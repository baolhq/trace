pub struct LinkService;

impl LinkService {
    pub fn extract_and_store(&self, _node_id: &str, _content: &str) {
        // TODO: parse links, resolve targets, replace_for_node in DB
    }

    pub fn get_backlinks(&self, _node_id: &str) {
        // TODO: delegate to LinksRepo
    }
}
