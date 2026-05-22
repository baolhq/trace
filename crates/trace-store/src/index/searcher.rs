/// Pooled Tantivy index readers for concurrent search.
pub struct IndexSearcher;

pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
}

impl IndexSearcher {
    pub fn search(&self, _query: &str, _limit: usize) -> Vec<SearchResult> {
        // TODO: parse query, execute via Tantivy QueryParser, collect top-k
        Vec::new()
    }
}
