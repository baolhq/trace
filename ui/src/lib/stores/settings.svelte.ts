import { invoke } from "@tauri-apps/api/core";
import type { AllSettings, Settings, SettingsScope } from "$lib/types";
import { DEFAULT_SETTINGS } from "$lib/types";

class SettingsStore {
  global: Settings = $state({ ...DEFAULT_SETTINGS });
  vault: Settings = $state({ ...DEFAULT_SETTINGS });
  merged: Settings = $state({ ...DEFAULT_SETTINGS });
  loaded = $state(false);

  #saveTimer: ReturnType<typeof setTimeout> | null = null;
  #pendingScope: SettingsScope | null = null;

  async load() {
    const r = await invoke<AllSettings>("get_settings");
    this.global = r.global;
    this.vault = r.vault;
    this.merged = r.merged;
    this.loaded = true;
  }

  set<K extends keyof Settings>(
    scope: SettingsScope,
    key: K,
    value: Settings[K],
  ) {
    if (scope === "global") {
      this.global = { ...this.global, [key]: value };
    } else {
      this.vault = { ...this.vault, [key]: value };
    }
    this.#scheduleSave(scope);
  }

  resetField<K extends keyof Settings>(scope: SettingsScope, key: K) {
    this.set(scope, key, DEFAULT_SETTINGS[key]);
  }

  #scheduleSave(scope: SettingsScope) {
    this.#pendingScope = scope;
    if (this.#saveTimer) clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(async () => {
      const savedScope = this.#pendingScope!;
      const settings = savedScope === "global" ? this.global : this.vault;
      try {
        await invoke("save_settings", { scope: savedScope, settings });
        const r = await invoke<AllSettings>("get_settings");
        this.merged = r.merged;
      } catch (e) {
        console.error("settings save failed:", e);
      }
      this.#saveTimer = null;
      this.#pendingScope = null;
    }, 400);
  }
}

export const settings = new SettingsStore();
