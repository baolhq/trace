use tauri::State;
use trace_core::model::Log;

use super::NodeInfo;
use crate::state::AppState;

#[tauri::command]
pub fn get_log_tree(state: State<'_, AppState>) -> Result<Vec<Log>, String> {
    state.log_service.tree().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_log(
    name: String,
    parent_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    state
        .log_service
        .create(&name, parent_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_log(id: i64, name: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .log_service
        .rename(id, &name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_log(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.log_service.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_log_members(
    log_id: i64,
    page: usize,
    state: State<'_, AppState>,
) -> Result<Vec<NodeInfo>, String> {
    const LIMIT: usize = 50;
    state
        .log_service
        .members(log_id, page, LIMIT)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_to_log(log_id: i64, node_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .log_service
        .add_node(log_id, &node_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_from_log(
    log_id: i64,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .log_service
        .remove_node(log_id, &node_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_in_log(
    log_id: i64,
    node_id: String,
    after_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .log_service
        .reorder_node(log_id, &node_id, after_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_log(
    id: i64,
    after_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .log_service
        .reorder(id, after_id)
        .map_err(|e| e.to_string())
}
