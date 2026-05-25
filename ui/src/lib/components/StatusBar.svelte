<script lang="ts">
    import TraceIcon from "@iconify-svelte/pajamas/file-tree";
    import SearchIcon from "@iconify-svelte/carbon/search";
    import OutlinesIcon from "@iconify-svelte/lucide/list-tree";
    import CircleFilledIcon from "@iconify-svelte/carbon/circle-filled";
    import { keybindings, formatCombo } from "$lib/keybindings";

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
        {#if keybindings.pendingChord}
            <span class="chord-hint">
                {formatCombo(keybindings.pendingChord)}...
            </span>
        {/if}
        <CircleFilledIcon
            class="save-dot {saving ? 'saving' : ''}"
            height="1em"
        />
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

    .chord-hint {
        font-size: 0.7rem;
        font-weight: 600;
        color: var(--cursor);
        letter-spacing: 0.05em;
        padding-right: 0.4rem;
    }

    .statusbar-section {
        display: flex;
        align-items: center;
        gap: 0.1rem;
    }

    .statusbar-right {
        padding-right: 0.3rem;
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

    :global(.save-dot) {
        color: var(--fg-success);
        flex-shrink: 0;
    }

    :global(.save-dot.saving) {
        color: var(--cursor);
    }
</style>
