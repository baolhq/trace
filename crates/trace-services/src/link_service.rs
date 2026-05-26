use std::sync::Arc;

use trace_core::{
    id::NodeId,
    markdown::links::extract_links,
    model::{LinkInfo, LinkType},
};
use trace_store::db::{links_repo::LinksRepo, nodes_repo::NodesRepo, Database};

use super::error::ServiceError;

pub struct LinkService {
    db: Arc<Database>,
}

impl LinkService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn links_repo(&self) -> LinksRepo {
        LinksRepo::new(Arc::clone(&self.db))
    }

    fn nodes_repo(&self) -> NodesRepo {
        NodesRepo::new(Arc::clone(&self.db))
    }

    /// Extracts links from `content`, resolves wiki-link titles to node IDs,
    /// and replaces the stored link set for `node_id`.
    pub fn extract_and_store(&self, node_id: &str, content: &str) -> Result<(), ServiceError> {
        let mut links = extract_links(node_id, content);
        let nodes_repo = self.nodes_repo();

        for link in &mut links {
            if link.link_type == LinkType::Wiki && link.to_id.is_none() {
                // target_raw for wiki links is "[[Title]]"
                if let Some(title) = link
                    .target_raw
                    .strip_prefix("[[")
                    .and_then(|s| s.strip_suffix("]]"))
                {
                    if let Ok(Some(id_str)) = nodes_repo.get_id_by_title(title) {
                        link.to_id = NodeId::new(id_str).ok();
                    }
                }
            }
        }

        self.links_repo()
            .replace_for_node(node_id, &links)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    /// Returns all notes that link TO `node_id`.
    pub fn get_backlinks(&self, node_id: &str) -> Result<Vec<LinkInfo>, ServiceError> {
        self.links_repo()
            .get_backlinks(node_id)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    /// Returns all links sourced FROM `node_id`.
    pub fn get_outgoing(&self, node_id: &str) -> Result<Vec<LinkInfo>, ServiceError> {
        self.links_repo()
            .get_outgoing(node_id)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    /// Resolves any dangling `[[title]]` links by pointing them at the newly-created node.
    /// Call this after a node is created so pre-existing links to its title become resolved.
    pub fn resolve_new_node(&self, title: &str, node_id: &str) -> Result<u64, ServiceError> {
        let target_raw = format!("[[{title}]]");
        self.links_repo()
            .resolve_unresolved(&target_raw, node_id)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }
}
