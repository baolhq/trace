use std::sync::Arc;

use rusqlite::params;
use trace_core::model::{Link, LinkInfo};

use super::Database;

pub struct LinksRepo {
    db: Arc<Database>,
}

impl LinksRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Replaces all links sourced from `source_id` in a single transaction.
    pub fn replace_for_node(&self, source_id: &str, links: &[Link]) -> Result<(), rusqlite::Error> {
        let mut conn = self.db.conn();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM links WHERE source_id = ?1", params![source_id])?;
        for link in links {
            tx.execute(
                "INSERT INTO links(source_id, target_id, target_raw, link_type)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    source_id,
                    link.to_id.as_ref().map(|id| id.as_str()),
                    &link.target_raw,
                    link.link_type as u8,
                ],
            )?;
        }
        tx.commit()
    }

    /// Returns all notes that link TO `target_id`, enriched with source titles.
    pub fn get_backlinks(&self, target_id: &str) -> Result<Vec<LinkInfo>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT l.source_id, n.title, l.target_raw, l.link_type
             FROM links l
             JOIN nodes n ON n.id = l.source_id
             WHERE l.target_id = ?1
             ORDER BY n.title",
        )?;
        let rows = stmt
            .query_map(params![target_id], |row| {
                Ok(LinkInfo {
                    node_id: Some(row.get(0)?),
                    title: Some(row.get(1)?),
                    target_raw: row.get(2)?,
                    link_type: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Returns all links sourced FROM `source_id`, enriched with target titles where resolved.
    pub fn get_outgoing(&self, source_id: &str) -> Result<Vec<LinkInfo>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT l.target_id, n.title, l.target_raw, l.link_type
             FROM links l
             LEFT JOIN nodes n ON n.id = l.target_id
             WHERE l.source_id = ?1
             ORDER BY COALESCE(n.title, l.target_raw)",
        )?;
        let rows = stmt
            .query_map(params![source_id], |row| {
                Ok(LinkInfo {
                    node_id: row.get(0)?,
                    title: row.get(1)?,
                    target_raw: row.get(2)?,
                    link_type: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Resolves dangling `[[target_raw]]` links by setting their target_id.
    /// Returns the number of rows updated.
    pub fn resolve_unresolved(
        &self,
        target_raw: &str,
        target_id: &str,
    ) -> Result<u64, rusqlite::Error> {
        let n = self.db.conn().execute(
            "UPDATE links SET target_id = ?1 WHERE target_raw = ?2 AND target_id IS NULL",
            params![target_id, target_raw],
        )?;
        Ok(n as u64)
    }
}
