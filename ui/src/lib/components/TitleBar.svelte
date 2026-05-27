<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount, onDestroy } from "svelte";
    import StarIcon from "@iconify-svelte/carbon/star";
    import StarFilledIcon from "@iconify-svelte/carbon/star-filled";
    import MacCommandIcon from "@iconify-svelte/carbon/mac-command";
    import Tooltip from "$lib/components/Tooltip.svelte";

    interface ActiveMeta {
        id: string;
        title: string;
        is_favorite: boolean;
    }

    interface RecentNode {
        id: string;
        title: string;
    }

    interface Command {
        id: string;
        label: string;
        hint?: string;
        available: boolean;
        action: () => void;
    }

    interface LogItem {
        id: number;
        name: string;
    }

    let {
        activeMeta = null,
        recentNodes = [],
        logItems = [],
        onOpenNode,
        onToggleFavorite,
        onNewNote,
        onOpenLog,
        fileSearchPing = 0,
        commandPalettePing = 0,
    }: {
        activeMeta: ActiveMeta | null;
        recentNodes: RecentNode[];
        logItems?: LogItem[];
        onOpenNode: (id: string) => void;
        onToggleFavorite: () => void;
        onNewNote?: () => void;
        onOpenLog?: (id: number) => void;
        fileSearchPing?: number;
        commandPalettePing?: number;
    } = $props();

    $effect(() => {
        if (fileSearchPing > 0) openDropdown("search");
    });

    $effect(() => {
        if (commandPalettePing > 0) openDropdown("commands");
    });

    const win = getCurrentWindow();
    let maximized = $state(false);
    let unlistenResize: (() => void) | undefined;

    type DropdownMode = "closed" | "search" | "commands" | "prompt";
    let dropdownMode: DropdownMode = $state("closed");
    let query = $state("");
    let searchInputEl: HTMLInputElement | null = $state(null);

    let promptStep: 0 | 1 = $state(0);
    let promptCount = "";
    let promptVaultPath = $state("");
    let promptError: string | null = $state(null);
    let promptBusy = $state(false);

    const isOpen = $derived(dropdownMode !== "closed");

    const filteredRecents = $derived(
        query.trim()
            ? recentNodes
                  .filter((n) =>
                      n.title
                          .toLowerCase()
                          .includes(query.trim().toLowerCase()),
                  )
                  .slice(0, 8)
            : recentNodes.slice(0, 8),
    );

    // Async trace suggestions (all notes, de-duped against recents)
    let traceResults: { id: string; title: string }[] = $state([]);
    let traceFetchTimer: ReturnType<typeof setTimeout> | null = null;

    $effect(() => {
        const q = query.trim();
        if (!q || dropdownMode !== "search") {
            traceResults = [];
            return;
        }
        if (traceFetchTimer) clearTimeout(traceFetchTimer);
        traceFetchTimer = setTimeout(async () => {
            const raw = await invoke<{ id: string; title: string }[]>(
                "suggest_nodes",
                { prefix: q },
            );
            const recentIds = new Set(filteredRecents.map((n) => n.id));
            traceResults = raw.filter((r) => !recentIds.has(r.id)).slice(0, 8);
        }, 60);
    });

    const filteredLogs = $derived(
        query.trim()
            ? logItems
                  .filter((l) =>
                      l.name.toLowerCase().includes(query.trim().toLowerCase()),
                  )
                  .slice(0, 6)
            : logItems.slice(0, 6),
    );

    const hasAnySearchResult = $derived(
        filteredRecents.length > 0 ||
            traceResults.length > 0 ||
            filteredLogs.length > 0,
    );

    const commands: Command[] = $derived([
        {
            id: "new-trace",
            label: "New trace",
            available: true,
            action: () => {
                closeDropdown();
                onNewNote?.();
            },
        },
        {
            id: "toggle-favorite",
            label: activeMeta?.is_favorite
                ? "Remove from favorites"
                : "Add to favorites",
            available: !!activeMeta,
            action: () => {
                closeDropdown();
                onToggleFavorite();
            },
        },
        {
            id: "settings",
            label: "Settings",
            hint: "coming soon",
            available: false,
            action: () => {},
        },
        {
            id: "dev-generate-vault",
            label: "Developer: Generate vault",
            available: true,
            action: async () => {
                dropdownMode = "prompt";
                promptStep = 0;
                promptCount = "";
                promptError = null;
                query = "";
                promptVaultPath = await invoke<string>("vault_path_cmd");
                setTimeout(() => searchInputEl?.focus(), 0);
            },
        },
    ]);

    const filteredCommands = $derived(
        query.trim()
            ? commands.filter((c) =>
                  c.label.toLowerCase().includes(query.trim().toLowerCase()),
              )
            : commands,
    );

    onMount(async () => {
        maximized = await win.isMaximized();
        unlistenResize = await win.onResized(async () => {
            maximized = await win.isMaximized();
        });
    });

    onDestroy(() => unlistenResize?.());

    function openDropdown(mode: Exclude<DropdownMode, "closed">) {
        dropdownMode = mode;
        query = mode === "search" ? (activeMeta?.title ?? "") : "";
        setTimeout(() => {
            searchInputEl?.focus();
            if (mode === "search") searchInputEl?.select();
        }, 0);
    }

    function closeDropdown() {
        dropdownMode = "closed";
        query = "";
    }

    function selectNode(id: string) {
        closeDropdown();
        onOpenNode(id);
    }

    function handleInputKey(e: KeyboardEvent) {
        if (e.key === "Escape") {
            closeDropdown();
            searchInputEl?.blur();
            e.stopPropagation();
            return;
        }
        if (e.key === "Enter" && dropdownMode === "prompt") {
            e.preventDefault();
            e.stopPropagation();
            handlePromptEnter();
        }
    }

    async function handlePromptEnter() {
        if (promptBusy) return;
        if (promptStep === 0) {
            promptCount = query;
            promptStep = 1;
            query = "";
        } else {
            promptBusy = true;
            promptError = null;
            const count = parseInt(promptCount || "100", 10);
            const dest = query.trim();
            try {
                await invoke("gen_vault_cmd", { count, dest });
                closeDropdown();
            } catch (e) {
                promptError = String(e);
            } finally {
                promptBusy = false;
            }
        }
    }

    function handleContainerFocusOut(e: FocusEvent) {
        const related = e.relatedTarget as HTMLElement | null;
        const current = e.currentTarget as HTMLElement;
        if (!related || !current.contains(related)) {
            closeDropdown();
        }
    }
