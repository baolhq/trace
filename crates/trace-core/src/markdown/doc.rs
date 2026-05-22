use serde::{Deserialize, Serialize};

/// Root document. `frontmatter` holds raw YAML bytes preserved verbatim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PmDoc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<String>,
    pub content: Vec<Block>,
}

// ── Block nodes ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    Heading(Heading),
    Paragraph(Paragraph),
    BulletList(BulletList),
    OrderedList(OrderedList),
    ListItem(ListItem),
    CodeBlock(CodeBlock),
    Blockquote(Blockquote),
    Table(Table),
    HorizontalRule,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8, // 1–6
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulletList {
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderedList {
    pub start: u64,
    pub items: Vec<ListItem>,
}

/// A list item may contain blocks (for nested lists) or just inlines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub content: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blockquote {
    pub content: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub head: Vec<TableRow>,
    pub body: Vec<TableRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

// ── Inline nodes ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Inline {
    Text(Text),
    HardBreak,
    WikiLink(WikiLink),
    Tag(Tag),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
}

/// `[[target]]` or `[[node:id]]`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WikiLink {
    pub target: String,
    /// True when the target is a stable node ID (`[[node:id]]`).
    #[serde(default)]
    pub is_id_ref: bool,
}

/// `#tagname`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

// ── Marks ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Mark {
    Bold,
    Italic,
    Strike,
    Code,
    Link(LinkMark),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkMark {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
