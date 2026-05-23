<script lang="ts">
    import {invoke} from "@tauri-apps/api/core";
    import {listen} from "@tauri-apps/api/event";
    import {onDestroy, onMount} from "svelte";
    import Editor from "$lib/editor/Editor.svelte";
    import {tipTapToPmDoc, type PmDoc} from "$lib/editor/doc";

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
        | { type: 'log'; log: LogTreeNode; depth: number }
        | { type: 'member'; node: NodeInfo; logId: number; depth: number };

    type ViewMode = { kind: 'editor' } | null;

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

    let newTitle = $state('');
    let newLogName = $state('');
    let creatingLog = $state(false);
    let renamingLogId: number | null = $state(null);
    let renamingLogName = $state('');

    // Panel collapse state
    let favoritesOpen = $state(true);
    let logsOpen = $state(true);
    let recentsOpen = $state(true);

    // Pointer-based drag (HTML5 DnD is unreliable in WebView2).
    let ptrDragId: string | null = null;
    let ptrStartX = 0;
    let ptrStartY = 0;
    let ptrDragging = false;
    let ptrCaptureEl: HTMLElement | null = null;
    let dragOverLogId: number | null = $state(null);

    let error = $state('');
    let saving = $state(false);


    // ── Derived ────────────────────────────────────────────────────────────

    const logTree: LogTreeNode[] = $derived(buildTree(allLogs, null));
    const flatItems: FlatSidebarItem[] = $derived(flattenTree(logTree, expandedLogs, logMembersMap));

    function buildTree(logs: Log[], parentId: number | null): LogTreeNode[] {
        return logs
            .filter(l => l.parent_id === parentId)
            .sort((a, b) => a.sort_key - b.sort_key)
            .map(l => ({...l, children: buildTree(logs, l.id)}));
    }

    function flattenTree(
        nodes: LogTreeNode[],
        expanded: Set<number>,
        membersMap: Record<number, NodeInfo[]>,
        depth = 0
    ): FlatSidebarItem[] {
        const result: FlatSidebarItem[] = [];
        for (const node of nodes) {
            result.push({type: 'log', log: node, depth});
            if (expanded.has(node.id)) {
                result.push(...flattenTree(node.children, expanded, membersMap, depth + 1));
                for (const member of (membersMap[node.id] ?? [])) {
                    result.push({type: 'member', node: member, logId: node.id, depth: depth + 1});
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
            const raw = await invoke<NodeInfo[]>('list_nodes');
            if (gen !== loadGen) return;
            const seen = new Set<string>();
            recentNodes = raw.filter(n => !seen.has(n.id) && seen.add(n.id) as unknown as boolean);
        } catch (e) {
            if (gen === loadGen) error = String(e);
        }
    }

    async function loadFavorites() {
        try {
            favorites = await invoke<NodeInfo[]>('list_favorites');
        } catch {
            // non-critical
        }
    }

    async function loadLogs() {
        try {
            allLogs = await invoke<Log[]>('get_log_tree');
        } catch (e) {
            error = String(e);
        }
    }

    async function loadLogMembers(logId: number) {
        try {
            const members = await invoke<NodeInfo[]>('get_log_members', {logId, page: 0});
            logMembersMap = {...logMembersMap, [logId]: members};
        } catch (e) {
            error = String(e);
        }
    }

    // ── Node actions ───────────────────────────────────────────────────────

    async function openNode(id: string) {
        if (activeNodeId === id && viewMode?.kind === 'editor') return;
        const hit = recentNodes.find(n => n.id === id);
        if (hit) recentNodes = [hit, ...recentNodes.filter(n => n.id !== id)];
        try {
            const res = await invoke<OpenNodeResponse>('open_node', {id});
            activeNodeId = id;
            activeMeta = res.meta;
            activeDoc = res.doc;
            viewMode = {kind: 'editor'};
            error = '';
        } catch (e) {
            error = String(e);
            loadRecents();
        }
    }

    async function createNode() {
        const title = newTitle.trim();
        if (!title) return;
        try {
            const id = await invoke<string>('create_node', {title});
            newTitle = '';
            await loadRecents();
            await openNode(id);
        } catch (e) {
            error = String(e);
        }
    }

    async function deleteNode(id: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke('delete_node', {id});
            if (activeNodeId === id) {
                activeNodeId = null;
                activeDoc = null;
                activeMeta = null;
                viewMode = null;
            }
            await Promise.all([loadRecents(), loadFavorites()]);
            const updated: Record<number, NodeInfo[]> = {};
            for (const [key, members] of Object.entries(logMembersMap)) {
                updated[Number(key)] = (members as NodeInfo[]).filter(n => n.id !== id);
            }
            logMembersMap = updated;
        } catch (e) {
            error = String(e);
        }
    }

    async function toggleFavorite(id: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            const newState = await invoke<boolean>('toggle_favorite', {id});
            await loadFavorites();
            recentNodes = recentNodes.map(n =>
                n.id === id ? {...n, is_favorite: newState} : n
            );
            const updated: Record<number, NodeInfo[]> = {};
            for (const [key, members] of Object.entries(logMembersMap)) {
                updated[Number(key)] = (members as NodeInfo[]).map(n =>
                    n.id === id ? {...n, is_favorite: newState} : n
                );
            }
            logMembersMap = updated;
        } catch (e) {
            error = String(e);
        }
    }

    async function handleSave(ttJson: object) {
        if (!activeNodeId) return;
        saving = true;
        try {
            const doc = tipTapToPmDoc(ttJson, activeDoc?.frontmatter);
            await invoke('save_node', {id: activeNodeId, doc});
        } catch (e) {
            error = String(e);
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
            await invoke('create_log', {name, parentId: null});
            newLogName = '';
            creatingLog = false;
            await loadLogs();
        } catch (e) {
            error = String(e);
        }
    }

    async function deleteLog(id: number, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke('delete_log', {id});
            expandedLogs.delete(id);
            expandedLogs = new Set(expandedLogs);
            const newMap = {...logMembersMap};
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
                await invoke('rename_log', {id: renamingLogId, name});
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
            await invoke('remove_from_log', {logId, nodeId});
            const current = logMembersMap[logId] ?? [];
            logMembersMap = {...logMembersMap, [logId]: current.filter(n => n.id !== nodeId)};
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
            const moved = Math.abs(e.clientX - ptrStartX) + Math.abs(e.clientY - ptrStartY);
            if (moved < 6) return;
            ptrDragging = true;
        }
        // data-log-id is on both log headers and member rows, so the whole
        // expanded section is a valid drop target.
        const target = document.elementFromPoint(e.clientX, e.clientY);
        const logEl = target?.closest('[data-log-id]') as HTMLElement | null;
        const found = logEl ? Number(logEl.dataset.logId) : null;
        dragOverLogId = found;
        // During pointer capture WebView2 uses the capturing element's cursor,
        // so we must set it inline directly on that element.
        if (ptrCaptureEl) ptrCaptureEl.style.cursor = found !== null ? 'copy' : 'not-allowed';
    }

    async function onNodePointerUp(id: string) {
        const wasDragging = ptrDragging;
        const targetLogId = dragOverLogId;
        ptrDragId = null;
        ptrDragging = false;
        dragOverLogId = null;
        if (ptrCaptureEl) {
            ptrCaptureEl.style.cursor = '';
            ptrCaptureEl = null;
        }
        if (wasDragging) {
            if (targetLogId !== null) {
                try {
                    await invoke('add_to_log', {logId: targetLogId, nodeId: id});
                    if (expandedLogs.has(targetLogId)) {
                        await loadLogMembers(targetLogId);
                    } else {
                        const newMap = {...logMembersMap};
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
            ptrCaptureEl.style.cursor = '';
            ptrCaptureEl = null;
        }
    }

    // ── Keyboard ───────────────────────────────────────────────────────────

    function handleCreateKey(e: KeyboardEvent) {
        if (e.key === 'Enter') createNode();
    }

    function handleNewLogKey(e: KeyboardEvent) {
        if (e.key === 'Enter') createLog();
        if (e.key === 'Escape') {
            creatingLog = false;
            newLogName = '';
        }
    }

    function handleRenameLogKey(e: KeyboardEvent) {
        if (e.key === 'Enter') commitRenameLog();
        if (e.key === 'Escape') renamingLogId = null;
    }

    // ── Lifecycle ──────────────────────────────────────────────────────────

    let unlisten: (() => void) | undefined;

    onMount(async () => {
        await Promise.all([loadRecents(), loadFavorites(), loadLogs()]);
        await invoke('frontend_ready');
        unlisten = await listen('nodes_changed', () => {
            loadRecents();
            loadFavorites();
        });
    });

    onDestroy(() => unlisten?.());
</script>

<div class="shell">
    <!-- ── Sidebar ── -->
    <aside class="sidebar">
        <div class="sidebar-fixed">
            <div class="sidebar-header">
                <span class="app-name">Trace</span>
            </div>

            <div class="create-row">
                <input
                        bind:value={newTitle}
                        onkeydown={handleCreateKey}
                        placeholder="New note…"
                />
                <button onclick={createNode} disabled={!newTitle.trim()} title="Create">+</button>
            </div>

            {#if error}
                <p class="sidebar-error">{error}</p>
            {/if}
        </div>

        <!-- Three collapsible panels; each open panel gets equal flex share -->
        <div class="sidebar-panels">

            <!-- Favorites panel -->
            <div class="panel" class:open={favoritesOpen}>
                <div class="panel-header" role="button" tabindex="0"
                     onclick={() => (favoritesOpen = !favoritesOpen)}
                     onkeydown={(e) => e.key === 'Enter' && (favoritesOpen = !favoritesOpen)}>
                    <span class="panel-arrow">{favoritesOpen ? '▾' : '▸'}</span>
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
                                        onpointerdown={(e) => onNodePointerDown(node.id, e)}
                                        onpointermove={onNodePointerMove}
                                        onpointerup={() => onNodePointerUp(node.id)}
                                        onpointercancel={onNodePointerCancel}
                                >
                                    <span class="node-title">{node.title}</span>
                                </button>
                                <button
                                        class="action-btn fav-btn fav-on"
                                        onclick={(e) => toggleFavorite(node.id, e)}
                                        title="Unfavorite"
                                        tabindex="-1"
                                >★
                                </button>
                                <button
                                        class="action-btn delete-btn"
                                        onclick={(e) => deleteNode(node.id, e)}
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
                <div class="panel-header" role="button" tabindex="0"
                     onclick={() => (logsOpen = !logsOpen)}
                     onkeydown={(e) => e.key === 'Enter' && (logsOpen = !logsOpen)}>
                    <span class="panel-arrow">{logsOpen ? '▾' : '▸'}</span>
                    <span class="panel-title">Logs</span>
                    <button class="section-add"
                            onclick={(e) => { e.stopPropagation(); creatingLog = true; }}
                            title="New log">+
                    </button>
                </div>
                {#if logsOpen}
                    <div class="panel-body">
                        {#each flatItems as item (item.type === 'log' ? `log-${item.log.id}` : `member-${item.logId}-${item.node.id}`)}
                            {#if item.type === 'log'}
                                <div
                                        class="log-item"
                                        class:drag-over={dragOverLogId === item.log.id}
                                        role="treeitem"
                                        aria-expanded={expandedLogs.has(item.log.id)}
                                        aria-selected={false}
                                        tabindex="-1"
                                        data-log-id={item.log.id}
                                        style="padding-left: {0.75 + item.depth * 0.85}rem"
                                >
                                    <button class="log-btn" onclick={() => openLog(item.log)}>
                                    <span class="log-arrow">
                                        {expandedLogs.has(item.log.id) ? '▾' : '▸'}
                                    </span>
                                        {#if renamingLogId === item.log.id}
                                            <input
                                                    class="log-rename-input"
                                                    bind:value={renamingLogName}
                                                    onkeydown={handleRenameLogKey}
                                                    onblur={commitRenameLog}
                                                    onclick={(e) => e.stopPropagation()}
                                                    use:focusOnMount
                                            />
                                        {:else}
                                            <span class="log-name">{item.log.name}</span>
                                        {/if}
                                    </button>
                                    <button
                                            class="action-btn log-action"
                                            onclick={(e) => startRenameLog(item.log, e)}
                                            title="Rename"
                                            tabindex="-1"
                                    >…
                                    </button>
                                    <button
                                            class="action-btn delete-btn log-action"
                                            onclick={(e) => deleteLog(item.log.id, e)}
                                            title="Delete log"
                                            tabindex="-1"
                                    >×
                                    </button>
                                </div>
                            {:else}
                                <div
                                        class="log-member-item"
                                        class:active={activeNodeId === item.node.id}
                                        class:drag-over={dragOverLogId === item.logId}
                                        role="option"
                                        aria-selected={activeNodeId === item.node.id}
                                        tabindex="-1"
                                        data-log-id={item.logId}
                                        style="padding-left: {0.75 + item.depth * 0.85}rem"
                                >
                                    <button
                                            class="node-btn"
                                            onpointerdown={(e) => onNodePointerDown(item.node.id, e)}
                                            onpointermove={onNodePointerMove}
                                            onpointerup={() => onNodePointerUp(item.node.id)}
                                            onpointercancel={onNodePointerCancel}
                                    >
                                        <span class="node-title">{item.node.title}</span>
                                    </button>
                                    <button
                                            class="action-btn fav-btn"
                                            class:fav-on={item.node.is_favorite}
                                            onclick={(e) => toggleFavorite(item.node.id, e)}
                                            title={item.node.is_favorite ? 'Unfavorite' : 'Favorite'}
                                            tabindex="-1"
                                    >★
                                    </button>
                                    <button
                                            class="action-btn remove-btn"
                                            onclick={(e) => removeFromLog(item.logId, item.node.id, e)}
                                            title="Remove from log"
                                            tabindex="-1"
                                    >×
                                    </button>
                                </div>
                            {/if}
                        {/each}

                        {#if creatingLog}
                            <div class="log-item log-create" style="padding-left: 0.75rem">
                                <input
                                        class="log-create-input"
                                        bind:value={newLogName}
                                        onkeydown={handleNewLogKey}
                                        onblur={() => { if (!newLogName.trim()) creatingLog = false; }}
                                        placeholder="Log name…"
                                        use:focusOnMount
                                />
                            </div>
                        {:else if flatItems.filter(i => i.type === 'log').length === 0}
                            <div class="empty-hint">Drag notes here to organize.</div>
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- Recents panel (no scroll — fade mask shows overflow) -->
            <div class="panel" class:open={recentsOpen}>
                <div class="panel-header" role="button" tabindex="0"
                     onclick={() => (recentsOpen = !recentsOpen)}
                     onkeydown={(e) => e.key === 'Enter' && (recentsOpen = !recentsOpen)}>
                    <span class="panel-arrow">{recentsOpen ? '▾' : '▸'}</span>
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
                                        onpointerdown={(e) => onNodePointerDown(node.id, e)}
                                        onpointermove={onNodePointerMove}
                                        onpointerup={() => onNodePointerUp(node.id)}
                                        onpointercancel={onNodePointerCancel}
                                >
                                    <span class="node-title">{node.title}</span>
                                </button>
                                <button
                                        class="action-btn fav-btn"
                                        class:fav-on={node.is_favorite}
                                        onclick={(e) => toggleFavorite(node.id, e)}
                                        title={node.is_favorite ? 'Unfavorite' : 'Favorite'}
                                        tabindex="-1"
                                >★
                                </button>
                                <button
                                        class="action-btn delete-btn"
                                        onclick={(e) => deleteNode(node.id, e)}
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
    </aside>

    <!-- ── Main pane ── -->
    <main class="main-pane">
        {#if viewMode?.kind === 'editor' && activeDoc && activeNodeId}
            {#if saving}
                <div class="save-indicator">Saving…</div>
            {/if}
            <Editor
                    nodeId={activeNodeId}
                    doc={activeDoc}
                    onSave={handleSave}
            />
        {:else}
            <div class="empty-state">
                <p>Select a note or create one to start writing.</p>
            </div>
        {/if}
    </main>
</div>

<!-- Svelte action: focus an element when it mounts -->
<script lang="ts" module>
    function focusOnMount(el: HTMLElement) {
        el.focus();
    }
</script>

<style>
    :global(*, *::before, *::after) {
        box-sizing: border-box;
    }

    :global(body) {
        margin: 0;
        font-family: system-ui, -apple-system, sans-serif;
        background: #fff;
        color: #1a1a1a;
    }

    .shell {
        display: flex;
        height: 100vh;
        overflow: hidden;
    }

    /* ── Sidebar ── */
    .sidebar {
        width: 240px;
        flex-shrink: 0;
        border-right: 1px solid #e8e8e8;
        background: #fafafa;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .sidebar-fixed {
        flex-shrink: 0;
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
        border-top: 1px solid #e8e8e8;
    }

    .panel.open {
        flex: 1 1 0;
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
        background: #f0f0f0;
    }

    .panel-arrow {
        font-size: 0.65rem;
        color: #999;
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
        color: #aaa;
    }

    .panel-body {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: #ddd transparent;
    }

    .panel-body.no-scroll {
        overflow: hidden;
        -webkit-mask-image: linear-gradient(to bottom, black 80%, transparent 100%);
        mask-image: linear-gradient(to bottom, black 80%, transparent 100%);
    }

    .sidebar-header {
        padding: 1rem 1rem 0.5rem;
        border-bottom: 1px solid #e8e8e8;
    }

    .app-name {
        font-size: 0.85rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: #888;
    }

    .create-row {
        display: flex;
        gap: 0.4rem;
        padding: 0.6rem 0.75rem;
    }

    .create-row input {
        flex: 1;
        padding: 0.3rem 0.5rem;
        font-size: 0.85rem;
        border: 1px solid #ddd;
        border-radius: 4px;
        background: #fff;
        outline: none;
    }

    .create-row input:focus {
        border-color: #4361ee;
    }

    .create-row button {
        padding: 0.3rem 0.55rem;
        font-size: 1rem;
        line-height: 1;
        border: 1px solid #ddd;
        border-radius: 4px;
        background: #fff;
        cursor: pointer;
        color: #555;
    }

    .create-row button:hover:not(:disabled) {
        background: #f0f0f0;
    }

    .create-row button:disabled {
        opacity: 0.4;
        cursor: default;
    }

    .sidebar-error {
        font-size: 0.75rem;
        color: #c00;
        padding: 0 0.75rem;
        margin: 0 0 0.5rem;
    }

    .section-add {
        margin-left: auto;
        background: none;
        border: none;
        font-size: 1rem;
        line-height: 1;
        color: #bbb;
        cursor: pointer;
        padding: 0 0.1rem;
    }

    .section-add:hover {
        color: #555;
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
        background: #efefef;
    }

    .node-item.active {
        background: #e8eeff;
    }

    .node-item.active .node-btn {
        color: #2d3fe6;
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
        color: inherit;
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
        color: #bbb;
    }

    .node-item:hover .action-btn,
    .log-item:hover .action-btn,
    .log-member-item:hover .action-btn {
        opacity: 1;
    }

    .fav-btn.fav-on {
        opacity: 1;
        color: #f5a623;
    }

    .fav-btn:hover {
        color: #f5a623 !important;
    }

    .delete-btn:hover {
        color: #c00;
    }

    .remove-btn:hover {
        color: #c00;
    }

    .empty-hint {
        padding: 0.4rem 0.75rem;
        font-size: 0.78rem;
        color: #bbb;
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
        background: #efefef;
    }

    .log-item.drag-over {
        background: #dce4ff;
        outline: 1px dashed #4361ee;
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
        color: #999;
        width: 10px;
        text-align: center;
    }

    .log-name {
        font-size: 0.875rem;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .log-action {
        opacity: 0;
    }

    .log-rename-input {
        flex: 1;
        font-size: 0.875rem;
        border: 1px solid #4361ee;
        border-radius: 3px;
        padding: 0.1rem 0.25rem;
        background: #fff;
        outline: none;
        min-width: 0;
    }

    .log-create {
        padding-top: 0.15rem;
        padding-bottom: 0.15rem;
    }

    .log-create-input {
        flex: 1;
        font-size: 0.875rem;
        border: 1px solid #4361ee;
        border-radius: 3px;
        padding: 0.2rem 0.4rem;
        background: #fff;
        outline: none;
        min-width: 0;
    }

    /* ── Inline log members ── */
    .log-member-item {
        display: flex;
        align-items: center;
        gap: 0.15rem;
        padding-right: 0.4rem;
        cursor: default;
        border-left: 2px solid #e8e8e8;
    }

    .log-member-item:hover {
        background: #efefef;
    }

    .log-member-item.drag-over {
        background: #dce4ff;
        outline: 1px dashed #4361ee;
        outline-offset: -1px;
    }

    .log-member-item.active {
        background: #e8eeff;
        border-left-color: #4361ee;
    }

    .log-member-item.active .node-btn {
        color: #2d3fe6;
    }

    /* ── Main pane ── */
    .main-pane {
        flex: 1;
        overflow: hidden;
        position: relative;
        display: flex;
        flex-direction: column;
    }

    .save-indicator {
        position: absolute;
        top: 0.75rem;
        right: 1rem;
        font-size: 0.75rem;
        color: #aaa;
        pointer-events: none;
        z-index: 1;
    }

    .empty-state {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #aaa;
        font-size: 0.9rem;
    }
</style>
