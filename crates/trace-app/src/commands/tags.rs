use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.tag_service.list_all().map_err(|e| e.to_string())
}
