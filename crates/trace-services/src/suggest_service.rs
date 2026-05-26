use std::sync::{Arc, RwLock};

use tracing::info;

use trace_store::db::Database;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeSuggestion {
    pub id: String,
    pub title: String,
}

pub struct SuggestService {
    db: Arc<Database>,
    nodes: Arc<RwLock<Vec<NodeSuggestion>>>,
    tags: Arc<RwLock<Vec<String>>>,
}

impl SuggestService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            nodes: Arc::new(RwLock::new(vec![])),
            tags: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Reload nodes and tags from the database.
    /// Must be called on startup and after any save that changes nodes or tags.
    pub fn rebuild(&self) {
        let (nodes, tags) = match self.load_from_db() {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("suggest: failed to load from DB: {}", e);
                return;
            }
        };
        info!(
            "suggest: index rebuilt ({} nodes, {} tags)",
            nodes.len(),
            tags.len()
        );
        *self.nodes.write().unwrap() = nodes;
        *self.tags.write().unwrap() = tags;
    }

    /// Returns up to `limit` nodes whose titles contain `query` (case-insensitive).
    pub fn suggest_nodes(&self, query: &str, limit: usize) -> Vec<NodeSuggestion> {
        let q = query.to_lowercase();
        self.nodes
            .read()
            .unwrap()
            .iter()
            .filter(|n| n.title.to_lowercase().contains(&q))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Returns up to `limit` tag names that contain `query` (case-insensitive).
    pub fn suggest_tags(&self, query: &str, limit: usize) -> Vec<String> {
        let q = query.to_lowercase();
        self.tags
            .read()
            .unwrap()
            .iter()
            .filter(|t| t.to_lowercase().contains(&q))
            .take(limit)
            .cloned()
            .collect()
    }

    fn load_from_db(&self) -> Result<(Vec<NodeSuggestion>, Vec<String>), rusqlite::Error> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare("SELECT id, title FROM nodes ORDER BY LOWER(title), title")?;
        let nodes: Vec<NodeSuggestion> = stmt
            .query_map([], |row| {
                Ok(NodeSuggestion {
                    id: row.get(0)?,
                    title: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t
             WHERE EXISTS (SELECT 1 FROM node_tags nt WHERE nt.tag_id = t.id)
             ORDER BY LOWER(t.name), t.name",
        )?;
        let tags: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok((nodes, tags))
    }
}
