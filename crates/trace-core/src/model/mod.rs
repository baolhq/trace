use serde::{Deserialize, Serialize};

use crate::id::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub path: String,
    pub title: String,
    pub created_at: i64,
    pub modified_at: i64,
    pub content_hash: String,
    pub byte_size: u64,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub from_id: NodeId,
    pub to_id: Option<NodeId>,
    pub target_raw: String,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LinkType {
    Wiki = 0,
    IdRef = 1,
    Embed = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_key: f64,
}

/// Lightweight node DTO used in list and member views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub is_favorite: bool,
}

impl From<Node> for NodeInfo {
    fn from(n: Node) -> Self {
        Self {
            id: n.id.to_string(),
            title: n.title,
            created_at: n.created_at,
            is_favorite: n.is_favorite,
        }
    }
}
