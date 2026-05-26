use std::{collections::HashSet, path::PathBuf, sync::Arc};

use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use trace_core::{
    hash::hash_content,
    id::NodeId,
    markdown::{extract_tags, parse::parse, title_from_path},
};
use trace_services::{events::CoreEvent, link_service::LinkService, tag_service::TagService};
use trace_store::db::Database;

use crate::util::{mtime_of, now_ms};

pub struct Scanner {
    vault_path: PathBuf,
    db: Arc<Database>,
    tag_service: TagService,
    link_service: LinkService,
    event_tx: broadcast::Sender<CoreEvent>,
}

impl Scanner {
    pub fn new(
        vault_path: PathBuf,
        db: Arc<Database>,
        tag_service: TagService,
        link_service: LinkService,
        event_tx: broadcast::Sender<CoreEvent>,
    ) -> Self {
        Self {
            vault_path,
            db,
            tag_service,
            link_service,
            event_tx,
        }
    }

    pub async fn run(&self) {
        info!("scanner: starting scan of {:?}", self.vault_path);
        let mut changed = 0usize;
        let mut seen: HashSet<String> = HashSet::new();

        for entry in WalkDir::new(&self.vault_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        {
            let abs = entry.path();
            let rel = match abs.strip_prefix(&self.vault_path) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            seen.insert(rel.clone());

            let bytes = match std::fs::read(abs) {
                Ok(b) => b,
                Err(e) => {
                    warn!("scanner: read error {rel}: {e}");
                    continue;
                }
            };
            let hash = hash_content(&bytes);
            let mtime_ms = entry.metadata().map(mtime_of).unwrap_or_else(|_| now_ms());

            let stored = self.db.get_content_hash(&rel);
            if stored.is_none() {
                if self.insert_node(&rel, &bytes, &hash, mtime_ms).is_some() {
                    changed += 1;
                }
            } else if stored.as_deref() != Some(hash.as_str()) {
                debug!("scanner: changed: {rel}");
                changed += 1;
                let content = std::str::from_utf8(&bytes).unwrap_or("");
                let node_id: Option<String> = {
                    let conn = self.db.conn();
                    conn.query_row(
                        "SELECT id FROM nodes WHERE path=?1",
                        rusqlite::params![rel],
                        |row| row.get(0),
                    )
                    .ok()
                };
                if let Some(node_id) = node_id {
                    let doc = parse(content);
                    let tags = extract_tags(&doc);
                    if let Err(e) = self.tag_service.sync_tags(&node_id, &tags) {
                        warn!("scanner: sync_tags failed for {rel}: {e}");
                    }
                    if let Err(e) = self.link_service.extract_and_store(&node_id, content) {
                        warn!("scanner: extract_and_store failed for {rel}: {e}");
                    }
                }
            }
        }

        let orphans: Vec<String> = {
            let conn = self.db.conn();
            let mut stmt = match conn.prepare("SELECT path FROM nodes") {
                Ok(s) => s,
                Err(e) => {
                    warn!("scanner: orphan query failed: {e}");
                    return;
                }
            };
            stmt.query_map([], |row| row.get(0))
                .unwrap_or_else(|_| unreachable!())
                .filter_map(|r| r.ok())
                .filter(|p: &String| !seen.contains(p))
                .collect()
        };

        for path in &orphans {
            let conn = self.db.conn();
            let deleted = conn
                .execute("DELETE FROM nodes WHERE path=?1", rusqlite::params![path])
                .map_or(0, |n| n);
            if deleted > 0 {
                info!("scanner: removed orphaned node: {path}");
            }
        }

        {
            let conn = self.db.conn();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO app_meta(key,value) VALUES('last_scan_ts',?1)",
                rusqlite::params![now_ms().to_string()],
            );
        }

        info!(
            "scanner: done — {changed} changed file(s), {} orphan(s) removed",
            orphans.len()
        );

        self.backfill_links_if_needed();

        let _ = self.event_tx.send(CoreEvent::ScanComplete);
    }

    /// One-time backfill: extracts links for every known node the first time
    /// this version of the app runs. Guarded by `links_version` in `app_meta`.
    fn backfill_links_if_needed(&self) {
        let already_done = {
            let conn = self.db.conn();
            conn.query_row(
                "SELECT value FROM app_meta WHERE key='links_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .as_deref()
                == Some("1")
        };

        if already_done {
            return;
        }

        info!("scanner: running one-time link backfill for all existing nodes");

        let nodes: Vec<(String, String)> = {
            let conn = self.db.conn();
            let mut stmt = match conn.prepare("SELECT id, path FROM nodes") {
                Ok(s) => s,
                Err(e) => {
                    warn!("scanner: backfill query failed: {e}");
                    return;
                }
            };
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap_or_else(|_| unreachable!())
                .filter_map(|r| r.ok())
                .collect()
        };

        let count = nodes.len();
        for (node_id, rel_path) in &nodes {
            let abs = self.vault_path.join(rel_path);
            match std::fs::read_to_string(&abs) {
                Ok(content) => {
                    if let Err(e) = self.link_service.extract_and_store(node_id, &content) {
                        warn!("scanner: backfill failed for {rel_path}: {e}");
                    }
                }
                Err(e) => warn!("scanner: backfill read error {rel_path}: {e}"),
            }
        }

        {
            let conn = self.db.conn();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO app_meta(key,value) VALUES('links_version','1')",
                [],
            );
        }

        info!("scanner: link backfill complete ({count} nodes)");
    }

    fn insert_node(&self, rel: &str, bytes: &[u8], hash: &str, mtime_ms: i64) -> Option<String> {
        let content = std::str::from_utf8(bytes).unwrap_or("");
        let title = title_from_path(rel);
        let id = NodeId::generate();
        let inserted = {
            let conn = self.db.conn();
            match conn.execute(
                "INSERT OR IGNORE INTO nodes(id, path, title, created_at, modified_at, content_hash, byte_size)
                 VALUES(?1, ?2, ?3, ?4, ?4, ?5, ?6)",
                rusqlite::params![id.as_str(), rel, title, mtime_ms, hash, bytes.len() as i64],
            ) {
                Ok(n) if n > 0 => {
                    debug!("scanner: inserted new node: {rel} ({id})");
                    true
                }
                Ok(_) => false,
                Err(e) => {
                    warn!("scanner: insert failed for {rel}: {e}");
                    false
                }
            }
        };
        if inserted {
            let doc = parse(content);
            let tags = extract_tags(&doc);
            if let Err(e) = self.tag_service.sync_tags(id.as_str(), &tags) {
                warn!("scanner: sync_tags failed for {rel}: {e}");
            }
            if let Err(e) = self.link_service.extract_and_store(id.as_str(), content) {
                warn!("scanner: extract_and_store failed for {rel}: {e}");
            }
            Some(id.to_string())
        } else {
            None
        }
    }
}
