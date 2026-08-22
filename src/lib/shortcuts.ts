// 全局快捷键清单与匹配逻辑（可配置的单一事实来源）。
//
// 扩展方式：在 SHORTCUTS 追加一条 ShortcutDef 即可，App 侧在
// useShortcuts 的 onAction 中处理对应 action。

export type ShortcutAction =
  | "previous"
  | "next"
  | "goToStart"
  | "goToEnd"
  | "undo"
  | "redo"
  | "flip"
  | "mirror"
  | "toggleAnalysis";

export interface ShortcutDef {
  action: ShortcutAction;
  /** 匹配用按键（KeyboardEvent.key 的值，如 "ArrowLeft"、"F"、" "）。 */
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  /** 展示标签（如 "Ctrl+Z"）。 */
  label: string;
  /** 中文描述（展示用）。 */
  description: string;
}

export const SHORTCUTS: ShortcutDef[] = [
  { action: "previous", key: "ArrowLeft", label: "←", description: "上一步" },
  { action: "next", key: "ArrowRight", label: "→", description: "下一步" },
  { action: "goToStart", key: "Home", label: "Home", description: "回到起点" },
  { action: "goToEnd", key: "End", label: "End", description: "走到终点" },
  { action: "flip", key: "F", label: "F", description: "翻转棋盘（180°）" },
  { action: "mirror", key: "M", label: "M", description: "左右镜像" },
  { action: "toggleAnalysis", key: " ", label: "Space", description: "开始 / 停止分析" },
  { action: "undo", key: "Z", ctrl: true, label: "Ctrl+Z", description: "悔棋" },
  { action: "redo", key: "Z", ctrl: true, shift: true, label: "Ctrl+Shift+Z", description: "重做" },
  { action: "redo", key: "Y", ctrl: true, label: "Ctrl+Y", description: "重做" },
];

interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}

/** 把键盘事件匹配为动作；未命中返回 null。metaKey 视同 Ctrl（macOS）。 */
export function resolveShortcut(event: KeyLike): ShortcutAction | null {
  const ctrl = event.ctrlKey || event.metaKey;
  const key = event.key.toLowerCase();
  for (const def of SHORTCUTS) {
    if (
      key === def.key.toLowerCase() &&
      ctrl === (def.ctrl ?? false) &&
      event.shiftKey === (def.shift ?? false)
    ) {
      return def.action;
    }
  }
  return null;
}
