<script lang="ts">
    import "../app.css";
    import "../global.css";
    import { onMount } from "svelte";
    import { initKeybindings } from "$lib/keybindings";

    let { children } = $props();

    onMount(() => initKeybindings());
</script>

{@render children()}
