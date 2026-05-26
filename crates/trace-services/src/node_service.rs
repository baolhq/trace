use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use tracing::info;

use trace_core::{
    hash::hash_content,
    id::NodeId,
    markdown::{
        doc::PmDoc,
        parse::{parse, parse_with_spans},
        serialize::{serialize, serialize_block_to_string},
    },
    model::{Node, NodeInfo},
};
use trace_store::{
    cache::MetadataCache,
    db::{nodes_repo::NodesRepo, Database},
    vault::{reader::VaultReader, writer::VaultWriter},
};

use super::error::ServiceError;

const METADATA_CACHE_CAPACITY: usize = 10_000;
const INVALID_TITLE_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|', '\0'];

pub struct NodeService {
    db: Arc<Database>,
    vault_root: PathBuf,
    cache: Mutex<MetadataCache>,
}

impl NodeService {
    pub fn new(db: Arc<Database>, vault_root: PathBuf) -> Self {
        Self {
            db,
            vault_root,
            cache: Mutex::new(MetadataCache::new(METADATA_CACHE_CAPACITY)),
        }
    }

    fn repo(&self) -> NodesRepo {
        NodesRepo::new(Arc::clone(&self.db))
    }

    fn reader(&self) -> VaultReader {
        VaultReader::new(&self.vault_root)
    }

    fn writer(&self) -> VaultWriter {
        VaultWriter::new(&self.vault_root)
    }

    pub fn create(&self, title: &str) -> Result<NodeId, ServiceError> {
        let title = title.trim();
        validate_title(title)?;

        let rel_path = format!("{title}.md");
        if self.reader().exists(&rel_path) {
            return Err(ServiceError::InvalidInput(format!(
                "a node with title {title:?} already exists"
            )));
        }

        let content = String::new();
        let bytes = content.as_bytes();
        let hash = hash_content(bytes);
        let id = NodeId::generate();
        let now = now_ms();

        self.writer().write_node(&rel_path, &content)?;

        let node = Node {
            id: id.clone(),
            path: rel_path,
            title: title.to_string(),
            created_at: now,
            modified_at: now,
            content_hash: hash,
            byte_size: bytes.len() as u64,
            is_favorite: false,
        };
        self.repo()
            .upsert(&node)
            .map_err(|e| ServiceError::Db(e.to_string()))?;
        self.cache.lock().unwrap().insert(node);

        info!("node_service: created {title:?} ({id})");
        Ok(id)
    }

