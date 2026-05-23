use tauri::State;
use trace_core::model::NodeInfo;

use crate::state::AppState;

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.tag_service.list_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_nodes_by_tag(
    tag: String,
    page: usize,
    state: State<'_, AppState>,
) -> Result<Vec<NodeInfo>, String> {
    const LIMIT: usize = 50;
    state
        .tag_service
        .nodes_by_tag(&tag, page, LIMIT)
        .map_err(|e| e.to_string())
}
