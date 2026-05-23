use std::sync::Arc;

use trace_core::model::{Log, NodeInfo};
use trace_store::db::{logs_repo::LogsRepo, Database};

use super::error::ServiceError;

pub struct LogService {
    db: Arc<Database>,
}

impl LogService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn repo(&self) -> LogsRepo {
        LogsRepo::new(Arc::clone(&self.db))
    }

    // ── Log lifecycle ────────────────────────────────────────────────────────

    pub fn create(&self, name: &str, parent_id: Option<i64>) -> Result<i64, ServiceError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ServiceError::InvalidInput(
                "log name cannot be empty".into(),
            ));
        }
        self.repo().create(name, parent_id).map_err(Into::into)
    }

    pub fn rename(&self, id: i64, name: &str) -> Result<(), ServiceError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ServiceError::InvalidInput(
                "log name cannot be empty".into(),
            ));
        }
        let found = self.repo().rename(id, name)?;
        if !found {
            return Err(ServiceError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), ServiceError> {
        let found = self.repo().delete(id)?;
        if !found {
            return Err(ServiceError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Returns all logs as a flat list sorted by sort_key. The caller builds
    /// the hierarchy using parent_id.
    pub fn tree(&self) -> Result<Vec<Log>, ServiceError> {
        self.repo().get_all().map_err(Into::into)
    }

    // ── Member management ────────────────────────────────────────────────────

    /// Appends `node_id` to the log (idempotent — no-op if already a member).
    pub fn add_node(&self, log_id: i64, node_id: &str) -> Result<(), ServiceError> {
        let repo = self.repo();
        if repo.is_member(log_id, node_id)? {
            return Ok(());
        }
        let sort_key = repo.max_member_sort_key(log_id)? + 1.0;
        repo.add_member(log_id, node_id, sort_key)?;
        Ok(())
    }

    pub fn remove_node(&self, log_id: i64, node_id: &str) -> Result<(), ServiceError> {
        self.repo().remove_member(log_id, node_id)?;
        Ok(())
    }

    /// Page-based member list (page 0-indexed, limit items per page).
    pub fn members(
        &self,
        log_id: i64,
        page: usize,
        limit: usize,
    ) -> Result<Vec<NodeInfo>, ServiceError> {
        self.repo()
            .members_paged(log_id, page, limit)
            .map_err(Into::into)
    }

    pub fn member_count(&self, log_id: i64) -> Result<usize, ServiceError> {
        self.repo().member_count(log_id).map_err(Into::into)
    }

    /// Moves `node_id` to appear immediately after `after_id` in sort order.
    /// `after_id = None` moves the node to the front.
    pub fn reorder_node(
        &self,
        log_id: i64,
        node_id: &str,
        after_id: Option<&str>,
    ) -> Result<(), ServiceError> {
        let repo = self.repo();
        let (prev, next) = repo.neighbor_sort_keys(log_id, after_id)?;
        let new_key = midpoint(prev, next);
        repo.update_member_sort_key(log_id, node_id, new_key)?;
        Ok(())
    }

    /// Moves a log to appear immediately after `after_id` among its siblings.
    /// `after_id = None` moves the log to the front of its sibling group.
    pub fn reorder(&self, id: i64, after_id: Option<i64>) -> Result<(), ServiceError> {
        let repo = self.repo();
        let all = repo.get_all()?;
        let log = all
            .iter()
            .find(|l| l.id == id)
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))?;
        let siblings: Vec<&Log> = all
            .iter()
            .filter(|l| l.parent_id == log.parent_id && l.id != id)
            .collect();

        let (prev, next) = match after_id {
            None => (None, siblings.first().map(|l| l.sort_key)),
            Some(aid) => {
                let pos = siblings.iter().position(|l| l.id == aid);
                let prev_key = pos.and_then(|i| siblings.get(i)).map(|l| l.sort_key);
                let next_key = pos.and_then(|i| siblings.get(i + 1)).map(|l| l.sort_key);
                (prev_key, next_key)
            }
        };
        let new_key = midpoint(prev, next);
        repo.update_sort_key(id, new_key)?;
        Ok(())
    }
}

/// Fractional-index midpoint. Falls back to endpoints ± 1.0 when one side is open.
fn midpoint(prev: Option<f64>, next: Option<f64>) -> f64 {
    match (prev, next) {
        (Some(a), Some(b)) => (a + b) / 2.0,
        (Some(a), None) => a + 1.0,
        (None, Some(b)) => b - 1.0,
        (None, None) => 0.0,
    }
}