</script>

<div class="titlebar" data-tauri-drag-region>
    <!-- Left: app label — same fixed width as win-controls to balance the center -->
    <div class="app-label" data-tauri-drag-region>
        <span class="app-name">Trace</span>
    </div>

    <!-- Left flex spacer -->
    <div class="drag-spacer" data-tauri-drag-region></div>

    <!-- Center compound box -->
    <div
        class="center-box"
        class:open={isOpen}
        onfocusout={handleContainerFocusOut}
    >
        <!-- Favorite icon (fixed width, left) -->
        <button
            class="side-icon fav-icon"
            class:fav-on={!!activeMeta?.is_favorite}
            onclick={activeMeta ? onToggleFavorite : undefined}
            disabled={!activeMeta}
            tabindex="-1"
        >
            <Tooltip
                description={activeMeta?.is_favorite
                    ? "Unfavorite"
                    : "Favorite"}
                shortcut={{ alt: true, key: "f" }}
            >
                {#if activeMeta?.is_favorite}
                    <StarFilledIcon height="1em" />
                {:else}
                    <StarIcon height="1em" />
                {/if}
            </Tooltip>
        </button>

        <!-- Filename / Search (flex center) -->
        <div
            class="center-area"
            role="button"
            tabindex="0"
            onclick={() => openDropdown("search")}
            onkeydown={(e) => e.key === "Enter" && openDropdown("search")}
        >
            <Tooltip
                description="Search file"
                shortcut={{ ctrl: true, key: "p" }}
            >
                {#if isOpen}
                    <input
                        class="search-input"
                        bind:this={searchInputEl}
                        bind:value={query}
                        onkeydown={handleInputKey}
                        onclick={(e) => e.stopPropagation()}
                        placeholder={dropdownMode === "commands"
                            ? "Run a command…"
                            : dropdownMode === "prompt"
                              ? promptStep === 0
                                  ? "Node count (default: 100)"
                                  : "Destination"
                              : "Search notes…"}
                        autocomplete="off"
                        spellcheck={false}
                    />
                {:else}
                    <span class="filename" class:placeholder={!activeMeta}>
                        {activeMeta ? activeMeta.title : "Open a note…"}
                    </span>
                {/if}
            </Tooltip>
        </div>

        <!-- Command palette icon (fixed width, right) -->
        <button
            class="side-icon cmd-icon"
            class:active={dropdownMode === "commands"}
            onclick={(e) => {
                e.stopPropagation();
                openDropdown("commands");
            }}
            tabindex="-1"
        >
            <Tooltip
                description="Command palette"
                shortcut={{ ctrl: true, shift: true, key: "p" }}
            >
                <MacCommandIcon height="1em" />
            </Tooltip>
        </button>

        <!-- Dropdown -->
        {#if isOpen}
            <div class="dropdown">
                {#if dropdownMode === "prompt"}
                    <p class="dropdown-section">
                        Generate vault — step {promptStep + 1}/2
                    </p>
                    <p class="prompt-desc">
                        {#if promptStep === 0}
                            Number of traces to generate
                        {:else}
                            Leave empty for {promptVaultPath || "the app vault"}
                        {/if}
                    </p>
                    {#if promptBusy}
                        <p class="prompt-desc">Generating…</p>
                    {/if}
                    {#if promptError}
                        <p class="prompt-error">{promptError}</p>
                    {/if}
                {:else if dropdownMode === "search"}
                    {#if hasAnySearchResult}
                        {#if query.trim()}
                            <!-- With a query: merge recents + suggestions under "Traces" -->
                            {@const allTraces = [
                                ...filteredRecents,
                                ...traceResults,
                            ]}
                            {#if allTraces.length > 0}
                                <p class="dropdown-section">Traces</p>
                                {#each allTraces as node (node.id)}
                                    <button
                                        class="dropdown-item"
                                        onmousedown={(e) => {
                                            e.preventDefault();
                                            selectNode(node.id);
                                        }}
                                    >
                                        <span class="dropdown-title"
                                            >{node.title}</span
                                        >
                                    </button>
                                {/each}
                            {/if}
                        {:else}
                            <!-- No query: show recents under "Recents" -->
                            {#if filteredRecents.length > 0}
                                <p class="dropdown-section">Recents</p>
                                {#each filteredRecents as node (node.id)}
                                    <button
                                        class="dropdown-item"
                                        onmousedown={(e) => {
                                            e.preventDefault();
                                            selectNode(node.id);
                                        }}
                                    >
                                        <span class="dropdown-title"
                                            >{node.title}</span
                                        >
                                    </button>
                                {/each}
                            {/if}
                        {/if}
                        {#if query.trim() && filteredLogs.length > 0}
                            <p class="dropdown-section">Logs</p>
                            {#each filteredLogs as log (log.id)}
                                <button
                                    class="dropdown-item"
                                    onmousedown={(e) => {
                                        e.preventDefault();
                                        closeDropdown();
                                        onOpenLog?.(log.id);
                                    }}
                                >
                                    <span class="dropdown-title"
                                        >{log.name}</span
                                    >
                                </button>
                            {/each}
                        {/if}
                    {:else if query.trim()}
                        <p class="dropdown-empty">No results for "{query}"</p>
                    {/if}
                {:else}
                    <p class="dropdown-section">Commands</p>
                    {#if filteredCommands.length > 0}
                        {#each filteredCommands as cmd (cmd.id)}
                            <button
                                class="dropdown-item"
                                class:cmd-unavailable={!cmd.available}
                                disabled={!cmd.available}
                                onmousedown={(e) => {
                                    e.preventDefault();
                                    if (cmd.available) cmd.action();
                                }}
                            >
                                <span class="dropdown-title">{cmd.label}</span>
                                {#if cmd.hint}
                                    <span class="cmd-hint">{cmd.hint}</span>
                                {/if}
                            </button>
                        {/each}
                    {:else}
                        <p class="dropdown-empty">
                            No commands match "{query}"
                        </p>
                    {/if}
                {/if}
            </div>
        {/if}
    </div>

    <!-- Right drag area -->
    <div class="drag-spacer" data-tauri-drag-region></div>

    <!-- Window controls -->
    <div class="win-controls">
        <button
            class="ctrl"
            onclick={() => win.minimize()}
            title="Minimize"
            aria-label="Minimize"
        >
            <svg viewBox="0 0 10 1" fill="currentColor" width="10" height="1">
                <rect width="10" height="1" />
            </svg>
        </button>
        <button
            class="ctrl"
            onclick={() => win.toggleMaximize()}
            title={maximized ? "Restore" : "Maximize"}
            aria-label={maximized ? "Restore" : "Maximize"}
        >
            {#if maximized}
                <svg
                    viewBox="0 0 10 10"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1"
                    width="10"
                    height="10"
                >
                    <rect x="2.5" y="0.5" width="7" height="7" />
                    <path d="M0.5,2.5 L0.5,9.5 L7.5,9.5 L7.5,7.5" />
                </svg>
            {:else}
                <svg
                    viewBox="0 0 10 10"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1"
                    width="10"
                    height="10"
                >
                    <rect x="0.5" y="0.5" width="9" height="9" />
                </svg>
            {/if}
        </button>
        <button
            class="ctrl ctrl-close"
            onclick={() => win.close()}
            title="Close"
            aria-label="Close"
        >
            <svg
                viewBox="0 0 10 10"
                stroke="currentColor"
                stroke-width="1.2"
                stroke-linecap="round"
                width="10"
                height="10"
            >
                <line x1="0.5" y1="0.5" x2="9.5" y2="9.5" />
                <line x1="9.5" y1="0.5" x2="0.5" y2="9.5" />
            </svg>
        </button>
    </div>
</div>

<style>
    .titlebar {
        height: var(--titlebar-height, 36px);
        flex-shrink: 0;
        display: flex;
        align-items: center;
        background: var(--bg-primary);
        border-bottom: 1px solid var(--bg-border);
        user-select: none;
        position: relative;
        z-index: 20;
    }

    /* Left app label — width must match win-controls (3 × 42px = 126px) */
    .app-label {
        width: 126px;
        flex-shrink: 0;
        height: 100%;
        display: flex;
        align-items: center;
        padding-left: 14px;
    }

    .app-name {
        font-size: 0.72rem;
        font-weight: 600;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--cursor);
        pointer-events: none;
    }

    .drag-spacer {
        flex: 1;
        height: 100%;
        min-width: 0;
    }

    /* ── Center compound box ── */

    .center-box {
        position: relative;
        display: flex;
        align-items: stretch;
        width: 400px;
        flex-shrink: 1;
        min-width: 200px;
        height: 24px;
        background: var(--bg-hover);
        border: 1px solid transparent;
        border-radius: 5px;
        transition:
            border-color 0.12s,
            background 0.12s;
    }

    .center-box:focus-within {
        border-color: var(--bg-border);
        background: var(--bg-primary);
    }

    /* When the dropdown is open, flatten bottom corners and drop the bottom border
       so the box and dropdown read as one connected surface. */
    .center-box.open,
    .center-box.open:focus-within {
        border-radius: 5px 5px 0 0;
        border-color: var(--bg-border);
        background: var(--bg-panel);
    }

    /* Fixed-width side icon slots */
    .side-icon {
        flex-shrink: 0;
        width: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        padding: 0;
        cursor: pointer;
        color: var(--cursor);
        transition: color 0.1s;
    }

    .side-icon:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    .fav-icon:not(:disabled):hover {
        color: var(--fg-muted);
    }

    .cmd-icon:hover,
    .cmd-icon.active {
        color: var(--fg-muted);
    }

    /* Vertical dividers between sections */
    .fav-icon {
        border-right: 1px solid var(--bg-border);
        border-radius: 4px 0 0 4px;
    }

    .cmd-icon {
        border-left: 1px solid var(--bg-border);
        border-radius: 0 4px 4px 0;
    }

    /* Center content */
    .center-area {
        flex: 1;
        min-width: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0 4px;
        overflow: hidden;
        cursor: text;
    }

    .filename {
        font-size: 0.8rem;
        color: var(--cursor);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        width: 100%;
        text-align: center;
        pointer-events: none;
    }

    .search-input {
        width: 100%;
        height: 100%;
        border: none;
        background: transparent;
        color: var(--fg-primary);
        font-size: 0.8rem;
        outline: none;
        text-align: center;
        padding: 0;
        caret-color: var(--fg-interactive);
    }

    .search-input::placeholder {
        color: var(--cursor);
    }

    /* ── Dropdown ── */

    .dropdown {
        position: absolute;
        top: calc(100% + 1px);
        left: -0.5px;
        right: -1px;
        background: var(--bg-panel);
        border: 1px solid var(--bg-border);
        border-top: none; /* merges flush with center-box — no seam border */
        border-radius: 0 0 6px 6px;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
        overflow: hidden;
        z-index: 100;
    }

    .dropdown-section {
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--cursor);
        margin: 0;
        padding: 0.45rem 0.7rem 0.2rem;
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        padding: 0.38rem 0.7rem;
        cursor: pointer;
        color: var(--fg-muted);
    }

    .dropdown-item:hover {
        background: var(--bg-hover);
        color: var(--fg-primary);
    }

    .dropdown-title {
        font-size: 0.83rem;
        display: block;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .dropdown-empty {
        font-size: 0.78rem;
        color: var(--cursor);
        text-align: center;
        padding: 0.55rem 0.7rem;
        margin: 0;
    }

    .cmd-unavailable {
        opacity: 0.4;
        cursor: not-allowed;
    }

    .cmd-unavailable:hover {
        background: none;
        color: var(--fg-muted);
    }

    .cmd-hint {
        font-size: 0.7rem;
        color: var(--cursor);
        margin-left: 0.5rem;
        flex-shrink: 0;
    }

    .prompt-desc {
        font-size: 0.78rem;
        color: var(--cursor);
        padding: 0.3rem 0.7rem 0.45rem;
        margin: 0;
        line-height: 1.4;
    }

    .prompt-error {
        font-size: 0.75rem;
        color: var(--fg-error);
        padding: 0.3rem 0.7rem;
        margin: 0;
    }

    /* ── Window controls ── */

    .win-controls {
        width: 126px; /* 3 × 42px — matches .app-label width to keep center box centered */
        flex-shrink: 0;
        display: flex;
        height: 100%;
        justify-content: flex-end;
    }

    .ctrl {
        width: 42px;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        color: var(--cursor);
        cursor: pointer;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .ctrl:hover {
        background: var(--bg-hover);
        color: var(--fg-muted);
    }

    .ctrl-close:hover {
        background: var(--fg-error);
        color: #fff;
    }
</style>
