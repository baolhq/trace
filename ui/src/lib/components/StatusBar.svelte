<script lang="ts">
    import TraceIcon from "@iconify-svelte/pajamas/file-tree";
    import SearchIcon from "@iconify-svelte/carbon/search";
    import OutlinesIcon from "@iconify-svelte/lucide/list-tree";

    let {
        sidebarMode,
        onModeChange,
        saving,
    }: {
        sidebarMode: "notes" | "search" | "outlines";
        onModeChange: (mode: "notes" | "search" | "outlines") => void;
        saving: boolean;
    } = $props();
</script>

<div class="statusbar">
    <div class="statusbar-section statusbar-left">
        <button
            class="mode-btn"
            class:active={sidebarMode === "notes"}
            onclick={() => onModeChange("notes")}
        >
            <TraceIcon height="1.5em" />
        </button>
        <button
            class="mode-btn"
            class:active={sidebarMode === "outlines"}
            onclick={() => onModeChange("outlines")}
        >
            <OutlinesIcon height="1.5em" />
        </button>
        <button
            class="mode-btn"
            class:active={sidebarMode === "search"}
            onclick={() => onModeChange("search")}
        >
            <SearchIcon height="1.5em" />
        </button>
    </div>
    <div class="statusbar-section statusbar-right">
        {#if saving}
            <span class="status-item">Saving…</span>
        {/if}
    </div>
</div>

<style>
    .statusbar {
        height: 32px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-top: 1px solid var(--bg-border);
        background: var(--bg-primary);
        padding: 0.5rem 0.5rem;
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
</style>
