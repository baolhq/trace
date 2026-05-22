use std::sync::OnceLock;

use regex::Regex;

use crate::id::NodeId;
use crate::model::{Link, LinkType};

static LINK_RE: OnceLock<Regex> = OnceLock::new();

fn link_re() -> &'static Regex {
    LINK_RE.get_or_init(|| {
        // Matches optional leading '!' then [[...]] with no nested brackets or newlines.
        Regex::new(r"(!?)\[\[([^\[\]\n]+?)]]").expect("invalid link regex")
    })
}

/// Extracts [[wiki]], [[node:id]], and ![[embed]] links from raw markdown.
pub fn extract_links(from_id: &str, content: &str) -> Vec<Link> {
    let from = match NodeId::new(from_id) {
        Ok(id) => id,
        Err(_) => return Vec::new(),
    };

    link_re()
        .captures_iter(content)
        .map(|cap| {
            let is_embed = &cap[1] == "!";
            let inner = cap[2].to_string();

            let (link_type, to_id) = if is_embed {
                (LinkType::Embed, None)
            } else if let Some(id_str) = inner.strip_prefix("node:") {
                let to = NodeId::new(id_str).ok();
                (LinkType::IdRef, to)
            } else {
                (LinkType::Wiki, None)
            };

            let target_raw = if is_embed {
                format!("![[{inner}]]")
            } else {
                format!("[[{inner}]]")
            };

            Link {
                from_id: from.clone(),
                to_id,
                target_raw,
                link_type,
            }
        })
        .collect()
}

pub fn is_wiki_link(text: &str) -> bool {
    text.starts_with("[[") && text.ends_with("]]")
}

pub fn is_tag(text: &str) -> bool {
    text.starts_with('#') && text.len() > 1
}

#[allow(dead_code)]
fn classify(raw: &str) -> LinkType {
    if raw.starts_with("![[") {
        LinkType::Embed
    } else if raw.contains(':') {
        LinkType::IdRef
    } else {
        LinkType::Wiki
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_link() {
        let links = extract_links("aabbccdd11223344", "See [[Foo Bar]] for details.");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Wiki);
        assert_eq!(links[0].target_raw, "[[Foo Bar]]");
    }

    #[test]
    fn node_id_ref() {
        let links = extract_links("aabbccdd11223344", "Link [[node:aabbccdd11223344]].");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::IdRef);
        assert!(links[0].to_id.is_some());
    }

    #[test]
    fn embed_link() {
        let links = extract_links("aabbccdd11223344", "![[image.png]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Embed);
    }

    #[test]
    fn multiple_links() {
        let links = extract_links("aabbccdd11223344", "[[A]] and [[B]] and ![[C]]");
        assert_eq!(links.len(), 3);
    }
}
