use std::sync::Arc;

use trace_store::db::Database;

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
