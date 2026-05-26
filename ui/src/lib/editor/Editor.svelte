<script lang="ts">
    import { onMount, onDestroy, untrack } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { Editor } from "@tiptap/core";
    import StarterKit from "@tiptap/starter-kit";
    import {
        Table,
        TableRow,
        TableHeader,
        TableCell,
    } from "@tiptap/extension-table";
    import Strike from "@tiptap/extension-strike";
    import { WikiLink } from "./extensions/WikiLink";
    import { Tag } from "./extensions/Tag";
    import { FindReplace, getFindReplaceState } from "./extensions/FindReplace";
    import FindBar from "$lib/components/FindBar.svelte";
    import Tooltip from "$lib/components/Tooltip.svelte";
    import { pmDocToTipTap, type PmDoc } from "./doc";
    import { editorLRU } from "./EditorLRU";

    interface Props {
        nodeId: string;
        doc: PmDoc;
        onSave: (doc: object, nodeId: string) => void;
        title: string;
        onRename: (newTitle: string) => Promise<void>;
        onNavigate?: (target: string, isIdRef: boolean) => void;
        existingTitles?: string[];
        findBarOpen?: boolean;
        findShowReplace?: boolean;
    }

    let {
        nodeId,
        doc,
        onSave,
        title,
        onRename,
        onNavigate,
        existingTitles = [],
        findBarOpen = $bindable(false),
        findShowReplace = $bindable(false),
    }: Props = $props();

    // ── Virtualized title ─────────────────────────────────────────────────────
    const INVALID_TITLE_RE = /[\\/:*?"<>|\x00-\x1f]/;

    const titlesLower = $derived(
        new Set(existingTitles.map((t) => t.toLowerCase())),
    );

    function validateTitle(t: string): string {
        if (!t) return "Title cannot be empty";
        if (INVALID_TITLE_RE.test(t))
            return "Title contains invalid characters";
        if (titlesLower.has(t.toLowerCase()))
            return "A note with this title already exists";
        return "";
    }

    let titleDraft = $state(untrack(() => title));
    let titleEditError = $state("");

    const inputError = $derived(validateTitle(titleDraft.trim()));
    const showError = $derived(
        !!titleEditError || (titleDraft !== title && !!inputError),
    );
    const errorMessage = $derived(titleEditError || inputError);

    $effect(() => {
        titleDraft = title;
        titleEditError = "";
    });

    function handleTitleInput() {
        if (titleEditError) titleEditError = "";
    }

    async function submitTitle() {
        const t = titleDraft.trim();
        if (inputError) {
            titleDraft = title;
            return;
        }
        if (t === title) return;
        titleEditError = "";
        try {
            await onRename(t);
        } catch (e) {
            titleEditError = String(e);
            titleDraft = title;
        }
    }

    function handleTitleKeydown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            e.preventDefault();
            editor
                ?.chain()
                .focus()
                .insertContentAt(0, { type: "paragraph" })
                .setTextSelection(1)
                .run();
        } else if (e.key === "Escape") {
            titleDraft = title;
            titleEditError = "";
            (e.target as HTMLInputElement).blur();
        }
    }

    let searchTerm = $state("");
    let replaceTerm = $state("");
    let matchCount = $state(0);
    let findCurrentIndex = $state(0);
    let matchCase = $state(false);
    let wholeWord = $state(false);
    let useRegex = $state(false);

    let prevNodeId = untrack(() => nodeId);
    let isSwapping = false;

    let container: HTMLDivElement; // scrollable wrapper — used for scroll tracking
    let editorMount: HTMLDivElement; // ProseMirror mounts here
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
                    { prefix: query },
                );
                suggItems = results.map((r) => ({ label: r.title, id: r.id }));
            } else {
                const results = await invoke<string[]>("suggest_tags", {
                    prefix: query,
                });
                suggItems = results.map((t) => ({ label: t }));
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
                    heading: { levels: [1, 2, 3, 4, 5, 6] },
                }),
                Strike,
                Table.configure({ resizable: false }),
                TableRow,
                TableHeader,
                TableCell,
                WikiLink.configure({ onNavigate }),
                Tag,
                FindReplace,
            ],
            content: pmDocToTipTap(initialDoc),
        });
        ed.on("update", ({ editor: e }) => {
            if (!isSwapping) scheduleSave();
            refreshSuggestion(e);
        });
        ed.on("selectionUpdate", ({ editor: e }) => {
            refreshSuggestion(e);
        });
        ed.on("transaction", ({ editor: e }) => {
            const ps = getFindReplaceState(e.state);
            matchCount = ps.matches.length;
            findCurrentIndex = ps.currentIndex;
        });
        return ed;
    }

    // ── Find bar handlers ─────────────────────────────────────────────────────────
    function execSearch(term: string) {
        editor?.commands.setSearchTerm(term, {
            matchCase,
            wholeWord,
            useRegex,
        });
    }

    function handleSearchChange(term: string) {
        searchTerm = term;
        execSearch(term);
    }

    function handleMatchCaseToggle() {
        matchCase = !matchCase;
        if (searchTerm) execSearch(searchTerm);
    }

    function handleWholeWordToggle() {
        wholeWord = !wholeWord;
        if (searchTerm) execSearch(searchTerm);
    }

    function handleUseRegexToggle() {
        useRegex = !useRegex;
        if (searchTerm) execSearch(searchTerm);
    }

    function handleClose() {
        findBarOpen = false;
        searchTerm = "";
        replaceTerm = "";
        matchCount = 0;
        findCurrentIndex = 0;
        editor?.commands.setSearchTerm("");
    }

    function scheduleSave() {
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(flushSave, AUTOSAVE_DELAY);
    }

    function flushSave(id: string = nodeId) {
        if (saveTimer) {
            clearTimeout(saveTimer);
            saveTimer = null;
        }
        if (!editor) return;
        onSave(editor.getJSON(), id);
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

            flushSave(prevNodeId);
            prevNodeId = newId;
            closeSuggestion();

            const cached = editorLRU.get(newId);
            const content = cached ? cached.json : pmDocToTipTap(doc);
            isSwapping = true;
            editor.commands.setContent(content, { emitUpdate: false });
            isSwapping = false;

            if (cached) {
                try {
                    editor.commands.setTextSelection({
                        from: cached.anchor,
                        to: cached.head,
                    });
                } catch {}
                container.scrollTop = cached.scrollTop;
            } else {
                editor.commands.setTextSelection(1);
                container.scrollTop = 0;
            }

            titleDraft = title;
            titleEditError = "";
        });
    });

    // ── Suggestion logic ──────────────────────────────────────────────────────────
    function refreshSuggestion(ed: Editor) {
        const { selection } = ed.state;
        if (!selection.empty) {
            closeSuggestion();
            return;
        }

        const anchor = selection.$from;
        const textBefore = anchor.parent.textContent.slice(
            0,
            anchor.parentOffset,
        );

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
            const prevChar =
                posBeforeHash > 0 ? textBefore[posBeforeHash - 1] : "";
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
            sugg = { ...sugg, active: false };
            suggItems = [];
        }
    }

    function applySuggestion(item: SuggItem) {
        if (!editor || !sugg.active) return;
        const to = editor.state.selection.$from.pos;
        const content =
            sugg.mode === "wiki"
                ? {
                      type: "wikiLink",
                      attrs: { target: item.label, isIdRef: false },
                  }
                : { type: "tag", attrs: { name: item.label } };
        editor
            .chain()
            .focus()
            .deleteRange({ from: sugg.from, to })
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
        editor = buildEditor(editorMount, doc);

        // Restore cached state if available (warm LRU)
        const cached = editorLRU.get(nodeId);
        if (cached) {
            editor.commands.setContent(cached.json, { emitUpdate: false });
            try {
                editor.commands.setTextSelection({
                    from: cached.anchor,
                    to: cached.head,
                });
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
        flushSave(nodeId);
        editor?.destroy();
        editor = null;
    });
</script>

<div class="editor-outer">
    {#if findBarOpen}
        <FindBar
            {searchTerm}
            {replaceTerm}
            {matchCount}
            {matchCase}
            {wholeWord}
            {useRegex}
            showReplace={findShowReplace}
            currentIndex={findCurrentIndex}
            onSearchChange={handleSearchChange}
            onReplaceChange={(v) => (replaceTerm = v)}
            onMatchCaseToggle={handleMatchCaseToggle}
            onWholeWordToggle={handleWholeWordToggle}
            onUseRegexToggle={handleUseRegexToggle}
            onToggleReplace={() => (findShowReplace = !findShowReplace)}
            onNext={() => editor?.commands.findNext()}
            onPrev={() => editor?.commands.findPrev()}
            onReplace={() => editor?.commands.replaceOne(replaceTerm)}
            onReplaceAll={() => editor?.commands.replaceAll(replaceTerm)}
            onClose={handleClose}
        />
    {/if}
    <div class="editor-wrap" bind:this={container}>
        <div class="editor-content">
            <div class="title-zone">
                <div class="title-tooltip-wrap">
                    <Tooltip description={errorMessage}>
                        <input
                            class="title-input"
                            class:invalid={showError}
                            bind:value={titleDraft}
                            oninput={handleTitleInput}
                            onblur={submitTitle}
                            onkeydown={handleTitleKeydown}
                            spellcheck={false}
                            autocomplete="off"
                        />
                    </Tooltip>
                </div>
            </div>
            <div bind:this={editorMount} spellcheck={false}></div>
        </div>
    </div>
</div>

<!-- Suggestion dropdown — rendered at fixed viewport coords of the cursor -->
{#if sugg.active && suggItems.length > 0}
    <div
        class="suggestion-popup"
        style="left: {sugg.left}px; top: {sugg.top}px"
        role="listbox"
        aria-label={sugg.mode === "wiki"
            ? "Note suggestions"
            : "Tag suggestions"}
    >
        {#each suggItems as item, i (item.label)}
            <!-- onmousedown + preventDefault keeps editor focus when clicking -->
            <button
                class="suggestion-item"
                class:selected={i === sugg.index}
                class:tag-item={sugg.mode === "tag"}
                role="option"
                aria-selected={i === sugg.index}
                onmousedown={(e) => {
                    e.preventDefault();
                    applySuggestion(item);
                }}
            >
                {#if sugg.mode === "tag"}<span class="tag-prefix">#</span
                    >{/if}{item.label}
            </button>
        {/each}
    </div>
{/if}

<style>
    .editor-outer {
        position: relative;
        width: 100%;
        height: 100%;
    }

    .editor-wrap {
        width: 100%;
        height: 100%;
        overflow-y: auto;
    }

    .editor-content {
        font-family: var(--font-content), monospace;
        max-width: 680px;
        margin: 0 auto;
        padding: 2rem 1.5rem 6rem;
        min-height: 100%;
    }

    .title-zone {
        margin-bottom: 0.25rem;
    }

    .title-input {
        width: 100%;
        font-size: 1.75rem;
        font-weight: 700;
        color: var(--fg-primary);
        background: transparent;
        border: none;
        outline: none;
        padding: 0;
        margin: 0 0 0.1rem;
        line-height: 1.25;
        caret-color: var(--fg-interactive);
    }

    .title-input::placeholder {
        color: var(--cursor);
    }

    .title-input.invalid {
        color: var(--fg-error);
        caret-color: var(--fg-error);
    }

    .title-tooltip-wrap {
        width: 100%;
    }

    .title-tooltip-wrap :global(.tooltip-root) {
        display: block;
        width: 100%;
    }

    .editor-content :global(.ProseMirror) {
        outline: none;
        font-size: 0.95rem;
        line-height: 1.7;
        color: var(--fg-primary);
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
        font-family: var(--font-mono);
        font-size: 0.875em;
        background: var(--bg-hover);
        border-radius: 3px;
        padding: 0.1em 0.3em;
    }

    .editor-content :global(.ProseMirror pre) {
        background: var(--bg-hover);
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
        border-left: 3px solid var(--bg-border);
        margin: 0 0 0.75rem;
        padding-left: 1rem;
        color: var(--fg-muted);
    }

    .editor-content :global(.ProseMirror hr) {
        border: none;
        border-top: 1px solid var(--bg-border);
        margin: 1.5rem 0;
    }

    .editor-content :global(.wiki-link) {
        display: inline-block;
        color: var(--fg-interactive);
        cursor: pointer;
        user-select: none;
    }

    .editor-content :global(.tag) {
        color: var(--accent);
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
        border: 1px solid var(--bg-border);
        padding: 0.4rem 0.6rem;
        text-align: left;
    }

    .editor-content :global(.ProseMirror th) {
        background: var(--bg-panel);
        font-weight: 600;
    }

    /* ── Find/replace highlights ── */
    .editor-content :global(.find-match) {
        background: rgba(215, 153, 33, 0.25);
        border-radius: 2px;
    }

    .editor-content :global(.find-current) {
        background: rgba(215, 153, 33, 0.6);
        border-radius: 2px;
        outline: 1px solid #d79921;
    }

    /* ── Suggestion dropdown ── */
    .suggestion-popup {
        position: fixed;
        z-index: 200;
        min-width: 200px;
        max-width: 320px;
        background: var(--bg-panel);
        border: 1px solid var(--bg-border);
        border-radius: 6px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
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
        color: var(--fg-primary);
    }

    .suggestion-item.selected,
    .suggestion-item:hover {
        background: var(--bg-active);
        color: var(--fg-interactive);
    }

    .tag-prefix {
        color: var(--accent);
        font-weight: 500;
    }

    .suggestion-item.tag-item.selected .tag-prefix,
    .suggestion-item.tag-item:hover .tag-prefix {
        color: var(--fg-interactive);
    }
</style>
