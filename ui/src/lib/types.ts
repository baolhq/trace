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
