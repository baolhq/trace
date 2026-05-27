<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import Editor from "$lib/editor/Editor.svelte";
    import TitleBar from "$lib/components/TitleBar.svelte";
    import FavoritesPanel from "$lib/components/FavoritesPanel.svelte";
    import LogsPanel from "$lib/components/LogsPanel.svelte";
    import RecentsPanel from "$lib/components/RecentsPanel.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import RightPanel from "$lib/components/RightPanel.svelte";
    import StatusBar from "$lib/components/StatusBar.svelte";
    import ContextMenu from "$lib/components/ContextMenu.svelte";
    import SettingsModal from "$lib/components/SettingsModal.svelte";
    import { notes } from "$lib/stores/notes.svelte";
    import { logs } from "$lib/stores/logs.svelte";
    import { settings } from "$lib/stores/settings.svelte";
    import { keybindings } from "$lib/keybindings";
    import Spinner from "$lib/components/Spinner.svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { cubicIn } from "svelte/easing";
    import type { SettingsScope } from "$lib/types";

    function growFade(_node: Element, { duration = 600 } = {}) {
        return {
            duration,
            easing: cubicIn,
            css: (t: number) =>
                `opacity: ${t}; transform: scale(${1 + (1 - t) * 0.18});`,
        };
    }

    let sidebarMode: "journal" | "traces" | "search" | "outlines" | null =
        $state("traces");
    let rightPanelMode: "links" | "backlinks" | null = $state(null);
    let findBarOpen = $state(false);
    let findShowReplace = $state(false);
    let fileSearchPing = $state(0);
    let commandPalettePing = $state(0);
    let backendReady = $state(false);
    let settingsOpen = $state(false);
    let settingsScope: SettingsScope = $state("global");

    let unlisten: (() => void) | undefined;
    let unregisterKeybindings: (() => void) | undefined;

    function toggleSidebar(tab: "journal" | "traces" | "search" | "outlines") {
        sidebarMode = sidebarMode === tab ? null : tab;
    }

    function toggleRightPanel(tab: "links" | "backlinks") {
        rightPanelMode = rightPanelMode === tab ? null : tab;
    }

    onMount(async () => {
        await new Promise(requestAnimationFrame);
        await getCurrentWindow().show();
        backendReady = await invoke("backend_ready");

        await Promise.all([
            notes.loadRecents(),
            notes.loadFavorites(),
            logs.loadLogs(),
            settings.load(),
        ]);
        unlisten = await listen("nodes_changed", () => {
            notes.loadRecents();
            notes.loadFavorites();
        });

        const offs = [
            keybindings.on("editor.new-trace", () =>
                notes.createUntitledNode(),
            ),
            keybindings.on("editor.journal", () => toggleSidebar("journal")),
            keybindings.on("panel.traces", () => toggleSidebar("traces")),
            keybindings.on("panel.outlines", () => toggleSidebar("outlines")),
            keybindings.on("panel.search", () => toggleSidebar("search")),
            keybindings.on("panel.links", () => toggleRightPanel("links")),
            keybindings.on("panel.backlinks", () =>
                toggleRightPanel("backlinks"),
            ),
            keybindings.on("app.focus-content", () => {
                document.querySelector<HTMLElement>(".ProseMirror")?.focus();
            }),
            keybindings.on("editor.find", () => {
                if (notes.viewMode?.kind === "editor") {
                    findBarOpen = true;
                    findShowReplace = false;
                }
            }),
            keybindings.on("editor.replace", () => {
                if (notes.viewMode?.kind === "editor") {
                    findBarOpen = true;
                    findShowReplace = true;
                }
            }),
            keybindings.on("app.file-search", () => fileSearchPing++),
            keybindings.on("app.command-palette", () => commandPalettePing++),
            keybindings.on("app.show-settings", () => {
                if (settingsOpen) {
                    settingsOpen = false;
                    invoke("open_settings_file", {
                        scope: settingsScope,
                    }).catch(console.error);
                } else {
                    settingsOpen = true;
                }
            }),
        ];
        unregisterKeybindings = () => offs.forEach((off) => off());
    });

    onDestroy(() => {
        unlisten?.();
        unregisterKeybindings?.();
    });
</script>

<div
    class="app-root"
    role="application"
    oncontextmenu={(e) => e.preventDefault()}
