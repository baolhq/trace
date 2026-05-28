<script lang="ts">
    let {
        value,
        options,
        onchange,
        minWidth = 130,
    }: {
        value: string;
        options: { value: string; label: string }[];
        onchange: (value: string) => void;
        minWidth?: number;
    } = $props();

    const selectedLabel = $derived(
        options.find((o) => o.value === value)?.label ?? "",
    );

    let open = $state(false);
    let triggerEl: HTMLButtonElement | null = $state(null);
    let listEl: HTMLElement | null = $state(null);
    let pos = $state({ top: 0, right: 0, minWidth: 0 });

    function toggle() {
        if (open) {
            open = false;
            return;
        }
        if (triggerEl) {
            const rect = triggerEl.getBoundingClientRect();
            const listH = Math.min(options.length * 28 + 10, 240);
            const spaceBelow = window.innerHeight - rect.bottom;
            const top =
                spaceBelow >= listH + 4
                    ? rect.bottom + 4
                    : Math.max(8, rect.top - listH - 4);
            pos = {
                top,
                right: window.innerWidth - rect.right,
                minWidth: rect.width,
            };
        }
        open = true;
    }

    function pick(v: string) {
        onchange(v);
        open = false;
    }

    $effect(() => {
        if (!open) return;
        function onClickOutside(e: MouseEvent) {
            const t = e.target as Node;
            if (!triggerEl?.contains(t) && !listEl?.contains(t)) open = false;
        }
        function onScroll(e: Event) {
            if (listEl?.contains(e.target as Node)) return;
            open = false;
        }
        function onKeydown(e: KeyboardEvent) {
            if (e.key === "Escape") {
                open = false;
                triggerEl?.focus();
                e.stopPropagation();
            }
        }
        document.addEventListener("click", onClickOutside);
        document.addEventListener("scroll", onScroll, { capture: true });
        document.addEventListener("keydown", onKeydown, { capture: true });
        return () => {
            document.removeEventListener("click", onClickOutside);
            document.removeEventListener("scroll", onScroll, { capture: true });
            document.removeEventListener("keydown", onKeydown, {
                capture: true,
            });
        };
    });
</script>

<div class="cs">
    <button
        class="cs-trigger"
        style:min-width="{minWidth}px"
        bind:this={triggerEl}
        onclick={toggle}
        type="button"
    >
        <span class="cs-label">{selectedLabel}</span>
        <svg
            class="cs-arrow"
            class:open
            viewBox="0 0 10 6"
            width="8"
            height="5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M1 1l4 4 4-4" />
        </svg>
    </button>

    {#if open}
        <div
            class="cs-list"
            bind:this={listEl}
            style:top="{pos.top}px"
            style:right="{pos.right}px"
            style:min-width="{pos.minWidth}px"
        >
            {#each options as opt}
                <button
                    class="cs-option"
                    class:selected={opt.value === value}
                    onclick={() => pick(opt.value)}
                    type="button">{opt.label}</button
                >
            {/each}
        </div>
    {/if}
</div>

<style>
    .cs {
        position: relative;
        display: inline-block;
    }

    .cs-trigger {
        height: 26px;
        padding: 0 0.5rem;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        color: var(--fg-primary);
        font-size: 0.8rem;
        line-height: 1;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        white-space: nowrap;
        width: 100%;
        transition: border-color 0.1s;
    }

    .cs-trigger:focus-visible {
        outline: none;
        border-color: var(--fg-interactive);
    }

    .cs-label {
        flex: 1;
        text-align: left;
        overflow: hidden;
        text-overflow: ellipsis;
        padding-top: 2px;
    }

    .cs-arrow {
        flex-shrink: 0;
        color: var(--cursor);
        transition: transform 0.15s;
    }

    .cs-arrow.open {
        transform: rotate(180deg);
    }

    .cs-list {
        position: fixed;
        z-index: 300;
        background: var(--bg-panel);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        max-height: 240px;
        overflow-y: auto;
        padding: 0.25rem 0;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .cs-option {
        display: block;
        width: 100%;
        padding: 0.3rem 0.75rem;
        background: none;
        border: none;
        color: var(--fg-muted);
        font-size: 0.8rem;
        text-align: left;
        cursor: pointer;
        white-space: nowrap;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .cs-option:hover {
        background: var(--bg-hover);
        color: var(--fg-primary);
    }

    .cs-option.selected {
        color: var(--fg-primary);
        background: color-mix(in srgb, var(--fg-interactive) 12%, transparent);
    }
</style>
