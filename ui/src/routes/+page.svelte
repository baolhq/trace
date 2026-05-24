<!-- Svelte action: focus an element when it mounts -->
<script lang="ts" module>
    function focusOnMount(el: HTMLElement) {
        el.focus();
    }
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import Editor from "$lib/editor/Editor.svelte";
    import TitleBar from "$lib/TitleBar.svelte";
    import { tipTapToPmDoc, type PmDoc } from "$lib/editor/doc";

    // ── Types ──────────────────────────────────────────────────────────────

    interface NodeInfo {
        id: string;
        title: string;
        created_at: number;
        is_favorite: boolean;
    }

    interface NodeMeta {
        id: string;
        title: string;
        created_at: number;
        modified_at: number;
        is_favorite: boolean;
    }

    interface OpenNodeResponse {
        meta: NodeMeta;
        doc: PmDoc;
    }

    interface Log {
        id: number;
        name: string;
        parent_id: number | null;
        sort_key: number;
    }

    interface LogTreeNode extends Log {
        children: LogTreeNode[];
    }

    type FlatSidebarItem =
        | { type: "log"; log: LogTreeNode; depth: number }
        | { type: "member"; node: NodeInfo; logId: number; depth: number };

    type ViewMode = { kind: "editor" } | null;

    // ── State ──────────────────────────────────────────────────────────────

    let recentNodes: NodeInfo[] = $state([]);
    let favorites: NodeInfo[] = $state([]);
    let allLogs: Log[] = $state([]);
    let logMembersMap: Record<number, NodeInfo[]> = $state({});

    let viewMode: ViewMode = $state(null);
    let activeNodeId: string | null = $state(null);
    let activeDoc: PmDoc | null = $state(null);
    let activeMeta: NodeMeta | null = $state(null);

    let expandedLogs: Set<number> = $state(new Set());

    let newLogName = $state("");
    let creatingLog = $state(false);
    let renamingLogId: number | null = $state(null);
    let renamingLogName = $state("");

    // Panel collapse state
    let favoritesOpen = $state(true);
    let logsOpen = $state(true);
    let recentsOpen = $state(true);

    // Sidebar mode
    let sidebarMode: "notes" | "search" = $state("notes");

    // Search panel state
    interface SearchHit {
        id: string;
        title: string;
        snippet: string;
    }
    type SearchSubMode = "search" | "replace";
    let searchSubMode: SearchSubMode = $state("search");
    let searchQuery = $state("");
    let replaceQuery = $state("");
    let searchRegex = $state(false);
    let searchResults: SearchHit[] = $state([]);
    let searchLoading = $state(false);
    let searchError: string | null = $state(null);
    let searchDebounce: ReturnType<typeof setTimeout> | null = null;

    $effect(() => {
        // Reset panel when switching to search mode
        if (sidebarMode === "search") {
            searchQuery = "";
            searchResults = [];
            searchError = null;
        }
    });

    $effect(() => {
        // Debounced auto-search on query change
        if (searchDebounce) clearTimeout(searchDebounce);
        const q = searchQuery;
        const rx = searchRegex;
        if (!q.trim()) {
            searchResults = [];
            searchError = null;
            return;
        }
        searchDebounce = setTimeout(() => runSearch(q, rx), 350);
    });

    async function runSearch(q: string, isRegex: boolean) {
        searchLoading = true;
        searchError = null;
        try {
            searchResults = await invoke("search_nodes", { query: q, isRegex });
        } catch (e) {
            searchError = String(e);
            searchResults = [];
        } finally {
            searchLoading = false;
        }
    }

    function handleSearchKey(e: KeyboardEvent) {
        if (e.key === "Enter") runSearch(searchQuery, searchRegex);
    }

    // Pointer-based drag (HTML5 DnD is unreliable in WebView2).
    let ptrDragId: string | null = null;
    let ptrStartX = 0;
    let ptrStartY = 0;
    let ptrDragging = false;
    let ptrCaptureEl: HTMLElement | null = null;
    let dragOverLogId: number | null = $state(null);

    let error = $state("");
    let saving = $state(false);
    let titleError = $state(false);

    // ── Derived ────────────────────────────────────────────────────────────

    const logTree: LogTreeNode[] = $derived(buildTree(allLogs, null));
    const flatItems: FlatSidebarItem[] = $derived(
        flattenTree(logTree, expandedLogs, logMembersMap),
    );

    function buildTree(logs: Log[], parentId: number | null): LogTreeNode[] {
        return logs
            .filter((l) => l.parent_id === parentId)
            .sort((a, b) => a.sort_key - b.sort_key)
            .map((l) => ({ ...l, children: buildTree(logs, l.id) }));
    }

    function flattenTree(
        nodes: LogTreeNode[],
        expanded: Set<number>,
        membersMap: Record<number, NodeInfo[]>,
        depth = 0,
    ): FlatSidebarItem[] {
        const result: FlatSidebarItem[] = [];
        for (const node of nodes) {
            result.push({ type: "log", log: node, depth });
            if (expanded.has(node.id)) {
                result.push(
                    ...flattenTree(
                        node.children,
                        expanded,
                        membersMap,
                        depth + 1,
                    ),
                );
                for (const member of membersMap[node.id] ?? []) {
                    result.push({
                        type: "member",
                        node: member,
                        logId: node.id,
                        depth: depth + 1,
                    });
                }
            }
        }
        return result;
    }

    // ── Data loading ───────────────────────────────────────────────────────

    let loadGen = 0;

    async function loadRecents() {
        const gen = ++loadGen;
        try {
            const raw = await invoke<NodeInfo[]>("list_nodes");
            if (gen !== loadGen) return;
            const seen = new Set<string>();
            recentNodes = raw.filter(
                (n) =>
                    !seen.has(n.id) && (seen.add(n.id) as unknown as boolean),
            );
        } catch (e) {
            if (gen === loadGen) error = String(e);
        }
    }

    async function loadFavorites() {
        try {
            favorites = await invoke<NodeInfo[]>("list_favorites");
        } catch {
            // non-critical
        }
    }

    async function loadLogs() {
        try {
            allLogs = await invoke<Log[]>("get_log_tree");
        } catch (e) {
            error = String(e);
        }
    }

    async function loadLogMembers(logId: number) {
        try {
            const members = await invoke<NodeInfo[]>("get_log_members", {
                logId,
                page: 0,
            });
            logMembersMap = { ...logMembersMap, [logId]: members };
        } catch (e) {
            error = String(e);
        }
    }

    // ── Node actions ───────────────────────────────────────────────────────

    async function openNode(id: string) {
        if (activeNodeId === id && viewMode?.kind === "editor") return;
        const hit = recentNodes.find((n) => n.id === id);
        if (hit) recentNodes = [hit, ...recentNodes.filter((n) => n.id !== id)];
        try {
            const res = await invoke<OpenNodeResponse>("open_node", { id });
            activeNodeId = id;
            activeMeta = res.meta;
            activeDoc = res.doc;
            viewMode = { kind: "editor" };
            error = "";
            titleError = false;
        } catch (e) {
            error = String(e);
            await loadRecents();
        }
    }

    async function createUntitledNode() {
        try {
            const id = await invoke<string>("create_node", {
                title: "Untitled",
            });
            await loadRecents();
            await openNode(id);
        } catch (e) {
            error = String(e);
        }
    }

    async function deleteNode(id: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("delete_node", { id });
            if (activeNodeId === id) {
                activeNodeId = null;
                activeDoc = null;
                activeMeta = null;
                viewMode = null;
            }
            await Promise.all([loadRecents(), loadFavorites()]);
            const updated: Record<number, NodeInfo[]> = {};
            for (const [key, members] of Object.entries(logMembersMap)) {
                updated[Number(key)] = (members as NodeInfo[]).filter(
                    (n) => n.id !== id,
                );
            }
            logMembersMap = updated;
        } catch (e) {
            error = String(e);
        }
    }

    async function toggleFavorite(id: string, e?: MouseEvent) {
        e?.stopPropagation();
        try {
            const newState = await invoke<boolean>("toggle_favorite", { id });
            await loadFavorites();
            recentNodes = recentNodes.map((n) =>
                n.id === id ? { ...n, is_favorite: newState } : n,
            );
            if (activeMeta && activeMeta.id === id) {
                activeMeta = { ...activeMeta, is_favorite: newState };
            }
            const updated: Record<number, NodeInfo[]> = {};
            for (const [key, members] of Object.entries(logMembersMap)) {
                updated[Number(key)] = (members as NodeInfo[]).map((n) =>
                    n.id === id ? { ...n, is_favorite: newState } : n,
                );
            }
            logMembersMap = updated;
        } catch (e) {
            error = String(e);
        }
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function extractTitleFromTt(ttJson: any): string {
        const first = ttJson?.content?.[0];
        if (!first) return "Untitled";
        return (
            (first.content ?? [])
                .filter((n: any) => n.type === "text")
                .map((n: any) => n.text ?? "")
                .join("")
                .trim() || "Untitled"
        );
    }

    async function handleSave(ttJson: object, saveNodeId: string) {
        if (!saveNodeId) return;
        saving = true;
        try {
            const doc = tipTapToPmDoc(ttJson, activeDoc?.frontmatter);
            await invoke("save_node", { id: saveNodeId, doc });
            // Discard result if the user switched notes while this save was in flight
            if (activeNodeId !== saveNodeId) return;
            titleError = false;
            const newTitle = extractTitleFromTt(ttJson);
            if (activeMeta?.id === saveNodeId)
                activeMeta = { ...activeMeta, title: newTitle };
            recentNodes = recentNodes.map((n) =>
                n.id === saveNodeId ? { ...n, title: newTitle } : n,
            );
            favorites = favorites.map((n) =>
                n.id === saveNodeId ? { ...n, title: newTitle } : n,
            );
            const updatedMap: Record<number, NodeInfo[]> = {};
            for (const [k, members] of Object.entries(logMembersMap)) {
                updatedMap[Number(k)] = (members as NodeInfo[]).map((n) =>
                    n.id === saveNodeId ? { ...n, title: newTitle } : n,
                );
            }
            logMembersMap = updatedMap;
        } catch (e) {
            if (activeNodeId !== saveNodeId) return;
            const msg = String(e);
            if (msg.includes("title invalid:")) {
                titleError = true;
            } else {
                error = msg;
            }
        } finally {
            saving = false;
        }
    }

    // ── Log actions ────────────────────────────────────────────────────────

    async function openLog(log: LogTreeNode) {
        const wasExpanded = expandedLogs.has(log.id);
        if (wasExpanded) {
            expandedLogs.delete(log.id);
        } else {
            expandedLogs.add(log.id);
            if (!logMembersMap[log.id]) {
                await loadLogMembers(log.id);
            }
        }
        expandedLogs = new Set(expandedLogs);
    }

    async function createLog() {
        const name = newLogName.trim();
        if (!name) {
            creatingLog = false;
            return;
        }
        try {
            await invoke("create_log", { name, parentId: null });
            newLogName = "";
            creatingLog = false;
            await loadLogs();
        } catch (e) {
            error = String(e);
        }
    }

    async function deleteLog(id: number, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("delete_log", { id });
            expandedLogs.delete(id);
            expandedLogs = new Set(expandedLogs);
            const newMap = { ...logMembersMap };
            delete newMap[id];
            logMembersMap = newMap;
            await loadLogs();
        } catch (e) {
            error = String(e);
        }
    }

    function startRenameLog(log: LogTreeNode, e: MouseEvent) {
        e.stopPropagation();
        renamingLogId = log.id;
        renamingLogName = log.name;
    }

    async function commitRenameLog() {
        if (renamingLogId === null) return;
        const name = renamingLogName.trim();
        if (name) {
            try {
                await invoke("rename_log", { id: renamingLogId, name });
                await loadLogs();
            } catch (e) {
                error = String(e);
            }
        }
        renamingLogId = null;
    }

    async function removeFromLog(logId: number, nodeId: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("remove_from_log", { logId, nodeId });
            const current = logMembersMap[logId] ?? [];
            logMembersMap = {
                ...logMembersMap,
                [logId]: current.filter((n) => n.id !== nodeId),
            };
        } catch (e) {
            error = String(e);
        }
    }

    // ── Drag-and-drop ──────────────────────────────────────────────────────

    function onNodePointerDown(id: string, e: PointerEvent) {
        if (e.button !== 0) return;
        ptrDragId = id;
        ptrStartX = e.clientX;
        ptrStartY = e.clientY;
        ptrDragging = false;
        ptrCaptureEl = e.currentTarget as HTMLElement;
        ptrCaptureEl.setPointerCapture(e.pointerId);
    }

    function onNodePointerMove(e: PointerEvent) {
        if (!ptrDragId) return;
        if (!ptrDragging) {
            const moved =
                Math.abs(e.clientX - ptrStartX) +
                Math.abs(e.clientY - ptrStartY);
            if (moved < 6) return;
            ptrDragging = true;
        }
        // data-log-id is on both log headers and member rows, so the whole
        // expanded section is a valid drop target.
        const target = document.elementFromPoint(e.clientX, e.clientY);
        const logEl = target?.closest("[data-log-id]") as HTMLElement | null;
        const found = logEl ? Number(logEl.dataset.logId) : null;
        dragOverLogId = found;
        // During pointer capture WebView2 uses the capturing element's cursor,
        // so we must set it inline directly on that element.
        if (ptrCaptureEl)
            ptrCaptureEl.style.cursor = found !== null ? "copy" : "not-allowed";
    }

    async function onNodePointerUp(id: string) {
        const wasDragging = ptrDragging;
        const targetLogId = dragOverLogId;
        ptrDragId = null;
        ptrDragging = false;
        dragOverLogId = null;
        if (ptrCaptureEl) {
            ptrCaptureEl.style.cursor = "";
            ptrCaptureEl = null;
        }
        if (wasDragging) {
            if (targetLogId !== null) {
                try {
                    await invoke("add_to_log", {
                        logId: targetLogId,
                        nodeId: id,
                    });
                    if (expandedLogs.has(targetLogId)) {
                        await loadLogMembers(targetLogId);
                    } else {
                        const newMap = { ...logMembersMap };
                        delete newMap[targetLogId];
                        logMembersMap = newMap;
                    }
                } catch (err) {
                    error = String(err);
                }
            }
        } else {
            await openNode(id);
        }
    }

    function onNodePointerCancel() {
        ptrDragId = null;
        ptrDragging = false;
        dragOverLogId = null;
        if (ptrCaptureEl) {
            ptrCaptureEl.style.cursor = "";
            ptrCaptureEl = null;
        }
    }

    // ── Keyboard ───────────────────────────────────────────────────────────

    function handleNewLogKey(e: KeyboardEvent) {
        if (e.key === "Enter") createLog();
        if (e.key === "Escape") {
            creatingLog = false;
            newLogName = "";
        }
    }

    function handleRenameLogKey(e: KeyboardEvent) {
        if (e.key === "Enter") commitRenameLog();
        if (e.key === "Escape") renamingLogId = null;
    }

    // ── Lifecycle ──────────────────────────────────────────────────────────

    let unlisten: (() => void) | undefined;

    onMount(async () => {
        await Promise.all([loadRecents(), loadFavorites(), loadLogs()]);
        await invoke("frontend_ready");
        unlisten = await listen("nodes_changed", () => {
            loadRecents();
            loadFavorites();
        });
    });

    onDestroy(() => unlisten?.());
