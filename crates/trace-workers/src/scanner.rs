use std::{collections::HashSet, path::PathBuf, sync::Arc};

use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use trace_core::{hash::hash_content, id::NodeId, markdown::extract_title};
use trace_services::events::CoreEvent;
use trace_store::db::Database;

use crate::util::{mtime_of, now_ms};

pub struct Scanner {
    vault_path: PathBuf,
    db: Arc<Database>,
    /// Sends relative paths of changed/new .md files to the indexer pipeline.
    tx: mpsc::Sender<String>,
    event_tx: broadcast::Sender<CoreEvent>,
}

impl Scanner {
    pub fn new(
        vault_path: PathBuf,
        db: Arc<Database>,
        tx: mpsc::Sender<String>,
        event_tx: broadcast::Sender<CoreEvent>,
    ) -> Self {
        Self { vault_path, db, tx, event_tx }
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
                self.insert_node(&rel, &bytes, &hash, mtime_ms);
                changed += 1;
                let _ = self.tx.send(rel).await;
            } else if stored.as_deref() != Some(hash.as_str()) {
                debug!("scanner: changed: {rel}");
                changed += 1;
                let _ = self.tx.send(rel).await;
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
            match conn.execute("DELETE FROM nodes WHERE path=?1", rusqlite::params![path]) {
                Ok(n) if n > 0 => info!("scanner: removed orphaned node: {path}"),
                _ => {}
            }
        }

        // Checkpoint last_scan_ts so cold-start skips unchanged files next time.
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
        let _ = self.event_tx.send(CoreEvent::ScanComplete);
    }

    fn insert_node(&self, rel: &str, bytes: &[u8], hash: &str, mtime_ms: i64) {
        let content = std::str::from_utf8(bytes).unwrap_or("");
        let title = extract_title(content, rel);
        let id = NodeId::generate();
        let conn = self.db.conn();
        match conn.execute(
            "INSERT OR IGNORE INTO nodes(id, path, title, created_at, modified_at, content_hash, byte_size)
             VALUES(?1, ?2, ?3, ?4, ?4, ?5, ?6)",
            rusqlite::params![id.as_str(), rel, title, mtime_ms, hash, bytes.len() as i64],
        ) {
            Ok(n) if n > 0 => debug!("scanner: inserted new node: {rel} ({id})"),
            Ok(_) => {}
            Err(e) => warn!("scanner: insert failed for {rel}: {e}"),
        }
    }
}
