<script lang="ts">
    import Panel from "./Panel.svelte";
    import NodeListItem from "./NodeListItem.svelte";
    import { notes } from "$lib/stores/notes.svelte";
</script>

<Panel title="Favorites">
    {#each notes.favorites as node (node.id)}
        <NodeListItem
            {node}
            isActive={notes.activeNodeId === node.id}
            onToggleFavorite={(e) => notes.toggleFavorite(node.id, e)}
            onDelete={(e) =>
                notes.deleteNode(node.id, e ?? new MouseEvent("click"))}
        />
    {/each}
</Panel>
