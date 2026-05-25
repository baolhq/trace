<script lang="ts">
    import { onMount } from "svelte";
    import ChevronDownIcon from "@iconify-svelte/lucide/chevron-down";
    import ChevronUpIcon from "@iconify-svelte/lucide/chevron-up";
    import ChevronRightIcon from "@iconify-svelte/lucide/chevron-right";
    import XIcon from "@iconify-svelte/lucide/x";
    import ReplaceIcon from "@iconify-svelte/lucide/replace";
    import ReplaceAllIcon from "@iconify-svelte/lucide/replace-all";

    interface Props {
        searchTerm: string;
        replaceTerm: string;
        matchCount: number;
        currentIndex: number;
        matchCase: boolean;
        wholeWord: boolean;
        useRegex: boolean;
        showReplace: boolean;
        onSearchChange: (v: string) => void;
        onReplaceChange: (v: string) => void;
        onMatchCaseToggle: () => void;
        onWholeWordToggle: () => void;
        onUseRegexToggle: () => void;
        onToggleReplace: () => void;
        onNext: () => void;
        onPrev: () => void;
        onReplace: () => void;
        onReplaceAll: () => void;
        onClose: () => void;
    }

    let {
        searchTerm,
        replaceTerm,
        matchCount,
        currentIndex,
        matchCase,
        wholeWord,
        useRegex,
        showReplace,
        onSearchChange,
        onReplaceChange,
        onMatchCaseToggle,
        onWholeWordToggle,
        onUseRegexToggle,
        onToggleReplace,
        onNext,
        onPrev,
        onReplace,
        onReplaceAll,
        onClose,
    }: Props = $props();

    let searchEl: HTMLInputElement;
    let replaceEl: HTMLInputElement | undefined = $state();

    onMount(() => {
        if (showReplace) {
            replaceEl?.focus();
            replaceEl?.select();
        } else {
            searchEl?.focus();
            searchEl?.select();
        }
    });

    // Focus replace input whenever the replace row becomes visible after mount.
    $effect(() => {
        if (showReplace) {
            replaceEl?.focus();
        }
    });

    function onSearchKeydown(e: KeyboardEvent) {
        if (e.key === "Tab" && !e.shiftKey && showReplace) {
            e.preventDefault();
            replaceEl?.focus();
        } else if (e.key === "Escape") {
            e.preventDefault();
            onClose();
        } else if (e.key === "Enter") {
            e.preventDefault();
            e.shiftKey ? onPrev() : onNext();
        }
    }

    function onReplaceKeydown(e: KeyboardEvent) {
        if (e.key === "Tab" && e.shiftKey) {
            e.preventDefault();
            searchEl?.focus();
        } else if (e.key === "Escape") {
            e.preventDefault();
            onClose();
        } else if (e.key === "Enter") {
            e.preventDefault();
            onReplace();
        }
    }

    const countLabel = $derived(
        matchCount === 0 ? "No results" : `${currentIndex + 1}/${matchCount}`,
    );
</script>

<!--
    Grid layout: [22px toggle/spacer] [1fr input] [auto controls]
    Both rows share the same grid columns so the inputs are always the same width.
