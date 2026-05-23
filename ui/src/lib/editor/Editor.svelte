<script lang="ts">
    import {onMount, onDestroy} from "svelte";
    import {Editor} from "@tiptap/core";
    import StarterKit from "@tiptap/starter-kit";
    import {Table, TableRow, TableHeader, TableCell} from "@tiptap/extension-table";
    import Strike from "@tiptap/extension-strike";
    import {WikiLink} from "./extensions/WikiLink";
    import {Tag} from "./extensions/Tag";
    import {pmDocToTipTap, type PmDoc} from "./doc";

    interface NodeSummary {
        id: string;
        title: string
    }

    interface Props {
        doc: PmDoc;
        nodeId: string;
        onSave: (doc: object) => void;
        nodes?: NodeSummary[];
        tags?: string[];
    }

    let {doc, nodeId, onSave, nodes = [], tags = []}: Props = $props();

    let container: HTMLDivElement;
    let editor: Editor | null = null;
    let saveTimer: ReturnType<typeof setTimeout> | null = null;

    const AUTOSAVE_DELAY = 400;

    // ── Suggestion state ─────────────────────────────────────────────────────────
    interface SuggItem {
        label: string;
        id?: string
    }

    let sugg = $state({
        active: false,
        mode: "wiki" as "wiki" | "tag",
        query: "",
        from: 0,
        left: 0,
        top: 0,
        index: 0,
    });

    let filteredItems = $derived((): SuggItem[] => {
        if (!sugg.active) return [];
        const q = sugg.query.toLowerCase();
        if (sugg.mode === "wiki") {
            return nodes
                .filter((n) => n.title.toLowerCase().includes(q))
                .slice(0, 8)
                .map((n) => ({label: n.title, id: n.id}));
        } else {
            return tags
                .filter((t) => t.toLowerCase().includes(q))
                .slice(0, 8)
                .map((t) => ({label: t}));
        }
    });

    // ── Editor setup ─────────────────────────────────────────────────────────────
    function buildEditor(element: HTMLElement, initialDoc: PmDoc): Editor {
        return new Editor({
            element,
            extensions: [
                StarterKit.configure({
                    strike: false,
                    heading: {levels: [1, 2, 3, 4, 5, 6]},
                }),
                Strike,
                Table.configure({resizable: false}),
                TableRow,
                TableHeader,
                TableCell,
                WikiLink,
                Tag,
            ],
            content: pmDocToTipTap(initialDoc),
            onUpdate: ({editor: ed}) => {
                scheduleSave();
                refreshSuggestion(ed);
            },
            onSelectionUpdate: ({editor: ed}) => {
                refreshSuggestion(ed);
            },
        });
    }

    function scheduleSave() {
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(flushSave, AUTOSAVE_DELAY);
    }

    function flushSave() {
        if (saveTimer) {
            clearTimeout(saveTimer);
            saveTimer = null;
        }
        if (!editor) return;
        onSave(editor.getJSON());
    }

    // ── Suggestion logic ──────────────────────────────────────────────────────────
    function refreshSuggestion(ed: Editor) {
        const {selection} = ed.state;
        if (!selection.empty) {
            closeSuggestion();
            return;
        }

        const anchor = selection.$from;
        const textBefore = anchor.parent.textContent.slice(0, anchor.parentOffset);

        // WikiLink: [[ ... (no closing ]])
        const wikiMatch = textBefore.match(/\[\[([^\]]*)$/);
        if (wikiMatch) {
            const coords = ed.view.coordsAtPos(anchor.pos);
            sugg = {
                active: true,
                mode: "wiki",
                query: wikiMatch[1],
                from: anchor.pos - wikiMatch[0].length,
                left: coords.left,
                top: coords.bottom + 4,
                index: 0,
            };
            return;
        }

        // Tag: #word (must be at start of text or preceded by whitespace)
        const tagMatch = textBefore.match(/#([\w-]*)$/);
        if (tagMatch) {
            const posBeforeHash = textBefore.length - tagMatch[0].length;
            const prevChar = posBeforeHash > 0 ? textBefore[posBeforeHash - 1] : "";
            if (prevChar === "" || prevChar === " " || prevChar === "\t") {
                const coords = ed.view.coordsAtPos(anchor.pos);
                sugg = {
                    active: true,
                    mode: "tag",
                    query: tagMatch[1],
                    from: anchor.pos - tagMatch[0].length,
                    left: coords.left,
                    top: coords.bottom + 4,
                    index: 0,
                };
                return;
            }
        }

        closeSuggestion();
    }

    function closeSuggestion() {
        if (sugg.active) sugg = {...sugg, active: false};
    }

    function applySuggestion(item: SuggItem) {
        if (!editor || !sugg.active) return;
        const to = editor.state.selection.$from.pos;
        const content =
            sugg.mode === "wiki"
                ? {type: "wikiLink", attrs: {target: item.label, isIdRef: false}}
                : {type: "tag", attrs: {name: item.label}};
        editor
            .chain()
            .focus()
            .deleteRange({from: sugg.from, to})
            .insertContentAt(sugg.from, content)
            .run();
        closeSuggestion();
    }

    // ── Keyboard interception (capture phase fires before ProseMirror) ────────────
    function onContainerKeydown(e: KeyboardEvent) {
        const items = filteredItems();
        if (!sugg.active || items.length === 0) return;
        if (e.key === "ArrowDown") {
            e.preventDefault();
            e.stopPropagation();
            sugg.index = Math.min(sugg.index + 1, items.length - 1);
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            e.stopPropagation();
            sugg.index = Math.max(sugg.index - 1, 0);
        } else if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            applySuggestion(items[sugg.index]);
        } else if (e.key === "Escape") {
            e.stopPropagation();
            closeSuggestion();
        }
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────────
    onMount(() => {
        editor = buildEditor(container, doc);
        // Capture phase on the container fires before the ProseMirror handler on
        // the inner contenteditable, letting us intercept arrow/enter/escape.
        container.addEventListener("keydown", onContainerKeydown, true);
    });

    onDestroy(() => {
        container?.removeEventListener("keydown", onContainerKeydown, true);
        flushSave();
        editor?.destroy();
        editor = null;
    });

    // Reload editor content when the doc prop changes (node switch).
    // Always evaluate pmDocToTipTap(doc) first so Svelte 5 tracks `doc` as a
    // reactive dependency — without this, the short-circuit `editor && doc`
    // would skip the read on the initial run (editor is null then) and Svelte
    // would never re-run the effect when doc changes later.
    $effect(() => {
        const content = pmDocToTipTap(doc);
        if (editor) {
            if (saveTimer) {
                clearTimeout(saveTimer);
                saveTimer = null;
            }
            // Suppress onUpdate so setContent doesn't trigger a spurious autosave.
            editor.commands.setContent(content, {emitUpdate: false});
        }
    });
</script>

<div class="editor-wrap">
    <div bind:this={container} class="editor-content"></div>
</div>

<!-- Suggestion dropdown — rendered at fixed viewport coords of the cursor -->
{#if sugg.active && filteredItems().length > 0}
    <div
            class="suggestion-popup"
            style="left: {sugg.left}px; top: {sugg.top}px"
            role="listbox"
            aria-label={sugg.mode === "wiki" ? "Note suggestions" : "Tag suggestions"}
    >
        {#each filteredItems() as item, i (item.label)}
            <!-- onmousedown + preventDefault keeps editor focus when clicking -->
            <button
                    class="suggestion-item"
                    class:selected={i === sugg.index}
                    class:tag-item={sugg.mode === "tag"}
                    role="option"
                    aria-selected={i === sugg.index}
                    onmousedown={(e) => { e.preventDefault(); applySuggestion(item); }}
            >
                {#if sugg.mode === "tag"}<span class="tag-prefix">#</span>{/if}{item.label}
            </button>
        {/each}
    </div>
{/if}

<style>
    .editor-wrap {
        width: 100%;
        height: 100%;
        overflow-y: auto;
    }

    .editor-content {
        max-width: 680px;
        margin: 0 auto;
        padding: 2rem 1.5rem 6rem;
        min-height: 100%;
    }

    :global(.editor-content .ProseMirror) {
        outline: none;
        font-size: 0.95rem;
        line-height: 1.7;
        color: #1a1a1a;
    }

    :global(.editor-content .ProseMirror h1) {
        font-size: 1.75rem;
        font-weight: 700;
        margin: 1.5rem 0 0.5rem;
    }

    :global(.editor-content .ProseMirror h2) {
        font-size: 1.4rem;
        font-weight: 600;
        margin: 1.25rem 0 0.4rem;
    }

    :global(.editor-content .ProseMirror h3) {
        font-size: 1.15rem;
        font-weight: 600;
        margin: 1rem 0 0.35rem;
    }

    :global(.editor-content .ProseMirror p) {
        margin: 0 0 0.75rem;
    }

    :global(.editor-content .ProseMirror ul),
    :global(.editor-content .ProseMirror ol) {
        padding-left: 1.5rem;
        margin: 0 0 0.75rem;
    }

    :global(.editor-content .ProseMirror li) {
        margin-bottom: 0.2rem;
    }

    :global(.editor-content .ProseMirror code) {
        font-family: ui-monospace, monospace;
        font-size: 0.875em;
        background: #f3f3f3;
        border-radius: 3px;
        padding: 0.1em 0.3em;
    }

    :global(.editor-content .ProseMirror pre) {
        background: #f6f6f6;
        border-radius: 6px;
        padding: 1rem;
        overflow-x: auto;
        margin: 0 0 1rem;
    }

    :global(.editor-content .ProseMirror pre code) {
        background: none;
        padding: 0;
        font-size: 0.875rem;
    }

    :global(.editor-content .ProseMirror blockquote) {
        border-left: 3px solid #ddd;
        margin: 0 0 0.75rem;
        padding-left: 1rem;
        color: #555;
    }

    :global(.editor-content .ProseMirror hr) {
        border: none;
        border-top: 1px solid #e0e0e0;
        margin: 1.5rem 0;
    }

    :global(.editor-content .wiki-link) {
        display: inline-block;
        background: #eef2ff;
        color: #4361ee;
        border-radius: 4px;
        padding: 0 0.3em;
        font-size: 0.9em;
        cursor: pointer;
        user-select: none;
    }

    :global(.editor-content .tag) {
        color: #7c3aed;
        font-weight: 500;
        cursor: pointer;
        user-select: none;
    }

    :global(.editor-content .ProseMirror table) {
        border-collapse: collapse;
        width: 100%;
        margin-bottom: 1rem;
    }

    :global(.editor-content .ProseMirror th),
    :global(.editor-content .ProseMirror td) {
        border: 1px solid #ddd;
        padding: 0.4rem 0.6rem;
        text-align: left;
    }

    :global(.editor-content .ProseMirror th) {
        background: #f9f9f9;
        font-weight: 600;
    }

    /* ── Suggestion dropdown ── */
    .suggestion-popup {
        position: fixed;
        z-index: 200;
        min-width: 200px;
        max-width: 320px;
        background: #fff;
        border: 1px solid #ddd;
        border-radius: 6px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
        overflow: hidden;
    }

    .suggestion-item {
        display: block;
        width: 100%;
        padding: 0.45rem 0.75rem;
        font-size: 0.875rem;
        text-align: left;
        background: none;
        border: none;
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        color: #1a1a1a;
    }

    .suggestion-item.selected,
    .suggestion-item:hover {
        background: #eef2ff;
        color: #2d3fe6;
    }

    .tag-prefix {
        color: #7c3aed;
        font-weight: 500;
    }

    .suggestion-item.tag-item.selected .tag-prefix,
    .suggestion-item.tag-item:hover .tag-prefix {
        color: #2d3fe6;
    }
</style>
