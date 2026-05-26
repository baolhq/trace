pub mod doc;
pub mod links;
pub mod parse;
pub mod serialize;

use std::collections::HashSet;

/// Collect every unique `#tag` name from a parsed document.
pub fn extract_tags(doc: &doc::PmDoc) -> Vec<String> {
    let mut set = HashSet::new();
    collect_blocks(&doc.content, &mut set);
    let mut tags: Vec<String> = set.into_iter().collect();
    tags.sort();
    tags
}

fn collect_blocks(blocks: &[doc::Block], out: &mut HashSet<String>) {
    use doc::Block;
    for b in blocks {
        match b {
            Block::Paragraph(p) => collect_inlines(&p.content, out),
            Block::Heading(h) => collect_inlines(&h.content, out),
            Block::BulletList(l) => l.items.iter().for_each(|i| collect_blocks(&i.content, out)),
            Block::OrderedList(l) => l.items.iter().for_each(|i| collect_blocks(&i.content, out)),
            Block::ListItem(i) => collect_blocks(&i.content, out),
            Block::Blockquote(b) => collect_blocks(&b.content, out),
            Block::Table(t) => t
                .head
                .iter()
                .chain(t.body.iter())
                .flat_map(|r| &r.cells)
                .for_each(|c| collect_inlines(&c.content, out)),
            Block::CodeBlock(_) | Block::HorizontalRule => {}
        }
    }
}

fn collect_inlines(inlines: &[doc::Inline], out: &mut HashSet<String>) {
    for i in inlines {
        if let doc::Inline::Tag(t) = i {
            out.insert(t.name.clone());
        }
    }
}

/// Derives the node title from its relative path (filename stem).
pub fn title_from_path(rel_path: &str) -> String {
    std::path::Path::new(rel_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| rel_path.to_string())
}
