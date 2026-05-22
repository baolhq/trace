use std::{path::PathBuf, sync::Arc};

use tokio::sync::broadcast;
use trace_services::events::CoreEvent;
use trace_store::db::Database;

/// Arc-shared handles to all services, passed into Tauri commands via managed state.
pub struct AppState {
    pub db: Arc<Database>,
    pub vault_path: PathBuf,
    pub event_tx: broadcast::Sender<CoreEvent>,
}

impl AppState {
    pub fn new(
        db: Arc<Database>,
        vault_path: PathBuf,
        event_tx: broadcast::Sender<CoreEvent>,
    ) -> Self {
        Self { db, vault_path, event_tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.event_tx.subscribe()
    }
}
