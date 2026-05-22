<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface NodeInfo {
    id: string;
    title: string;
    created_at: number;
  }

  let nodes: NodeInfo[] = $state([]);
  let newTitle = $state("");
  let error = $state("");

  async function loadNodes() {
    try {
      nodes = await invoke("list_nodes");
    } catch (e) {
      error = String(e);
    }
  }

  async function createNode() {
    const title = newTitle.trim();
    if (!title) return;
    try {
      await invoke("create_node", { title });
      newTitle = "";
      await loadNodes();
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteNode(id: string) {
    try {
      await invoke("delete_node", { id });
      await loadNodes();
    } catch (e) {
      error = String(e);
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Enter") createNode();
  }

  onMount(async () => {
    // Load initial data first so the window appears with content, not blank.
    await loadNodes();
    // Signal Rust: frontend is rendered and data is ready.
    // Rust will show the window now that both sides are done.
    await invoke("frontend_ready");
  });
</script>

<main>
  <h1>Trace</h1>

  <div class="create-row">
    <input
      bind:value={newTitle}
      onkeydown={handleKey}
      placeholder="Node title..."
    />
    <button onclick={createNode} disabled={!newTitle.trim()}>
      Create trace
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if nodes.length === 0}
    <p class="empty">No nodes yet.</p>
  {:else}
    <ul>
      {#each nodes as node (node.id)}
        <li>
          <div class="node-info">
            <span class="title">{node.title}</span>
            <span class="id">{node.id}</span>
          </div>
          <button class="delete" onclick={() => deleteNode(node.id)}>
            Delete
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  main {
    max-width: 600px;
    margin: 2rem auto;
    padding: 0 1rem;
    font-family: system-ui, sans-serif;
  }

  h1 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .create-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }

  input {
    flex: 1;
    padding: 0.4rem 0.6rem;
    font-size: 0.95rem;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  button {
    padding: 0.4rem 0.8rem;
    font-size: 0.95rem;
    border: 1px solid #999;
    border-radius: 4px;
    cursor: pointer;
    background: #f5f5f5;
  }

  button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
  }

  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .title {
    font-size: 0.95rem;
  }

  .id {
    font-size: 0.7rem;
    color: #999;
    font-family: monospace;
  }

  .delete {
    font-size: 0.8rem;
    color: #c00;
    border-color: #c00;
    padding: 0.25rem 0.5rem;
  }

  .empty {
    color: #999;
    font-size: 0.9rem;
  }

  .error {
    color: #c00;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
</style>
