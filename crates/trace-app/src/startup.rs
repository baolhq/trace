use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;
use trace_services::{
    events::CoreEvent, link_service::LinkService, log_service::LogService,
    node_service::NodeService, search_service::SearchService, suggest_service::SuggestService,
    tag_service::TagService,
};
use trace_store::db::{migrations, Database};
use trace_workers::{FileSync, Scanner, Watcher};
use tracing::{info, instrument};

use crate::state::AppState;

#[instrument(skip_all, fields(db = ?db_path))]
pub fn init(vault_path: PathBuf, db_path: PathBuf, app_handle: AppHandle) -> AppState {
    info!("startup: opening database at {:?}", db_path);
    let db = Arc::new(Database::open(&db_path).expect("failed to open database"));
    {
        let conn = db.conn();
        migrations::run(&conn).expect("migrations failed");
    }
    info!("startup: schema up to date");

    let (event_tx, event_rx) = broadcast::channel::<CoreEvent>(512);
    let mut ui_rx = event_tx.subscribe();

    let (ready_tx, ready_rx) = tokio::sync::watch::channel(false);

    let scanner = Scanner::new(
        vault_path.clone(),
        Arc::clone(&db),
        TagService::new(Arc::clone(&db)),
        LinkService::new(Arc::clone(&db)),
        event_tx.clone(),
    );
    tauri::async_runtime::spawn(async move {
        scanner.run().await;
        let _ = ready_tx.send(true);
    });

    let watcher = Watcher::new(vault_path.clone(), event_tx.clone());
    tauri::async_runtime::spawn(async move { watcher.run().await });

    let mut file_sync = FileSync::new(
        vault_path.clone(),
        Arc::clone(&db),
        TagService::new(Arc::clone(&db)),
        LinkService::new(Arc::clone(&db)),
        event_tx.clone(),
        event_rx,
    );
    tauri::async_runtime::spawn(async move { file_sync.run().await });

    tauri::async_runtime::spawn(async move {
        loop {
            match ui_rx.recv().await {
                Ok(CoreEvent::NodesChanged | CoreEvent::ScanComplete) => {
                    let _ = app_handle.emit("nodes_changed", ());
                }
                Ok(CoreEvent::LinksUpdated { id }) => {
                    let _ = app_handle.emit("links_updated", id);
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {}
            }
        }
    });

    let node_service = NodeService::new(Arc::clone(&db), vault_path.clone());
    let link_service = LinkService::new(Arc::clone(&db));
    let tag_service = TagService::new(Arc::clone(&db));
    let log_service = LogService::new(Arc::clone(&db));
    let suggest_service = SuggestService::new(Arc::clone(&db));
    suggest_service.rebuild();

    let search_service = SearchService::new(Arc::clone(&db), vault_path.clone());

    info!("startup: all services ready");

    AppState::new(
        db,
        vault_path,
        event_tx,
        node_service,
        link_service,
        tag_service,
        log_service,
        suggest_service,
        search_service,
        ready_rx,
    )
}
