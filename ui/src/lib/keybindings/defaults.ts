import type { KeybindingEntry } from "./types";

export const DEFAULT_PROFILE: KeybindingEntry[] = [
  // App
  { id: "app.open-vault", combo: [{ key: "k", ctrl: true }, { key: "o", ctrl: true }] },
  { id: "app.reveal-vault", combo: [{ key: "k", ctrl: true }, { key: "v", ctrl: true }] },
  // TODO: app.show-settings — double-press Ctrl+, to open TOML not yet implemented
  { id: "app.show-settings", combo: { key: ",", ctrl: true } },
  { id: "app.show-version", combo: { key: "F1" } },
  { id: "app.file-search", combo: { key: "p", ctrl: true } },
  // TODO: app.file-search — Shift+Shift alias not yet implemented
  { id: "app.command-palette", combo: { key: "p", ctrl: true, shift: true } },

  // Panels
  { id: "panel.search", combo: { key: "f", ctrl: true, shift: true } },
  { id: "panel.replace", combo: { key: "h", ctrl: true, shift: true } },
  // TODO: panel.toggle-left — sidebar toggle not yet implemented
  { id: "panel.toggle-left", combo: { key: ",", ctrl: true, shift: true } },
  // TODO: panel.toggle-right — right panel toggle not yet implemented
  { id: "panel.toggle-right", combo: { key: ".", ctrl: true, shift: true } },
  { id: "panel.traces", combo: { key: "t", alt: true } },
  // TODO: panel.outlines — outlines panel not yet implemented
  { id: "panel.outlines", combo: { key: "o", alt: true } },
  // TODO: panel.links — links panel not yet implemented
  { id: "panel.links", combo: { key: "l", alt: true } },
  // TODO: panel.backlinks — backlinks panel not yet implemented
  { id: "panel.backlinks", combo: { key: "b", alt: true } },
  // TODO: panel.duplicate — panel duplication not yet implemented
  { id: "panel.duplicate", combo: { key: "d", ctrl: true } },

  // Editor — navigation & lifecycle
  // TODO: editor.journal — Journal page not yet implemented
  { id: "editor.journal", combo: { key: "j", alt: true } },
  // TODO: editor.journal-latest — Journal page not yet implemented
  { id: "editor.journal-latest", combo: { key: "j", alt: true, shift: true } },
  { id: "editor.new-trace", combo: { key: "n", ctrl: true } },
  { id: "editor.close", combo: { key: "w", ctrl: true } },
  { id: "editor.find", combo: { key: "f", ctrl: true } },
  { id: "editor.replace", combo: { key: "h", ctrl: true } },
  { id: "editor.focus-content", combo: { key: "e", ctrl: true } },
  // TODO: editor.focus-title — title focus + select not yet implemented
  { id: "editor.focus-title", combo: { key: "F2" } },

  // Editor — line operations
  { id: "editor.move-line-up", combo: { key: "ArrowUp", alt: true } },
  { id: "editor.move-line-down", combo: { key: "ArrowDown", alt: true } },
  { id: "editor.duplicate-block", combo: { key: "ArrowDown", alt: true, shift: true } },

  // Editor — formatting
  { id: "editor.toggle-bold", combo: { key: "b", ctrl: true } },
  { id: "editor.toggle-italic", combo: { key: "i", ctrl: true } },
  { id: "editor.toggle-underline", combo: { key: "u", ctrl: true } },
  // TODO: editor.toggle-strike — needs TipTap extension
  { id: "editor.toggle-strike", combo: { key: "s", alt: true, shift: true } },

  // Editor — font zoom
  { id: "editor.zoom-in", combo: { key: "=", ctrl: true } },
  { id: "editor.zoom-out", combo: { key: "-", ctrl: true } },
  // TODO: editor.zoom-reset — needs default CSS font size extraction
  { id: "editor.zoom-reset", combo: { key: "0", ctrl: true } },

  // Editor — links & blocks
  { id: "editor.create-link", combo: [{ key: "k", ctrl: true }, { key: "l", ctrl: true }] },
  { id: "editor.remove-link", combo: [{ key: "k", ctrl: true }, { key: "d", ctrl: true }] },
  // TODO: editor.select-block — action semantics not yet settled
  { id: "editor.select-block", combo: [{ key: "k", ctrl: true }, { key: "p", ctrl: true }] },

  // Editor — insert
  // TODO: editor.insert-date — needs persistent datetime format setting
  { id: "editor.insert-date", combo: { key: "d", ctrl: true, shift: true } },
  { id: "editor.insert-time", combo: { key: "t", ctrl: true, shift: true } },

  // Editor — view modes
  { id: "editor.source-mode", combo: [{ key: "k", ctrl: true }, { key: "s", ctrl: true }] },
  { id: "editor.live-mode", combo: [{ key: "k", ctrl: true }, { key: "e", ctrl: true }] },
  { id: "editor.reader-mode", combo: [{ key: "k", ctrl: true }, { key: "r", ctrl: true }] },
  { id: "editor.cycle-mode", combo: [{ key: "k", ctrl: true }, { key: "m", ctrl: true }] },

  // Editor — misc
  { id: "editor.open-link", combo: { key: "Enter", alt: true } },
  // TODO: editor.toggle-favorite — requires selected note context
  { id: "editor.toggle-favorite", combo: { key: "f", alt: true } },

  // Navigation
  // TODO: navigation.back / navigation.forward — navigation memory not yet implemented
  { id: "navigation.back", combo: { key: "ArrowLeft", alt: true } },
  { id: "navigation.forward", combo: { key: "ArrowRight", alt: true } },
  { id: "navigation.refresh", combo: { key: "F5" } },

  // Logs
  { id: "log.new-log", combo: [{ key: "k", ctrl: true }, { key: "n", ctrl: true }] },
  { id: "log.add-to", combo: [{ key: "k", ctrl: true }, { key: "a", ctrl: true }] },

  // View
  { id: "view.toggle-fullscreen", combo: { key: "F11" } },
  { id: "view.toggle-zen", combo: [{ key: "k", ctrl: true }, { key: "z", ctrl: true }] },
  // TODO: view.reset-layout — needs persistent layout implementation
  { id: "view.reset-layout", combo: [{ key: "k", ctrl: true }, { key: "0", ctrl: true }] },
  { id: "view.zoom-in", combo: { key: "=", ctrl: true, shift: true } },
  { id: "view.zoom-out", combo: { key: "-", ctrl: true, shift: true } },
  { id: "view.zoom-reset", combo: { key: "0", ctrl: true, shift: true } },
];
