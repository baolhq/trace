use std::sync::Arc;

use rusqlite::params;
use trace_core::model::{Log, NodeInfo};

use super::{nodes_repo::row_to_node_info, Database};

pub struct LogsRepo {
    db: Arc<Database>,
}

impl LogsRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // ── Log CRUD ────────────────────────────────────────────────────────────

    pub fn create(&self, name: &str, parent_id: Option<i64>) -> Result<i64, rusqlite::Error> {
        let sort_key = self.next_sibling_sort_key(parent_id)?;
        self.db.conn().execute(
            "INSERT INTO logs(name, parent_id, sort_key) VALUES(?1, ?2, ?3)",
            params![name, parent_id, sort_key],
        )?;
        Ok(self.db.conn().last_insert_rowid())
    }

    pub fn get_all(&self) -> Result<Vec<Log>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt =
            conn.prepare("SELECT id, name, parent_id, sort_key FROM logs ORDER BY sort_key")?;
        let logs = stmt
            .query_map([], row_to_log)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(logs)
    }

    pub fn rename(&self, id: i64, name: &str) -> Result<bool, rusqlite::Error> {
        let n = self
            .db
            .conn()
            .execute("UPDATE logs SET name=?1 WHERE id=?2", params![name, id])?;
        Ok(n > 0)
    }

    pub fn delete(&self, id: i64) -> Result<bool, rusqlite::Error> {
        let n = self
            .db
            .conn()
            .execute("DELETE FROM logs WHERE id=?1", params![id])?;
        Ok(n > 0)
    }

    pub fn update_sort_key(&self, id: i64, sort_key: f64) -> Result<(), rusqlite::Error> {
        self.db.conn().execute(
            "UPDATE logs SET sort_key=?1 WHERE id=?2",
            params![sort_key, id],
        )?;
        Ok(())
    }

    // ── Member CRUD ─────────────────────────────────────────────────────────

    pub fn add_member(
        &self,
        log_id: i64,
        node_id: &str,
        sort_key: f64,
    ) -> Result<(), rusqlite::Error> {
        self.db.conn().execute(
            "INSERT OR IGNORE INTO log_members(log_id, node_id, sort_key) VALUES(?1, ?2, ?3)",
            params![log_id, node_id, sort_key],
        )?;
        Ok(())
    }

    pub fn remove_member(&self, log_id: i64, node_id: &str) -> Result<bool, rusqlite::Error> {
        let n = self.db.conn().execute(
            "DELETE FROM log_members WHERE log_id=?1 AND node_id=?2",
            params![log_id, node_id],
        )?;
        Ok(n > 0)
    }

    pub fn is_member(&self, log_id: i64, node_id: &str) -> Result<bool, rusqlite::Error> {
        let n: i64 = self.db.conn().query_row(
            "SELECT COUNT(*) FROM log_members WHERE log_id=?1 AND node_id=?2",
            params![log_id, node_id],
            |row| row.get(0),
        )?;
        Ok(n > 0)
    }

    /// Paged member list ordered by sort_key. Page is 0-indexed.
    pub fn members_paged(
        &self,
        log_id: i64,
        page: usize,
        limit: usize,
    ) -> Result<Vec<NodeInfo>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.created_at, n.is_favorite
             FROM log_members lm
             JOIN nodes n ON n.id = lm.node_id
             WHERE lm.log_id = ?1
             ORDER BY lm.sort_key
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt
            .query_map(
                params![log_id, limit as i64, (page * limit) as i64],
                row_to_node_info,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn member_count(&self, log_id: i64) -> Result<usize, rusqlite::Error> {
        let n: i64 = self.db.conn().query_row(
            "SELECT COUNT(*) FROM log_members WHERE log_id=?1",
            params![log_id],
            |row| row.get(0),
        )?;
        Ok(n as usize)
    }

    pub fn max_member_sort_key(&self, log_id: i64) -> Result<f64, rusqlite::Error> {
        let v: Option<f64> = self.db.conn().query_row(
            "SELECT MAX(sort_key) FROM log_members WHERE log_id=?1",
            params![log_id],
            |row| row.get(0),
        )?;
        Ok(v.unwrap_or(0.0))
    }

    /// Returns (sort_key_of_after, sort_key_of_item_following_after).
    /// `after_id = None` means "insert at the beginning".
    pub fn neighbor_sort_keys(
        &self,
        log_id: i64,
        after_id: Option<&str>,
    ) -> Result<(Option<f64>, Option<f64>), rusqlite::Error> {
        let conn = self.db.conn();
        match after_id {
            None => {
                // Inserting before the first item — predecessor is None, successor is min.
                let min: Option<f64> = conn.query_row(
                    "SELECT MIN(sort_key) FROM log_members WHERE log_id=?1",
                    params![log_id],
                    |row| row.get(0),
                )?;
                Ok((None, min))
            }
            Some(after) => {
                let after_key: Option<f64> = conn
                    .query_row(
                        "SELECT sort_key FROM log_members WHERE log_id=?1 AND node_id=?2",
                        params![log_id, after],
                        |row| row.get(0),
                    )
                    .ok();
                let next_key: Option<f64> = conn
                    .query_row(
                        "SELECT sort_key FROM log_members
                         WHERE log_id=?1 AND sort_key > ?2
                         ORDER BY sort_key LIMIT 1",
                        params![log_id, after_key.unwrap_or(0.0)],
                        |row| row.get(0),
                    )
                    .ok();
                Ok((after_key, next_key))
            }
        }
    }

    pub fn update_member_sort_key(
        &self,
        log_id: i64,
        node_id: &str,
        sort_key: f64,
    ) -> Result<(), rusqlite::Error> {
        self.db.conn().execute(
            "UPDATE log_members SET sort_key=?1 WHERE log_id=?2 AND node_id=?3",
            params![sort_key, log_id, node_id],
        )?;
        Ok(())
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Computes sort_key for a new log appended after siblings with the same parent.
    fn next_sibling_sort_key(&self, parent_id: Option<i64>) -> Result<f64, rusqlite::Error> {
        let max: Option<f64> = match parent_id {
            None => self.db.conn().query_row(
                "SELECT MAX(sort_key) FROM logs WHERE parent_id IS NULL",
                [],
                |row| row.get(0),
            )?,
            Some(pid) => self.db.conn().query_row(
                "SELECT MAX(sort_key) FROM logs WHERE parent_id=?1",
                params![pid],
                |row| row.get(0),
            )?,
        };
        Ok(max.unwrap_or(0.0) + 1.0)
    }
}

fn row_to_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<Log> {
    Ok(Log {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        sort_key: row.get(3)?,
    })
}
