export interface SelectionSnapshot {
  text: string;
  x: number;
  y: number;
  context: string;
}

export function readSelectionSnapshot(container: HTMLElement | null): SelectionSnapshot | null {
  const selection = window.getSelection();
  const text = selection?.toString().trim() ?? "";
  if (!selection || text.length < 2 || !container) {
    return null;
  }
  const anchorInContainer = selection.anchorNode ? container.contains(selection.anchorNode) : false;
  const focusInContainer = selection.focusNode ? container.contains(selection.focusNode) : false;
  if (!anchorInContainer && !focusInContainer) {
    return null;
  }
  const range = selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
  const rect = range?.getBoundingClientRect() ?? range?.getClientRects()[0];
  if (!rect) {
    return null;
  }
  const popoverWidth = Math.min(340, window.innerWidth - 24);
  const left = Math.max(12, Math.min(rect.left + rect.width / 2 - popoverWidth / 2, window.innerWidth - popoverWidth - 12));
  const top = Math.max(12, Math.min(rect.bottom + 10, window.innerHeight - 80));
  return {
    text: text.slice(0, 120),
    x: left,
    y: top,
    context: container.textContent?.slice(0, 800) ?? ""
  };
}
