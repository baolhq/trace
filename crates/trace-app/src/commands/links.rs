use tauri::State;
use trace_core::model::LinkInfo;

use crate::state::AppState;

#[tauri::command]
pub fn get_links(node_id: String, state: State<'_, AppState>) -> Result<Vec<LinkInfo>, String> {
    state
        .link_service
        .get_outgoing(&node_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_backlinks(node_id: String, state: State<'_, AppState>) -> Result<Vec<LinkInfo>, String> {
    state
        .link_service
        .get_backlinks(&node_id)
        .map_err(|e| e.to_string())
}
