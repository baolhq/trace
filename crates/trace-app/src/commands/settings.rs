use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use trace_services::settings_service::Settings;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct AllSettings {
    pub global: Settings,
    pub vault: Settings,
    pub merged: Settings,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AllSettings {
    AllSettings {
        global: state.settings_service.load_global(),
        vault: state.settings_service.load_vault(),
        merged: state.settings_service.merged(),
    }
}

#[tauri::command]
pub fn save_settings(
    scope: String,
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    match scope.as_str() {
        "global" => state.settings_service.save_global(&settings),
        "vault" => state.settings_service.save_vault(&settings),
        _ => Err(format!("unknown scope: {scope}")),
    }
}

#[tauri::command]
pub fn open_settings_file(
    scope: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let path = state.settings_service.ensure_and_get_path(&scope)?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}
