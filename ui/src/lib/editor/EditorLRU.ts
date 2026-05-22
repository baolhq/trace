import type { Editor } from "@tiptap/core";

const CAPACITY = 8;

interface Entry {
  id: string;
  editor: Editor;
}

/**
 * Holds detached TipTap editor instances so switching back to a recently
 * viewed node skips re-parsing. Entries are evicted LRU when cap is reached.
 */
export class EditorLRU {
  private entries: Entry[] = [];

  get(id: string): Editor | null {
    const idx = this.entries.findIndex((e) => e.id === id);
    if (idx === -1) return null;
    // Move to front (most recently used)
    const [entry] = this.entries.splice(idx, 1);
    this.entries.unshift(entry);
    return entry.editor;
  }

  put(id: string, editor: Editor): void {
    // Remove stale entry for same id if present
    this.entries = this.entries.filter((e) => e.id !== id);

    if (this.entries.length >= CAPACITY) {
      const evicted = this.entries.pop()!;
      evicted.editor.destroy();
    }

    this.entries.unshift({ id, editor });
  }

  invalidate(id: string): void {
    const idx = this.entries.findIndex((e) => e.id === id);
    if (idx !== -1) {
      this.entries[idx].editor.destroy();
      this.entries.splice(idx, 1);
    }
  }

  destroy(): void {
    for (const { editor } of this.entries) editor.destroy();
    this.entries = [];
  }
}
