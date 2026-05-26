import { Node, mergeAttributes, nodeInputRule } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";

export interface WikiLinkOptions {
  HTMLAttributes: Record<string, unknown>;
  onNavigate?: (target: string, isIdRef: boolean) => void;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    wikiLink: {
      insertWikiLink: (target: string, isIdRef?: boolean) => ReturnType;
    };
  }
}

export const WikiLink = Node.create<WikiLinkOptions>({
  name: "wikiLink",
  group: "inline",
  inline: true,
  atom: true, // non-editable single unit

  addOptions() {
    return {
      HTMLAttributes: {},
      onNavigate: undefined,
    };
  },

  addAttributes() {
    return {
      target: { default: null },
      isIdRef: { default: false },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-wiki-link]" }];
  },

  renderHTML({ node, HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        "data-wiki-link": node.attrs.target,
        "data-id-ref": node.attrs.isIdRef ? "true" : null,
        class: "wiki-link",
      }),
      node.attrs.isIdRef
        ? `[[node:${node.attrs.target}]]`
        : `[[${node.attrs.target}]]`,
    ];
  },

  addInputRules() {
    // Converts [[target]] typed by the user into a wikiLink atom node.
    return [
      nodeInputRule({
        find: /\[\[([^\]]+)]]$/,
        type: this.type,
        getAttributes: (match) => {
          const raw = match[1];
          const isIdRef = raw.startsWith("node:");
          return { target: isIdRef ? raw.slice(5) : raw, isIdRef };
        },
      }),
    ];
  },

  addCommands() {
    return {
      insertWikiLink:
        (target: string, isIdRef = false) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs: { target, isIdRef },
          });
        },
    };
  },

  addProseMirrorPlugins() {
    const { onNavigate } = this.options;
    return [
      new Plugin({
        key: new PluginKey("wikiLinkClick"),
        props: {
          handleClick(_view, _pos, event) {
            const el = event.target as HTMLElement;
            const span = el.closest("[data-wiki-link]") as HTMLElement | null;
            if (!span) return false;
            const target = span.getAttribute("data-wiki-link");
            const isIdRef = span.getAttribute("data-id-ref") === "true";
            if (target && onNavigate) {
              onNavigate(target, isIdRef);
              return true;
            }
            return false;
          },
        },
      }),
    ];
  },
});
