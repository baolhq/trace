use std::ops::Range;

use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag as CmTag, TagEnd,
};

use super::doc::*;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> PmDoc {
    let (frontmatter, body) = split_frontmatter(input);
    let (content, _) = parse_blocks(body);
    PmDoc { frontmatter, content }
}

/// Like `parse`, but also returns byte spans for each top-level block.
/// Spans are relative to `input` (not the body after frontmatter).
/// Use `input[span]` to extract original bytes for a block.
pub fn parse_with_spans(input: &str) -> (PmDoc, Vec<Range<usize>>) {
    let (frontmatter, body) = split_frontmatter(input);
    let body_offset = input.len() - body.len();
    let (content, spans) = parse_blocks(body);
    let adjusted: Vec<Range<usize>> = spans
        .into_iter()
        .map(|s| (s.start + body_offset)..(s.end + body_offset))
        .collect();
    (PmDoc { frontmatter, content }, adjusted)
}

// ── Frontmatter ───────────────────────────────────────────────────────────────

fn split_frontmatter(src: &str) -> (Option<String>, &str) {
    let src = src.trim_start_matches('\u{feff}');
    if !src.starts_with("---") {
        return (None, src);
    }
    let rest = &src[3..];
    let rest = match rest.strip_prefix('\n').or_else(|| rest.strip_prefix("\r\n")) {
        Some(s) => s,
        None => return (None, src),
    };
    for (pos, _) in rest.match_indices("---") {
        let at_line_start = pos == 0 || rest.as_bytes().get(pos - 1) == Some(&b'\n');
        if !at_line_start {
            continue;
        }
        let yaml = rest[..pos].trim_end_matches(['\n', '\r']).to_string();
        let after = &rest[pos + 3..];
        let after = after
            .strip_prefix('\n')
            .or_else(|| after.strip_prefix("\r\n"))
            .unwrap_or(after);
        return (Some(yaml), after);
    }
    (None, src)
}

// ── Block parsing ─────────────────────────────────────────────────────────────

/// Returns `(blocks, spans)` where spans are byte ranges relative to `src`.
/// `spans[i]` covers the source bytes for `blocks[i]`, including its trailing `\n`.
fn parse_blocks(src: &str) -> (Vec<Block>, Vec<Range<usize>>) {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<(Event<'_>, Range<usize>)> =
        Parser::new_ext(src, opts).into_offset_iter().collect();
    let mut ctx = Ctx { src, events: &events, pos: 0 };
    ctx.collect_top_level_blocks()
}

struct Ctx<'a> {
    src: &'a str,
    events: &'a [(Event<'a>, Range<usize>)],
    pos: usize,
}

