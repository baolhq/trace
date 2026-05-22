use std::{path::PathBuf, sync::Arc, time::SystemTime};

use tokio::sync::broadcast;
use tracing::{info, warn};

use trace_core::{hash::hash_content, id::NodeId};
use trace_services::events::CoreEvent;
use trace_store::db::Database;

use crate::scanner::extract_title;

pub struct FileSync {
    vault_path: PathBuf,
    db: Arc<Database>,
    tx: broadcast::Sender<CoreEvent>,
    rx: broadcast::Receiver<CoreEvent>,
}

impl FileSync {
    pub fn new(
        vault_path: PathBuf,
        db: Arc<Database>,
        tx: broadcast::Sender<CoreEvent>,
        rx: broadcast::Receiver<CoreEvent>,
    ) -> Self {
        Self {
            vault_path,
            db,
            tx,
            rx,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.rx.recv().await {
                Ok(CoreEvent::FileChanged { path, .. }) => self.handle(path).await,
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(
                        "file_sync: lagged, missed {n} events — some changes may require restart"
                    );
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }

    async fn handle(&self, abs_path: String) {
        let path = std::path::Path::new(&abs_path);

        if path.extension().map_or(true, |e| e != "md") {
            return;
        }

        let rel = match path.strip_prefix(&self.vault_path) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => return,
        };

        if path.exists() {
            self.sync_created(&rel, path);
        } else {
            self.sync_deleted(&rel);
        }
    }

    fn sync_created(&self, rel: &str, abs: &std::path::Path) {
        let bytes = match std::fs::read(abs) {
            Ok(b) => b,
            Err(e) => {
                warn!("file_sync: read error {rel}: {e}");
                return;
            }
        };
        let hash = hash_content(&bytes);

        let stored_hash = self.db.get_content_hash(rel);
        if stored_hash.as_deref() == Some(hash.as_str()) {
            // Same hash is no-op; suppresses events for files we just wrote ourselves.
            return;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        if stored_hash.is_some() {
            let conn = self.db.conn();
            let _ = conn.execute(
                "UPDATE nodes SET content_hash=?1, byte_size=?2, modified_at=?3 WHERE path=?4",
                rusqlite::params![hash, bytes.len() as i64, now, rel],
            );
            info!("file_sync: updated node: {rel}");
            let _ = self.tx.send(CoreEvent::NodesChanged);
        } else {
            let content = std::str::from_utf8(&bytes).unwrap_or("");
            let title = extract_title(content, rel);
            let id = NodeId::generate();
            let conn = self.db.conn();
            match conn.execute(
                "INSERT OR IGNORE INTO nodes(id, path, title, created_at, modified_at, content_hash, byte_size)
                 VALUES(?1, ?2, ?3, ?4, ?4, ?5, ?6)",
                rusqlite::params![id.as_str(), rel, title, now, hash, bytes.len() as i64],
            ) {
                Ok(n) if n > 0 => {
                    info!("file_sync: inserted new node: {rel} ({id})");
                    let _ = self.tx.send(CoreEvent::NodesChanged);
                }
                Ok(_) => {}
                Err(e) => warn!("file_sync: insert failed for {rel}: {e}"),
            }
        }
    }

    fn sync_deleted(&self, rel: &str) {
        let conn = self.db.conn();
        match conn.execute("DELETE FROM nodes WHERE path=?1", rusqlite::params![rel]) {
            Ok(n) if n > 0 => {
                info!("file_sync: removed deleted node: {rel}");
                let _ = self.tx.send(CoreEvent::NodesChanged);
            }
            _ => {}
        }
    }
}
