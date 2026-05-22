use std::{collections::HashSet, path::PathBuf, sync::Arc, time::SystemTime};

use tokio::sync::mpsc;
use tracing::{info, warn};
use walkdir::WalkDir;

use trace_core::{hash::hash_content, id::NodeId};
use trace_store::db::Database;

pub struct Scanner {
    vault_path: PathBuf,
    db: Arc<Database>,
    /// Sends relative paths of changed/new .md files to the indexer pipeline.
    tx: mpsc::Sender<String>,
}

impl Scanner {
    pub fn new(vault_path: PathBuf, db: Arc<Database>, tx: mpsc::Sender<String>) -> Self {
        Self { vault_path, db, tx }
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

            let stored = self.db.get_content_hash(&rel);
            if stored.is_none() {
                // New file not in DB — insert it.
                self.insert_node(&rel, &bytes, &hash);
                changed += 1;
                let _ = self.tx.send(rel).await;
            } else if stored.as_deref() != Some(hash.as_str()) {
                // Known file with different content.
                info!("scanner: changed: {rel}");
                changed += 1;
                let _ = self.tx.send(rel).await;
            }
        }

        // Remove DB records for files that no longer exist on disk.
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
        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        {
            let conn = self.db.conn();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO app_meta(key,value) VALUES('last_scan_ts',?1)",
                rusqlite::params![now_ms.to_string()],
            );
        }

        info!(
            "scanner: done — {changed} changed file(s), {} orphan(s) removed",
            orphans.len()
        );
    }

    fn insert_node(&self, rel: &str, bytes: &[u8], hash: &str) {
        let content = std::str::from_utf8(bytes).unwrap_or("");
        let title = extract_title(content, rel);
        let id = NodeId::generate();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let conn = self.db.conn();
        match conn.execute(
            "INSERT OR IGNORE INTO nodes(id, path, title, created_at, modified_at, content_hash, byte_size)
             VALUES(?1, ?2, ?3, ?4, ?4, ?5, ?6)",
            rusqlite::params![id.as_str(), rel, title, now, hash, bytes.len() as i64],
        ) {
            Ok(n) if n > 0 => info!("scanner: inserted new node: {rel} ({id})"),
            Ok(_) => {}
            Err(e) => warn!("scanner: insert failed for {rel}: {e}"),
        }
    }
}

pub(crate) fn extract_title(content: &str, rel_path: &str) -> String {
    content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l[2..].trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::path::Path::new(rel_path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel_path.to_string())
        })
}
