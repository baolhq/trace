use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, mpsc};
use trace_services::events::CoreEvent;
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

    // scan_tx is consumed by the Scanner; the receiver end lives here until
    // the Indexer worker is wired up in Phase 5.
    let (scan_tx, _scan_rx) = mpsc::channel::<String>(256);

    let scanner = Scanner::new(vault_path.clone(), Arc::clone(&db), scan_tx);
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

    // Forward NodesChanged events to the Tauri frontend so the UI can refresh.
    let mut ui_rx = event_tx.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match ui_rx.recv().await {
                Ok(CoreEvent::NodesChanged) => {
                    let _ = app_handle.emit("nodes_changed", ());
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {}
            }
        }
    });

    info!("startup: scanner, watcher, and file-sync spawned");
    AppState::new(db, vault_path, event_tx)
}
