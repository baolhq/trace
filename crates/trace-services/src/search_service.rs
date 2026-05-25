use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
};

use rayon::prelude::*;
use trace_store::{
    db::{nodes_repo::NodesRepo, Database},
    vault::reader::VaultReader,
};

pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

pub struct SearchService {
    db: Arc<Database>,
    vault_root: PathBuf,
}

impl SearchService {
    pub fn new(db: Arc<Database>, vault_root: PathBuf) -> Self {
        Self { db, vault_root }
    }

    pub fn search(
        &self,
        query: &str,
        is_regex: bool,
        match_case: bool,
        whole_word: bool,
        limit: usize,
    ) -> Result<Vec<SearchHit>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let pattern = build_pattern(query, is_regex, whole_word);
        self.search_regex(&pattern, match_case, limit)
    }

    pub fn search_async(
        &self,
        query: &str,
        is_regex: bool,
        match_case: bool,
        whole_word: bool,
        limit: usize,
        epoch: u64,
        current_epoch: Arc<AtomicU64>,
        on_hit: impl Fn(SearchHit) + Send + Sync,
    ) -> Result<(), String> {
        if query.trim().is_empty() {
            return Ok(());
        }
        let pattern = build_pattern(query, is_regex, whole_word);
        let re = regex::RegexBuilder::new(&pattern)
            .case_insensitive(!match_case)
            .build()
            .map_err(|e| e.to_string())?;
        let repo = NodesRepo::new(Arc::clone(&self.db));
        let nodes = repo.list_all_paths().map_err(|e| e.to_string())?;
        let reader = VaultReader::new(&self.vault_root);
        let sent = AtomicUsize::new(0);

        nodes.into_par_iter().for_each(|(id, path, title)| {
            if current_epoch.load(Ordering::Relaxed) != epoch {
                return;
            }
            if sent.load(Ordering::Relaxed) >= limit {
                return;
            }
            let raw = reader.read_node(&path).unwrap_or_default();
            let content = strip_front(&raw);
            if let Some(m) = re.find(content) {
                if sent.fetch_add(1, Ordering::Relaxed) >= limit {
                    return;
                }
                if current_epoch.load(Ordering::Relaxed) != epoch {
                    return;
                }
                on_hit(SearchHit {
                    id,
                    title,
                    snippet: make_snippet(content, m.start(), m.end()),
                });
            }
        });

        Ok(())
    }

    fn search_regex(
        &self,
        pattern: &str,
        match_case: bool,
        limit: usize,
    ) -> Result<Vec<SearchHit>, String> {
        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(!match_case)
            .build()
            .map_err(|e| e.to_string())?;
        let repo = NodesRepo::new(Arc::clone(&self.db));
        let nodes = repo.list_all_paths().map_err(|e| e.to_string())?;
        let reader = VaultReader::new(&self.vault_root);

        let mut hits: Vec<SearchHit> = nodes
            .into_par_iter()
            .filter_map(|(id, path, title)| {
                let raw = reader.read_node(&path).unwrap_or_default();
                let content = strip_front(&raw);
                re.find(content).map(|m| {
                    let snippet = make_snippet(content, m.start(), m.end());
                    SearchHit { id, title, snippet }
                })
            })
            .collect();

        hits.truncate(limit);
        Ok(hits)
    }
}

fn build_pattern(query: &str, is_regex: bool, whole_word: bool) -> String {
    if is_regex {
        if whole_word {
            format!(r"\b(?:{query})\b")
        } else {
            query.to_owned()
        }
    } else {
        let escaped = regex::escape(query);
        if whole_word {
            format!(r"\b(?:{escaped})\b")
        } else {
            escaped
        }
    }
}

fn strip_front(text: &str) -> &str {
    let mut s = text;
    if s.starts_with("---") {
        if let Some(rel) = s[3..].find("\n---") {
            let after = &s[3 + rel + 4..];
            s = after.trim_start_matches(|c| c == '\r' || c == '\n');
        }
    }
    s = s.trim_start();
    if s.starts_with('#') {
        if let Some(nl) = s.find('\n') {
            s = &s[nl + 1..];
        } else {
            return "";
        }
    }
    s.trim_start_matches(|c: char| c == '\n' || c == '\r')
}

fn make_snippet(text: &str, match_start: usize, match_end: usize) -> String {
    const PRE: usize = 20;
    const POST: usize = 120;

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
