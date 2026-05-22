use trace_core::model::Node;

pub struct NodesRepo;

impl NodesRepo {
    pub fn upsert(&self, _node: &Node) {
        // TODO: INSERT OR REPLACE INTO nodes ...
    }

    pub fn get_by_id(&self, _id: &str) -> Option<Node> {
        // TODO: SELECT * FROM nodes WHERE id = ?
        None
    }

    pub fn list_recent(&self, _limit: usize) -> Vec<Node> {
        // TODO: SELECT * FROM nodes ORDER BY modified_at DESC LIMIT ?
        Vec::new()
    }

    pub fn delete(&self, _id: &str) {
        // TODO: DELETE FROM nodes WHERE id = ?
    }
}