    pub fn get_meta(&self, id: &str) -> Result<Node, ServiceError> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(node) = cache.get(id) {
                return Ok(node.clone());
            }
        }
        let node = self
            .repo()
            .get_by_id(id)
            .map_err(|e| ServiceError::Db(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))?;
        self.cache.lock().unwrap().insert(node.clone());
        Ok(node)
    }

    pub fn read_body(&self, id: &str) -> Result<String, ServiceError> {
        let meta = self.get_meta(id)?;
        Ok(self.reader().read_node(&meta.path)?)
    }

    pub fn read_doc(&self, id: &str) -> Result<PmDoc, ServiceError> {
        let body = self.read_body(id)?;
        Ok(parse(&body))
    }

    pub fn save_doc(&self, id: &str, doc: &PmDoc) -> Result<(), ServiceError> {
        // Attempt block-level dirty tracking: preserve original bytes for unchanged
        // blocks so autosaves don't reformat untouched content in Git diffs.
        if let Ok(old_body) = self.read_body(id) {
            let (old_doc, spans) = parse_with_spans(&old_body);
            if old_doc.frontmatter == doc.frontmatter
                && old_doc.content.len() == doc.content.len()
                && !spans.is_empty()
            {
                let mut stitched = String::with_capacity(old_body.len());

                if let Some(fm) = &doc.frontmatter {
                    stitched.push_str("---\n");
                    stitched.push_str(fm);
                    stitched.push('\n');
                    stitched.push_str("---\n\n");
                }

                let last = doc.content.len().saturating_sub(1);
                for (i, (new_block, old_block)) in
                    doc.content.iter().zip(old_doc.content.iter()).enumerate()
                {
                    if new_block == old_block {
                        // Unchanged: copy original bytes, normalised to exactly one trailing \n
                        let raw = &old_body[spans[i].clone()];
                        let trimmed = raw.trim_end_matches('\n');
                        stitched.push_str(trimmed);
                        stitched.push('\n');
                    } else {
                        // Changed: canonical serialisation
                        stitched.push_str(&serialize_block_to_string(new_block));
                    }
                    if i < last {
                        stitched.push('\n'); // canonical blank-line separator
                    }
                }

                return self.save(id, &stitched);
            }
        }

        // Fallback: frontmatter changed, block count differs, or file unreadable
        let body = serialize(doc);
        self.save(id, &body)
    }

    pub fn save(&self, id: &str, body: &str) -> Result<(), ServiceError> {
        let meta = self.get_meta(id)?;
        let bytes = body.as_bytes();
        let new_hash = hash_content(bytes);

        if new_hash == meta.content_hash {
            return Ok(());
        }

        self.writer().write_node(&meta.path, body)?;

        let updated = Node {
            id: meta.id.clone(),
            path: meta.path.clone(),
            title: meta.title.clone(),
            created_at: meta.created_at,
            modified_at: now_ms(),
            content_hash: new_hash,
            byte_size: bytes.len() as u64,
            is_favorite: meta.is_favorite,
        };
        self.repo()
            .upsert(&updated)
            .map_err(|e| ServiceError::Db(e.to_string()))?;
        self.cache.lock().unwrap().insert(updated);

        info!("node_service: saved {id}");
        Ok(())
    }

    pub fn rename(&self, id: &str, new_title: &str) -> Result<(), ServiceError> {
        let new_title = new_title.trim();
        validate_title(new_title)?;

        let meta = self.get_meta(id)?;
        if new_title == meta.title {
            return Ok(());
        }

        let new_path = format!("{new_title}.md");
        if self.reader().exists(&new_path) {
            return Err(ServiceError::TitleInvalid(
                "a note with this title already exists".into(),
            ));
        }

        let body = self.reader().read_node(&meta.path)?;
        self.writer().write_node(&new_path, &body)?;
        let _ = self.writer().delete_node(&meta.path);

        let updated = Node {
            id: meta.id.clone(),
            path: new_path.clone(),
            title: new_title.to_string(),
            created_at: meta.created_at,
            modified_at: now_ms(),
            content_hash: meta.content_hash,
            byte_size: meta.byte_size,
            is_favorite: meta.is_favorite,
        };
        self.repo()
            .upsert(&updated)
            .map_err(|e| ServiceError::Db(e.to_string()))?;
        self.cache.lock().unwrap().insert(updated);

        info!("node_service: renamed {:?} -> {:?}", meta.path, new_path);
        Ok(())
    }

    pub fn list_all_titles(&self) -> Result<Vec<String>, ServiceError> {
        self.repo()
            .list_all_titles()
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    pub fn delete(&self, id: &str) -> Result<(), ServiceError> {
        let meta = self.get_meta(id)?;
        // File may already be gone if deleted externally — that's fine.
        let _ = self.writer().delete_node(&meta.path);
        self.repo()
            .delete(id)
            .map_err(|e| ServiceError::Db(e.to_string()))?;
        self.cache.lock().unwrap().invalidate(id);
        info!("node_service: deleted {id}");
        Ok(())
    }

    pub fn record_recent(&self, id: &str) -> Result<(), ServiceError> {
        self.repo()
            .record_recent(id, now_ms())
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    pub fn list_recent_opened(&self, limit: usize) -> Result<Vec<Node>, ServiceError> {
        self.repo()
            .list_recent_opened(limit)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    /// Flips is_favorite for `id` and returns the new state.
    pub fn toggle_favorite(&self, id: &str) -> Result<bool, ServiceError> {
        let new_state = self
            .repo()
            .toggle_favorite(id)
            .map_err(|e| ServiceError::Db(e.to_string()))?;
        // Invalidate cached metadata so subsequent get_meta returns the new state.
        self.cache.lock().unwrap().invalidate(id);
        Ok(new_state)
    }

    pub fn list_favorites(&self) -> Result<Vec<Node>, ServiceError> {
        self.repo()
            .list_favorites()
            .map_err(|e| ServiceError::Db(e.to_string()))
    }

    pub fn list_recent_info(&self, limit: usize) -> Result<Vec<NodeInfo>, ServiceError> {
        self.repo()
            .list_recent_info(limit)
            .map_err(|e| ServiceError::Db(e.to_string()))
    }
}

fn validate_title(title: &str) -> Result<(), ServiceError> {
    if title.is_empty() {
        return Err(ServiceError::InvalidInput("title is empty".into()));
    }
    if title
        .chars()
        .any(|c| INVALID_TITLE_CHARS.contains(&c) || c.is_control())
    {
        return Err(ServiceError::InvalidInput(format!(
            "title contains invalid characters: {title:?}"
        )));
    }
    Ok(())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
