<script lang="ts">
    import StarIcon from "@iconify-svelte/carbon/star";
    import StarFilledIcon from "@iconify-svelte/carbon/star-filled";
    import CloseIcon from "@iconify-svelte/carbon/close";
    import DocumentIcon from "@iconify-svelte/carbon/document";
    import TrashIcon from "@iconify-svelte/carbon/trash-can";
    import SubtractIcon from "@iconify-svelte/carbon/subtract-alt";
    import { logs } from "$lib/stores/logs.svelte";
    import { notes } from "$lib/stores/notes.svelte";
    import { contextMenu } from "$lib/stores/contextMenu.svelte";
    import type { NodeInfo } from "$lib/types";

    let {
        node,
        isActive,
        isDragOver = false,
        logId = undefined,
        paddingLeft = undefined,
        showBorderLeft = false,
        favPersistent = false,
        onToggleFavorite,
        onDelete = undefined,
        onRemoveFromLog = undefined,
    }: {
        node: NodeInfo;
        isActive: boolean;
        isDragOver?: boolean;
        logId?: number;
        paddingLeft?: string;
        showBorderLeft?: boolean;
        favPersistent?: boolean;
        onToggleFavorite: (e?: MouseEvent) => void;
        onDelete?: (e?: MouseEvent) => void;
        onRemoveFromLog?: (e: MouseEvent) => void;
    } = $props();
</script>

{#snippet iconOpen()}<DocumentIcon height="1em" />{/snippet}
{#snippet iconFav()}<StarIcon height="1em" />{/snippet}
{#snippet iconUnfav()}<StarFilledIcon height="1em" />{/snippet}
{#snippet iconRemove()}<SubtractIcon height="1em" />{/snippet}
{#snippet iconDelete()}<TrashIcon height="1em" />{/snippet}

<div
    class="node-item"
    class:active={isActive}
    class:drag-over={isDragOver}
    class:border-left={showBorderLeft}
    role="option"
    aria-selected={isActive}
    tabindex="-1"
    data-log-id={logId}
    style:padding-left={paddingLeft}
    oncontextmenu={(e) => {
        e.preventDefault();
        contextMenu.open(e.clientX, e.clientY, [
            {
                kind: "action",
                label: "Open",
                icon: iconOpen,
                action: () => notes.openNode(node.id),
            },
            { kind: "separator" },
            {
                kind: "action",
                label: node.is_favorite ? "Unfavorite" : "Favorite",
                icon: node.is_favorite ? iconUnfav : iconFav,
                action: () => onToggleFavorite(),
            },
            { kind: "separator" },
            ...(onRemoveFromLog
                ? [
                      {
                          kind: "action" as const,
                          label: "Remove from log",
                          icon: iconRemove,
                          action: () =>
                              onRemoveFromLog!(new MouseEvent("click")),
                      },
                  ]
                : []),
            ...(onDelete
                ? [
                      {
                          kind: "action" as const,
                          label: "Delete",
                          icon: iconDelete,
                          danger: true,
                          action: () => onDelete!(new MouseEvent("click")),
                      },
                  ]
                : []),
        ]);
    }}
>
    <button
        class="node-btn"
        onpointerdown={(e) => logs.onNodePointerDown(node.id, e)}
        onpointermove={(e) => logs.onNodePointerMove(e)}
        onpointerup={() => logs.onNodePointerUp(node.id)}
        onpointercancel={() => logs.onNodePointerCancel()}
    >
        <span class="node-title">{node.title}</span>
    </button>
    <button
        class="action-btn fav-btn"
        class:fav-on={favPersistent && node.is_favorite}
        onclick={(e) => onToggleFavorite(e)}
        title={node.is_favorite ? "Unfavorite" : "Favorite"}
        tabindex="-1"
    >
        {#if node.is_favorite}
            <StarFilledIcon height="1em" />
        {:else}
            <StarIcon height="1em" />
        {/if}
    </button>
    {#if onRemoveFromLog}
        <button
            class="action-btn remove-btn"
            onclick={(e) => onRemoveFromLog!(e)}
            title="Remove from log"
            tabindex="-1"><CloseIcon height="1em" /></button
        >
    {:else if onDelete}
        <button
            class="action-btn delete-btn"
            onclick={(e) => onDelete!(e)}
            title="Delete"
            tabindex="-1"><CloseIcon height="1em" /></button
        >
    {/if}
</div>

<style>
    .node-item {
        display: flex;
        align-items: center;
        gap: 0.15rem;
        padding: 0 0.4rem 0 0;
        cursor: default;
    }

    .node-item.border-left {
        border-left: 2px solid var(--bg-border);
    }

    .node-item:hover {
        background: var(--bg-hover);
    }

    .node-item.active {
        background: var(--bg-active);
    }

    .node-item.drag-over {
        background: var(--bg-active);
        outline: 1px dashed var(--fg-interactive);
        outline-offset: -1px;
    }

    .node-item.active .node-btn {
        color: var(--fg-interactive);
    }

    .node-item:hover .action-btn {
        opacity: 1;
    }

    .fav-btn.fav-on {
        opacity: 1;
    }

    .node-btn {
        flex: 1;
        background: none;
        border: none;
        text-align: left;
        cursor: pointer;
        padding: 0.4rem 0.3rem 0.4rem 0.75rem;
        overflow: hidden;
        min-width: 0;
        height: 34px;
        display: flex;
        align-items: center;
        user-select: none;
        touch-action: none;
    }

    .node-title {
        display: block;
        font-size: 0.875rem;
        font-family: var(--font-ui);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--cursor);
    }
</style>
