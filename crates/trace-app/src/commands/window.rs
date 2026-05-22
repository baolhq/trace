use tauri::{AppHandle, Manager, State};
use tracing::{info, warn};

use crate::state::AppState;

/// Called by the frontend once its first render + data load is complete.
/// Shows the window only when both the backend and frontend are ready,
/// preventing a blank or partially-loaded window from ever being visible.
#[tauri::command]
pub fn frontend_ready(state: State<'_, AppState>, app: AppHandle) {
    if !state.is_backend_ready() {
        // Shouldn't happen with synchronous startup, but safe-guards against
        // a future async startup path.
        warn!("window: frontend ready but backend is not — window stays hidden");
        return;
    }

    match app.get_webview_window("main") {
        Some(window) => {
            window.show().ok();
            info!("window: shown");
        }
        None => warn!("window: 'main' window not found"),
    }
}
