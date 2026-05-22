use std::num::NonZeroUsize;

use lru::LruCache;
use trace_core::model::Node;

pub struct MetadataCache {
    inner: LruCache<String, Node>,
}

impl MetadataCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap());
        Self { inner: LruCache::new(cap) }
    }

    pub fn get(&mut self, id: &str) -> Option<&Node> {
        self.inner.get(id)
    }

    pub fn insert(&mut self, node: Node) {
        self.inner.put(node.id.as_str().to_owned(), node);
    }

    pub fn invalidate(&mut self, id: &str) {
        self.inner.pop(id);
    }
}
