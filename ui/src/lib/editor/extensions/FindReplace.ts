import { Extension } from "@tiptap/core";
import { Plugin, PluginKey, TextSelection } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Node as PmNode } from "@tiptap/pm/model";

export interface SearchOptions {
  matchCase?: boolean;
  wholeWord?: boolean;
  useRegex?: boolean;
}

export interface FindReplacePluginState {
  searchTerm: string;
  options: SearchOptions;
  matches: { from: number; to: number }[];
  currentIndex: number;
}

export const findReplaceKey = new PluginKey<FindReplacePluginState>(
  "findReplace",
);

export function getFindReplaceState(state: {
  doc: unknown;
}): FindReplacePluginState {
  return (
    findReplaceKey.getState(
      state as Parameters<typeof findReplaceKey.getState>[0],
    ) ?? {
      searchTerm: "",
      options: {},
      matches: [],
      currentIndex: 0,
    }
  );
}

function buildMatches(
  doc: PmNode,
  searchTerm: string,
  options: SearchOptions = {},
): { from: number; to: number }[] {
  if (!searchTerm) return [];
  const { matchCase = false, wholeWord = false, useRegex = false } = options;

  let pattern: RegExp;
  try {
    let source = useRegex
      ? searchTerm
      : searchTerm.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    if (wholeWord) source = `\\b${source}\\b`;
    pattern = new RegExp(source, matchCase ? "g" : "gi");
  } catch {
    return [];
  }

  const results: { from: number; to: number }[] = [];
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return;
    pattern.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = pattern.exec(node.text)) !== null) {
      if (m[0].length === 0) {
        pattern.lastIndex++;
        continue;
      }
      results.push({ from: pos + m.index, to: pos + m.index + m[0].length });
    }
  });
  return results;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    findReplace: {
      setSearchTerm: (term: string, options?: SearchOptions) => ReturnType;
      findNext: () => ReturnType;
      findPrev: () => ReturnType;
      replaceOne: (replacement: string) => ReturnType;
      replaceAll: (replacement: string) => ReturnType;
    };
  }
}

export const FindReplace = Extension.create({
  name: "findReplace",

  addCommands() {
    return {
      setSearchTerm:
        (term: string, options?: SearchOptions) =>
        ({ dispatch, state }) => {
          const opts = options ?? {};
          const matches = buildMatches(state.doc, term, opts);
          if (dispatch) {
            const tr = state.tr.setMeta(findReplaceKey, {
              searchTerm: term,
              options: opts,
              matches,
              currentIndex: 0,
            });
            if (matches.length > 0) {
              tr.setSelection(
                TextSelection.create(state.doc, matches[0].from, matches[0].to),
              );
              tr.scrollIntoView();
            }
            dispatch(tr);
          }
          return true;
        },

      findNext:
        () =>
        ({ dispatch, state }) => {
          const ps = getFindReplaceState(state);
          if (!ps.matches.length) return false;
          const nextIdx = (ps.currentIndex + 1) % ps.matches.length;
          const match = ps.matches[nextIdx];
          if (dispatch) {
            const tr = state.tr.setMeta(findReplaceKey, {
              ...ps,
              currentIndex: nextIdx,
            });
            tr.setSelection(
              TextSelection.create(state.doc, match.from, match.to),
            );
            tr.scrollIntoView();
            dispatch(tr);
          }
          return true;
        },

      findPrev:
        () =>
        ({ dispatch, state }) => {
          const ps = getFindReplaceState(state);
          if (!ps.matches.length) return false;
          const prevIdx =
            (ps.currentIndex - 1 + ps.matches.length) % ps.matches.length;
          const match = ps.matches[prevIdx];
          if (dispatch) {
            const tr = state.tr.setMeta(findReplaceKey, {
              ...ps,
              currentIndex: prevIdx,
            });
            tr.setSelection(
              TextSelection.create(state.doc, match.from, match.to),
            );
            tr.scrollIntoView();
            dispatch(tr);
          }
          return true;
        },

      replaceOne:
        (replacement: string) =>
        ({ dispatch, state }) => {
          const ps = getFindReplaceState(state);
          if (!ps.matches.length) return false;
          const match = ps.matches[ps.currentIndex];
          if (dispatch) {
            const tr = state.tr;
            if (replacement) {
              tr.replaceWith(
                match.from,
                match.to,
                state.schema.text(replacement),
              );
            } else {
              tr.delete(match.from, match.to);
            }
            dispatch(tr);
          }
          return true;
        },

      replaceAll:
        (replacement: string) =>
        ({ dispatch, state }) => {
          const ps = getFindReplaceState(state);
          if (!ps.matches.length) return false;
          if (dispatch) {
            let tr = state.tr;
            for (let i = ps.matches.length - 1; i >= 0; i--) {
              const m = ps.matches[i];
              if (replacement) {
                tr = tr.replaceWith(
                  m.from,
                  m.to,
                  state.schema.text(replacement),
                );
              } else {
                tr = tr.delete(m.from, m.to);
              }
            }
            dispatch(tr);
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: findReplaceKey,
        state: {
          init(): FindReplacePluginState {
            return {
              searchTerm: "",
              options: {},
              matches: [],
              currentIndex: 0,
            };
          },
          apply(tr, value): FindReplacePluginState {
            const meta = tr.getMeta(findReplaceKey) as
              | FindReplacePluginState
              | undefined;
            if (meta) return meta;
            if (tr.docChanged && value.searchTerm) {
              const matches = buildMatches(
                tr.doc,
                value.searchTerm,
                value.options,
              );
              return {
                ...value,
                matches,
                currentIndex: Math.min(
                  value.currentIndex,
                  Math.max(0, matches.length - 1),
                ),
              };
            }
            return value;
          },
        },
        props: {
          decorations(state) {
            const ps = findReplaceKey.getState(state);
            if (!ps?.searchTerm || !ps.matches.length)
              return DecorationSet.empty;
            const decos = ps.matches.map((m, i) =>
              Decoration.inline(m.from, m.to, {
                class: i === ps.currentIndex ? "find-current" : "find-match",
              }),
            );
            return DecorationSet.create(state.doc, decos);
          },
        },
      }),
    ];
  },
});
