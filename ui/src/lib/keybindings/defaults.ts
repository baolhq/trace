import type { KeybindingEntry } from "./types";

export const DEFAULT_PROFILE: KeybindingEntry[] = [
  {
    id: "app.new-note",
    combo: { key: "n", ctrl: true },
    description: "Create a new note",
  },
  {
    id: "app.search",
    combo: { key: "f", ctrl: true, shift: true },
    description: "Open search panel",
  },
  {
    id: "app.sidebar.notes",
    combo: { key: "1", ctrl: true },
    description: "Switch sidebar to notes view",
  },
  {
    id: "app.sidebar.search",
    combo: { key: "2", ctrl: true },
    description: "Switch sidebar to search view",
  },
  {
    id: "app.focus-editor",
    combo: { key: "e", ctrl: true },
    description: "Focus the editor",
  },
  // Two-step chords — first key is the leader (Ctrl+K here).
  {
    id: "log.new-log",
    combo: [
      { key: "k", ctrl: true },
      { key: "n", ctrl: true },
    ],
    description: "Create a new log  (Ctrl+K  Ctrl+N)",
  },
];
