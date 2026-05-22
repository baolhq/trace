use std::{path::PathBuf, sync::Arc, time::SystemTime};

use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use trace_core::hash::hash_content;
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

            let bytes = match std::fs::read(abs) {
                Ok(b) => b,
                Err(e) => { warn!("scanner: read error {rel}: {e}"); continue; }
            };
            let hash = hash_content(&bytes);

            let stored: Option<String> = {
                let conn = self.db.conn();
                conn.query_row(
                    "SELECT content_hash FROM nodes WHERE path=?1",
                    rusqlite::params![rel],
                    |row| row.get(0),
                )
                .ok()
            };

            if stored.as_deref() != Some(&hash) {
                debug!("scanner: changed: {rel}");
                changed += 1;
                // Ignore send errors — receiver may not be wired yet (Phase 1).
                let _ = self.tx.send(rel).await;
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

        info!("scanner: done — {changed} changed file(s) enqueued");
    }
}