impl<'a> Ctx<'a> {
    fn peek(&self) -> Option<&Event<'a>> {
        self.events.get(self.pos).map(|(e, _)| e)
    }

    fn next_event(&mut self) -> Option<&Event<'a>> {
        let e = self.events.get(self.pos).map(|(e, _)| e);
        self.pos += 1;
        e
    }

    /// Byte offset where the current (not-yet-consumed) event starts.
    fn byte_start(&self) -> usize {
        self.events.get(self.pos).map(|(_, r)| r.start).unwrap_or(self.src.len())
    }

    /// Byte offset where the last consumed event ends, extended to include
    /// one trailing `\n` if the very next byte in `src` is `\n`.
    /// This captures each block's own line terminator without the blank-line gap.
    fn last_byte_end(&self) -> usize {
        if self.pos == 0 {
            return 0;
        }
        let raw = self.events.get(self.pos - 1).map(|(_, r)| r.end).unwrap_or(0);
        if self.src.as_bytes().get(raw) == Some(&b'\n') {
            raw + 1
        } else {
            raw
        }
    }

    // ── collect_top_level_blocks: records byte spans per block ────────────────

    fn collect_top_level_blocks(&mut self) -> (Vec<Block>, Vec<Range<usize>>) {
        let mut blocks = Vec::new();
        let mut spans: Vec<Range<usize>> = Vec::new();

        loop {
            let start = self.byte_start();
            match self.next_block() {
                None => break,
                Some(block) => {
                    let end = self.last_byte_end();
                    blocks.push(block);
                    spans.push(start..end);
                }
            }
        }

        (blocks, spans)
    }

    // ── collect_blocks: recursive (no span tracking needed) ──────────────────

    fn collect_blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        while let Some(block) = self.next_block() {
            blocks.push(block);
        }
        blocks
    }

    // ── next_block: parse exactly one block, return None at boundary ──────────

    fn next_block(&mut self) -> Option<Block> {
        loop {
            match self.peek() {
                None | Some(Event::End(_)) => return None,
                Some(Event::Start(tag)) => {
                    let tag = tag.clone();
                    return Some(self.parse_block(tag));
                }
                Some(Event::Rule) => {
                    self.next_event();
                    return Some(Block::HorizontalRule);
                }
                _ => {
                    self.next_event();
                }
            }
        }
    }

    fn parse_block(&mut self, tag: CmTag<'a>) -> Block {
        match tag {
            CmTag::Heading { level, .. } => {
                let level = heading_level(level);
                self.next_event();
                let end_tag = TagEnd::Heading(match level {
                    1 => HeadingLevel::H1,
                    2 => HeadingLevel::H2,
                    3 => HeadingLevel::H3,
                    4 => HeadingLevel::H4,
                    5 => HeadingLevel::H5,
                    _ => HeadingLevel::H6,
                });
                let content = self.collect_inlines(Some(end_tag));
                Block::Heading(Heading { level, content })
            }
            CmTag::Paragraph => {
                self.next_event();
                let content = self.collect_inlines(Some(TagEnd::Paragraph));
                if content.is_empty() {
                    // Empty paragraph — skip (shouldn't normally happen, but be safe)
                    Block::Paragraph(Paragraph { content: vec![] })
                } else {
                    Block::Paragraph(Paragraph { content })
                }
            }
            CmTag::BlockQuote(_) => {
                self.next_event();
                let inner = self.collect_blocks();
                self.next_event(); // End(BlockQuote)
                Block::Blockquote(Blockquote { content: inner })
            }
            CmTag::List(start_num) => {
                self.next_event();
                let items = self.collect_list_items();
                self.next_event(); // End(List)
                if let Some(start) = start_num {
                    Block::OrderedList(OrderedList { start, items })
                } else {
                    Block::BulletList(BulletList { items })
                }
            }
            CmTag::CodeBlock(kind) => {
                let language = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let s = lang.trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    }
                    CodeBlockKind::Indented => None,
                };
                self.next_event();
                let mut code = String::new();
                loop {
                    match self.next_event() {
                        Some(Event::Text(t)) => code.push_str(t),
                        Some(Event::End(TagEnd::CodeBlock)) | None => break,
                        _ => {}
                    }
                }
                if code.ends_with('\n') {
                    code.pop();
                }
                Block::CodeBlock(CodeBlock { language, code })
            }
            CmTag::Table(_) => {
                self.next_event();
                let (head, body) = self.collect_table();
                Block::Table(Table { head, body })
            }
            _ => {
                self.next_event();
                // Unknown block type — emit empty paragraph as placeholder
                Block::Paragraph(Paragraph { content: vec![] })
            }
        }
    }

    fn collect_list_items(&mut self) -> Vec<ListItem> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Event::Start(CmTag::Item)) => {
                    self.next_event(); // consume Start(Item)

                    let is_tight = matches!(
                        self.peek(),
                        Some(Event::Text(_))
                            | Some(Event::Code(_))
                            | Some(Event::SoftBreak)
                            | Some(Event::HardBreak)
                            | Some(Event::Start(
                                CmTag::Strong
                                    | CmTag::Emphasis
                                    | CmTag::Strikethrough
                                    | CmTag::Link { .. }
                            ))
                    );

                    let content = if is_tight {
                        let inlines = self.collect_inlines(None);
                        if inlines.is_empty() {
                            vec![]
                        } else {
                            vec![Block::Paragraph(Paragraph { content: inlines })]
                        }
                    } else {
                        self.collect_blocks()
                    };

                    if matches!(self.peek(), Some(Event::End(TagEnd::Item))) {
                        self.next_event();
                    }
                    items.push(ListItem { content });
                }
                Some(Event::End(_)) | None => break,
                _ => {
                    self.next_event();
                }
            }
        }
        items
    }

    fn collect_table(&mut self) -> (Vec<TableRow>, Vec<TableRow>) {
        let mut head = Vec::new();
        let mut body = Vec::new();
        let mut in_head = false;
        loop {
            match self.peek().cloned() {
                None | Some(Event::End(TagEnd::Table)) => {
                    self.next_event();
                    break;
                }
                Some(Event::Start(CmTag::TableHead)) => {
                    self.next_event();
                    in_head = true;
                }
                Some(Event::End(TagEnd::TableHead)) => {
                    self.next_event();
                    in_head = false;
                }
                Some(Event::Start(CmTag::TableRow)) => {
                    self.next_event();
                    let row = self.collect_table_row();
                    if in_head { head.push(row) } else { body.push(row) }
                }
                Some(Event::End(TagEnd::TableRow)) => {
                    self.next_event();
                }
                _ => {
                    self.next_event();
                }
            }
        }
        (head, body)
    }

    fn collect_table_row(&mut self) -> TableRow {
        let mut cells = Vec::new();
        loop {
            match self.peek().cloned() {
                None | Some(Event::End(TagEnd::TableRow)) => {
                    self.next_event();
                    break;
                }
                Some(Event::Start(CmTag::TableCell)) => {
                    self.next_event();
                    let content = self.collect_inlines(Some(TagEnd::TableCell));
                    cells.push(TableCell { content });
                }
                _ => {
                    self.next_event();
                }
            }
        }
        TableRow { cells }
    }

    /// Collect inline content until `end` tag (or End(_) when `end` is None).
    fn collect_inlines(&mut self, end: Option<TagEnd>) -> Vec<Inline> {
        let mut b = InlineBuilder::default();
        loop {
            match self.peek().cloned() {
                None => break,
                Some(Event::End(TagEnd::Strong)) => {
                    self.next_event();
                    b.pop_mark(|m| m == &Mark::Bold);
                }
                Some(Event::End(TagEnd::Emphasis)) => {
                    self.next_event();
                    b.pop_mark(|m| m == &Mark::Italic);
                }
                Some(Event::End(TagEnd::Strikethrough)) => {
                    self.next_event();
                    b.pop_mark(|m| m == &Mark::Strike);
                }
                Some(Event::End(TagEnd::Link)) => {
                    self.next_event();
                    b.pop_mark(|m| matches!(m, Mark::Link(_)));
                }
                Some(Event::End(e)) => {
                    if end.map_or(true, |expected| e == expected) {
                        self.next_event();
                    }
                    break;
                }
                Some(Event::Text(t)) => {
                    self.next_event();
                    b.push_text(&t);
                }
                Some(Event::Code(t)) => {
                    self.next_event();
                    b.flush();
                    b.result.push(Inline::Text(Text {
                        text: t.to_string(),
                        marks: vec![Mark::Code],
                    }));
                }
                Some(Event::SoftBreak) => {
                    self.next_event();
                    b.push_text(" ");
                }
                Some(Event::HardBreak) => {
                    self.next_event();
                    b.flush();
                    b.result.push(Inline::HardBreak);
                }
                Some(Event::Start(CmTag::Strong)) => {
                    self.next_event();
                    b.push_mark(Mark::Bold);
                }
                Some(Event::Start(CmTag::Emphasis)) => {
                    self.next_event();
                    b.push_mark(Mark::Italic);
                }
                Some(Event::Start(CmTag::Strikethrough)) => {
                    self.next_event();
                    b.push_mark(Mark::Strike);
                }
                Some(Event::Start(CmTag::Link { dest_url, title, .. })) => {
                    self.next_event();
                    b.push_mark(Mark::Link(LinkMark {
                        href: dest_url.to_string(),
                        title: if title.is_empty() { None } else { Some(title.to_string()) },
                    }));
                }
                _ => {
                    self.next_event();
                }
            }
        }
        b.finish()
    }
}

