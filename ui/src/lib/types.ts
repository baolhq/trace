import type { PmDoc } from "$lib/editor/doc";

export interface NodeInfo {
  id: string;
  title: string;
  created_at: number;
  is_favorite: boolean;
}

export interface NodeMeta {
  id: string;
  title: string;
  created_at: number;
  modified_at: number;
  is_favorite: boolean;
}

export interface OpenNodeResponse {
  meta: NodeMeta;
  doc: PmDoc;
}

export interface Log {
  id: number;
  name: string;
  parent_id: number | null;
  sort_key: number;
}

export interface LogTreeNode extends Log {
  children: LogTreeNode[];
}

export type FlatSidebarItem =
  | { type: "log"; log: LogTreeNode; depth: number }
  | { type: "member"; node: NodeInfo; logId: number; depth: number };

export type ViewMode = { kind: "editor" } | null;

export interface SearchHit {
  id: string;
  title: string;
  snippet: string;
}

export type SearchSubMode = "search" | "replace";

export interface Settings {
  ui_font: string;
  ui_font_size: number;
  content_font: string;
  content_font_size: number;
  mono_font: string;
  mono_font_size: number;
  zoom_level: number;
  inline_title: boolean;
  content_line_height: "compact" | "standard" | "comfortable";
  theme: string;
  wikilink_brackets: boolean;
  download_remote_images: boolean;
  date_format: string;
  epoch_precision: "seconds" | "milliseconds";
  spellcheck: boolean;
  spellcheck_languages: string[];
  close_to_tray: boolean;
  autostart: boolean;
  native_app_bar: boolean;
  multiple_instances: boolean;
  soft_delete: boolean;
  auto_backup: boolean;
  backup_frequency: "daily" | "weekly" | "monthly";
  zen_app_bar: "show" | "hide" | "peek";
  zen_sidebar: "show" | "hide" | "peek";
  zen_right_panel: "show" | "hide" | "peek";
  zen_status_bar: "show" | "hide" | "peek";
  vim_escaper: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  ui_font: "'Roboto Serif', serif",
  ui_font_size: 16,
  content_font: "'Atkinson Hyperlegible', sans-serif",
  content_font_size: 16,
  mono_font: "'Martian Mono', monospace",
  mono_font_size: 16,
  zoom_level: 1.0,
  inline_title: true,
  content_line_height: "standard",
  theme: "trace-dark",
  wikilink_brackets: true,
  download_remote_images: true,
  date_format: "MMMM d, yyyy",
  epoch_precision: "seconds",
  spellcheck: false,
  spellcheck_languages: ["en-US"],
  close_to_tray: true,
  autostart: false,
  native_app_bar: false,
  multiple_instances: true,
  soft_delete: true,
  auto_backup: false,
  backup_frequency: "daily",
  zen_app_bar: "peek",
  zen_sidebar: "peek",
  zen_right_panel: "peek",
  zen_status_bar: "peek",
  vim_escaper: false,
};

export interface AllSettings {
  global: Settings;
  vault: Settings;
  merged: Settings;
}

export type SettingsScope = "global" | "vault";
