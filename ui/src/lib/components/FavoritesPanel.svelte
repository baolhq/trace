<script lang="ts">
    import Panel from "./Panel.svelte";
    import { notes } from "$lib/stores/notes.svelte";
    import { logs } from "$lib/stores/logs.svelte";
</script>

<Panel title="Favorites">
    {#each notes.favorites as node (node.id)}
        <div
            class="node-item"
            class:active={notes.activeNodeId === node.id}
            role="option"
            aria-selected={notes.activeNodeId === node.id}
            tabindex="-1"
        >
            <button
                class="node-btn"
                onpointerdown={(e) => logs.onNodePointerDown(node.id, e)}
                onpointermove={(e) => logs.onNodePointerMove(e)}
                onpointerup={() => logs.onNodePointerUp(node.id)}
                onpointercancel={() => logs.onNodePointerCancel()}
            >
                <span class="node-title">{node.title}</span>
            </button>
            <button
                class="action-btn fav-btn fav-on"
                onclick={(e) => notes.toggleFavorite(node.id, e)}
                title="Unfavorite"
                tabindex="-1"
                >★
            </button>
            <button
                class="action-btn delete-btn"
                onclick={(e) => notes.deleteNode(node.id, e)}
                title="Delete"
                tabindex="-1"
                >×
            </button>
        </div>
    {/each}
</Panel>

<style>
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

    .node-item:hover .action-btn {
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
</style>
