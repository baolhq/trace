<script lang="ts">
    import {onMount, onDestroy, untrack} from "svelte";
    import {invoke} from "@tauri-apps/api/core";
    import {Editor} from "@tiptap/core";
    import StarterKit from "@tiptap/starter-kit";
    import {Table, TableRow, TableHeader, TableCell} from "@tiptap/extension-table";
    import Strike from "@tiptap/extension-strike";
    import {WikiLink} from "./extensions/WikiLink";
    import {Tag} from "./extensions/Tag";
    import {pmDocToTipTap, type PmDoc} from "./doc";
    import {editorLRU} from "./EditorLRU";

    interface Props {
        nodeId: string;
        doc: PmDoc;
        onSave: (doc: object) => void;
    }

    let {nodeId, doc, onSave}: Props = $props();

    let prevNodeId = untrack(() => nodeId);
    let isSwapping = false;

    let container: HTMLDivElement;
    let editor: Editor | null = null;
    let saveTimer: ReturnType<typeof setTimeout> | null = null;

    const AUTOSAVE_DELAY = 400;

    // ── Suggestion state ─────────────────────────────────────────────────────────
    interface SuggItem {
        label: string;
        id?: string;
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

    // FST-backed suggestions fetched from backend
    let suggItems: SuggItem[] = $state([]);
    let suggFetchTimer: ReturnType<typeof setTimeout> | null = null;

    function fetchSuggestions(mode: "wiki" | "tag", query: string) {
        if (suggFetchTimer) clearTimeout(suggFetchTimer);
        suggFetchTimer = setTimeout(async () => {
            if (!sugg.active) return;
            if (mode === "wiki") {
                const results = await invoke<{ id: string; title: string }[]>(
                    "suggest_nodes",
                    {prefix: query},
                );
                suggItems = results.map((r) => ({label: r.title, id: r.id}));
            } else {
                const results = await invoke<string[]>("suggest_tags", {prefix: query});
                suggItems = results.map((t) => ({label: t}));
            }
            sugg.index = 0;
        }, 60);
    }

    // ── Editor setup ─────────────────────────────────────────────────────────────
    function buildEditor(element: HTMLElement, initialDoc: PmDoc): Editor {
        const ed = new Editor({
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
        });
        ed.on("update", ({editor: e}) => {
            if (!isSwapping) scheduleSave();
            refreshSuggestion(e);
        });
        ed.on("selectionUpdate", ({editor: e}) => {
            refreshSuggestion(e);
        });
        return ed;
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

    // ── Note switching ────────────────────────────────────────────────────────────
    $effect(() => {
        const newId = nodeId; // outside untrack → reactive on nodeId
        untrack(() => {
            if (!editor || newId === prevNodeId) return;

            editorLRU.put(prevNodeId, {
                json: editor.getJSON(),
                anchor: editor.state.selection.anchor,
                head: editor.state.selection.head,
                scrollTop: container?.scrollTop ?? 0,
            });

            flushSave();
            prevNodeId = newId;
            closeSuggestion();

            const cached = editorLRU.get(newId);
            const content = cached ? cached.json : pmDocToTipTap(doc);
            isSwapping = true;
            editor.commands.setContent(content, {emitUpdate: false});
            isSwapping = false;

            if (cached) {
                try {
                    editor.commands.setTextSelection({from: cached.anchor, to: cached.head});
                } catch {
                }
                container.scrollTop = cached.scrollTop;
            } else {
                editor.commands.setTextSelection(1);
                container.scrollTop = 0;
            }
        });
    });

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
            const query = wikiMatch[1];
            sugg = {
                active: true,
                mode: "wiki",
                query,
                from: anchor.pos - wikiMatch[0].length,
                left: coords.left,
                top: coords.bottom + 4,
                index: 0,
            };
            fetchSuggestions("wiki", query);
            return;
        }

        // Tag: #word (must be at start of text or preceded by whitespace)
        const tagMatch = textBefore.match(/#([\w-]*)$/);
        if (tagMatch) {
            const posBeforeHash = textBefore.length - tagMatch[0].length;
            const prevChar = posBeforeHash > 0 ? textBefore[posBeforeHash - 1] : "";
            if (prevChar === "" || prevChar === " " || prevChar === "\t") {
                const coords = ed.view.coordsAtPos(anchor.pos);
                const query = tagMatch[1];
                sugg = {
                    active: true,
                    mode: "tag",
                    query,
                    from: anchor.pos - tagMatch[0].length,
                    left: coords.left,
                    top: coords.bottom + 4,
                    index: 0,
                };
                fetchSuggestions("tag", query);
                return;
            }
        }

        closeSuggestion();
    }

    function closeSuggestion() {
        if (sugg.active) {
            sugg = {...sugg, active: false};
            suggItems = [];
        }
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
        if (!sugg.active || suggItems.length === 0) return;
        if (e.key === "ArrowDown") {
            e.preventDefault();
            e.stopPropagation();
            sugg.index = Math.min(sugg.index + 1, suggItems.length - 1);
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            e.stopPropagation();
            sugg.index = Math.max(sugg.index - 1, 0);
        } else if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            applySuggestion(suggItems[sugg.index]);
        } else if (e.key === "Escape") {
            e.stopPropagation();
            closeSuggestion();
        }
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────────
    onMount(() => {
        editor = buildEditor(container, doc);

        // Restore cached state if available (warm LRU)
        const cached = editorLRU.get(nodeId);
        if (cached) {
            editor.commands.setContent(cached.json, {emitUpdate: false});
            try {
                editor.commands.setTextSelection({from: cached.anchor, to: cached.head});
            } catch {
                // Selection may be out of bounds if content changed externally
            }
            container.scrollTop = cached.scrollTop;
        }

        container.addEventListener("keydown", onContainerKeydown, true);
    });

    onDestroy(() => {
        // Save editor state to LRU before destruction
        if (editor && nodeId) {
            editorLRU.put(nodeId, {
                json: editor.getJSON(),
                anchor: editor.state.selection.anchor,
                head: editor.state.selection.head,
                scrollTop: container?.scrollTop ?? 0,
            });
        }
        container?.removeEventListener("keydown", onContainerKeydown, true);
        flushSave();
        editor?.destroy();
        editor = null;
    });
</script>

<div class="editor-wrap">
    <div bind:this={container} class="editor-content"></div>
</div>

<!-- Suggestion dropdown — rendered at fixed viewport coords of the cursor -->
{#if sugg.active && suggItems.length > 0}
    <div
            class="suggestion-popup"
            style="left: {sugg.left}px; top: {sugg.top}px"
            role="listbox"
            aria-label={sugg.mode === "wiki" ? "Note suggestions" : "Tag suggestions"}
    >
        {#each suggItems as item, i (item.label)}
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

    /*noinspection CssUnusedSymbol*/
    .editor-content :global(.ProseMirror) {
        outline: none;
        font-size: 0.95rem;
        line-height: 1.7;
        color: #1a1a1a;
    }

    .editor-content :global(.ProseMirror h1) {
        font-size: 1.75rem;
        font-weight: 700;
        margin: 1.5rem 0 0.5rem;
    }

    .editor-content :global(.ProseMirror h2) {
        font-size: 1.4rem;
        font-weight: 600;
        margin: 1.25rem 0 0.4rem;
    }

    .editor-content :global(.ProseMirror h3) {
        font-size: 1.15rem;
        font-weight: 600;
        margin: 1rem 0 0.35rem;
    }

    .editor-content :global(.ProseMirror p) {
        margin: 0 0 0.75rem;
    }

    .editor-content :global(.ProseMirror ul),
    .editor-content :global(.ProseMirror ol) {
        padding-left: 1.5rem;
        margin: 0 0 0.75rem;
    }

    .editor-content :global(.ProseMirror li) {
        margin-bottom: 0.2rem;
    }

    .editor-content :global(.ProseMirror code) {
        font-family: ui-monospace, monospace;
        font-size: 0.875em;
        background: #f3f3f3;
        border-radius: 3px;
        padding: 0.1em 0.3em;
    }

    .editor-content :global(.ProseMirror pre) {
        background: #f6f6f6;
        border-radius: 6px;
        padding: 1rem;
        overflow-x: auto;
        margin: 0 0 1rem;
    }

    .editor-content :global(.ProseMirror pre code) {
        background: none;
        padding: 0;
        font-size: 0.875rem;
    }

    .editor-content :global(.ProseMirror blockquote) {
        border-left: 3px solid #ddd;
        margin: 0 0 0.75rem;
        padding-left: 1rem;
        color: #555;
    }

    .editor-content :global(.ProseMirror hr) {
        border: none;
        border-top: 1px solid #e0e0e0;
        margin: 1.5rem 0;
    }

    /*noinspection CssUnusedSymbol*/
    .editor-content :global(.wiki-link) {
        display: inline-block;
        background: #eef2ff;
        color: #4361ee;
        border-radius: 4px;
        padding: 0 0.3em;
        font-size: 0.9em;
        cursor: pointer;
        user-select: none;
    }

    /*noinspection CssUnusedSymbol*/
    .editor-content :global(.tag) {
        color: #7c3aed;
        font-weight: 500;
        cursor: pointer;
        user-select: none;
    }

    .editor-content :global(.ProseMirror table) {
        border-collapse: collapse;
        width: 100%;
        margin-bottom: 1rem;
    }

    .editor-content :global(.ProseMirror th),
    .editor-content :global(.ProseMirror td) {
        border: 1px solid #ddd;
        padding: 0.4rem 0.6rem;
        text-align: left;
    }

    .editor-content :global(.ProseMirror th) {
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
