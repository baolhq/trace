<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import { Table, TableRow, TableHeader, TableCell } from "@tiptap/extension-table";
  import Strike from "@tiptap/extension-strike";
  import { WikiLink } from "./extensions/WikiLink";
  import { Tag } from "./extensions/Tag";
  import { pmDocToTipTap, type PmDoc } from "./doc";

  interface Props {
    doc: PmDoc;
    nodeId: string;
    onSave: (doc: object) => void;
  }

  let { doc, nodeId, onSave }: Props = $props();

  let container: HTMLDivElement;
  let editor: Editor | null = null;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const AUTOSAVE_DELAY = 400;

  function buildEditor(element: HTMLElement, initialDoc: PmDoc): Editor {
    return new Editor({
      element,
      extensions: [
        StarterKit.configure({
          // Disable Strike from StarterKit — added separately below
          strike: false,
          heading: { levels: [1, 2, 3, 4, 5, 6] },
        }),
        Strike,
        Table.configure({ resizable: false }),
        TableRow,
        TableHeader,
        TableCell,
        WikiLink,
        Tag,
      ],
      content: pmDocToTipTap(initialDoc),
      onUpdate: () => scheduleSave(),
    });
  }

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, AUTOSAVE_DELAY);
  }

  function flushSave() {
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    if (!editor) return;
    onSave(editor.getJSON());
  }

  onMount(() => {
    editor = buildEditor(container, doc);
  });

  onDestroy(() => {
    flushSave();
    editor?.destroy();
    editor = null;
  });

  // When nodeId changes, reload content (cache miss — caller already did LRU check)
  $effect(() => {
    if (editor && doc) {
      flushSave(); // save current node before switching
      editor.commands.setContent(pmDocToTipTap(doc));
    }
  });
</script>

<div class="editor-wrap">
  <div bind:this={container} class="editor-content"></div>
</div>

<style>
  .editor-wrap {
    width: 100%;
    height: 100%;
    overflow-y: auto;
  }

  .editor-content {
    max-width: 680px;
    margin: 0 auto;
    padding: 2rem 1.5rem 6rem;
    min-height: 100%;
  }

  :global(.editor-content .ProseMirror) {
    outline: none;
    font-size: 0.95rem;
    line-height: 1.7;
    color: #1a1a1a;
  }

  :global(.editor-content .ProseMirror h1) { font-size: 1.75rem; font-weight: 700; margin: 1.5rem 0 0.5rem; }
  :global(.editor-content .ProseMirror h2) { font-size: 1.4rem;  font-weight: 600; margin: 1.25rem 0 0.4rem; }
  :global(.editor-content .ProseMirror h3) { font-size: 1.15rem; font-weight: 600; margin: 1rem 0 0.35rem; }

  :global(.editor-content .ProseMirror p)  { margin: 0 0 0.75rem; }
  :global(.editor-content .ProseMirror ul),
  :global(.editor-content .ProseMirror ol) { padding-left: 1.5rem; margin: 0 0 0.75rem; }
  :global(.editor-content .ProseMirror li) { margin-bottom: 0.2rem; }

  :global(.editor-content .ProseMirror code) {
    font-family: ui-monospace, monospace;
    font-size: 0.875em;
    background: #f3f3f3;
    border-radius: 3px;
    padding: 0.1em 0.3em;
  }

  :global(.editor-content .ProseMirror pre) {
    background: #f6f6f6;
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
    margin: 0 0 1rem;
  }

  :global(.editor-content .ProseMirror pre code) {
    background: none;
    padding: 0;
    font-size: 0.875rem;
  }

  :global(.editor-content .ProseMirror blockquote) {
    border-left: 3px solid #ddd;
    margin: 0 0 0.75rem;
    padding-left: 1rem;
    color: #555;
  }

  :global(.editor-content .ProseMirror hr) {
    border: none;
    border-top: 1px solid #e0e0e0;
    margin: 1.5rem 0;
  }

  :global(.editor-content .wiki-link) {
    display: inline-block;
    background: #eef2ff;
    color: #4361ee;
    border-radius: 4px;
    padding: 0 0.3em;
    font-size: 0.9em;
    cursor: pointer;
    user-select: none;
  }

  :global(.editor-content .tag) {
    color: #7c3aed;
    font-weight: 500;
    cursor: pointer;
    user-select: none;
  }

  :global(.editor-content .ProseMirror table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 1rem;
  }

  :global(.editor-content .ProseMirror th),
  :global(.editor-content .ProseMirror td) {
    border: 1px solid #ddd;
    padding: 0.4rem 0.6rem;
    text-align: left;
  }

  :global(.editor-content .ProseMirror th) {
    background: #f9f9f9;
    font-weight: 600;
  }
</style>