// ── InlineBuilder ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct InlineBuilder {
    buf: String,
    marks: Vec<Mark>,
    result: Vec<Inline>,
}

impl InlineBuilder {
    fn push_text(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    fn push_mark(&mut self, mark: Mark) {
        self.flush();
        self.marks.push(mark);
    }

    fn pop_mark(&mut self, pred: impl Fn(&Mark) -> bool) {
        self.flush();
        if let Some(pos) = self.marks.iter().rposition(|m| pred(m)) {
            self.marks.remove(pos);
        }
    }

    fn flush(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.buf);
        scan_inline_text(&mut self.result, &self.marks, &text);
    }

    fn finish(mut self) -> Vec<Inline> {
        self.flush();
        self.result
    }
}

// ── Text scanner ──────────────────────────────────────────────────────────────

fn scan_inline_text(out: &mut Vec<Inline>, marks: &[Mark], text: &str) {
    let mut rest = text;
    while !rest.is_empty() {
        let wiki_pos = rest.find("[[");
        let tag_pos = find_tag_start(rest);

        match (wiki_pos, tag_pos) {
            (Some(w), Some(t)) if t < w => {
                emit_text(out, marks, &rest[..t]);
                let after = &rest[t + 1..];
                let len = tag_name_len(after);
                out.push(Inline::Tag(Tag { name: after[..len].to_string() }));
                rest = &after[len..];
            }
            (Some(w), _) => {
                emit_text(out, marks, &rest[..w]);
                let after = &rest[w + 2..];
                if let Some(end) = after.find("]]") {
                    let inner = &after[..end];
                    let is_id_ref = inner.starts_with("node:");
                    let target =
                        if is_id_ref { inner[5..].to_string() } else { inner.to_string() };
                    out.push(Inline::WikiLink(WikiLink { target, is_id_ref }));
                    rest = &after[end + 2..];
                } else {
                    emit_text(out, marks, "[[");
                    rest = after;
                }
            }
            (None, Some(t)) => {
                emit_text(out, marks, &rest[..t]);
                let after = &rest[t + 1..];
                let len = tag_name_len(after);
                out.push(Inline::Tag(Tag { name: after[..len].to_string() }));
                rest = &after[len..];
            }
            (None, None) => {
                emit_text(out, marks, rest);
                break;
            }
        }
    }
}

