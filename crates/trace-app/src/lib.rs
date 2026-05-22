mod commands;
mod startup;
mod state;

use crate::commands::nodes::{create_node, delete_node, list_nodes, open_node, save_node};
use crate::commands::window::frontend_ready;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let vault_path = app_dir.join("vault");
            let db_path = app_dir.join("metadata.db");
            std::fs::create_dir_all(&vault_path)?;

            let state = startup::init(vault_path, db_path, app.handle().clone());
            state.mark_backend_ready();
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_nodes,
            open_node,
            save_node,
            create_node,
            delete_node,
            frontend_ready,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
