use std::{path::PathBuf, sync::Arc};

use tokio::sync::{broadcast, mpsc};
use tracing::{info, instrument};
use trace_services::events::CoreEvent;
use trace_store::db::{migrations, Database};
use trace_workers::{Scanner, Watcher};

use crate::state::AppState;

#[instrument(skip_all, fields(db = ?db_path))]
pub fn init(vault_path: PathBuf, db_path: PathBuf) -> AppState {
    info!("startup: opening database at {:?}", db_path);
    let db = Arc::new(Database::open(&db_path).expect("failed to open database"));
    {
        let conn = db.conn();
        migrations::run(&conn).expect("migrations failed");
    }
    info!("startup: schema up to date");

    let (event_tx, _event_rx) = broadcast::channel::<CoreEvent>(512);

    // scan_tx is consumed by the Scanner; the receiver end lives here until
    // the Indexer worker is wired up in Phase 5.
    let (scan_tx, _scan_rx) = mpsc::channel::<String>(256);

    let scanner = Scanner::new(vault_path.clone(), Arc::clone(&db), scan_tx);
    tauri::async_runtime::spawn(async move { scanner.run().await });

    let watcher = Watcher::new(vault_path.clone(), event_tx.clone());
    tauri::async_runtime::spawn(async move { watcher.run().await });

    info!("startup: scanner and watcher spawned");
    AppState::new(db, vault_path, event_tx)
}
