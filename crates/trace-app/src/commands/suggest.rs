use tauri::State;
use trace_services::suggest_service::NodeSuggestion;

use crate::state::AppState;

const SUGGEST_LIMIT: usize = 8;

#[tauri::command]
pub fn suggest_nodes(prefix: String, state: State<'_, AppState>) -> Vec<NodeSuggestion> {
    state.suggest_service.suggest_nodes(&prefix, SUGGEST_LIMIT)
}

#[tauri::command]
pub fn suggest_tags(prefix: String, state: State<'_, AppState>) -> Vec<String> {
    state.suggest_service.suggest_tags(&prefix, SUGGEST_LIMIT)
}
