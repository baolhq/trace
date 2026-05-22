import { Node, mergeAttributes } from "@tiptap/core";

export interface TagOptions {
  HTMLAttributes: Record<string, unknown>;
}

export const Tag = Node.create<TagOptions>({
  name: "tag",
  group: "inline",
  inline: true,
  atom: true,

  addOptions() {
    return { HTMLAttributes: {} };
  },

  addAttributes() {
    return {
      name: { default: null },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-tag]" }];
  },

  renderHTML({ node, HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        "data-tag": node.attrs.name,
        class: "tag",
      }),
      `#${node.attrs.name}`,
    ];
  },
});
