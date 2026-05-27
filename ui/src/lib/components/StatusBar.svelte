<script lang="ts">
    import JournalIcon from "@iconify-svelte/lucide/scroll-text";
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
        rightPanelMode,
        onRightPanelChange,
        saving,
    }: {
        sidebarMode: "journal" | "traces" | "search" | "outlines";
        onModeChange: (
            mode: "journal" | "traces" | "search" | "outlines",
        ) => void;
        rightPanelMode: "links" | "backlinks" | null;
        onRightPanelChange: (mode: "links" | "backlinks" | null) => void;
        saving: boolean;
    } = $props();

    function togglePanel(tab: "links" | "backlinks") {
        onRightPanelChange(rightPanelMode === tab ? null : tab);
    }
</script>

<div class="statusbar">
    <div class="statusbar-section statusbar-left">
        <Tooltip description="Journal" shortcut={{ alt: true, key: "j" }}>
            <button
                class="mode-btn"
                class:active={sidebarMode === "journal"}
                onclick={() => onModeChange("journal")}
            >
                <JournalIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Traces" shortcut={{ alt: true, key: "t" }}>
            <button
                class="mode-btn"
                class:active={sidebarMode === "traces"}
                onclick={() => onModeChange("traces")}
            >
                <TraceIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Outline" shortcut={{ alt: true, key: "o" }}>
            <button
                class="mode-btn"
                class:active={sidebarMode === "outlines"}
                onclick={() => onModeChange("outlines")}
            >
                <OutlinesIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip
            description="Search"
            shortcut={{ ctrl: true, shift: true, key: "f" }}
        >
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
        <Tooltip description="Links" shortcut={{ alt: true, key: "l" }}>
            <button
                class="status-btn"
                class:active={rightPanelMode === "links"}
                onclick={() => togglePanel("links")}
            >
                <LinkIcon height="1em" />
            </button>
        </Tooltip>
        <Tooltip description="Backlinks" shortcut={{ alt: true, key: "b" }}>
            <button
                class="status-btn"
                class:active={rightPanelMode === "backlinks"}
                onclick={() => togglePanel("backlinks")}
            >
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

    .status-btn.active {
        color: var(--fg-interactive);
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
