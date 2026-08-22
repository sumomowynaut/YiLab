// 全局快捷键 hook：订阅 window keydown，命中 SHORTCUTS 后回调。

import { useEffect, useRef } from "react";
import { resolveShortcut, type ShortcutAction } from "../lib/shortcuts";

export interface ShortcutHandlers {
  onAction: (action: ShortcutAction) => void;
  /** 返回 true 表示忽略本次按键（例如焦点在输入框/文本域时）。 */
  shouldIgnore?: (event: KeyboardEvent) => boolean;
}

export function useShortcuts(handlers: ShortcutHandlers): void {
  const ref = useRef(handlers);
  ref.current = handlers;

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const h = ref.current;
      if (h.shouldIgnore?.(event)) {
        return;
      }
      const action = resolveShortcut(event);
      if (!action) {
        return;
      }
      event.preventDefault();
      h.onAction(action);
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
