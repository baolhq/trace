// Mirror of trace-core::markdown::doc — must stay in sync with Rust serde output.

export interface PmDoc {
  frontmatter?: string;
  content: Block[];
}

export type Block =
  | { type: "Heading"; level: number; content: Inline[] }
  | { type: "Paragraph"; content: Inline[] }
  | { type: "BulletList"; items: ListItem[] }
  | { type: "OrderedList"; start: number; items: ListItem[] }
  | { type: "ListItem"; content: Block[] }
  | { type: "CodeBlock"; language?: string; code: string }
  | { type: "Blockquote"; content: Block[] }
  | { type: "Table"; head: TableRow[]; body: TableRow[] }
  | { type: "HorizontalRule" };

export interface ListItem {
  content: Block[];
}

export interface TableRow {
  cells: TableCell[];
}

export interface TableCell {
  content: Inline[];
}

export type Inline =
  | { type: "Text"; text: string; marks?: Mark[] }
  | { type: "HardBreak" }
  | { type: "WikiLink"; target: string; is_id_ref?: boolean }
  | { type: "Tag"; name: string };

export type Mark =
  | { type: "Bold" }
  | { type: "Italic" }
  | { type: "Strike" }
  | { type: "Code" }
  | { type: "Link"; href: string; title?: string };

/**
 * Convert a PmDoc (from Rust) into a TipTap-compatible JSON document.
 * TipTap uses ProseMirror's JSON format which differs slightly from our
 * internal representation (different node name casing, attrs structure).
 */
export function pmDocToTipTap(doc: PmDoc): object {
  return {
    type: "doc",
    content: doc.content.map(blockToTt),
  };
}

function blockToTt(block: Block): object {
  switch (block.type) {
    case "Heading":
      return {
        type: "heading",
        attrs: { level: block.level },
        content: block.content.map(inlineToTt),
      };
    case "Paragraph":
      return {
        type: "paragraph",
        content: block.content.length ? block.content.map(inlineToTt) : undefined,
      };
    case "BulletList":
      return {
        type: "bulletList",
        content: block.items.map((item) => listItemToTt(item)),
      };
    case "OrderedList":
      return {
        type: "orderedList",
        attrs: { start: block.start },
        content: block.items.map((item) => listItemToTt(item)),
      };
    case "ListItem":
      return {
        type: "listItem",
        content: block.content.map(blockToTt),
      };
    case "CodeBlock":
      return {
        type: "codeBlock",
        attrs: { language: block.language ?? null },
        content: [{ type: "text", text: block.code }],
      };
    case "Blockquote":
      return {
        type: "blockquote",
        content: block.content.map(blockToTt),
      };
    case "Table":
      return {
        type: "table",
        content: [
          ...block.head.map((row) => tableRowToTt(row, true)),
          ...block.body.map((row) => tableRowToTt(row, false)),
        ],
      };
    case "HorizontalRule":
      return { type: "horizontalRule" };
  }
}

function listItemToTt(item: ListItem): object {
  return {
    type: "listItem",
    content: item.content.map(blockToTt),
  };
}

function tableRowToTt(row: TableRow, isHeader: boolean): object {
  return {
    type: "tableRow",
    content: row.cells.map((cell) => ({
      type: isHeader ? "tableHeader" : "tableCell",
      content: cell.content.length
        ? [{ type: "paragraph", content: cell.content.map(inlineToTt) }]
        : [{ type: "paragraph" }],
    })),
  };
}

function inlineToTt(inline: Inline): object {
  switch (inline.type) {
    case "Text":
      return {
        type: "text",
        text: inline.text,
        marks: inline.marks?.map(markToTt),
      };
    case "HardBreak":
      return { type: "hardBreak" };
    case "WikiLink":
      return {
        type: "wikiLink",
        attrs: { target: inline.target, isIdRef: inline.is_id_ref ?? false },
      };
    case "Tag":
      return { type: "tag", attrs: { name: inline.name } };
  }
}

function markToTt(mark: Mark): object {
  switch (mark.type) {
    case "Bold":   return { type: "bold" };
    case "Italic": return { type: "italic" };
    case "Strike": return { type: "strike" };
    case "Code":   return { type: "code" };
    case "Link":   return { type: "link", attrs: { href: mark.href, title: mark.title ?? null } };
  }
}
