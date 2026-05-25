<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        title,
        noScroll = false,
        children,
        actions,
    }: {
        title: string;
        noScroll?: boolean;
        children: Snippet;
        actions?: Snippet;
    } = $props();

    let open = $state(true);
</script>

<div class="panel" class:open>
    <div
        class="panel-header"
        role="button"
        tabindex="0"
        onclick={() => (open = !open)}
        onkeydown={(e) => e.key === "Enter" && (open = !open)}
    >
        <span class="panel-arrow">{open ? "▾" : "▸"}</span>
        <span class="panel-title">{title}</span>
        {@render actions?.()}
    </div>
    {#if open}
        <div class="panel-body" class:no-scroll={noScroll}>
            {@render children()}
        </div>
    {/if}
</div>

<style>
    .panel {
        display: flex;
        flex-direction: column;
        min-height: 0;
        flex: 0 0 auto;
        border-top: 1px solid var(--bg-border);
    }

    .panel.open {
        flex: 1 1 0;
    }

    .panel:first-child {
        border-top: none;
    }

    .panel-header {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.45rem 0.75rem;
        cursor: pointer;
        user-select: none;
        flex-shrink: 0;
    }

    .panel-header:hover {
        background: var(--bg-hover);
    }

    .panel-arrow {
        font-size: 0.65rem;
        color: var(--cursor);
        width: 10px;
        text-align: center;
        flex-shrink: 0;
    }

    .panel-title {
        flex: 1;
        font-size: 0.7rem;
        font-weight: 600;
        letter-spacing: 0.07em;
        text-transform: uppercase;
        color: var(--cursor);
    }

    .panel-body {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .panel-body.no-scroll {
        overflow: hidden;
        -webkit-mask-image: linear-gradient(
            to bottom,
            black 80%,
            transparent 100%
        );
        mask-image: linear-gradient(to bottom, black 80%, transparent 100%);
    }
</style>
