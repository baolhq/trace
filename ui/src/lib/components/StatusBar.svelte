<script lang="ts">
    let {
        sidebarMode,
        onModeChange,
        saving,
    }: {
        sidebarMode: "notes" | "search";
        onModeChange: (mode: "notes" | "search") => void;
        saving: boolean;
    } = $props();
</script>

<div class="statusbar">
    <div class="statusbar-section statusbar-left">
        <button
            class="mode-btn"
            class:active={sidebarMode === "notes"}
            onclick={() => onModeChange("notes")}>Notes</button
        >
        <button
            class="mode-btn"
            class:active={sidebarMode === "search"}
            onclick={() => onModeChange("search")}>Search</button
        >
    </div>
    <div class="statusbar-section statusbar-right">
        {#if saving}
            <span class="status-item">Saving…</span>
        {/if}
    </div>
</div>

<style>
    .statusbar {
        height: 24px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-top: 1px solid var(--bg-border);
        background: var(--bg-panel);
        padding: 0 0.5rem;
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
