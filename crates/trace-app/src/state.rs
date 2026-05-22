use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::broadcast;
use trace_services::{events::CoreEvent, node_service::NodeService};
use trace_store::db::Database;

pub struct AppState {
    pub db: Arc<Database>,
    pub vault_path: PathBuf,
    pub event_tx: broadcast::Sender<CoreEvent>,
    pub node_service: NodeService,
    backend_ready: AtomicBool,
}

impl AppState {
    pub fn new(
        db: Arc<Database>,
        vault_path: PathBuf,
        event_tx: broadcast::Sender<CoreEvent>,
        node_service: NodeService,
    ) -> Self {
        Self { db, vault_path, event_tx, node_service, backend_ready: AtomicBool::new(false) }
    }

    pub fn mark_backend_ready(&self) {
        self.backend_ready.store(true, Ordering::Release);
    }

    pub fn is_backend_ready(&self) -> bool {
        self.backend_ready.load(Ordering::Acquire)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.event_tx.subscribe()
    }
}
