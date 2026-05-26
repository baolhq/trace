use tauri::State;
use tracing::{info, warn};

use trace_core::{
    markdown::{doc::PmDoc, extract_tags},
    model::Node,
};

use super::NodeInfo;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct OpenNodeResponse {
    pub meta: Node,
    pub doc: PmDoc,
}

#[tauri::command]
pub fn list_nodes(state: State<'_, AppState>) -> Result<Vec<NodeInfo>, String> {
    state
        .node_service
        .list_recent_info(20)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_node(id: String, state: State<'_, AppState>) -> Result<OpenNodeResponse, String> {
    let meta = state
        .node_service
        .get_meta(&id)
        .map_err(|e| e.to_string())?;
    let doc = state
        .node_service
        .read_doc(&id)
        .map_err(|e| e.to_string())?;
    if let Err(e) = state.node_service.record_recent(&id) {
        warn!("record_recent failed for {id}: {e}");
    }
    Ok(OpenNodeResponse { meta, doc })
}

#[tauri::command]
pub fn save_node(id: String, doc: PmDoc, state: State<'_, AppState>) -> Result<(), String> {
    state
        .node_service
        .save_doc(&id, &doc)
        .map_err(|e| e.to_string())?;
    let tags = extract_tags(&doc);
    if let Err(e) = state.tag_service.sync_tags(&id, &tags) {
        warn!("sync_tags failed for {id}: {e}");
    }
    state.suggest_service.rebuild();
    Ok(())
}

#[tauri::command]
pub fn create_node(title: String, state: State<'_, AppState>) -> Result<String, String> {
    let id = state
        .node_service
        .create(&title)
        .map_err(|e| e.to_string())?;
    state.suggest_service.rebuild();
    info!("command: created node {title:?} ({id})");
    Ok(id.to_string())
}

#[tauri::command]
pub fn rename_node(id: String, title: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .node_service
        .rename(&id, &title)
        .map_err(|e| e.to_string())?;
    state.suggest_service.rebuild();
    info!("command: renamed node {id} to {title:?}");
    Ok(())
}

#[tauri::command]
pub fn delete_node(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.node_service.delete(&id).map_err(|e| e.to_string())?;
    state.suggest_service.rebuild();
    info!("command: deleted node {id}");
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    state
        .node_service
        .toggle_favorite(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> Result<Vec<NodeInfo>, String> {
    let nodes = state
        .node_service
        .list_favorites()
        .map_err(|e| e.to_string())?;
    Ok(nodes.into_iter().map(NodeInfo::from).collect())
}
