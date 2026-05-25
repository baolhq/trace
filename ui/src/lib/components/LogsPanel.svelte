<script lang="ts">
    import Panel from "./Panel.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { focusOnMount } from "$lib/actions";
    import { notes } from "$lib/stores/notes.svelte";
    import { logs, buildTree, flattenTree } from "$lib/stores/logs.svelte";

    const logTree = $derived(buildTree(logs.allLogs, null));
    const flatItems = $derived(
        flattenTree(logTree, logs.expandedLogs, logs.logMembersMap),
    );

    let newLogName = $state("");
    let creatingLog = $state(false);
    let renamingLogId: number | null = $state(null);
    let renamingLogName = $state("");

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
            await logs.loadLogs();
        } catch (e) {
            notes.error = String(e);
        }
    }

    async function deleteLog(id: number, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("delete_log", { id });
            logs.expandedLogs.delete(id);
            logs.expandedLogs = new Set(logs.expandedLogs);
            const newMap = { ...logs.logMembersMap };
            delete newMap[id];
            logs.logMembersMap = newMap;
            await logs.loadLogs();
        } catch (e) {
            notes.error = String(e);
        }
    }

    function startRenameLog(log: { id: number; name: string }, e: MouseEvent) {
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
                await logs.loadLogs();
            } catch (e) {
                notes.error = String(e);
            }
        }
        renamingLogId = null;
    }

    async function removeFromLog(logId: number, nodeId: string, e: MouseEvent) {
        e.stopPropagation();
        try {
            await invoke("remove_from_log", { logId, nodeId });
            const current = logs.logMembersMap[logId] ?? [];
            logs.logMembersMap = {
                ...logs.logMembersMap,
                [logId]: current.filter((n) => n.id !== nodeId),
            };
        } catch (e) {
            notes.error = String(e);
        }
    }

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
</script>

<Panel title="Logs">
    {#snippet actions()}
        <button
            class="section-add"
            onclick={(e) => {
                e.stopPropagation();
                creatingLog = true;
            }}
            title="New log"
            >+
        </button>
    {/snippet}

    {#each flatItems as item (item.type === "log" ? `log-${item.log.id}` : `member-${item.logId}-${item.node.id}`)}
        {#if item.type === "log"}
            <div
                class="log-item"
                class:drag-over={logs.dragOverLogId === item.log.id}
                role="treeitem"
                aria-expanded={logs.expandedLogs.has(item.log.id)}
                aria-selected={false}
                tabindex="-1"
                data-log-id={item.log.id}
                style="padding-left: {0.75 + item.depth * 0.85}rem"
            >
                <button class="log-btn" onclick={() => logs.openLog(item.log)}>
                    <span class="log-arrow">
                        {logs.expandedLogs.has(item.log.id) ? "▾" : "▸"}
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
                class:active={notes.activeNodeId === item.node.id}
                class:drag-over={logs.dragOverLogId === item.logId}
                role="option"
                aria-selected={notes.activeNodeId === item.node.id}
                tabindex="-1"
                data-log-id={item.logId}
                style="padding-left: {0.75 + item.depth * 0.85}rem"
            >
                <button
                    class="node-btn"
                    onpointerdown={(e) =>
                        logs.onNodePointerDown(item.node.id, e)}
                    onpointermove={(e) => logs.onNodePointerMove(e)}
                    onpointerup={() => logs.onNodePointerUp(item.node.id)}
                    onpointercancel={() => logs.onNodePointerCancel()}
                >
                    <span class="node-title">{item.node.title}</span>
                </button>
                <button
                    class="action-btn fav-btn"
                    class:fav-on={item.node.is_favorite}
                    onclick={(e) => notes.toggleFavorite(item.node.id, e)}
                    title={item.node.is_favorite ? "Unfavorite" : "Favorite"}
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
                onblur={() => {
                    if (!newLogName.trim()) creatingLog = false;
                }}
                placeholder="Log name…"
                use:focusOnMount
            />
        </div>
    {:else if flatItems.filter((i) => i.type === "log").length === 0}
        <div class="empty-hint">Drag notes here to organize.</div>
    {/if}
</Panel>

<style>
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

    .empty-hint {
        padding: 0.4rem 0.75rem;
        font-size: 0.78rem;
        color: var(--cursor);
    }

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
</style>
