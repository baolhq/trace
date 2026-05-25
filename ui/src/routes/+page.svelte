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
    import StatusBar from "$lib/components/StatusBar.svelte";
    import { notes } from "$lib/stores/notes.svelte";
    import { logs } from "$lib/stores/logs.svelte";
    import { keybindings } from "$lib/keybindings";

    let sidebarMode: "notes" | "search" | "outlines" = $state("notes");
    let findBarOpen = $state(false);
    let findShowReplace = $state(false);

    let unlisten: (() => void) | undefined;
    let unregisterKeybindings: (() => void) | undefined;

    onMount(async () => {
        await Promise.all([
            notes.loadRecents(),
            notes.loadFavorites(),
            logs.loadLogs(),
        ]);
        await invoke("frontend_ready");
        unlisten = await listen("nodes_changed", () => {
            notes.loadRecents();
            notes.loadFavorites();
        });

        const off1 = keybindings.on("app.new-note", () =>
            notes.createUntitledNode(),
        );
        const off2 = keybindings.on(
            "app.search",
            () => (sidebarMode = "search"),
        );
        const off3 = keybindings.on(
            "app.sidebar.notes",
            () => (sidebarMode = "notes"),
        );
        const off4 = keybindings.on(
            "app.sidebar.search",
            () => (sidebarMode = "search"),
        );
        const off5 = keybindings.on("app.focus-editor", () => {
            document.querySelector<HTMLElement>(".ProseMirror")?.focus();
        });
        const off6 = keybindings.on("editor.find", () => {
            if (notes.viewMode?.kind === "editor") {
                findBarOpen = true;
                findShowReplace = false;
            }
        });
        const off7 = keybindings.on("editor.replace", () => {
            if (notes.viewMode?.kind === "editor") {
                findBarOpen = true;
                findShowReplace = true;
            }
        });
        unregisterKeybindings = () => {
            off1();
            off2();
            off3();
            off4();
            off5();
            off6();
            off7();
        };
    });

    onDestroy(() => {
        unlisten?.();
        unregisterKeybindings?.();
    });
</script>

<div class="app-root">
    <TitleBar
        activeMeta={notes.activeMeta}
        recentNodes={notes.recentNodes}
        onOpenNode={(id) => notes.openNode(id)}
        onToggleFavorite={() =>
            notes.activeMeta && notes.toggleFavorite(notes.activeMeta.id)}
        onNewNote={() => notes.createUntitledNode()}
    />

    <div class="shell">
        <aside class="sidebar">
            {#if notes.error}
                <p class="sidebar-error">{notes.error}</p>
            {/if}

            {#if sidebarMode === "notes"}
                <div class="sidebar-panels">
                    <FavoritesPanel />
                    <LogsPanel />
                    <RecentsPanel />
                </div>
            {:else if sidebarMode === "search"}
                <SearchPanel />
            {/if}
        </aside>

        <main class="main-pane">
            {#if notes.viewMode?.kind === "editor" && notes.activeDoc && notes.activeNodeId}
                <Editor
                    nodeId={notes.activeNodeId}
                    doc={notes.activeDoc}
                    onSave={(ttJson, nodeId) =>
                        notes.handleSave(ttJson, nodeId)}
                    titleError={notes.titleError}
                    bind:findBarOpen
                    bind:findShowReplace
                />
            {:else}
                <div class="empty-state">
                    <p>Select a note or create one to start writing.</p>
                </div>
            {/if}
        </main>
    </div>

    <StatusBar
        {sidebarMode}
        onModeChange={(m) => (sidebarMode = m)}
        saving={notes.saving}
    />
</div>

<style>
    :global(*, *::before, *::after) {
        box-sizing: border-box;
    }

    :global(body) {
        margin: 0;
        font-family:
            system-ui,
            -apple-system,
            sans-serif;
        background: var(--bg-primary);
        color: var(--fg-primary);
    }

    .app-root {
        display: flex;
        flex-direction: column;
        height: 100vh;
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
