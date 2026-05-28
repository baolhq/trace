export interface DropdownPos {
  top: number;
  right: number;
  minWidth: number;
}

export function positionDropdown(
  triggerEl: HTMLElement,
  estimatedHeight: number,
  buffer = 4,
): DropdownPos {
  const rect = triggerEl.getBoundingClientRect();
  const spaceBelow = window.innerHeight - rect.bottom;
  const top =
    spaceBelow >= estimatedHeight + buffer
      ? rect.bottom + buffer
      : Math.max(8, rect.top - estimatedHeight - buffer);
  return { top, right: window.innerWidth - rect.right, minWidth: rect.width };
}
