pub struct SearchService;

pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

impl SearchService {
    pub fn query(&self, _text: &str, _limit: usize) -> Vec<SearchHit> {
        // TODO: delegate to IndexSearcher, merge with metadata from NodesRepo
        Vec::new()
    }
}
