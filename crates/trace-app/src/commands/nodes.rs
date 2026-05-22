use std::time::SystemTime;

use tauri::State;
use tracing::info;

use trace_core::{hash::hash_content, id::NodeId};
use trace_store::vault::writer::VaultWriter;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub title: String,
    pub created_at: i64,
}

#[tauri::command]
pub fn list_nodes(state: State<'_, AppState>) -> Result<Vec<NodeInfo>, String> {
    let conn = state.db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, created_at FROM nodes ORDER BY modified_at DESC",
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
pub fn create_node(title: String, state: State<'_, AppState>) -> Result<String, String> {
    let id = NodeId::generate();
    let rel_path = format!("{}.md", id);
    let content = format!("# {}\n", title.trim());
    let bytes = content.as_bytes();
    let hash = hash_content(bytes);

    let writer = VaultWriter::new(&state.vault_path);
    writer.write_node(&rel_path, &content).map_err(|e| e.to_string())?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    {
        let conn = state.db.conn();
        conn.execute(
            "INSERT INTO nodes(id, path, title, created_at, modified_at, content_hash, byte_size)
             VALUES(?1, ?2, ?3, ?4, ?4, ?5, ?6)",
            rusqlite::params![
                id.as_str(),
                rel_path,
                title.trim(),
                now,
                hash,
                bytes.len() as i64
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    info!("created node: {} ({})", title.trim(), id);
    Ok(id.to_string())
}

#[tauri::command]
pub fn delete_node(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let rel_path: Option<String> = {
        let conn = state.db.conn();
        conn.query_row(
            "SELECT path FROM nodes WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok()
    };

    if let Some(path) = rel_path {
        // File may already be gone (deleted externally) — that's fine.
        let _ = VaultWriter::new(&state.vault_path).delete_node(&path);

        let conn = state.db.conn();
        conn.execute("DELETE FROM nodes WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;

        info!("deleted node: {id}");
    }

    Ok(())
}
