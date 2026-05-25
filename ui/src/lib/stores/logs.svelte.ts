import { invoke } from "@tauri-apps/api/core";
import type { Log, LogTreeNode, FlatSidebarItem, NodeInfo } from "$lib/types";

// Imported lazily — only used inside method bodies, not at module init time.
// This creates a circular dep with notes.svelte.ts, which is safe because
// neither class constructor references the other store at construction time.
import { notes } from "$lib/stores/notes.svelte";

export function buildTree(logs: Log[], parentId: number | null): LogTreeNode[] {
  return logs
    .filter((l) => l.parent_id === parentId)
    .sort((a, b) => a.sort_key - b.sort_key)
    .map((l) => ({ ...l, children: buildTree(logs, l.id) }));
}

export function flattenTree(
  nodes: LogTreeNode[],
  expanded: Set<number>,
  membersMap: Record<number, NodeInfo[]>,
  depth = 0,
): FlatSidebarItem[] {
  const result: FlatSidebarItem[] = [];
  for (const node of nodes) {
    result.push({ type: "log", log: node, depth });
    if (expanded.has(node.id)) {
      result.push(
        ...flattenTree(node.children, expanded, membersMap, depth + 1),
      );
      for (const member of membersMap[node.id] ?? []) {
        result.push({
          type: "member",
          node: member,
          logId: node.id,
          depth: depth + 1,
        });
      }
    }
  }
  return result;
}

class LogsStore {
  allLogs: Log[] = $state([]);
  logMembersMap: Record<number, NodeInfo[]> = $state({});
  expandedLogs: Set<number> = $state(new Set());
  dragOverLogId: number | null = $state(null);

  // Non-reactive pointer drag state
  #ptrDragId: string | null = null;
  #ptrStartX = 0;
  #ptrStartY = 0;
  #ptrDragging = false;
  #ptrCaptureEl: HTMLElement | null = null;

  filterAllMembers(pred: (n: NodeInfo) => boolean) {
    const updated: Record<number, NodeInfo[]> = {};
    for (const [key, members] of Object.entries(this.logMembersMap)) {
      updated[Number(key)] = members.filter(pred);
    }
    this.logMembersMap = updated;
  }

  patchAllMembers(fn: (n: NodeInfo) => NodeInfo) {
    const updated: Record<number, NodeInfo[]> = {};
    for (const [key, members] of Object.entries(this.logMembersMap)) {
      updated[Number(key)] = members.map(fn);
    }
    this.logMembersMap = updated;
  }

  async loadLogs() {
    try {
      this.allLogs = await invoke<Log[]>("get_log_tree");
    } catch (e) {
      notes.error = String(e);
    }
  }

  async loadLogMembers(logId: number) {
    try {
      const members = await invoke<NodeInfo[]>("get_log_members", {
        logId,
        page: 0,
      });
      this.logMembersMap = { ...this.logMembersMap, [logId]: members };
    } catch (e) {
      notes.error = String(e);
    }
  }

  async openLog(log: LogTreeNode) {
    const wasExpanded = this.expandedLogs.has(log.id);
    if (wasExpanded) {
      this.expandedLogs.delete(log.id);
    } else {
      this.expandedLogs.add(log.id);
      if (!this.logMembersMap[log.id]) {
        await this.loadLogMembers(log.id);
      }
    }
    this.expandedLogs = new Set(this.expandedLogs);
  }

  onNodePointerDown(id: string, e: PointerEvent) {
    if (e.button !== 0) return;
    this.#ptrDragId = id;
    this.#ptrStartX = e.clientX;
    this.#ptrStartY = e.clientY;
    this.#ptrDragging = false;
    this.#ptrCaptureEl = e.currentTarget as HTMLElement;
    this.#ptrCaptureEl.setPointerCapture(e.pointerId);
  }

  onNodePointerMove(e: PointerEvent) {
    if (!this.#ptrDragId) return;
    if (!this.#ptrDragging) {
      const moved =
        Math.abs(e.clientX - this.#ptrStartX) +
        Math.abs(e.clientY - this.#ptrStartY);
      if (moved < 6) return;
      this.#ptrDragging = true;
    }
    const target = document.elementFromPoint(e.clientX, e.clientY);
    const logEl = target?.closest("[data-log-id]") as HTMLElement | null;
    const found = logEl ? Number(logEl.dataset.logId) : null;
    this.dragOverLogId = found;
    if (this.#ptrCaptureEl)
      this.#ptrCaptureEl.style.cursor = found !== null ? "copy" : "not-allowed";
  }

  async onNodePointerUp(id: string) {
    const wasDragging = this.#ptrDragging;
    const targetLogId = this.dragOverLogId;
    this.#ptrDragId = null;
    this.#ptrDragging = false;
    this.dragOverLogId = null;
    if (this.#ptrCaptureEl) {
      this.#ptrCaptureEl.style.cursor = "";
      this.#ptrCaptureEl = null;
    }
    if (wasDragging) {
      if (targetLogId !== null) {
        try {
          await invoke("add_to_log", {
            logId: targetLogId,
            nodeId: id,
          });
          if (this.expandedLogs.has(targetLogId)) {
            await this.loadLogMembers(targetLogId);
          } else {
            const newMap = { ...this.logMembersMap };
            delete newMap[targetLogId];
            this.logMembersMap = newMap;
          }
        } catch (err) {
          notes.error = String(err);
        }
      }
    } else {
      await notes.openNode(id);
    }
  }

  onNodePointerCancel() {
    this.#ptrDragId = null;
    this.#ptrDragging = false;
    this.dragOverLogId = null;
    if (this.#ptrCaptureEl) {
      this.#ptrCaptureEl.style.cursor = "";
      this.#ptrCaptureEl = null;
    }
  }
}

export const logs = new LogsStore();
