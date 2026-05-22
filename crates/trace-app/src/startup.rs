use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, mpsc};
use trace_services::{events::CoreEvent, node_service::NodeService};
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

    // Subscribe before spawning workers so no events (including ScanComplete)
    // are missed due to the broadcast channel's drop-if-no-subscriber behaviour.
    let mut ui_rx = event_tx.subscribe();

    let (scan_tx, _scan_rx) = mpsc::channel::<String>(256);

    let scanner = Scanner::new(vault_path.clone(), Arc::clone(&db), scan_tx, event_tx.clone());
    tauri::async_runtime::spawn(async move { scanner.run().await });

    let watcher = Watcher::new(vault_path.clone(), event_tx.clone());
    tauri::async_runtime::spawn(async move { watcher.run().await });

    let mut file_sync = FileSync::new(
        vault_path.clone(),
        Arc::clone(&db),
        event_tx.clone(),
        event_rx,
    );
    tauri::async_runtime::spawn(async move { file_sync.run().await });

    // Forward node-relevant events to the frontend so the UI can refresh its list.
    // ScanComplete triggers a refresh after the initial vault scan finishes.
    // NodesChanged covers external file-system changes detected by FileSync.
    tauri::async_runtime::spawn(async move {
        loop {
            match ui_rx.recv().await {
                Ok(CoreEvent::NodesChanged | CoreEvent::ScanComplete) => {
                    let _ = app_handle.emit("nodes_changed", ());
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {}
            }
        }
    });

    let node_service = NodeService::new(Arc::clone(&db), vault_path.clone());
    info!("startup: scanner, watcher, file-sync, and node-service ready");

    AppState::new(db, vault_path, event_tx, node_service)
}