</script>

<div class="app-root">
    <TitleBar
        {activeMeta}
        {recentNodes}
        onOpenNode={openNode}
        onToggleFavorite={() => activeMeta && toggleFavorite(activeMeta.id)}
        onNewNote={createUntitledNode}
    />

    <div class="shell">
        <!-- ── Sidebar ── -->
        <aside class="sidebar">
            {#if error}
                <p class="sidebar-error">{error}</p>
            {/if}

            <!-- Three collapsible panels; each open panel gets equal flex share -->
            {#if sidebarMode === "notes"}
                <div class="sidebar-panels">
                    <!-- Favorites panel -->
                    <div class="panel" class:open={favoritesOpen}>
                        <div
                            class="panel-header"
                            role="button"
                            tabindex="0"
                            onclick={() => (favoritesOpen = !favoritesOpen)}
                            onkeydown={(e) =>
                                e.key === "Enter" &&
                                (favoritesOpen = !favoritesOpen)}
                        >
                            <span class="panel-arrow"
                                >{favoritesOpen ? "▾" : "▸"}</span
                            >
                            <span class="panel-title">Favorites</span>
                        </div>
                        {#if favoritesOpen}
                            <div class="panel-body">
                                {#each favorites as node (node.id)}
                                    <div
                                        class="node-item"
                                        class:active={activeNodeId === node.id}
                                        role="option"
                                        aria-selected={activeNodeId === node.id}
                                        tabindex="-1"
                                    >
                                        <button
                                            class="node-btn"
                                            onpointerdown={(e) =>
                                                onNodePointerDown(node.id, e)}
                                            onpointermove={onNodePointerMove}
                                            onpointerup={() =>
                                                onNodePointerUp(node.id)}
                                            onpointercancel={onNodePointerCancel}
                                        >
                                            <span class="node-title"
                                                >{node.title}</span
                                            >
                                        </button>
                                        <button
                                            class="action-btn fav-btn fav-on"
                                            onclick={(e) =>
                                                toggleFavorite(node.id, e)}
                                            title="Unfavorite"
                                            tabindex="-1"
                                            >★
                                        </button>
                                        <button
                                            class="action-btn delete-btn"
                                            onclick={(e) =>
                                                deleteNode(node.id, e)}
                                            title="Delete"
                                            tabindex="-1"
                                            >×
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>

                    <!-- Logs panel -->
                    <div class="panel" class:open={logsOpen}>
                        <div
                            class="panel-header"
                            role="button"
                            tabindex="0"
                            onclick={() => (logsOpen = !logsOpen)}
                            onkeydown={(e) =>
                                e.key === "Enter" && (logsOpen = !logsOpen)}
                        >
                            <span class="panel-arrow"
                                >{logsOpen ? "▾" : "▸"}</span
                            >
                            <span class="panel-title">Logs</span>
                            <button
                                class="section-add"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    creatingLog = true;
                                }}
                                title="New log"
                                >+
                            </button>
                        </div>
                        {#if logsOpen}
                            <div class="panel-body">
                                {#each flatItems as item (item.type === "log" ? `log-${item.log.id}` : `member-${item.logId}-${item.node.id}`)}
                                    {#if item.type === "log"}
                                        <div
                                            class="log-item"
                                            class:drag-over={dragOverLogId ===
                                                item.log.id}
                                            role="treeitem"
                                            aria-expanded={expandedLogs.has(
                                                item.log.id,
                                            )}
                                            aria-selected={false}
                                            tabindex="-1"
                                            data-log-id={item.log.id}
                                            style="padding-left: {0.75 +
                                                item.depth * 0.85}rem"
                                        >
                                            <button
                                                class="log-btn"
                                                onclick={() =>
                                                    openLog(item.log)}
                                            >
                                                <span class="log-arrow">
                                                    {expandedLogs.has(
                                                        item.log.id,
                                                    )
                                                        ? "▾"
                                                        : "▸"}
                                                </span>
                                                {#if renamingLogId === item.log.id}
                                                    <input
                                                        class="log-rename-input"
                                                        bind:value={
                                                            renamingLogName
                                                        }
                                                        onkeydown={handleRenameLogKey}
                                                        onblur={commitRenameLog}
                                                        onclick={(e) =>
                                                            e.stopPropagation()}
                                                        use:focusOnMount
                                                    />
                                                {:else}
                                                    <span class="log-name"
                                                        >{item.log.name}</span
                                                    >
                                                {/if}
                                            </button>
                                            <button
                                                class="action-btn log-action"
                                                onclick={(e) =>
                                                    startRenameLog(item.log, e)}
                                                title="Rename"
                                                tabindex="-1"
                                                >…
                                            </button>
                                            <button
                                                class="action-btn delete-btn log-action"
                                                onclick={(e) =>
                                                    deleteLog(item.log.id, e)}
                                                title="Delete log"
                                                tabindex="-1"
                                                >×
                                            </button>
                                        </div>
                                    {:else}
                                        <div
                                            class="log-member-item"
                                            class:active={activeNodeId ===
                                                item.node.id}
                                            class:drag-over={dragOverLogId ===
                                                item.logId}
                                            role="option"
                                            aria-selected={activeNodeId ===
                                                item.node.id}
                                            tabindex="-1"
                                            data-log-id={item.logId}
                                            style="padding-left: {0.75 +
                                                item.depth * 0.85}rem"
                                        >
                                            <button
                                                class="node-btn"
                                                onpointerdown={(e) =>
                                                    onNodePointerDown(
                                                        item.node.id,
                                                        e,
                                                    )}
                                                onpointermove={onNodePointerMove}
                                                onpointerup={() =>
                                                    onNodePointerUp(
                                                        item.node.id,
                                                    )}
                                                onpointercancel={onNodePointerCancel}
                                            >
                                                <span class="node-title"
                                                    >{item.node.title}</span
                                                >
                                            </button>
                                            <button
                                                class="action-btn fav-btn"
                                                class:fav-on={item.node
                                                    .is_favorite}
                                                onclick={(e) =>
                                                    toggleFavorite(
                                                        item.node.id,
                                                        e,
                                                    )}
                                                title={item.node.is_favorite
                                                    ? "Unfavorite"
                                                    : "Favorite"}
                                                tabindex="-1"
                                                >★
                                            </button>
                                            <button
                                                class="action-btn remove-btn"
                                                onclick={(e) =>
                                                    removeFromLog(
                                                        item.logId,
                                                        item.node.id,
                                                        e,
                                                    )}
                                                title="Remove from log"
                                                tabindex="-1"
                                                >×
                                            </button>
                                        </div>
                                    {/if}
                                {/each}

                                {#if creatingLog}
                                    <div
                                        class="log-item log-create"
                                        style="padding-left: 0.75rem"
                                    >
                                        <input
                                            class="log-create-input"
                                            bind:value={newLogName}
                                            onkeydown={handleNewLogKey}
                                            onblur={() => {
                                                if (!newLogName.trim())
                                                    creatingLog = false;
                                            }}
                                            placeholder="Log name…"
                                            use:focusOnMount
                                        />
                                    </div>
                                {:else if flatItems.filter((i) => i.type === "log").length === 0}
                                    <div class="empty-hint">
                                        Drag notes here to organize.
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>

                    <!-- Recents panel (no scroll — fade mask shows overflow) -->
                    <div class="panel" class:open={recentsOpen}>
                        <div
                            class="panel-header"
                            role="button"
                            tabindex="0"
                            onclick={() => (recentsOpen = !recentsOpen)}
                            onkeydown={(e) =>
                                e.key === "Enter" &&
                                (recentsOpen = !recentsOpen)}
                        >
                            <span class="panel-arrow"
                                >{recentsOpen ? "▾" : "▸"}</span
                            >
                            <span class="panel-title">Recents</span>
                        </div>
                        {#if recentsOpen}
                            <div class="panel-body no-scroll">
                                {#each recentNodes as node (node.id)}
                                    <div
                                        class="node-item"
                                        class:active={activeNodeId === node.id}
                                        role="option"
                                        aria-selected={activeNodeId === node.id}
                                        tabindex="-1"
                                    >
                                        <button
                                            class="node-btn"
                                            onpointerdown={(e) =>
                                                onNodePointerDown(node.id, e)}
                                            onpointermove={onNodePointerMove}
                                            onpointerup={() =>
                                                onNodePointerUp(node.id)}
                                            onpointercancel={onNodePointerCancel}
                                        >
                                            <span class="node-title"
                                                >{node.title}</span
                                            >
                                        </button>
                                        <button
                                            class="action-btn fav-btn"
                                            class:fav-on={node.is_favorite}
                                            onclick={(e) =>
                                                toggleFavorite(node.id, e)}
                                            title={node.is_favorite
                                                ? "Unfavorite"
                                                : "Favorite"}
                                            tabindex="-1"
                                            >★
                                        </button>
                                        <button
                                            class="action-btn delete-btn"
                                            onclick={(e) =>
                                                deleteNode(node.id, e)}
                                            title="Delete"
                                            tabindex="-1"
                                            >×
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                </div>
            {:else if sidebarMode === "search"}
                <div class="sidebar-search">
                    <!-- Sub-mode tabs -->
                    <div class="search-tabs">
                        <button
                            class="search-tab"
                            class:active={searchSubMode === "search"}
                            onclick={() => (searchSubMode = "search")}
                            >Search</button
                        >
                        <button
                            class="search-tab"
                            class:active={searchSubMode === "replace"}
                            onclick={() => (searchSubMode = "replace")}
                            >Replace</button
                        >
                    </div>

                    <!-- Search input -->
                    <div class="search-input-row">
                        <input
                            class="search-input"
                            bind:value={searchQuery}
                            placeholder="Search notes…"
                            onkeydown={handleSearchKey}
                            use:focusOnMount
                        />
                        <button
                            class="search-opt-btn"
                            class:active={searchRegex}
                            onclick={() => {
                                searchRegex = !searchRegex;
                            }}
                            title="Use regular expression">.*</button
                        >
                    </div>

                    <!-- Replace input (replace mode only) -->
                    {#if searchSubMode === "replace"}
                        <div class="search-input-row">
                            <input
                                class="search-input"
                                bind:value={replaceQuery}
                                placeholder="Replace with…"
                            />
                        </div>
                    {/if}

                    <!-- Results -->
                    <div class="search-results">
                        {#if searchLoading}
                            <p class="search-status">Searching…</p>
                        {:else if searchError}
                            <p class="search-status search-error">
                                {searchError}
                            </p>
                        {:else if searchResults.length === 0 && searchQuery.trim()}
                            <p class="search-status">No results</p>
                        {:else}
                            {#each searchResults as hit (hit.id)}
                                <div
                                    class="search-hit"
                                    class:active={activeNodeId === hit.id}
                                    role="option"
                                    aria-selected={activeNodeId === hit.id}
                                    tabindex="0"
                                    onclick={() => openNode(hit.id)}
                                    onkeydown={(e) =>
                                        e.key === "Enter" && openNode(hit.id)}
                                >
                                    <div class="search-hit-title">
                                        {hit.title}
                                    </div>
                                    <div class="search-hit-snippet">
                                        {@html hit.snippet}
                                    </div>
                                </div>
                            {/each}
                        {/if}
                    </div>
                </div>
            {/if}
        </aside>

        <!-- ── Main pane ── -->
        <main class="main-pane">
            {#if viewMode?.kind === "editor" && activeDoc && activeNodeId}
                <Editor
                    nodeId={activeNodeId}
                    doc={activeDoc}
                    onSave={handleSave}
                    {titleError}
                />
            {:else}
                <div class="empty-state">
                    <p>Select a note or create one to start writing.</p>
                </div>
            {/if}
        </main>
    </div>

    <!-- ── Status bar ── -->
    <div class="statusbar">
        <div class="statusbar-section statusbar-left">
            <button
                class="mode-btn"
                class:active={sidebarMode === "notes"}
                onclick={() => (sidebarMode = "notes")}>Notes</button
            >
            <button
                class="mode-btn"
                class:active={sidebarMode === "search"}
                onclick={() => (sidebarMode = "search")}>Search</button
            >
        </div>
        <div class="statusbar-section statusbar-right">
            {#if saving}
                <span class="status-item">Saving…</span>
            {/if}
        </div>
    </div>
</div>

<style>
    :global(*, *::before, *::after) {
        box-sizing: border-box;
    }

    :global(body) {
        margin: 0;
        font-family:
            system-ui,
            -apple-system,
            sans-serif;
        background: var(--bg-primary);
        color: var(--fg-primary);
    }

    .app-root {
        display: flex;
        flex-direction: column;
        height: 100vh;
        overflow: hidden;
    }

    .shell {
        display: flex;
        flex: 1;
        min-height: 0;
        overflow: hidden;
    }

    /* ── Sidebar ── */
    .sidebar {
        width: 240px;
        flex-shrink: 0;
        border-right: 1px solid var(--bg-border);
        background: var(--bg-primary);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* ── Panels ── */
    .sidebar-panels {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* Collapsed: only the header row. Open: equal share of remaining space. */
    .panel {
        display: flex;
        flex-direction: column;
        min-height: 0;
        flex: 0 0 auto;
        border-top: 1px solid var(--bg-border);
    }

    .panel.open {
        flex: 1 1 0;
    }

    .panel:first-child {
        border-top: none;
    }

    .panel-header {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.45rem 0.75rem;
        cursor: pointer;
        user-select: none;
        flex-shrink: 0;
    }

    .panel-header:hover {
        background: var(--bg-hover);
    }

    .panel-arrow {
        font-size: 0.65rem;
        color: var(--cursor);
        width: 10px;
        text-align: center;
        flex-shrink: 0;
    }

    .panel-title {
        flex: 1;
        font-size: 0.7rem;
        font-weight: 600;
        letter-spacing: 0.07em;
        text-transform: uppercase;
        color: var(--cursor);
    }

    .panel-body {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .panel-body.no-scroll {
        overflow: hidden;
        -webkit-mask-image: linear-gradient(
            to bottom,
            black 80%,
            transparent 100%
        );
        mask-image: linear-gradient(to bottom, black 80%, transparent 100%);
    }

    .sidebar-error {
        font-size: 0.75rem;
        color: var(--fg-error);
        padding: 0 0.75rem;
        margin: 0 0 0.5rem;
    }

    .section-add {
        margin-left: auto;
        background: none;
        border: none;
        font-size: 1rem;
        line-height: 1;
        color: var(--cursor);
        cursor: pointer;
        padding: 0 0.1rem;
    }

    .section-add:hover {
        color: var(--fg-muted);
    }

    /* ── Node items (shared by favorites + recents) ── */
    .node-item {
        display: flex;
        align-items: center;
        gap: 0.15rem;
        padding: 0 0.4rem 0 0;
        cursor: default;
    }

    .node-item:hover {
        background: var(--bg-hover);
    }

    .node-item.active {
        background: var(--bg-active);
    }

    .node-item.active .node-btn {
        color: var(--fg-interactive);
    }

    .node-btn {
        flex: 1;
        background: none;
        border: none;
        text-align: left;
        cursor: pointer;
        padding: 0.4rem 0.3rem 0.4rem 0.75rem;
        overflow: hidden;
        min-width: 0;
        height: 34px;
        display: flex;
        align-items: center;
        user-select: none;
        touch-action: none;
    }

    .node-title {
        display: block;
        font-size: 0.875rem;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--cursor);
    }

    .action-btn {
        flex-shrink: 0;
        opacity: 0;
        background: none;
        border: none;
        font-size: 0.85rem;
        line-height: 1;
        cursor: pointer;
        padding: 0 0.15rem;
        transition: opacity 0.1s;
        color: var(--cursor);
    }

    .node-item:hover .action-btn,
    .log-item:hover .action-btn,
    .log-member-item:hover .action-btn {
        opacity: 1;
    }

    .fav-btn.fav-on {
        opacity: 1;
        color: var(--fg-warning);
    }

    .fav-btn:hover {
        color: var(--fg-warning) !important;
    }

    .delete-btn:hover {
        color: var(--fg-error);
    }

    .remove-btn:hover {
        color: var(--fg-error);
    }

    .empty-hint {
        padding: 0.4rem 0.75rem;
        font-size: 0.78rem;
        color: var(--cursor);
    }

    /* ── Log tree items ── */
    .log-item {
        display: flex;
        align-items: center;
        gap: 0.15rem;
        padding-right: 0.4rem;
        cursor: default;
    }

    .log-item:hover {
        background: var(--bg-hover);
    }

    .log-item.drag-over {
        background: var(--bg-active);
        outline: 1px dashed var(--fg-interactive);
        outline-offset: -1px;
    }

    .log-btn {
        flex: 1;
        background: none;
        border: none;
        text-align: left;
        cursor: pointer;
        padding: 0.35rem 0.3rem 0.35rem 0;
        display: flex;
        align-items: center;
        gap: 0.3rem;
        min-width: 0;
    }

    .log-arrow {
        font-size: 0.65rem;
        flex-shrink: 0;
        color: var(--cursor);
        width: 10px;
        text-align: center;
    }

    .log-name {
        font-size: 0.875rem;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--cursor);
    }

    .log-action {
        opacity: 0;
    }

    .log-rename-input {
        flex: 1;
        font-size: 0.875rem;
        border: 1px solid var(--fg-interactive);
        border-radius: 3px;
        padding: 0.1rem 0.25rem;
        background: var(--bg-primary);
        color: var(--fg-primary);
        outline: none;
        min-width: 0;
    }

    .log-rename-input::placeholder {
        color: var(--cursor);
    }

    .log-create {
        padding-top: 0.15rem;
        padding-bottom: 0.15rem;
    }

    .log-create-input {
        flex: 1;
        font-size: 0.875rem;
        border: 1px solid var(--fg-interactive);
        border-radius: 3px;
        padding: 0.2rem 0.4rem;
        background: var(--bg-primary);
        color: var(--fg-primary);
        outline: none;
        min-width: 0;
    }

    .log-create-input::placeholder {
        color: var(--cursor);
    }

    /* ── Inline log members ── */
    .log-member-item {
        display: flex;
        align-items: center;
        gap: 0.15rem;
        padding-right: 0.4rem;
        cursor: default;
        border-left: 2px solid var(--bg-border);
    }

    .log-member-item:hover {
        background: var(--bg-hover);
    }

    .log-member-item.drag-over {
        background: var(--bg-active);
        outline: 1px dashed var(--fg-interactive);
        outline-offset: -1px;
    }

    .log-member-item.active {
        background: var(--bg-active);
    }

    .log-member-item.active .node-btn {
        color: var(--fg-interactive);
    }

    /* ── Main pane ── */
    .main-pane {
        flex: 1;
        overflow: hidden;
        position: relative;
        display: flex;
        flex-direction: column;
    }

    /* ── Status bar ── */
    .statusbar {
        height: 24px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-top: 1px solid var(--bg-border);
        background: var(--bg-panel);
        padding: 0 0.5rem;
    }

    .statusbar-section {
        display: flex;
        align-items: center;
        gap: 0.1rem;
    }

    .mode-btn {
        background: none;
        border: none;
        font-size: 0.7rem;
        font-weight: 500;
        letter-spacing: 0.04em;
        color: var(--cursor);
        padding: 0.15rem 0.5rem;
        border-radius: 3px;
        cursor: pointer;
        user-select: none;
    }

    .mode-btn:hover {
        color: var(--fg-muted);
        background: var(--bg-hover);
    }

    .mode-btn.active {
        color: var(--fg-interactive);
    }

    .status-item {
        font-size: 0.7rem;
        color: var(--cursor);
        padding: 0 0.25rem;
    }

    /* ── Search panel ── */
    .sidebar-search {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .search-tabs {
        display: flex;
        border-bottom: 1px solid var(--bg-border);
        flex-shrink: 0;
    }

    .search-tab {
        flex: 1;
        background: none;
        border: none;
        padding: 0.4rem 0;
        font-size: 0.72rem;
        font-weight: 500;
        letter-spacing: 0.04em;
        color: var(--cursor);
        cursor: pointer;
        border-bottom: 2px solid transparent;
        margin-bottom: -1px;
        transition: color 0.1s;
    }

    .search-tab:hover {
        color: var(--fg-muted);
    }
    .search-tab.active {
        color: var(--fg-interactive);
        border-bottom-color: var(--fg-interactive);
    }

    .search-input-row {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.4rem 0.5rem;
        border-bottom: 1px solid var(--bg-border);
        flex-shrink: 0;
    }

    .search-input {
        flex: 1;
        background: var(--bg-hover);
        border: 1px solid transparent;
        border-radius: 3px;
        padding: 0.25rem 0.4rem;
        font-size: 0.82rem;
        color: var(--fg-muted);
        outline: none;
        min-width: 0;
    }

    .search-input:focus {
        border-color: var(--fg-interactive);
    }

    .search-input::placeholder {
        color: var(--cursor);
    }

    .search-opt-btn {
        flex-shrink: 0;
        background: none;
        border: 1px solid transparent;
        border-radius: 3px;
        padding: 0.2rem 0.35rem;
        font-size: 0.78rem;
        font-family: monospace;
        color: var(--cursor);
        cursor: pointer;
        transition:
            color 0.1s,
            background 0.1s;
    }

    .search-opt-btn:hover {
        color: var(--fg-muted);
        background: var(--bg-hover);
    }
    .search-opt-btn.active {
        color: var(--fg-interactive);
        background: var(--bg-active);
        border-color: var(--fg-interactive);
    }

    .search-results {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .search-status {
        padding: 0.6rem 0.75rem;
        font-size: 0.78rem;
        color: var(--cursor);
        margin: 0;
    }

    .search-error {
        color: var(--fg-error);
    }

    .search-hit {
        padding: 0.45rem 0.75rem;
        cursor: pointer;
        border-bottom: 1px solid var(--bg-border);
    }

    .search-hit:hover {
        background: var(--bg-hover);
    }
    .search-hit.active {
        background: var(--bg-active);
    }

    .search-hit-title {
        font-size: 0.82rem;
        color: var(--fg-muted);
        font-weight: 500;
        margin-bottom: 0.2rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .search-hit-snippet {
        font-size: 0.72rem;
        color: var(--cursor);
        line-height: 1.4;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    :global(.search-hit-snippet b) {
        color: var(--fg-warning);
        font-weight: 600;
    }

    .empty-state {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--cursor);
        font-size: 0.9rem;
    }
</style>