-->
<div class="find-bar">
    <!-- Find row -->
    <button
        class="icon-btn toggle-replace"
        class:active={showReplace}
        title={showReplace ? "Hide replace" : "Show replace"}
        onclick={onToggleReplace}
    >
        <ChevronRightIcon
            height="1em"
            style="transform: rotate({showReplace
                ? 90
                : 0}deg); transition: transform 0.15s"
        />
    </button>

    <input
        bind:this={searchEl}
        class="find-input"
        type="text"
        placeholder="Find"
        value={searchTerm}
        oninput={(e) => onSearchChange((e.target as HTMLInputElement).value)}
        onkeydown={onSearchKeydown}
        spellcheck="false"
    />

    <div class="find-controls">
        <button
            class="icon-btn opt-btn"
            class:active={matchCase}
            title="Match case"
            onclick={onMatchCaseToggle}
        >
            <span class="opt-label">Aa</span>
        </button>
        <button
            class="icon-btn opt-btn"
            class:active={wholeWord}
            title="Match whole word"
            onclick={onWholeWordToggle}
        >
            <span class="opt-label">ab</span>
        </button>
        <button
            class="icon-btn opt-btn"
            class:active={useRegex}
            title="Use regular expression"
            onclick={onUseRegexToggle}
        >
            <span class="opt-label">.*</span>
        </button>
        <span
            class="match-count"
            class:no-match={matchCount === 0 && searchTerm !== ""}
        >
            {searchTerm ? countLabel : ""}
        </span>
        <button
            class="icon-btn"
            title="Previous match (Shift+Enter)"
            onclick={onPrev}
            disabled={matchCount === 0}
        >
            <ChevronUpIcon height="1em" />
        </button>
        <button
            class="icon-btn"
            title="Next match (Enter)"
            onclick={onNext}
            disabled={matchCount === 0}
        >
            <ChevronDownIcon height="1em" />
        </button>
        <button
            class="icon-btn close-btn"
            title="Close (Esc)"
            onclick={onClose}
        >
            <XIcon height="1em" />
        </button>
    </div>

    <!-- Replace row -->
    {#if showReplace}
        <span class="row-spacer"></span>

        <input
            bind:this={replaceEl}
            class="find-input"
            type="text"
            placeholder="Replace"
            value={replaceTerm}
            oninput={(e) =>
                onReplaceChange((e.target as HTMLInputElement).value)}
            onkeydown={onReplaceKeydown}
            spellcheck="false"
        />

        <div class="replace-actions">
            <button
                class="icon-btn"
                title="Replace (Enter)"
                onclick={onReplace}
                disabled={matchCount === 0}
            >
                <ReplaceIcon height="1em" />
            </button>
            <button
                class="icon-btn"
                title="Replace all"
                onclick={onReplaceAll}
                disabled={matchCount === 0}
            >
                <ReplaceAllIcon height="1em" />
            </button>
        </div>
    {/if}
</div>

<style>
    /* Grid: col1=toggle(22px), col2=input(1fr), col3=controls(auto) */
    .find-bar {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        z-index: 50;
        background: var(--bg-primary);
        padding: 0.3rem 0.5rem;
        display: grid;
        grid-template-columns: 22px 1fr auto;
        align-items: center;
        gap: 0.25rem;
    }

    .find-input {
        min-width: 0;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 3px;
        color: var(--fg-primary);
        font-size: 0.85rem;
        padding: 0.2rem 0.5rem;
        outline: none;
    }

    .find-input:focus {
        border-color: var(--fg-interactive);
    }

    .find-controls,
    .replace-actions {
        display: flex;
        align-items: center;
        gap: 0.25rem;
    }

    .match-count {
        font-size: 0.75rem;
        color: var(--fg-muted);
        white-space: nowrap;
        min-width: 3.5rem;
        text-align: right;
        flex-shrink: 0;
    }

    .match-count.no-match {
        color: var(--fg-error);
    }

    .icon-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        padding: 0;
        background: none;
        border: none;
        border-radius: 3px;
        color: var(--fg-muted);
        cursor: pointer;
        flex-shrink: 0;
    }

    .icon-btn:hover:not(:disabled) {
        background: var(--bg-active);
        color: var(--fg-primary);
    }

    .icon-btn:disabled {
        color: var(--cursor);
        cursor: default;
    }

    .opt-btn {
        width: auto;
        min-width: 22px;
        padding: 0 4px;
        height: 22px;
    }

    .opt-btn.active {
        background: var(--bg-active);
        color: var(--fg-interactive);
    }

    .opt-label {
        font-size: 0.72rem;
        font-family: ui-monospace, monospace;
        font-weight: 600;
        line-height: 1;
        letter-spacing: 0;
    }

    .toggle-replace {
        color: var(--cursor);
    }

    .toggle-replace.active {
        color: var(--fg-muted);
    }

    .close-btn {
        color: var(--cursor);
    }

    .close-btn:hover {
        color: var(--fg-error);
    }
</style>
