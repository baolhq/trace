use std::sync::Arc;

use trace_core::model::NodeInfo;
use trace_store::db::{nodes_repo::row_to_node_info, Database};

pub struct TagService {
    db: Arc<Database>,
}

impl TagService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Upsert a node's tags. Inserts new tag names, replaces the node's
    /// associations, then prunes orphan tag rows (tags with no node).
    pub fn sync_tags(&self, node_id: &str, tags: &[String]) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();

        for name in tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags(name) VALUES (?1)",
                rusqlite::params![name],
            )?;
        }

        conn.execute(
            "DELETE FROM node_tags WHERE node_id = ?1",
            rusqlite::params![node_id],
        )?;

        for name in tags {
            conn.execute(
                "INSERT OR IGNORE INTO node_tags(node_id, tag_id)
                 SELECT ?1, id FROM tags WHERE name = ?2",
                rusqlite::params![node_id, name],
            )?;
        }

        // Remove tags that are no longer referenced by any node.
        conn.execute(
            "DELETE FROM tags WHERE id NOT IN (SELECT DISTINCT tag_id FROM node_tags)",
            [],
        )?;

        Ok(())
    }

    /// Paged list of nodes that carry `tag` (page 0-indexed).
    pub fn nodes_by_tag(
        &self,
        tag: &str,
        page: usize,
        limit: usize,
    ) -> Result<Vec<NodeInfo>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.created_at, n.is_favorite
             FROM nodes n
             JOIN node_tags nt ON nt.node_id = n.id
             JOIN tags t ON t.id = nt.tag_id
             WHERE t.name = ?1
             ORDER BY n.modified_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt
            .query_map(
                rusqlite::params![tag, limit as i64, (page * limit) as i64],
                row_to_node_info,
            )?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// All tag names that appear in at least one node, sorted alphabetically.
    pub fn list_all(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t
             WHERE EXISTS (SELECT 1 FROM node_tags nt WHERE nt.tag_id = t.id)
             ORDER BY t.name",
        )?;
        let tags = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    }
}
