<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { focusOnMount } from "$lib/actions";
    import { notes } from "$lib/stores/notes.svelte";
    import type { SearchHit, SearchSubMode } from "$lib/types";

    let searchSubMode: SearchSubMode = $state("search");
    let searchQuery = $state("");
    let replaceQuery = $state("");
    let searchMatchCase = $state(false);
    let searchWholeWord = $state(false);
    let searchRegex = $state(false);
    let searchResults: SearchHit[] = $state([]);
    let searchLoading = $state(false);
    let searchError: string | null = $state(null);

    let searchDebounce: ReturnType<typeof setTimeout> | null = null;

    $effect(() => {
        if (searchDebounce) clearTimeout(searchDebounce);
        const q = searchQuery;
        const rx = searchRegex;
        const mc = searchMatchCase;
        const ww = searchWholeWord;
        if (!q.trim()) {
            searchResults = [];
            searchError = null;
            return;
        }
        searchDebounce = setTimeout(() => runSearch(q, rx, mc, ww), 350);
    });

    async function runSearch(
        q: string,
        isRegex: boolean,
        matchCase: boolean,
        wholeWord: boolean,
    ) {
        searchLoading = true;
        searchError = null;
        try {
            searchResults = await invoke("search_nodes", {
                query: q,
                isRegex,
                matchCase,
                wholeWord,
            });
        } catch (e) {
            searchError = String(e);
            searchResults = [];
        } finally {
            searchLoading = false;
        }
    }

    function handleSearchKey(e: KeyboardEvent) {
        if (e.key === "Enter")
            runSearch(
                searchQuery,
                searchRegex,
                searchMatchCase,
                searchWholeWord,
            );
    }
</script>

<div class="sidebar-search">
    <div class="search-tabs">
        <button
            class="search-tab"
            class:active={searchSubMode === "search"}
            onclick={() => (searchSubMode = "search")}>Search</button
        >
        <button
            class="search-tab"
            class:active={searchSubMode === "replace"}
            onclick={() => (searchSubMode = "replace")}>Replace</button
        >
    </div>

    <div
        class="search-input-row"
        class:error={!!searchError}
        data-error={searchError ?? undefined}
    >
        <input
            class="search-input"
            bind:value={searchQuery}
            placeholder="Search notes…"
            onkeydown={handleSearchKey}
            use:focusOnMount
        />
        <button
            class="search-opt-btn"
            class:active={searchMatchCase}
            onclick={() => (searchMatchCase = !searchMatchCase)}
            title="Match case">Aa</button
        >
        <button
            class="search-opt-btn"
            class:active={searchWholeWord}
            onclick={() => (searchWholeWord = !searchWholeWord)}
            title="Match whole word">ab</button
        >
        <button
            class="search-opt-btn"
            class:active={searchRegex}
            onclick={() => (searchRegex = !searchRegex)}
            title="Use regular expression">.*</button
        >
    </div>

    {#if searchSubMode === "replace"}
        <div class="search-input-row">
            <input
                class="search-input"
                bind:value={replaceQuery}
                placeholder="Replace with…"
            />
        </div>
    {/if}

    <div class="search-results">
        {#if searchLoading}
            <p class="search-status">Searching…</p>
        {:else if searchResults.length === 0 && searchQuery.trim() && !searchError}
            <p class="search-status">No results</p>
        {:else}
            {#each searchResults as hit (hit.id)}
                <div
                    class="search-hit"
                    class:active={notes.activeNodeId === hit.id}
                    role="option"
                    aria-selected={notes.activeNodeId === hit.id}
                    tabindex="0"
                    onclick={() => notes.openNode(hit.id)}
                    onkeydown={(e) =>
                        e.key === "Enter" && notes.openNode(hit.id)}
                >
                    <div class="search-hit-title">{hit.title}</div>
                    <div class="search-hit-snippet">
                        {@html hit.snippet}
                    </div>
                </div>
            {/each}
        {/if}
    </div>
</div>

<style>
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

    .search-tab:first-child {
        border-right: 1px solid var(--bg-border);
    }

    .search-tab:hover {
        color: var(--fg-muted);
    }

    .search-tab.active {
        color: var(--fg-muted);
        border-bottom-color: var(--bg-primary);
    }

    .search-input-row {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.4rem 0.5rem;
        border-bottom: 1px solid var(--bg-border);
        flex-shrink: 0;
        position: relative;
    }

    .search-input-row.error .search-input {
        border-color: var(--fg-error);
    }

    .search-input-row.error:hover::after {
        content: attr(data-error);
        position: absolute;
        top: calc(100% + 2px);
        left: 0;
        right: 0;
        background: var(--bg-primary);
        border: 1px solid var(--fg-error);
        border-radius: 3px;
        padding: 0.3rem 0.5rem;
        font-size: 0.73rem;
        color: var(--fg-error);
        z-index: 20;
        white-space: pre-wrap;
        word-break: break-all;
        pointer-events: none;
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
</style>
