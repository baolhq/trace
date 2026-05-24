use std::{path::PathBuf, sync::Arc};

use regex::Regex;
use trace_store::{
    db::{nodes_repo::NodesRepo, Database},
    index::SearchIndex,
    vault::reader::VaultReader,
};

pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

pub struct SearchService {
    index: Arc<SearchIndex>,
    db: Arc<Database>,
    vault_root: PathBuf,
}

impl SearchService {
    pub fn new(index: Arc<SearchIndex>, db: Arc<Database>, vault_root: PathBuf) -> Self {
        Self {
            index,
            db,
            vault_root,
        }
    }

    /// Bulk-indexes all vault notes from DB. Called once at startup.
    pub fn build_index(&self) {
        let repo = NodesRepo::new(Arc::clone(&self.db));
        let reader = VaultReader::new(&self.vault_root);
        match repo.list_all_for_index() {
            Ok(nodes) => {
                let count = nodes.len();
                for (id, path, title, modified_at, tags_str) in nodes {
                    let body = reader.read_node(&path).unwrap_or_default();
                    self.index
                        .index_node(&id, &title, &body, &tags_str, modified_at);
                }
                self.index.commit();
                tracing::info!("search: indexed {} notes", count);
            }
            Err(e) => tracing::warn!("search: build_index failed: {}", e),
        }
    }

    /// Re-indexes a single note from disk after it has been saved.
    pub fn index_from_vault(&self, id: &str) -> Result<(), String> {
        let repo = NodesRepo::new(Arc::clone(&self.db));
        let Some((path, title, modified_at, tags_str)) =
            repo.get_for_index(id).map_err(|e| e.to_string())?
        else {
            return Ok(());
        };
        let body = VaultReader::new(&self.vault_root)
            .read_node(&path)
            .map_err(|e| e.to_string())?;
        self.index
            .index_node(id, &title, &body, &tags_str, modified_at);
        // TODO: debounce commits (~1–2 s) instead of committing on every save.
        self.index.commit();
        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        is_regex: bool,
        limit: usize,
    ) -> Result<Vec<SearchHit>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        if is_regex {
            self.search_regex(query, limit)
        } else {
            Ok(self.search_text(query, limit))
        }
    }

    /// Tantivy full-text search. Snippets are pulled from vault files on demand.
    fn search_text(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let hits = self.index.search(query, limit);
        let reader = VaultReader::new(&self.vault_root);
        let terms = extract_terms(query);

        hits.into_iter()
            .map(|r| {
                let snippet = vault_snippet(&self.db, &reader, &r.id, &terms);
                SearchHit {
                    id: r.id,
                    title: r.title,
                    snippet,
                }
            })
            .collect()
    }

    /// Regex scan across all vault files. Returns matches with highlighted snippets.
    fn search_regex(&self, pattern: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
        let re = Regex::new(pattern).map_err(|e| e.to_string())?;
        let repo = NodesRepo::new(Arc::clone(&self.db));
        let nodes = repo.list_all_paths().map_err(|e| e.to_string())?;
        let reader = VaultReader::new(&self.vault_root);

        let mut hits = Vec::new();
        for (id, path, title) in nodes {
            if hits.len() >= limit {
                break;
            }
            let raw = reader.read_node(&path).unwrap_or_default();
            // Search only in body content — skip title heading shown in result UI
            let content = strip_front(&raw);
            if let Some(m) = re.find(content) {
                let snippet = make_snippet(content, m.start(), m.end());
                hits.push(SearchHit { id, title, snippet });
            }
        }
        Ok(hits)
    }
}

/// Extract a snippet from the vault file for a Tantivy text-search hit.
/// Strips the title heading so the snippet shows body content, not a duplicate title.
fn vault_snippet(db: &Database, reader: &VaultReader, node_id: &str, terms: &[String]) -> String {
    let path: Option<String> = db
        .conn()
        .query_row(
            "SELECT path FROM nodes WHERE id = ?1",
            rusqlite::params![node_id],
            |row| row.get(0),
        )
        .ok();
    let Some(path) = path else {
        return String::new();
    };
    let raw = reader.read_node(&path).unwrap_or_default();
    let content = strip_front(&raw);

    let content_lower = content.to_lowercase();
    for term in terms {
        if let Some(pos) = content_lower.find(term.as_str()) {
            return make_snippet(content, pos, pos + term.len());
        }
    }
    // Fallback: first ~160 chars of content (not the raw file start)
    let preview: String = content.chars().take(160).collect();
    html_escape(&preview)
}

/// Strip YAML frontmatter and the first H1 heading from a vault file's raw text,
/// returning a slice into the body content that follows.
fn strip_front(text: &str) -> &str {
    let mut s = text;
    // Skip YAML frontmatter (--- ... ---)
    if s.starts_with("---") {
        if let Some(rel) = s[3..].find("\n---") {
            let after = &s[3 + rel + 4..]; // past the closing \n---
            s = after.trim_start_matches(|c| c == '\r' || c == '\n');
        }
    }
    // Skip the first heading line (the note title)
    s = s.trim_start();
    if s.starts_with('#') {
        if let Some(nl) = s.find('\n') {
            s = &s[nl + 1..];
        } else {
            return "";
        }
    }
    // Drop any leading blank lines
    s.trim_start_matches(|c: char| c == '\n' || c == '\r')
}

/// Tokenize a Tantivy query string into individual search terms for snippet matching.
fn extract_terms(query: &str) -> Vec<String> {
    query
        .split(|c: char| c.is_whitespace() || c == '"' || c == '+' || c == '-')
        .map(|t| t.trim_matches(':').to_lowercase())
        .filter(|t| t.len() > 1)
        .collect()
}

fn make_snippet(text: &str, match_start: usize, match_end: usize) -> String {
    const PRE: usize = 80;
    const POST: usize = 160;

    let pre_chars: Vec<char> = text[..match_start].chars().rev().take(PRE).collect();
    let has_pre = pre_chars.len() == PRE;
    let pre: String = pre_chars.into_iter().rev().collect();

    let post_chars: Vec<char> = text[match_end..].chars().take(POST).collect();
    let has_post = post_chars.len() == POST;
    let post: String = post_chars.into_iter().collect();

    format!(
        "{}{}<b>{}</b>{}{}",
        if has_pre { "…" } else { "" },
        html_escape(&pre),
        html_escape(&text[match_start..match_end]),
        html_escape(&post),
        if has_post { "…" } else { "" },
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
