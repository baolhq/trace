use tauri::State;
use tracing::{info, warn};

use trace_core::{
    markdown::{doc::PmDoc, extract_tags},
    model::Node,
};

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub title: String,
    pub created_at: i64,
}

#[derive(serde::Serialize)]
pub struct OpenNodeResponse {
    pub meta: Node,
    pub doc: PmDoc,
}

#[tauri::command]
pub fn list_nodes(state: State<'_, AppState>) -> Result<Vec<NodeInfo>, String> {
    let conn = state.db.conn();
    // Return the 20 most-recently-touched notes. Recently-opened float to the top;
    // unvisited notes fall back to modified_at order.
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.created_at
             FROM nodes n
             LEFT JOIN recent_nodes rn ON n.id = rn.node_id
             ORDER BY rn.opened_at DESC NULLS LAST, n.modified_at DESC, n.title
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let nodes = stmt
        .query_map([], |row| {
            Ok(NodeInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(nodes)
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
pub fn delete_node(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.node_service.delete(&id).map_err(|e| e.to_string())?;
    state.suggest_service.rebuild();
    info!("command: deleted node {id}");
    Ok(())
}
