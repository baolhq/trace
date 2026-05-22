use super::doc::*;

/// Canonical PmDoc → Markdown. Deterministic: same input → same output.
pub fn serialize(doc: &PmDoc) -> String {
    let mut out = String::new();

    if let Some(fm) = &doc.frontmatter {
        out.push_str("---\n");
        out.push_str(fm);
        out.push('\n');
        out.push_str("---\n\n");
    }

    let last = doc.content.len().saturating_sub(1);
    for (i, block) in doc.content.iter().enumerate() {
        serialize_block(&mut out, block, 0);
        if i < last {
            out.push('\n');
        }
    }

    out
}

fn serialize_block(out: &mut String, block: &Block, depth: usize) {
    match block {
        Block::Heading(h) => {
            let prefix = "#".repeat(h.level as usize);
            out.push_str(&prefix);
            out.push(' ');
            serialize_inlines(out, &h.content);
            out.push('\n');
        }
        Block::Paragraph(p) => {
            serialize_inlines(out, &p.content);
            out.push('\n');
        }
        Block::BulletList(list) => {
            for item in &list.items {
                serialize_list_item(out, item, depth, None);
            }
        }
        Block::OrderedList(list) => {
            for (i, item) in list.items.iter().enumerate() {
                let num = list.start + i as u64;
                serialize_list_item(out, item, depth, Some(num));
            }
        }
        Block::ListItem(item) => {
            // Direct ListItem (shouldn't appear at top level, but handle gracefully)
            serialize_list_item(out, item, depth, None);
        }
        Block::CodeBlock(cb) => {
            let lang = cb.language.as_deref().unwrap_or("");
            out.push_str("```");
            out.push_str(lang);
            out.push('\n');
            out.push_str(&cb.code);
            out.push('\n');
            out.push_str("```\n");
        }
        Block::Blockquote(bq) => {
            let mut inner = String::new();
            let last = bq.content.len().saturating_sub(1);
            for (i, b) in bq.content.iter().enumerate() {
                serialize_block(&mut inner, b, depth);
                if i < last {
                    inner.push('\n');
                }
            }
            for line in inner.lines() {
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
        }
        Block::Table(table) => {
            serialize_table(out, table);
        }
        Block::HorizontalRule => {
            out.push_str("---\n");
        }
    }
}

fn serialize_list_item(out: &mut String, item: &ListItem, depth: usize, num: Option<u64>) {
    let indent = "  ".repeat(depth);
    let marker = match num {
        Some(n) => format!("{}{}. ", indent, n),
        None => format!("{}- ", indent),
    };

    // Flatten: if the item has a single paragraph, inline it with the marker.
    // If it has nested blocks, the first paragraph is inlined and the rest are indented.
    let mut content_iter = item.content.iter().peekable();
    let mut first = true;

    while let Some(block) = content_iter.next() {
        if first {
            first = false;
            if let Block::Paragraph(p) = block {
                out.push_str(&marker);
                serialize_inlines(out, &p.content);
                out.push('\n');
                continue;
            }
        }
        // Nested block: indent it
        let mut inner = String::new();
        serialize_block(&mut inner, block, depth + 1);
        for line in inner.lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        if content_iter.peek().is_some() {
            out.push('\n');
        }
    }
}

fn serialize_table(out: &mut String, table: &Table) {
    if table.head.is_empty() && table.body.is_empty() {
        return;
    }

    let col_count = table
        .head
        .first()
        .map(|r| r.cells.len())
        .or_else(|| table.body.first().map(|r| r.cells.len()))
        .unwrap_or(0);

    // Collect all cell strings first so we can compute column widths
    let head_cells: Vec<Vec<String>> = table
        .head
        .iter()
        .map(|row| row.cells.iter().map(|c| inline_str(&c.content)).collect())
        .collect();

    let body_cells: Vec<Vec<String>> = table
        .body
        .iter()
        .map(|row| row.cells.iter().map(|c| inline_str(&c.content)).collect())
        .collect();

    let mut widths = vec![3usize; col_count]; // min width 3 for `---`
    for row in head_cells.iter().chain(body_cells.iter()) {
        for (j, cell) in row.iter().enumerate() {
            if j < widths.len() {
                widths[j] = widths[j].max(cell.len());
            }
        }
    }

    let render_row = |cells: &[String]| -> String {
        let mut s = String::from("| ");
        for (j, cell) in cells.iter().enumerate() {
            let w = widths.get(j).copied().unwrap_or(3);
            s.push_str(&format!("{:<w$} | ", cell, w = w));
        }
        s.trim_end().to_string() + "\n"
    };

    for row in &head_cells {
        out.push_str(&render_row(row));
    }

    // Separator
    out.push('|');
    for &w in &widths {
        out.push(' ');
        out.push_str(&"-".repeat(w));
        out.push_str(" |");
    }
    out.push('\n');

    for row in &body_cells {
        out.push_str(&render_row(row));
    }
}

// ── Inline serialization ──────────────────────────────────────────────────────

fn serialize_inlines(out: &mut String, inlines: &[Inline]) {
    out.push_str(&inline_str(inlines));
}

fn inline_str(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                let text = apply_marks(&t.text, &t.marks);
                s.push_str(&text);
            }
            Inline::HardBreak => s.push_str("  \n"),
            Inline::WikiLink(wl) => {
                if wl.is_id_ref {
                    s.push_str(&format!("[[node:{}]]", wl.target));
                } else {
                    s.push_str(&format!("[[{}]]", wl.target));
                }
            }
            Inline::Tag(t) => {
                s.push('#');
                s.push_str(&t.name);
            }
        }
    }
    s
}

