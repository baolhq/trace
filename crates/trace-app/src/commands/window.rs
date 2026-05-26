use tauri::State;

use crate::state::AppState;

#[tauri::command]
/// Called by the frontend to ensure that backend has finished database
/// migrations, indexing, backfilling.. and is ready to show main UI.
pub async fn backend_ready(state: State<'_, AppState>) -> Result<bool, ()> {
    let mut rx = state.ready_rx.clone();
    let _ = rx.wait_for(|v| *v).await;
    Ok(true)
}
