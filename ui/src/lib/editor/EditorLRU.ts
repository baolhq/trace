const CAPACITY = 8;

export interface CachedEditorState {
    json: object;
    anchor: number;
    head: number;
    scrollTop: number;
}

interface Entry {
    id: string;
    state: CachedEditorState;
}

/**
 * Module-level LRU cache for editor state (JSON content + cursor + scroll).
 * Survives Svelte component remounts so switching back to a recent note
 * restores cursor position and scroll without a round-trip to disk.
 */
class EditorStateCache {
    private entries: Entry[] = [];

    get(id: string): CachedEditorState | null {
        const idx = this.entries.findIndex((e) => e.id === id);
        if (idx === -1) return null;
        // Move to front (most recently used)
        const [entry] = this.entries.splice(idx, 1);
        this.entries.unshift(entry);
        return entry.state;
    }

    put(id: string, state: CachedEditorState): void {
        this.entries = this.entries.filter((e) => e.id !== id);
        if (this.entries.length >= CAPACITY) {
            this.entries.pop(); // evict LRU
        }
        this.entries.unshift({ id, state });
    }

    invalidate(id: string): void {
        this.entries = this.entries.filter((e) => e.id !== id);
    }
}

export const editorLRU = new EditorStateCache();