fn find_tag_start(s: &str) -> Option<usize> {
    let mut prev: Option<char> = None;
    for (i, c) in s.char_indices() {
        if c == '#' {
            let at_boundary = prev.map_or(true, |p| p.is_whitespace());
            if at_boundary {
                let after = &s[i + 1..];
                if after.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
                    return Some(i);
                }
            }
        }
        prev = Some(c);
    }
    None
}

fn tag_name_len(s: &str) -> usize {
    s.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-').unwrap_or(s.len())
}

fn emit_text(out: &mut Vec<Inline>, marks: &[Mark], text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Inline::Text(prev)) = out.last_mut() {
        if prev.marks == marks {
            prev.text.push_str(text);
            return;
        }
    }
    out.push(Inline::Text(Text { text: text.to_string(), marks: marks.to_vec() }));
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading() {
        let doc = parse("# Hello");
        assert!(matches!(&doc.content[0], Block::Heading(h) if h.level == 1));
    }

    #[test]
    fn bold_italic() {
        let doc = parse("**bold** and *italic*");
        let Block::Paragraph(p) = &doc.content[0] else { panic!() };
        let Inline::Text(t) = &p.content[0] else { panic!("{:?}", p.content) };
        assert_eq!(t.marks, vec![Mark::Bold]);
        let Inline::Text(t2) = &p.content[2] else { panic!() };
        assert_eq!(t2.marks, vec![Mark::Italic]);
    }

    #[test]
    fn wiki_link() {
        let doc = parse("See [[My Note]] for details.");
        let Block::Paragraph(p) = &doc.content[0] else { panic!() };
        assert!(
            p.content.iter().any(|i| matches!(i, Inline::WikiLink(w) if w.target == "My Note")),
            "wiki link not found in: {:?}",
            p.content
        );
    }

    #[test]
    fn tag_inline() {
        let doc = parse("tagged with #rust today");
        let Block::Paragraph(p) = &doc.content[0] else { panic!() };
        assert!(p.content.iter().any(|i| matches!(i, Inline::Tag(t) if t.name == "rust")));
    }

    #[test]
    fn frontmatter_preserved() {
        let src = "---\ntitle: Test\n---\n# Hello";
        let doc = parse(src);
        assert_eq!(doc.frontmatter.as_deref(), Some("title: Test"));
        assert!(matches!(doc.content[0], Block::Heading(_)));
    }

    #[test]
    fn code_block() {
        let src = "```rust\nfn main() {}\n```";
        let doc = parse(src);
        let Block::CodeBlock(cb) = &doc.content[0] else { panic!() };
        assert_eq!(cb.language.as_deref(), Some("rust"));
        assert_eq!(cb.code, "fn main() {}");
    }

    #[test]
    fn table() {
        let src = "| A | B |\n|---|---|\n| 1 | 2 |";
        assert!(matches!(parse(src).content[0], Block::Table(_)));
    }

    #[test]
    fn tight_list() {
        let src = "- alpha\n- beta\n";
        let doc = parse(src);
        let Block::BulletList(list) = &doc.content[0] else { panic!("{:?}", doc.content) };
        assert_eq!(list.items.len(), 2);
        let Block::Paragraph(p) = &list.items[0].content[0] else {
            panic!("{:?}", list.items[0])
        };
        assert!(matches!(&p.content[0], Inline::Text(t) if t.text == "alpha"));
    }

    #[test]
    fn spans_cover_blocks() {
        let src = "# Heading\n\nParagraph.\n";
        let (doc, spans) = parse_with_spans(src);
        assert_eq!(doc.content.len(), 2);
        assert_eq!(spans.len(), 2);
        // Each span must be non-empty and within bounds
        for span in &spans {
            assert!(!span.is_empty());
            assert!(span.end <= src.len());
        }
        // Heading span should contain "# Heading"
        assert!(src[spans[0].clone()].contains("Heading"));
        // Paragraph span should contain "Paragraph."
        assert!(src[spans[1].clone()].contains("Paragraph"));
    }

    #[test]
    fn spans_with_frontmatter() {
        let src = "---\ntitle: T\n---\n\n# Hello\n\nWorld\n";
        let (doc, spans) = parse_with_spans(src);
        assert_eq!(doc.content.len(), 2);
        assert_eq!(spans.len(), 2);
        assert!(src[spans[0].clone()].contains("Hello"));
        assert!(src[spans[1].clone()].contains("World"));
    }
}
