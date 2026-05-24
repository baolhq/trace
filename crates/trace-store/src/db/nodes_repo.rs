use std::sync::Arc;

use rusqlite::params;
use trace_core::{
    id::NodeId,
    model::{Node, NodeInfo},
};

use super::Database;

pub fn row_to_node_info(row: &rusqlite::Row<'_>) -> rusqlite::Result<NodeInfo> {
    Ok(NodeInfo {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        is_favorite: row.get::<_, i64>(3)? != 0,
    })
}

pub struct NodesRepo {
    db: Arc<Database>,
}

impl NodesRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn upsert(&self, node: &Node) -> Result<(), rusqlite::Error> {
        self.db.conn().execute(
            "INSERT OR REPLACE INTO nodes
             (id, path, title, created_at, modified_at, content_hash, byte_size, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                node.id.as_str(),
                node.path,
                node.title,
                node.created_at,
                node.modified_at,
                node.content_hash,
                node.byte_size as i64,
                node.is_favorite as i64,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Node>, rusqlite::Error> {
        match self.db.conn().query_row(
            "SELECT id, path, title, created_at, modified_at, content_hash, byte_size, is_favorite
             FROM nodes WHERE id = ?1",
            params![id],
            row_to_node,
        ) {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let n = self
            .db
            .conn()
            .execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    pub fn record_recent(&self, node_id: &str, opened_at: i64) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR REPLACE INTO recent_nodes(node_id, opened_at) VALUES (?1, ?2)",
            params![node_id, opened_at],
        )?;
        conn.execute(
            "DELETE FROM recent_nodes WHERE node_id NOT IN (
                SELECT node_id FROM recent_nodes ORDER BY opened_at DESC LIMIT 50
             )",
            [],
        )?;
        Ok(())
    }

    /// Flips is_favorite and returns the new state.
    pub fn toggle_favorite(&self, id: &str) -> Result<bool, rusqlite::Error> {
        self.db.conn().execute(
            "UPDATE nodes SET is_favorite = 1 - is_favorite WHERE id = ?1",
            params![id],
        )?;
        let new_state: i64 = self.db.conn().query_row(
            "SELECT is_favorite FROM nodes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(new_state != 0)
    }

    pub fn list_favorites(&self) -> Result<Vec<Node>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, title, created_at, modified_at, content_hash, byte_size, is_favorite
             FROM nodes WHERE is_favorite = 1 ORDER BY modified_at DESC",
        )?;
        let nodes = stmt
            .query_map([], row_to_node)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(nodes)
    }

    pub fn list_recent_info(&self, limit: usize) -> Result<Vec<NodeInfo>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.created_at, n.is_favorite
             FROM nodes n
             LEFT JOIN recent_nodes rn ON n.id = rn.node_id
             ORDER BY rn.opened_at DESC NULLS LAST, n.modified_at DESC, n.title
             LIMIT ?1",
        )?;
        let nodes = stmt
            .query_map(params![limit as i64], row_to_node_info)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(nodes)
    }

    /// Returns (id, path, title) for every node — used for bulk indexing.
    pub fn list_all_paths(&self) -> Result<Vec<(String, String, String)>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt =
            conn.prepare("SELECT id, path, title FROM nodes ORDER BY modified_at DESC")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Returns (id, path, title, modified_at, tags_str) for every node.
    /// `tags_str` is space-joined tag names for Tantivy indexing.
    pub fn list_all_for_index(
        &self,
    ) -> Result<Vec<(String, String, String, i64, String)>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.path, n.title, n.modified_at,
                    COALESCE(GROUP_CONCAT(t.name, ' '), '') AS tags_str
             FROM nodes n
             LEFT JOIN node_tags nt ON nt.node_id = n.id
             LEFT JOIN tags t ON t.id = nt.tag_id
             GROUP BY n.id
             ORDER BY n.modified_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Returns (path, title, modified_at, tags_str) for one node.
    pub fn get_for_index(
        &self,
        id: &str,
    ) -> Result<Option<(String, String, i64, String)>, rusqlite::Error> {
        let conn = self.db.conn();
        match conn.query_row(
            "SELECT n.path, n.title, n.modified_at,
                    COALESCE(GROUP_CONCAT(t.name, ' '), '') AS tags_str
             FROM nodes n
             LEFT JOIN node_tags nt ON nt.node_id = n.id
             LEFT JOIN tags t ON t.id = nt.tag_id
             WHERE n.id = ?1
             GROUP BY n.id",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn list_recent_opened(&self, limit: usize) -> Result<Vec<Node>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.path, n.title, n.created_at, n.modified_at,
                    n.content_hash, n.byte_size, n.is_favorite
             FROM recent_nodes r
             JOIN nodes n ON n.id = r.node_id
             ORDER BY r.opened_at DESC
             LIMIT ?1",
        )?;
        let nodes = stmt
            .query_map(params![limit as i64], row_to_node)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(nodes)
    }
}

fn row_to_node(row: &rusqlite::Row<'_>) -> rusqlite::Result<Node> {
    let id_str: String = row.get(0)?;
    Ok(Node {
        id: NodeId::new(id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        path: row.get(1)?,
        title: row.get(2)?,
        created_at: row.get(3)?,
        modified_at: row.get(4)?,
        content_hash: row.get(5)?,
        byte_size: row.get::<_, i64>(6)? as u64,
        is_favorite: row.get::<_, i64>(7)? != 0,
    })
}
