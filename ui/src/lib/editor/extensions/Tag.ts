import { Node, mergeAttributes, InputRule } from "@tiptap/core";

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

  addInputRules() {
    return [
      new InputRule({
        // Matches #word followed by a space. The handler fires before the
        // space is inserted (ProseMirror suppresses it when we return steps),
        // so range covers only "#word" — we replace it and re-insert the space.
        find: /#([\w-]+)\s$/,
        handler: ({ state, range, match }) => {
          const name = match[1];
          const { tr, schema } = state;
          const tagNode = schema.nodes[this.name].create({ name });
          tr.replaceWith(range.from, range.to, tagNode);
          tr.insertText(" ", range.from + tagNode.nodeSize);
        },
      }),
    ];
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
