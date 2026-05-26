import { invoke } from "@tauri-apps/api/core";
import { tipTapToPmDoc, type PmDoc } from "$lib/editor/doc";
import { logs } from "$lib/stores/logs.svelte";
import type {
  NodeInfo,
  NodeMeta,
  OpenNodeResponse,
  ViewMode,
} from "$lib/types";

class NotesStore {
  recentNodes: NodeInfo[] = $state([]);
  favorites: NodeInfo[] = $state([]);
  viewMode: ViewMode = $state(null);
  activeNodeId: string | null = $state(null);
  activeDoc: PmDoc | null = $state(null);
  activeMeta: NodeMeta | null = $state(null);
  error = $state("");
  saving = $state(false);

  #loadGen = 0;

  async loadRecents() {
    const gen = ++this.#loadGen;
    try {
      const raw = await invoke<NodeInfo[]>("list_nodes");
      if (gen !== this.#loadGen) return;
      const seen = new Set<string>();
      this.recentNodes = raw.filter(
        (n) => !seen.has(n.id) && (seen.add(n.id) as unknown as boolean),
      );
    } catch (e) {
      if (gen === this.#loadGen) this.error = String(e);
    }
  }

  async loadFavorites() {
    try {
      this.favorites = await invoke<NodeInfo[]>("list_favorites");
    } catch {
      // non-critical
    }
  }

  async openNode(id: string) {
    if (this.activeNodeId === id && this.viewMode?.kind === "editor") return;
    const hit = this.recentNodes.find((n) => n.id === id);
    if (hit)
      this.recentNodes = [hit, ...this.recentNodes.filter((n) => n.id !== id)];
    try {
      const res = await invoke<OpenNodeResponse>("open_node", { id });
      this.activeNodeId = id;
      this.activeMeta = res.meta;
      this.activeDoc = res.doc;
      this.viewMode = { kind: "editor" };
      this.error = "";
    } catch (e) {
      this.error = String(e);
      await this.loadRecents();
    }
  }

  async createUntitledNode() {
    try {
      const id = await invoke<string>("create_node", {
        title: "Untitled",
      });
      await this.loadRecents();
      await this.openNode(id);
    } catch (e) {
      this.error = String(e);
    }
  }

  async deleteNode(id: string, e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke("delete_node", { id });
      if (this.activeNodeId === id) {
        this.activeNodeId = null;
        this.activeDoc = null;
        this.activeMeta = null;
        this.viewMode = null;
      }
      await Promise.all([this.loadRecents(), this.loadFavorites()]);
      logs.filterAllMembers((n) => n.id !== id);
    } catch (e) {
      this.error = String(e);
    }
  }

  async toggleFavorite(id: string, e?: MouseEvent) {
    e?.stopPropagation();
    try {
      const newState = await invoke<boolean>("toggle_favorite", { id });
      await this.loadFavorites();
      this.recentNodes = this.recentNodes.map((n) =>
        n.id === id ? { ...n, is_favorite: newState } : n,
      );
      if (this.activeMeta && this.activeMeta.id === id) {
        this.activeMeta = { ...this.activeMeta, is_favorite: newState };
      }
      logs.patchAllMembers((n) =>
        n.id === id ? { ...n, is_favorite: newState } : n,
      );
    } catch (e) {
      this.error = String(e);
    }
  }

  async handleSave(ttJson: object, saveNodeId: string) {
    if (!saveNodeId) return;
    this.saving = true;
    try {
      const doc = tipTapToPmDoc(ttJson, this.activeDoc?.frontmatter);
      await invoke("save_node", { id: saveNodeId, doc });
    } catch (e) {
      if (this.activeNodeId !== saveNodeId) return;
      this.error = String(e);
    } finally {
      this.saving = false;
    }
  }

  async renameNode(id: string, newTitle: string) {
    await invoke("rename_node", { id, title: newTitle });
    if (this.activeMeta?.id === id)
      this.activeMeta = { ...this.activeMeta, title: newTitle };
    this.recentNodes = this.recentNodes.map((n) =>
      n.id === id ? { ...n, title: newTitle } : n,
    );
    this.favorites = this.favorites.map((n) =>
      n.id === id ? { ...n, title: newTitle } : n,
    );
    logs.patchAllMembers((n) => (n.id === id ? { ...n, title: newTitle } : n));
  }
}

export const notes = new NotesStore();
