use trace_core::model::Link;

pub struct LinksRepo;

impl LinksRepo {
    pub fn replace_for_node(&self, _source_id: &str, _links: &[Link]) {
        // TODO: DELETE + INSERT in a transaction
    }

    pub fn get_backlinks(&self, _target_id: &str) -> Vec<Link> {
        // TODO: SELECT * FROM links WHERE target_id = ?
        Vec::new()
    }

    pub fn resolve_unresolved(&self, _target_raw: &str, _target_id: &str) {
        // TODO: UPDATE links SET target_id=? WHERE target_raw=? AND target_id IS NULL
    }
}
