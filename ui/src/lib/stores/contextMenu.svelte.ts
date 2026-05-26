import type { Snippet } from "svelte";

export type ContextMenuItem =
  | {
      kind: "action";
      label: string;
      icon?: Snippet;
      action: () => void;
      danger?: boolean;
    }
  | { kind: "separator" };

class ContextMenuStore {
  visible = $state(false);
  x = $state(0);
  y = $state(0);
  items: ContextMenuItem[] = $state([]);

  open(x: number, y: number, items: ContextMenuItem[]) {
    this.x = Math.round(x);
    this.y = Math.round(y);
    this.items = items;
    this.visible = true;
  }

  close() {
    this.visible = false;
  }
}

export const contextMenu = new ContextMenuStore();
