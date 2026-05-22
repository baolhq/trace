use std::{path::Path, sync::Mutex};

use tracing::debug;

pub mod folders_repo;
pub mod links_repo;
pub mod migrations;
pub mod nodes_repo;

/// Single-connection SQLite database with WAL mode and FK enforcement.
pub struct Database {
    conn: Mutex<rusqlite::Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        debug!("db: opening {:?}", path);
        let conn = rusqlite::Connection::open(path)?;
        Self::apply_pragmas(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.conn.lock().expect("db mutex poisoned")
    }

    fn apply_pragmas(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", "5000")?;
        Ok(())
    }

    pub fn get_content_hash(&self, path: &str) -> Option<String> {
        self.conn()
            .query_row(
                "SELECT content_hash FROM nodes WHERE path=?1",
                rusqlite::params![path],
                |row| row.get(0),
            )
            .ok()
    }
}
