import type { Action } from "svelte/action";

interface ClickOutsideParams {
  onClose: () => void;
  exclude?: (HTMLElement | null)[];
  closeOnEscape?: boolean;
  returnFocusTo?: HTMLElement | null;
  closeOnScroll?: boolean;
}

export const clickOutside: Action<HTMLElement, ClickOutsideParams> = (
  node,
  params,
) => {
  let {
    onClose,
    exclude = [],
    closeOnEscape = false,
    returnFocusTo,
    closeOnScroll = false,
  } = params;

  function handleClick(e: MouseEvent) {
    const t = e.target as Node;
    const excl = exclude.filter((el): el is HTMLElement => el !== null);
    if (!node.contains(t) && !excl.some((el) => el.contains(t))) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
      returnFocusTo?.focus();
      e.stopPropagation();
    }
  }

  function handleScroll(e: Event) {
    if (node.contains(e.target as Node)) return;
    onClose();
  }

  document.addEventListener("click", handleClick);
  if (closeOnEscape)
    document.addEventListener("keydown", handleKeydown, { capture: true });
  if (closeOnScroll)
    document.addEventListener("scroll", handleScroll, { capture: true });

  return {
    destroy() {
      document.removeEventListener("click", handleClick);
      if (closeOnEscape)
        document.removeEventListener("keydown", handleKeydown, {
          capture: true,
        });
      if (closeOnScroll)
        document.removeEventListener("scroll", handleScroll, {
          capture: true,
        });
    },
    update(newParams: ClickOutsideParams) {
      onClose = newParams.onClose;
      exclude = newParams.exclude ?? [];
      returnFocusTo = newParams.returnFocusTo;
    },
  };
};
