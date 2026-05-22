pub struct TagService;

impl TagService {
    pub fn sync_tags(&self, _node_id: &str, _tags: &[&str]) {
        // TODO: upsert tags, sync node_tags join table
    }

    pub fn list_all(&self) -> Vec<String> {
        // TODO: SELECT name FROM tags ORDER BY name
        Vec::new()
    }
}
