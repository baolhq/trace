use std::sync::{Arc, RwLock};

use fst::{Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use fst::automaton::Str;
use tracing::info;

use trace_store::db::Database;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeSuggestion {
    pub id: String,
    pub title: String,
}

struct SuggestIndex {
    title_map: Map<Vec<u8>>,
    nodes: Vec<NodeSuggestion>,
    tag_map: Map<Vec<u8>>,
    tags: Vec<String>,
}

pub struct SuggestService {
    db: Arc<Database>,
    index: Arc<RwLock<Option<SuggestIndex>>>,
}

impl SuggestService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db, index: Arc::new(RwLock::new(None)) }
    }

    /// Rebuild the FST index from the current database state.
    /// Must be called on startup and after any save that changes nodes or tags.
    pub fn rebuild(&self) {
        let (nodes, tags) = match self.load_from_db() {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("suggest: failed to load from DB: {}", e);
                return;
            }
        };

        let node_count = nodes.len();
        let tag_count = tags.len();

        let Some(idx) = build_index(nodes, tags) else { return };
        *self.index.write().unwrap() = Some(idx);
        info!("suggest: index rebuilt ({} nodes, {} tags)", node_count, tag_count);
    }

    /// Returns up to `limit` nodes whose titles start with `prefix` (case-insensitive).
    pub fn suggest_nodes(&self, prefix: &str, limit: usize) -> Vec<NodeSuggestion> {
        let guard = self.index.read().unwrap();
        let Some(idx) = guard.as_ref() else { return vec![] };

        let prefix_lc = prefix.to_lowercase();
        let automaton = Str::new(&prefix_lc).starts_with();
        let mut stream = idx.title_map.search(automaton).into_stream();
        let mut results = Vec::new();
        while let Some((_, i)) = stream.next() {
            if let Some(node) = idx.nodes.get(i as usize) {
                results.push(node.clone());
            }
            if results.len() >= limit {
                break;
            }
        }
        results
    }

    /// Returns up to `limit` tag names that start with `prefix` (case-insensitive).
    pub fn suggest_tags(&self, prefix: &str, limit: usize) -> Vec<String> {
        let guard = self.index.read().unwrap();
        let Some(idx) = guard.as_ref() else { return vec![] };

        let prefix_lc = prefix.to_lowercase();
        let automaton = Str::new(&prefix_lc).starts_with();
        let mut stream = idx.tag_map.search(automaton).into_stream();
        let mut results = Vec::new();
        while let Some((_, i)) = stream.next() {
            if let Some(tag) = idx.tags.get(i as usize) {
                results.push(tag.clone());
            }
            if results.len() >= limit {
                break;
            }
        }
        results
    }

    fn load_from_db(&self) -> Result<(Vec<NodeSuggestion>, Vec<String>), rusqlite::Error> {
        let conn = self.db.conn();

        let mut stmt =
            conn.prepare("SELECT id, title FROM nodes ORDER BY LOWER(title), title")?;
        let nodes: Vec<NodeSuggestion> = stmt
            .query_map([], |row| {
                Ok(NodeSuggestion { id: row.get(0)?, title: row.get(1)? })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t
             WHERE EXISTS (SELECT 1 FROM node_tags nt WHERE nt.tag_id = t.id)
             ORDER BY LOWER(t.name), t.name",
        )?;
        let tags: Vec<String> =
            stmt.query_map([], |row| row.get(0))?.filter_map(|r| r.ok()).collect();

        Ok((nodes, tags))
    }
}

fn build_index(nodes: Vec<NodeSuggestion>, tags: Vec<String>) -> Option<SuggestIndex> {
    // Sort by lowercase key — FST requires lexicographic order with unique keys.
    let mut node_pairs: Vec<(String, NodeSuggestion)> =
        nodes.into_iter().map(|n| (n.title.to_lowercase(), n)).collect();
    node_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    // Deduplicate by lowercase key (node titles are unique but may collide when lowercased)
    node_pairs.dedup_by(|a, b| a.0 == b.0);

    let mut title_builder = MapBuilder::memory();
    for (i, (key, _)) in node_pairs.iter().enumerate() {
        if let Err(e) = title_builder.insert(key.as_bytes(), i as u64) {
            tracing::warn!("suggest: FST title insert error: {}", e);
            return None;
        }
    }
    let title_bytes = match title_builder.into_inner() {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("suggest: FST title build error: {}", e);
            return None;
        }
    };
    let title_map = match Map::new(title_bytes) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("suggest: FST title map error: {}", e);
            return None;
        }
    };
    let nodes_vec: Vec<NodeSuggestion> = node_pairs.into_iter().map(|(_, n)| n).collect();

    let mut tag_pairs: Vec<(String, String)> =
        tags.into_iter().map(|t| (t.to_lowercase(), t)).collect();
    tag_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    tag_pairs.dedup_by(|a, b| a.0 == b.0);

    let mut tag_builder = MapBuilder::memory();
    for (i, (key, _)) in tag_pairs.iter().enumerate() {
        if let Err(e) = tag_builder.insert(key.as_bytes(), i as u64) {
            tracing::warn!("suggest: FST tag insert error: {}", e);
            return None;
        }
    }
    let tag_bytes = match tag_builder.into_inner() {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("suggest: FST tag build error: {}", e);
            return None;
        }
    };
    let tag_map = match Map::new(tag_bytes) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("suggest: FST tag map error: {}", e);
            return None;
        }
    };
    let tags_vec: Vec<String> = tag_pairs.into_iter().map(|(_, t)| t).collect();

    Some(SuggestIndex { title_map, nodes: nodes_vec, tag_map, tags: tags_vec })
}