>
    {#if backendReady}
        <TitleBar
            activeMeta={notes.activeMeta}
            recentNodes={notes.recentNodes}
            logItems={logs.allLogs}
            onOpenNode={(id) => notes.openNode(id)}
            onToggleFavorite={() =>
                notes.activeMeta && notes.toggleFavorite(notes.activeMeta.id)}
            onNewNote={() => notes.createUntitledNode()}
            onOpenLog={(id) => {
                sidebarMode = "traces";
                const log = logs.allLogs.find((l) => l.id === id);
                if (log) logs.openLog({ ...log, children: [] });
            }}
            onOpenSettings={() => (settingsOpen = true)}
            {fileSearchPing}
            {commandPalettePing}
        />

        <div class="shell">
            {#if sidebarMode}
                <aside class="sidebar">
                    {#if notes.error}
                        <p class="sidebar-error">{notes.error}</p>
                    {/if}

                    {#if sidebarMode === "traces"}
                        <div class="sidebar-panels">
                            <FavoritesPanel />
                            <LogsPanel />
                            <RecentsPanel />
                        </div>
                    {:else if sidebarMode === "search"}
                        <SearchPanel />
                    {/if}
                </aside>
            {/if}

            <main class="main-pane">
                {#if notes.viewMode?.kind === "editor" && notes.activeDoc && notes.activeNodeId}
                    <Editor
                        nodeId={notes.activeNodeId}
                        doc={notes.activeDoc}
                        onSave={(ttJson, nodeId) =>
                            notes.handleSave(ttJson, nodeId)}
                        title={notes.activeMeta?.title ?? ""}
                        onRename={async (t) => {
                            if (notes.activeMeta)
                                await notes.renameNode(notes.activeMeta.id, t);
                        }}
                        onNavigate={async (target, isIdRef) => {
                            if (isIdRef) {
                                await notes.openNode(target);
                            } else {
                                const id = await invoke<string | null>(
                                    "get_node_id_by_title",
                                    { title: target },
                                );
                                if (id) await notes.openNode(id);
                            }
                        }}
                        existingTitles={notes.allTitles.filter(
                            (t) => t !== (notes.activeMeta?.title ?? ""),
                        )}
                        bind:findBarOpen
                        bind:findShowReplace
                    />
                {:else}
                    <div class="empty-state">
                        <p>Select a note or create one to start writing.</p>
                    </div>
                {/if}
            </main>

            {#if rightPanelMode}
                <RightPanel
                    mode={rightPanelMode}
                    nodeId={notes.activeNodeId ?? null}
                />
            {/if}
        </div>

        <StatusBar
            {sidebarMode}
            onModeChange={(m) => (sidebarMode = m)}
            {rightPanelMode}
            onRightPanelChange={(m) => (rightPanelMode = m)}
            saving={notes.saving}
        />
    {/if}

    {#if !backendReady}
        <div class="spinner-overlay" out:growFade>
            <Spinner />
        </div>
    {/if}
</div>

<SettingsModal
    bind:open={settingsOpen}
    onOpenRawToml={(scope: SettingsScope) => {
        settingsScope = scope;
        invoke("open_settings_file", { scope }).catch(console.error);
    }}
/>

<ContextMenu />

<style>
    :global(*, *::before, *::after) {
        box-sizing: border-box;
    }

    :global(::selection) {
        background: var(--selection);
    }

    :global(body) {
        margin: 0;
        font-family: var(--font-ui);
        font-feature-settings: "lnum", "tnum";
        background: var(--bg-primary);
        color: var(--fg-primary);
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .app-root {
        display: flex;
        flex-direction: column;
        height: 100vh;
        overflow: hidden;
        position: relative;
    }

    .spinner-overlay {
        position: absolute;
        inset: 0;
        z-index: 99;
        background: var(--bg-primary);
        overflow: hidden;
    }

    .shell {
        display: flex;
        flex: 1;
        min-height: 0;
        overflow: hidden;
    }

    .sidebar {
        width: 240px;
        flex-shrink: 0;
        border-right: 1px solid var(--bg-border);
        background: var(--bg-primary);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .sidebar-panels {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .sidebar-error {
        font-size: 0.75rem;
        color: var(--fg-error);
        padding: 0 0.75rem;
        margin: 0 0 0.5rem;
    }

    .main-pane {
        flex: 1;
        overflow: hidden;
        position: relative;
        display: flex;
        flex-direction: column;
    }

    .empty-state {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--cursor);
        font-size: 0.9rem;
    }
</style>
