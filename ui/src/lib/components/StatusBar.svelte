<script lang="ts">
    import TraceIcon from "@iconify-svelte/pajamas/file-tree";
    import SearchIcon from "@iconify-svelte/carbon/search";
    import OutlinesIcon from "@iconify-svelte/lucide/list-tree";
    import LinkIcon from "@iconify-svelte/lucide/link";
    import BacklinkIcon from "@iconify-svelte/lucide/link-2";
    import CircleFilledIcon from "@iconify-svelte/carbon/circle-filled";
    import Tooltip from "$lib/components/Tooltip.svelte";
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
        <Tooltip description="Notes" shortcut={{ ctrl: true, key: "1" }}>
            <button
                class="mode-btn"
                class:active={sidebarMode === "notes"}
                onclick={() => onModeChange("notes")}
            >
                <TraceIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Outline">
            <button
                class="mode-btn"
                class:active={sidebarMode === "outlines"}
                onclick={() => onModeChange("outlines")}
            >
                <OutlinesIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Search" shortcut={{ ctrl: true, key: "2" }}>
            <button
                class="mode-btn"
                class:active={sidebarMode === "search"}
                onclick={() => onModeChange("search")}
            >
                <SearchIcon height="1em" />
            </button>
        </Tooltip>
    </div>

    <div class="statusbar-section statusbar-right">
        {#if keybindings.pendingChord}
            <span class="chord-hint">
                {formatCombo(keybindings.pendingChord)}...
            </span>
        {/if}
        <Tooltip description="Links">
            <button class="status-btn">
                <LinkIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Backlinks">
            <button class="status-btn">
                <BacklinkIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description={saving ? "Saving…" : "Saved"}>
            <CircleFilledIcon
                class="save-dot {saving ? 'saving' : ''}"
                height="1em"
            />
        </Tooltip>
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

    .status-btn {
        display: flex;
        align-items: center;
        background: none;
        border: none;
        color: var(--cursor);
        padding: 0.25rem 0.5rem;
        border-radius: 3px;
        cursor: pointer;
    }

    .status-btn:hover {
        color: var(--fg-muted);
        background: var(--bg-hover);
    }

    :global(.save-dot) {
        color: var(--fg-success);
        flex-shrink: 0;
        margin-left: 0.2em;
    }

    :global(.save-dot.saving) {
        color: var(--cursor);
    }
</style>
