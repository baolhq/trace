import { DEFAULT_PROFILE } from "./defaults";
import type { ActionId, HandlerFn, KeyCombo, KeybindingEntry } from "./types";

// ── helpers ──────────────────────────────────────────────────────────────────

function toSteps(combo: KeyCombo | KeyCombo[]): KeyCombo[] {
  return Array.isArray(combo) ? combo : [combo];
}

function comboFromEvent(e: KeyboardEvent): KeyCombo {
  return {
    key: e.key,
    ctrl: e.ctrlKey || undefined,
    shift: e.shiftKey || undefined,
    alt: e.altKey || undefined,
    meta: e.metaKey || undefined,
  };
}

function combosMatch(a: KeyCombo, b: KeyCombo): boolean {
  return (
    a.key.toLowerCase() === b.key.toLowerCase() &&
    !!a.ctrl === !!b.ctrl &&
    !!a.shift === !!b.shift &&
    !!a.alt === !!b.alt &&
    !!a.meta === !!b.meta
  );
}

export function formatCombo(combo: KeyCombo): string {
  const parts: string[] = [];
  if (combo.ctrl) parts.push("Ctrl");
  if (combo.shift) parts.push("Shift");
  if (combo.alt) parts.push("Alt");
  if (combo.meta) parts.push("Meta");
  parts.push(combo.key === " " ? "Space" : combo.key.toUpperCase());
  return parts.join("+");
}

// ── registry ─────────────────────────────────────────────────────────────────

class KeybindingRegistry {
  bindings = $state<KeybindingEntry[]>([...DEFAULT_PROFILE]);

  /**
   * Non-null while waiting for the second key of a chord.
   * Reactive so a status bar can display "Ctrl+K …" as a hint.
   */
  pendingChord = $state<KeyCombo | null>(null);

  readonly #handlers = new Map<ActionId, HandlerFn[]>();
  #chordTimer: ReturnType<typeof setTimeout> | null = null;

  /**
   * Register a handler for an action. Returns an unsubscribe function.
   * The most recently registered handler wins when multiple are present.
   */
  on(id: ActionId, handler: HandlerFn): () => void {
    if (!this.#handlers.has(id)) this.#handlers.set(id, []);
    this.#handlers.get(id)!.push(handler);
    return () => {
      const list = this.#handlers.get(id);
      if (!list) return;
      const idx = list.lastIndexOf(handler);
      if (idx !== -1) list.splice(idx, 1);
    };
  }

  /** Override the combo for an action at runtime (single or chord). */
  rebind(id: ActionId, combo: KeyCombo | KeyCombo[]): void {
    this.bindings = this.bindings.map((b) =>
      b.id === id ? { ...b, combo } : b,
    );
  }

  // ── event handler ─────────────────────────────────────────────────────────

  handle(event: KeyboardEvent): void {
    const current = comboFromEvent(event);

    // Escape always cancels a pending chord without processing further.
    if (event.key === "Escape" && this.pendingChord !== null) {
      event.preventDefault();
      this.#clearChord();
      return;
    }

    // ── chord: waiting for second key ─────────────────────────────────────
    if (this.pendingChord !== null) {
      const leader = this.pendingChord;
      this.#clearChord();

      for (const b of this.bindings) {
        const steps = toSteps(b.combo);
        if (
          steps.length === 2 &&
          combosMatch(leader, steps[0]) &&
          combosMatch(current, steps[1])
        ) {
          this.#fire(b, event);
          return;
        }
      }
      // No chord matched — absorb the key press, chord is cancelled.
      return;
    }

    // ── check if current key is a chord leader ────────────────────────────
    const isLeader = this.bindings.some((b) => {
      const steps = toSteps(b.combo);
      return steps.length > 1 && combosMatch(current, steps[0]);
    });
    if (isLeader) {
      event.preventDefault();
      this.pendingChord = current;
      this.#chordTimer = setTimeout(() => this.#clearChord(), 1500);
      return;
    }

    // ── single-step matching ──────────────────────────────────────────────
    for (const b of this.bindings) {
      const steps = toSteps(b.combo);
      if (steps.length === 1 && combosMatch(current, steps[0])) {
        this.#fire(b, event);
        return;
      }
    }
  }

  // ── private ───────────────────────────────────────────────────────────────

  #clearChord(): void {
    if (this.#chordTimer !== null) {
      clearTimeout(this.#chordTimer);
      this.#chordTimer = null;
    }
    this.pendingChord = null;
  }

  #fire(binding: KeybindingEntry, event: KeyboardEvent): void {
    const handlers = this.#handlers.get(binding.id);
    if (!handlers?.length) return;
    event.preventDefault();
    handlers[handlers.length - 1](event);
  }
}

export const keybindings = new KeybindingRegistry();

/** Mount a capture-phase global listener. Call the returned cleanup in onDestroy. */
export function initKeybindings(): () => void {
  const onKeyDown = (e: KeyboardEvent) => keybindings.handle(e);
  window.addEventListener("keydown", onKeyDown, true);
  return () => window.removeEventListener("keydown", onKeyDown, true);
}
