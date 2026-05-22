pub struct NodeService;

impl NodeService {
    pub fn create(&self, _title: &str, _body: &str) {
        // TODO: generate id, write to vault, upsert DB, index
    }

    pub fn save(&self, _id: &str, _body: &str) {
        // TODO: hash check, atomic vault write, update DB, re-index
    }

    pub fn delete(&self, _id: &str) {
        // TODO: remove from vault, DB, and index
    }
}
