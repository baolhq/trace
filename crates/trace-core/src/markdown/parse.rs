use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag as CmTag, TagEnd,
};

use super::doc::*;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> PmDoc {
    let (frontmatter, body) = split_frontmatter(input);
    let content = parse_blocks(body);
    PmDoc { frontmatter, content }
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

fn parse_blocks(src: &str) -> Vec<Block> {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<Event<'_>> = Parser::new_ext(src, opts).collect();
    let mut ctx = Ctx { events: &events, pos: 0 };
    ctx.collect_blocks()
}

struct Ctx<'a> {
    events: &'a [Event<'a>],
    pos: usize,
}

impl<'a> Ctx<'a> {
    fn peek(&self) -> Option<&Event<'a>> {
        self.events.get(self.pos)
    }

    fn next_event(&mut self) -> Option<&Event<'a>> {
        let e = self.events.get(self.pos);
        self.pos += 1;
        e
    }

    fn collect_blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        loop {
            match self.peek() {
                None | Some(Event::End(_)) => break,
                Some(Event::Start(tag)) => {
                    let tag = tag.clone();
                    match &tag {
                        CmTag::Heading { level, .. } => {
                            let level = heading_level(*level);
                            self.next_event();
                            let content = self.collect_inlines(Some(TagEnd::Heading(
                                match level {
                                    1 => HeadingLevel::H1,
                                    2 => HeadingLevel::H2,
                                    3 => HeadingLevel::H3,
                                    4 => HeadingLevel::H4,
                                    5 => HeadingLevel::H5,
                                    _ => HeadingLevel::H6,
                                },
                            )));
                            blocks.push(Block::Heading(Heading { level, content }));
                        }
                        CmTag::Paragraph => {
                            self.next_event();
                            let content = self.collect_inlines(Some(TagEnd::Paragraph));
                            if !content.is_empty() {
                                blocks.push(Block::Paragraph(Paragraph { content }));
                            }
                        }
                        CmTag::BlockQuote(_) => {
                            self.next_event();
                            let inner = self.collect_blocks();
                            self.next_event(); // End(BlockQuote)
                            blocks.push(Block::Blockquote(Blockquote { content: inner }));
                        }
                        CmTag::List(start_num) => {
                            let start_num = *start_num;
                            self.next_event();
                            let items = self.collect_list_items();
                            self.next_event(); // End(List)
                            if let Some(start) = start_num {
                                blocks.push(Block::OrderedList(OrderedList { start, items }));
                            } else {
                                blocks.push(Block::BulletList(BulletList { items }));
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
                            if code.ends_with('\n') { code.pop(); }
                            blocks.push(Block::CodeBlock(CodeBlock { language, code }));
                        }
                        CmTag::Table(_) => {
                            self.next_event();
                            let (head, body) = self.collect_table();
                            blocks.push(Block::Table(Table { head, body }));
                        }
                        _ => { self.next_event(); }
                    }
                }
                Some(Event::Rule) => {
                    self.next_event();
                    blocks.push(Block::HorizontalRule);
                }
                _ => { self.next_event(); }
            }
        }
        blocks
    }

    fn collect_list_items(&mut self) -> Vec<ListItem> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Event::Start(CmTag::Item)) => {
                    self.next_event(); // consume Start(Item)

                    // Tight list items emit text/inlines directly (no Paragraph wrapper).
                    // Loose list items emit Start(Paragraph) first.
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
                        let inlines = self.collect_inlines(None); // stops at End(Item)
                        if inlines.is_empty() {
                            vec![]
                        } else {
                            vec![Block::Paragraph(Paragraph { content: inlines })]
                        }
                    } else {
                        self.collect_blocks()
                    };

                    // consume End(Item)
                    if matches!(self.peek(), Some(Event::End(TagEnd::Item))) {
                        self.next_event();
                    }
                    items.push(ListItem { content });
                }
                Some(Event::End(_)) | None => break,
                _ => { self.next_event(); }
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
                Some(Event::End(TagEnd::TableRow)) => { self.next_event(); }
                _ => { self.next_event(); }
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
                _ => { self.next_event(); }
            }
        }
        TableRow { cells }
    }

    /// Collect inline content until `end` tag (or End(Item)/End(_) when `end` is None).
    /// Buffers consecutive Text events so [[wiki]] spanning split events is detected.
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
                _ => { self.next_event(); }
            }
        }
        b.finish()
    }
}

// ── InlineBuilder — buffers text before scanning for wiki links / tags ────────

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
        if self.buf.is_empty() { return; }
        let text = std::mem::take(&mut self.buf);
        scan_inline_text(&mut self.result, &self.marks, &text);
    }

    fn finish(mut self) -> Vec<Inline> {
        self.flush();
        self.result
    }
}

// ── Text scanner — splits [[wiki]], #tags out of a text run ──────────────────

fn scan_inline_text(out: &mut Vec<Inline>, marks: &[Mark], text: &str) {
    let mut rest = text;
    while !rest.is_empty() {
        // Prefer [[...]] over #tag (detect whichever comes first)
        let wiki_pos = rest.find("[[");
        let tag_pos = find_tag_start(rest);

        match (wiki_pos, tag_pos) {
            (Some(w), Some(t)) if t < w => {
                // Tag comes first
                emit_text(out, marks, &rest[..t]);
                let after = &rest[t + 1..];
                let len = tag_name_len(after);
                out.push(Inline::Tag(Tag { name: after[..len].to_string() }));
                rest = &after[len..];
            }
            (Some(w), _) => {
                // Wiki link comes first (or only wiki found)
                emit_text(out, marks, &rest[..w]);
                let after = &rest[w + 2..];
                if let Some(end) = after.find("]]") {
                    let inner = &after[..end];
                    let is_id_ref = inner.starts_with("node:");
                    let target = if is_id_ref { inner[5..].to_string() } else { inner.to_string() };
                    out.push(Inline::WikiLink(WikiLink { target, is_id_ref }));
                    rest = &after[end + 2..];
                } else {
                    // Unmatched `[[` — treat as literal
                    emit_text(out, marks, "[[");
                    rest = after;
                }
            }
            (None, Some(t)) => {
                // Only a tag
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
    s.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(s.len())
}

fn emit_text(out: &mut Vec<Inline>, marks: &[Mark], text: &str) {
    if text.is_empty() { return; }
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
        HeadingLevel::H1 => 1, HeadingLevel::H2 => 2, HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4, HeadingLevel::H5 => 5, HeadingLevel::H6 => 6,
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
            "wiki link not found in: {:?}", p.content
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
        let Block::Paragraph(p) = &list.items[0].content[0] else { panic!("{:?}", list.items[0]) };
        assert!(matches!(&p.content[0], Inline::Text(t) if t.text == "alpha"));
    }
}
