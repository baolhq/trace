<script lang="ts">
    import { contextMenu } from "$lib/stores/contextMenu.svelte";

    let menuEl: HTMLElement | null = $state(null);

    // Clamp to viewport and reveal after mount so the initial position flash
    // is invisible (opacity starts at 0, set to 1 here after measuring).
    $effect(() => {
        if (!contextMenu.visible || !menuEl) return;
        const { width, height } = menuEl.getBoundingClientRect();
        const PAD = 4;
        const x = Math.round(
            Math.min(contextMenu.x, window.innerWidth - width - PAD),
        );
        const y = Math.round(
            Math.min(contextMenu.y, window.innerHeight - height - PAD),
        );
        menuEl.style.left = `${x}px`;
        menuEl.style.top = `${y}px`;
        menuEl.style.opacity = "1";
        menuEl.focus();
    });

    // Close on outside pointer-down (covers clicking blank areas that don't focus).
    $effect(() => {
        if (!contextMenu.visible) return;
        function onOutside(e: PointerEvent) {
            if (!menuEl?.contains(e.target as Node)) contextMenu.close();
        }
        document.addEventListener("pointerdown", onOutside, true);
        return () =>
            document.removeEventListener("pointerdown", onOutside, true);
    });

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            e.stopPropagation();
            contextMenu.close();
        }
    }
</script>

{#if contextMenu.visible}
    <div
        bind:this={menuEl}
        class="context-menu"
        role="menu"
        tabindex="-1"
        style="left: {contextMenu.x}px; top: {contextMenu.y}px"
        onkeydown={onKeyDown}
        oncontextmenu={(e) => e.preventDefault()}
        onfocusout={(e) => {
            if (!menuEl?.contains(e.relatedTarget as Node)) contextMenu.close();
        }}
    >
        {#each contextMenu.items as item}
            {#if item.kind === "separator"}
                <div class="separator" role="separator"></div>
            {:else}
                <button
                    class="menu-item"
                    class:danger={item.danger}
                    role="menuitem"
                    onmousedown={(e) => {
                        if (e.button !== 0) return;
                        item.action();
                        contextMenu.close();
                    }}
                >
                    {#if item.icon}
                        {@render item.icon()}
                    {/if}
                    <span>{item.label}</span>
                </button>
            {/if}
        {/each}
    </div>
{/if}

<style>
    .context-menu {
        position: fixed;
        z-index: 9999;
        opacity: 0;
        transition: opacity 0.08s ease;
        background: var(--bg-panel);
        border: 1px solid var(--bg-border);
        border-radius: 5px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
        padding: 4px 0;
        min-width: 160px;
        outline: none;
    }

    .menu-item {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        width: 100%;
        background: none;
        border: none;
        height: 28px;
        padding: 0 0.75rem;
        font-size: 0.875rem;
        font-family: var(--font-ui);
        text-align: left;
        cursor: pointer;
        color: var(--fg-muted);
        white-space: nowrap;
        user-select: none;
    }

    .menu-item:hover {
        background: var(--bg-hover);
        color: var(--fg-primary);
    }

    .menu-item.danger {
        color: var(--fg-error);
    }

    .separator {
        height: 0;
        border-top: 1px solid var(--bg-border);
        margin: 4px 0;
    }
</style>
