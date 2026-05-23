<script lang="ts">
    import {invoke} from "@tauri-apps/api/core";
    import {listen} from "@tauri-apps/api/event";
    import {onDestroy, onMount} from "svelte";
    import Editor from "$lib/editor/Editor.svelte";
    import {tipTapToPmDoc, type PmDoc} from "$lib/editor/doc";

    interface NodeMeta {
        id: string;
        title: string;
        created_at: number;
        modified_at: number;
    }

    interface OpenNodeResponse {
        meta: NodeMeta;
        doc: PmDoc;
    }

    let nodes: NodeMeta[] = $state([]);
    let activeId: string | null = $state(null);
    let activeDoc: PmDoc | null = $state(null);
    let activeMeta: NodeMeta | null = $state(null);
    let newTitle = $state("");
    let error = $state("");
    let saving = $state(false);

    async function loadNodes() {
        try {
            nodes = await invoke("list_nodes");
        } catch (e) {
            error = String(e);
        }
    }

    async function openNode(id: string) {
        if (activeId === id) return;
        try {
            const res: OpenNodeResponse = await invoke("open_node", {id});
            activeId = id;
            activeMeta = res.meta;
            activeDoc = res.doc;
            error = "";
        } catch (e) {
            error = String(e);
        }
    }

    async function createNode() {
        const title = newTitle.trim();
        if (!title) return;
        try {
            const id: string = await invoke("create_node", {title});
            newTitle = "";
            await loadNodes();
            await openNode(id);
        } catch (e) {
            error = String(e);
        }
    }

    async function deleteNode(id: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("delete_node", {id});
            if (activeId === id) {
                activeId = null;
                activeDoc = null;
                activeMeta = null;
            }
            await loadNodes();
        } catch (e) {
            error = String(e);
        }
    }

    async function handleSave(ttJson: object) {
        if (!activeId) return;
        saving = true;
        try {
            const doc = tipTapToPmDoc(ttJson, activeDoc?.frontmatter);
            await invoke("save_node", {id: activeId, doc});
        } catch (e) {
            error = String(e);
        } finally {
            saving = false;
        }
    }

    function handleKey(e: KeyboardEvent) {
        if (e.key === "Enter") createNode();
    }

    let unlisten: (() => void) | undefined;

    onMount(async () => {
        await loadNodes();
        await invoke("frontend_ready");
        unlisten = await listen("nodes_changed", () => {
            loadNodes();
        });
    });

    onDestroy(() => unlisten?.());
</script>

<div class="shell">
    <!-- Sidebar -->
    <aside class="sidebar">
        <div class="sidebar-header">
            <span class="app-name">Trace</span>
        </div>

        <div class="create-row">
            <input
                    bind:value={newTitle}
                    onkeydown={handleKey}
                    placeholder="New note..."
            />
            <button onclick={createNode} disabled={!newTitle.trim()} title="Create">+</button>
        </div>

        {#if error}
            <p class="sidebar-error">{error}</p>
        {/if}

        <ul class="node-list">
            {#each nodes as node (node.id)}
                <li class="node-item" class:active={activeId === node.id}>
                    <button class="node-btn" onclick={() => openNode(node.id)}>
                        <span class="node-title">{node.title}</span>
                    </button>
                    <button
                            class="delete-btn"
                            onclick={(e) => deleteNode(node.id, e)}
                            title="Delete"
                            tabindex="-1"
                    >×
                    </button>
                </li>
            {/each}

            {#if nodes.length === 0}
                <li class="empty-hint">No notes yet.</li>
            {/if}
        </ul>
    </aside>

    <!-- Main pane -->
    <main class="main-pane">
        {#if activeDoc && activeId}
            {#if saving}
                <div class="save-indicator">Saving…</div>
            {/if}
            {#key activeId}
                <Editor
                        nodeId={activeId}
                        doc={activeDoc}
                        onSave={handleSave}
                />
            {/key}
        {:else}
            <div class="empty-state">
                <p>Select a note or create one to start writing.</p>
            </div>
        {/if}
    </main>
</div>

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
        display: flex;
        flex-direction: column;
        background: #fafafa;
        overflow: hidden;
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

    .node-list {
        flex: 1;
        overflow-y: auto;
        list-style: none;
        margin: 0;
        padding: 0.25rem 0;
    }

    .node-item {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0 0.4rem 0 0;
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
        padding: 0.4rem 0.5rem 0.4rem 0.75rem;
        overflow: hidden;
        min-width: 0;
    }

    .node-title {
        display: block;
        font-size: 0.875rem;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: inherit;
    }

    .delete-btn {
        flex-shrink: 0;
        opacity: 0;
        background: none;
        border: none;
        font-size: 1rem;
        line-height: 1;
        color: #999;
        cursor: pointer;
        padding: 0 0.1rem;
        transition: opacity 0.1s;
    }

    .node-item:hover .delete-btn {
        opacity: 1;
    }

    .delete-btn:hover {
        color: #c00;
    }

    .empty-hint {
        padding: 0.5rem 0.75rem;
        font-size: 0.8rem;
        color: #aaa;
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
