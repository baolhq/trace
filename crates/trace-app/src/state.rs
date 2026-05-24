use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::broadcast;
use trace_services::{
    events::CoreEvent, log_service::LogService, node_service::NodeService,
    search_service::SearchService, suggest_service::SuggestService, tag_service::TagService,
};
use trace_store::db::Database;

pub struct AppState {
    pub db: Arc<Database>,
    pub vault_path: PathBuf,
    pub event_tx: broadcast::Sender<CoreEvent>,
    pub node_service: NodeService,
    pub tag_service: TagService,
    pub log_service: LogService,
    pub suggest_service: SuggestService,
    pub search_service: SearchService,
    backend_ready: AtomicBool,
}

impl AppState {
    pub fn new(
        db: Arc<Database>,
        vault_path: PathBuf,
        event_tx: broadcast::Sender<CoreEvent>,
        node_service: NodeService,
        tag_service: TagService,
        log_service: LogService,
        suggest_service: SuggestService,
        search_service: SearchService,
    ) -> Self {
        Self {
            db,
            vault_path,
            event_tx,
            node_service,
            tag_service,
            log_service,
            suggest_service,
            search_service,
            backend_ready: AtomicBool::new(false),
        }
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
