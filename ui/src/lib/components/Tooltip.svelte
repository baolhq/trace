<script lang="ts">
    import MacCommandIcon from "@iconify-svelte/carbon/mac-command";
    import type { KeyCombo } from "$lib/keybindings";
    import type { Snippet } from "svelte";

    type ShortcutPart = { kind: "cmd" } | { kind: "text"; label: string };

    let {
        description,
        shortcut,
        children,
    }: {
        description: string;
        shortcut?: KeyCombo;
        children: Snippet;
    } = $props();

    let visible = $state(false);
    let cursorX = $state(0);
    let cursorY = $state(0);
    let tooltipEl: HTMLElement | null = $state(null);
    let showTimer: ReturnType<typeof setTimeout> | null = null;

    const OFFSET_Y = 12;
    const PAD = 8;
    const DELAY = 500;

    const parts = $derived(buildParts(shortcut));

    function buildParts(combo: KeyCombo | undefined): ShortcutPart[] {
        if (!combo) return [];
        const result: ShortcutPart[] = [];
        if (combo.ctrl || combo.meta) result.push({ kind: "cmd" });
        if (combo.shift) result.push({ kind: "text", label: "Shift" });
        if (combo.alt) result.push({ kind: "text", label: "Alt" });
        result.push({
            kind: "text",
            label: combo.key === " " ? "Space" : combo.key.toUpperCase(),
        });
        return result;
    }

    function onMouseEnter(e: MouseEvent) {
        if (!description) return;
        cursorX = e.clientX;
        cursorY = e.clientY;
        showTimer = setTimeout(() => {
            visible = true;
        }, DELAY);
    }

    function onMouseMove(e: MouseEvent) {
        if (visible) return;
        cursorX = e.clientX;
        cursorY = e.clientY;
    }

    function onMouseLeave() {
        if (showTimer !== null) {
            clearTimeout(showTimer);
            showTimer = null;
        }
        visible = false;
    }

    // After the tooltip mounts, measure it and snap to a clamped position.
    // Starts at opacity 0 so this first-frame reposition is invisible.
    $effect(() => {
        if (!visible || !tooltipEl) return;
        const { width, height } = tooltipEl.getBoundingClientRect();
        let x = cursorX - width / 2;
        let y = cursorY - height - OFFSET_Y;
        x = Math.max(PAD, Math.min(x, window.innerWidth - width - PAD));
        if (y < PAD) y = cursorY + OFFSET_Y; // flip below when near top edge
        tooltipEl.style.left = `${x}px`;
        tooltipEl.style.top = `${y}px`;
        tooltipEl.style.opacity = "1";
    });
</script>

<div
    class="tooltip-root"
    onmouseenter={onMouseEnter}
    onmousemove={onMouseMove}
    onmouseleave={onMouseLeave}
    role="none"
>
    {@render children()}
    {#if visible}
        <div bind:this={tooltipEl} class="tooltip" role="tooltip">
            <span class="description">{description}</span>
            {#if parts.length}
                <span class="shortcut">
                    {#each parts as part, i}
                        {#if i > 0}<span class="sep">+</span>{/if}
                        {#if part.kind === "cmd"}
                            <MacCommandIcon height="1em" />
                        {:else}
                            <span class="key">{part.label}</span>
                        {/if}
                    {/each}
                </span>
            {/if}
        </div>
    {/if}
</div>

<style>
    .tooltip-root {
        position: relative;
        display: inline-flex;
        align-items: center;
    }

    .tooltip {
        position: fixed;
        opacity: 0; /* invisible until $effect clamps and reveals it */
        transition: opacity 0.12s ease;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.25rem 0.55rem;
        border-radius: 5px;
        background: var(--bg-primary);
        border: 1px solid var(--bg-border);
        font-size: 0.8rem;
        white-space: nowrap;
        pointer-events: none;
        user-select: none;
        z-index: 9999;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
        color: var(--fg-primary);
    }

    .shortcut {
        display: flex;
        align-items: center;
        gap: 0.15rem;
    }

    .sep {
        opacity: 0.5;
        font-size: 0.65rem;
    }

    .key {
        font-weight: 500;
    }
</style>
