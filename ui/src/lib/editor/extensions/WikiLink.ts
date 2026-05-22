import { Node, mergeAttributes } from "@tiptap/core";

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
});
