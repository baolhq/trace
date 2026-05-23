// Mirror of trace-core::markdown::doc — must stay in sync with Rust serde output.
// Rust uses #[serde(tag = "type", rename_all = "camelCase")] so all type tags are camelCase.

export interface PmDoc {
    frontmatter?: string;
    content: Block[];
}

export type Block =
    | { type: "heading"; level: number; content: Inline[] }
    | { type: "paragraph"; content: Inline[] }
    | { type: "bulletList"; items: ListItem[] }
    | { type: "orderedList"; start: number; items: ListItem[] }
    | { type: "listItem"; content: Block[] }
    | { type: "codeBlock"; language?: string; code: string }
    | { type: "blockquote"; content: Block[] }
    | { type: "table"; head: TableRow[]; body: TableRow[] }
    | { type: "horizontalRule" };

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
    | { type: "text"; text: string; marks?: Mark[] }
    | { type: "hardBreak" }
    | { type: "wikiLink"; target: string; is_id_ref?: boolean }
    | { type: "tag"; name: string };

export type Mark =
    | { type: "bold" }
    | { type: "italic" }
    | { type: "strike" }
    | { type: "code" }
    | { type: "link"; href: string; title?: string };

// ── PmDoc → TipTap JSON ────────────────────────────────────────────────────────

export function pmDocToTipTap(doc: PmDoc): object {
    return {
        type: "doc",
        content: doc.content.map(blockToTt),
    };
}

function blockToTt(block: Block): object {
    switch (block.type) {
        case "heading":
            return {
                type: "heading",
                attrs: {level: block.level},
                content: block.content.map(inlineToTt),
            };
        case "paragraph":
            return {
                type: "paragraph",
                content: block.content.length ? block.content.map(inlineToTt) : undefined,
            };
        case "bulletList":
            return {
                type: "bulletList",
                content: block.items.map(listItemToTt),
            };
        case "orderedList":
            return {
                type: "orderedList",
                attrs: {start: block.start},
                content: block.items.map(listItemToTt),
            };
        case "listItem":
            return {
                type: "listItem",
                content: block.content.map(blockToTt),
            };
        case "codeBlock":
            return {
                type: "codeBlock",
                attrs: {language: block.language ?? null},
                content: [{type: "text", text: block.code}],
            };
        case "blockquote":
            return {
                type: "blockquote",
                content: block.content.map(blockToTt),
            };
        case "table":
            return {
                type: "table",
                content: [
                    ...block.head.map((row) => tableRowToTt(row, true)),
                    ...block.body.map((row) => tableRowToTt(row, false)),
                ],
            };
        case "horizontalRule":
            return {type: "horizontalRule"};
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
                ? [{type: "paragraph", content: cell.content.map(inlineToTt)}]
                : [{type: "paragraph"}],
        })),
    };
}

function inlineToTt(inline: Inline): object {
    switch (inline.type) {
        case "text":
            return {
                type: "text",
                text: inline.text,
                marks: inline.marks?.map(markToTt),
            };
        case "hardBreak":
            return {type: "hardBreak"};
        case "wikiLink":
            return {
                type: "wikiLink",
                attrs: {target: inline.target, isIdRef: inline.is_id_ref ?? false},
            };
        case "tag":
            return {type: "tag", attrs: {name: inline.name}};
    }
}

function markToTt(mark: Mark): object {
    switch (mark.type) {
        case "bold":
            return {type: "bold"};
        case "italic":
            return {type: "italic"};
        case "strike":
            return {type: "strike"};
        case "code":
            return {type: "code"};
        case "link":
            return {type: "link", attrs: {href: mark.href, title: mark.title ?? null}};
    }
}

// ── TipTap JSON → PmDoc ────────────────────────────────────────────────────────

/**
 * Convert TipTap's editor.getJSON() back to the PmDoc format Rust expects.
 * `frontmatter` is preserved from the original doc — it is not editable in TipTap.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function tipTapToPmDoc(ttJson: any, frontmatter?: string): PmDoc {
    const content: Block[] = (ttJson.content ?? [])
        .map(ttBlockToPm)
        .filter(notNull);
    return frontmatter !== undefined ? {frontmatter, content} : {content};
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function ttBlockToPm(node: any): Block | null {
    switch (node.type) {
        case "heading":
            return {
                type: "heading",
                level: node.attrs?.level ?? 1,
                content: (node.content ?? []).map(ttInlineToPm).filter(notNull),
            };
        case "paragraph":
            return {
                type: "paragraph",
                content: (node.content ?? []).map(ttInlineToPm).filter(notNull),
            };
        case "bulletList":
            return {
                type: "bulletList",
                items: (node.content ?? []).map(ttListItemToPm).filter(notNull),
            };
        case "orderedList":
            return {
                type: "orderedList",
                start: node.attrs?.start ?? 1,
                items: (node.content ?? []).map(ttListItemToPm).filter(notNull),
            };
        case "listItem":
            return {
                type: "listItem",
                content: (node.content ?? []).map(ttBlockToPm).filter(notNull),
            };
        case "codeBlock":
            return {
                type: "codeBlock",
                language: node.attrs?.language ?? undefined,
                code: node.content?.[0]?.text ?? "",
            };
        case "blockquote":
            return {
                type: "blockquote",
                content: (node.content ?? []).map(ttBlockToPm).filter(notNull),
            };
        case "table": {
            const head: TableRow[] = [];
            const body: TableRow[] = [];
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            for (const row of node.content ?? []) {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                const isHeader = (row.content ?? []).some((c: any) => c.type === "tableHeader");
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                const cells: TableCell[] = (row.content ?? []).map((cell: any) => {
                    const para = cell.content?.[0];
                    const inlines: Inline[] = (para?.content ?? [])
                        .map(ttInlineToPm)
                        .filter(notNull);
                    return {content: inlines};
                });
                if (isHeader) {
                    head.push({cells});
                } else {
                    body.push({cells});
                }
            }
            return {type: "table", head, body};
        }
        case "horizontalRule":
            return {type: "horizontalRule"};
        default:
            return null;
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function ttListItemToPm(node: any): ListItem | null {
    if (node.type !== "listItem") return null;
    return {content: (node.content ?? []).map(ttBlockToPm).filter(notNull)};
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function ttInlineToPm(node: any): Inline | null {
    switch (node.type) {
        case "text":
            return {
                type: "text",
                text: node.text ?? "",
                marks: (node.marks ?? []).map(ttMarkToPm).filter(notNull),
            };
        case "hardBreak":
            return {type: "hardBreak"};
        case "wikiLink":
            return {
                type: "wikiLink",
                target: node.attrs?.target ?? "",
                is_id_ref: node.attrs?.isIdRef ?? false,
            };
        case "tag":
            return {type: "tag", name: node.attrs?.name ?? ""};
        default:
            return null;
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function ttMarkToPm(mark: any): Mark | null {
    switch (mark.type) {
        case "bold":
            return {type: "bold"};
        case "italic":
            return {type: "italic"};
        case "strike":
            return {type: "strike"};
        case "code":
            return {type: "code"};
        case "link":
            return {type: "link", href: mark.attrs?.href ?? "", title: mark.attrs?.title ?? undefined};
        default:
            return null;
    }
}

function notNull<T>(v: T | null): v is T {
    return v !== null;
}
