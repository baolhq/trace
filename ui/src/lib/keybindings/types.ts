export interface KeyCombo {
  key: string; // matches event.key, case-insensitive
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
}

export type ActionId = string;

export interface KeybindingEntry {
  id: ActionId;
  combo: KeyCombo | KeyCombo[]; // KeyCombo[] = chord sequence
  description: string;
}

export type HandlerFn = (event: KeyboardEvent) => void;
