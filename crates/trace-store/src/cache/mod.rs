use std::collections::HashMap;

use trace_core::model::Node;

/// LRU-backed in-memory cache for node metadata and parsed bodies.
pub struct MetadataCache {
    entries: HashMap<String, Node>,
    capacity: usize,
}

impl MetadataCache {
    pub fn new(capacity: usize) -> Self {
        Self { entries: HashMap::new(), capacity }
    }

    pub fn get(&self, id: &str) -> Option<&Node> {
        self.entries.get(id)
    }

    pub fn insert(&mut self, node: Node) {
        if self.entries.len() >= self.capacity {
            // TODO: evict LRU entry
        }
        self.entries.insert(node.id.as_str().to_owned(), node);
    }

    pub fn invalidate(&mut self, id: &str) {
        self.entries.remove(id);
    }
}
