use std::{
    path::PathBuf,
    sync::{atomic::AtomicU64, Arc},
};

use tokio::sync::{broadcast, watch};
use trace_services::{
    events::CoreEvent, link_service::LinkService, log_service::LogService,
    node_service::NodeService, search_service::SearchService, settings_service::SettingsService,
    suggest_service::SuggestService, tag_service::TagService,
};
use trace_store::db::Database;

pub struct AppState {
    pub db: Arc<Database>,
    pub vault_path: PathBuf,
    pub event_tx: broadcast::Sender<CoreEvent>,
    pub node_service: NodeService,
    pub link_service: LinkService,
    pub tag_service: TagService,
    pub log_service: LogService,
    pub suggest_service: SuggestService,
    pub search_service: SearchService,
    pub settings_service: SettingsService,
    pub search_epoch: Arc<AtomicU64>,
    pub ready_rx: watch::Receiver<bool>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Arc<Database>,
        vault_path: PathBuf,
        event_tx: broadcast::Sender<CoreEvent>,
        node_service: NodeService,
        link_service: LinkService,
        tag_service: TagService,
        log_service: LogService,
        suggest_service: SuggestService,
        search_service: SearchService,
        settings_service: SettingsService,
        ready_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            db,
            vault_path,
            event_tx,
            node_service,
            link_service,
            tag_service,
            log_service,
            suggest_service,
            search_service,
            settings_service,
            search_epoch: Arc::new(AtomicU64::new(0)),
            ready_rx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.event_tx.subscribe()
    }
}
