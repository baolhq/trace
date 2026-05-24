/// Result returned by the index layer. No snippet — snippets are extracted
/// from vault files by the service layer (body is not stored in the index).
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub score: f32,
}
