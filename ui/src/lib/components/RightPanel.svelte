<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onDestroy } from "svelte";
    import LinkIcon from "@iconify-svelte/lucide/link";
    import BacklinkIcon from "@iconify-svelte/lucide/link-2";
    import { notes } from "$lib/stores/notes.svelte";

    let {
        mode,
        nodeId,
    }: {
        mode: "links" | "backlinks";
        nodeId: string | null;
    } = $props();

    interface LinkInfo {
        node_id: string | null;
        title: string | null;
        target_raw: string;
        link_type: number;
    }

    let items: LinkInfo[] = $state([]);

    async function load() {
        if (!nodeId) {
            items = [];
            return;
        }
        try {
            const cmd = mode === "links" ? "get_links" : "get_backlinks";
            items = await invoke<LinkInfo[]>(cmd, { nodeId });
        } catch {
            items = [];
        }
    }

    $effect(() => {
        void load();
    });

    const unlistenPromise = listen<string>("links_updated", (event) => {
        if (event.payload === nodeId) void load();
    });

    onDestroy(() => {
        unlistenPromise.then((off) => off());
    });

    function displayTitle(item: LinkInfo): string {
        return item.title ?? item.target_raw;
    }

    function openItem(item: LinkInfo) {
        if (item.node_id) notes.openNode(item.node_id);
    }
</script>

<aside class="right-panel">
    <div class="rp-body">
        {#if items.length === 0}
            {#if mode === "links"}
                <div class="empty-state">
                    <LinkIcon height="1.25em" class="empty-icon" />
                    <p class="empty-title">No outgoing links</p>
                    <p class="empty-hint">
                        Use <code>[[Note title]]</code> in the editor to link to another
                        note.
                    </p>
                </div>
            {:else}
                <div class="empty-state">
                    <BacklinkIcon height="1.25em" class="empty-icon" />
                    <p class="empty-title">No backlinks</p>
                    <p class="empty-hint">
                        Other notes that link to this one will appear here.
                    </p>
                </div>
            {/if}
        {:else}
            <ul class="link-list">
                {#each items as item (item.target_raw + (item.node_id ?? ""))}
                    <li>
                        <button
                            class="link-item"
                            class:unresolved={!item.node_id}
                            onclick={() => openItem(item)}
                            disabled={!item.node_id}
                        >
                            {displayTitle(item)}
                            {#if !item.node_id}
                                <span class="unresolved-badge">missing</span>
                            {/if}
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </div>
</aside>

<style>
    .right-panel {
        width: 240px;
        flex-shrink: 0;
        border-left: 1px solid var(--bg-border);
        background: var(--bg-primary);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .rp-body {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        gap: 0.5rem;
        padding: 2rem 1.25rem;
        text-align: center;
        color: var(--cursor);
    }

    :global(.empty-icon) {
        opacity: 0.4;
        margin-bottom: 0.25rem;
    }

    .empty-title {
        margin: 0;
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--fg-muted);
    }

    .empty-hint {
        margin: 0;
        font-size: 0.72rem;
        line-height: 1.5;
        color: var(--cursor);
    }

    .empty-hint code {
        font-family: monospace;
        font-size: 0.7rem;
        background: var(--bg-hover);
        padding: 0.1em 0.3em;
        border-radius: 3px;
    }

    .link-list {
        list-style: none;
        margin: 0;
        padding: 0.25rem 0;
    }

    .link-list li {
        display: flex;
    }

    .link-item {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 0.4rem;
        background: none;
        border: none;
        text-align: left;
        padding: 0.3rem 0.75rem;
        font-size: 0.8rem;
        color: var(--fg-primary);
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .link-item:hover:not(:disabled) {
        background: var(--bg-hover);
        color: var(--fg-interactive);
    }

    .link-item.unresolved {
        color: var(--cursor);
        cursor: default;
    }

    .unresolved-badge {
        flex-shrink: 0;
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--cursor);
        background: var(--bg-hover);
        border-radius: 3px;
        padding: 0.1em 0.35em;
    }
</style>