fn apply_marks(text: &str, marks: &[Mark]) -> String {
    // Apply outermost → innermost. Order: link > strike > code > bold > italic.
    // Code mark is special: no nesting allowed inside it.
    if marks.contains(&Mark::Code) {
        return format!("`{}`", text);
    }

    let mut s = text.to_string();
    // Link wraps everything
    let link = marks.iter().find_map(|m| {
        if let Mark::Link(l) = m { Some(l) } else { None }
    });

    // Apply character-level marks before wrapping in link
    if marks.contains(&Mark::Strike) {
        s = format!("~~{}~~", s);
    }
    if marks.contains(&Mark::Bold) && marks.contains(&Mark::Italic) {
        s = format!("***{}***", s);
    } else if marks.contains(&Mark::Bold) {
        s = format!("**{}**", s);
    } else if marks.contains(&Mark::Italic) {
        s = format!("*{}*", s);
    }

    if let Some(lm) = link {
        if let Some(title) = &lm.title {
            s = format!("[{}]({} \"{}\")", s, lm.href, title);
        } else {
            s = format!("[{}]({})", s, lm.href);
        }
    }

    s
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse::parse;

    fn roundtrip(src: &str) -> String {
        serialize(&parse(src))
    }

    #[test]
    fn heading_roundtrip() {
        let src = "# Hello World\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn paragraph_roundtrip() {
        let src = "Just a paragraph.\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn bold_roundtrip() {
        let src = "**bold text**\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn italic_roundtrip() {
        let src = "*italic text*\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn strikethrough_roundtrip() {
        let src = "~~struck~~\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn code_block_roundtrip() {
        let src = "```rust\nfn main() {}\n```\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn wiki_link_roundtrip() {
        let src = "See [[My Note]] for details.\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn tag_roundtrip() {
        let src = "tagged with #rust today\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn frontmatter_preserved() {
        let src = "---\ntitle: Test\n---\n\n# Hello\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn bullet_list_roundtrip() {
        let src = "- alpha\n- beta\n- gamma\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn ordered_list_roundtrip() {
        let src = "1. first\n2. second\n3. third\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn blockquote_roundtrip() {
        let src = "> quoted text\n";
        assert_eq!(roundtrip(src), src);
    }

    #[test]
    fn horizontal_rule() {
        let src = "---\n";
        // A bare `---` with no following content is a thematic break in CM
        let doc = parse(src);
        assert!(matches!(doc.content[0], Block::HorizontalRule));
    }

    #[test]
    fn parse_serialize_parse_stable() {
        // Second parse must equal first parse (idempotent)
        let src = "---\ntitle: T\n---\n\n# Heading\n\nParagraph with **bold** and [[Link]].\n\n- item one\n- item two\n";
        let doc1 = parse(src);
        let mid = serialize(&doc1);
        let doc2 = parse(&mid);
        assert_eq!(doc1, doc2);
    }
}
