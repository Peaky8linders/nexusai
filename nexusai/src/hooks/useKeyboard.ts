import { useEffect } from "react";

type KeyHandler = (e: KeyboardEvent) => void;

/**
 * Global keyboard shortcut hook.
 * Supports modifier keys: ctrl/cmd + key combos.
 */
export function useKeyboard(shortcuts: Record<string, KeyHandler>) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Skip if user is typing in an input
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
        // Only allow Escape in inputs
        if (e.key !== "Escape") return;
      }

      const isMod = e.metaKey || e.ctrlKey;
      let key = "";

      if (isMod) key += "mod+";
      if (e.shiftKey) key += "shift+";
      key += e.key.toLowerCase();

      const fn = shortcuts[key];
      if (fn) {
        e.preventDefault();
        fn(e);
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [shortcuts]);
}

/**
 * Focus trap for modal dialogs.
 * Keeps tab focus within the container element.
 */
export function useFocusTrap(containerRef: React.RefObject<HTMLElement | null>, active: boolean) {
  useEffect(() => {
    if (!active || !containerRef.current) return;

    const container = containerRef.current;
    const focusable = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    // Focus first element on mount
    first.focus();

    const handler = (e: KeyboardEvent) => {
      if (e.key !== "Tab") return;

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    };

    container.addEventListener("keydown", handler);
    return () => container.removeEventListener("keydown", handler);
  }, [active, containerRef]);
}
