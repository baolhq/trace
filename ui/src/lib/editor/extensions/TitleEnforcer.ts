import type { EditorState, Transaction } from "@tiptap/pm/state";
import { Extension } from "@tiptap/core";
import {
  AllSelection,
  Plugin,
  PluginKey,
  TextSelection,
} from "@tiptap/pm/state";

const KEY = new PluginKey("titleEnforcer");

export const TitleEnforcer = Extension.create({
  name: "titleEnforcer",

  addKeyboardShortcuts() {
    return {
      "Mod-a": () => {
        const { state } = this.editor;
        const { doc, selection } = state;
        const firstNode = doc.firstChild;

        if (!firstNode || firstNode.type !== state.schema.nodes.heading) {
          return false;
        }

        const titleFrom = 1;
        const titleTo = 1 + firstNode.content.size;
        const hasBody = doc.content.size > firstNode.nodeSize;

        // Anchor inside the title?
        const inTitle = selection.$from.before(1) === 0;

        if (inTitle) {
          if (selection.from === titleFrom && selection.to === titleTo) {
            // Title already selected — expand to all
            this.editor.view.dispatch(
              state.tr.setSelection(new AllSelection(doc)),
            );
          } else {
            this.editor.view.dispatch(
              state.tr.setSelection(
                TextSelection.create(doc, titleFrom, titleTo),
              ),
            );
          }
        } else {
          if (!hasBody) {
            this.editor.view.dispatch(
              state.tr.setSelection(new AllSelection(doc)),
            );
            return true;
          }
          const bodyFrom = firstNode.nodeSize + 1;
          const bodyTo = doc.content.size - 1;
          if (selection.from <= bodyFrom && selection.to >= bodyTo) {
            // Body already selected — expand to all
            this.editor.view.dispatch(
              state.tr.setSelection(new AllSelection(doc)),
            );
          } else {
            this.editor.view.dispatch(
              state.tr.setSelection(
                TextSelection.create(doc, bodyFrom, bodyTo),
              ),
            );
          }
        }
        return true;
      },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: KEY,
        appendTransaction(
          transactions: readonly Transaction[],
          _oldState: EditorState,
          newState: EditorState,
        ) {
          if (!transactions.some((tr: Transaction) => tr.docChanged))
            return null;

          const firstNode = newState.doc.firstChild;
          if (!firstNode) return null;

          const { heading, paragraph } = newState.schema.nodes;
          if (firstNode.type === heading && firstNode.attrs.level === 1)
            return null;

          // Only correct paragraphs or mis-levelled headings; leave other block types alone
          if (firstNode.type !== paragraph && firstNode.type !== heading)
            return null;

          return newState.tr.setNodeMarkup(0, heading, { level: 1 });
        },
      }),
    ];
  },
});
