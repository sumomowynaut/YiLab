// 设置面板：主题、引擎参数、快捷键说明（集中设置入口）。

import { SHORTCUTS } from "../../lib/shortcuts";
import {
  DEPTH_OPTIONS,
  HASH_OPTIONS,
  MULTIPV_OPTIONS,
  THREADS_OPTIONS,
  useEngineStore,
} from "../../stores/useEngineStore";
import { useThemeStore } from "../../stores/useThemeStore";
import { Button } from "../ui/button";

/** 设置面板（主题 + 引擎参数 + 快捷键）。 */
export function SettingsPanel() {
  const theme = useThemeStore((state) => state.theme);
  const toggleTheme = useThemeStore((state) => state.toggleTheme);
  const settings = useEngineStore((state) => state.settings);
  const applySettings = useEngineStore((state) => state.applySettings);

  return (
    <div data-testid="settings-panel" className="flex flex-col gap-3 rounded border p-3">
      <span className="text-sm font-semibold">设置</span>

      <div className="flex flex-col gap-2">
        <span className="text-xs text-muted-foreground">外观</span>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid="settings-theme"
            onClick={toggleTheme}
          >
            {theme === "dark" ? "☀ 浅色模式" : "🌙 深色模式"}
          </Button>
          <span className="text-xs text-muted-foreground">
            当前：{theme === "dark" ? "深色" : "浅色"}
          </span>
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <span className="text-xs text-muted-foreground">引擎参数</span>

        <label className="flex items-center gap-2 text-xs">
          引擎路径
          <input
            data-testid="settings-program"
            value={settings.programPath}
            onChange={(e) => void applySettings({ programPath: e.currentTarget.value })}
            placeholder="粘贴 pikafish-xxx.exe 完整路径"
            className="h-7 flex-1 rounded border border-input bg-background px-2 text-xs"
          />
        </label>

        <div className="grid grid-cols-2 gap-2 text-xs">
          <label className="flex items-center gap-2">
            线程
            <select
              data-testid="settings-threads"
              value={settings.threads}
              onChange={(e) => void applySettings({ threads: Number(e.currentTarget.value) })}
              className="h-7 w-20 rounded border border-input bg-background px-1"
            >
              {THREADS_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2">
            哈希(MB)
            <select
              data-testid="settings-hash"
              value={settings.hash}
              onChange={(e) => void applySettings({ hash: Number(e.currentTarget.value) })}
              className="h-7 w-20 rounded border border-input bg-background px-1"
            >
              {HASH_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2">
            深度
            <select
              data-testid="settings-depth"
              value={settings.depth ?? 0}
              onChange={(e) =>
                void applySettings({
                  depth: Number(e.currentTarget.value) > 0 ? Number(e.currentTarget.value) : null,
                })
              }
              className="h-7 w-20 rounded border border-input bg-background px-1"
            >
              {DEPTH_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n === 0 ? "无限" : n}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2">
            MultiPV
            <select
              data-testid="settings-multipv"
              value={settings.multipv}
              onChange={(e) => void applySettings({ multipv: Number(e.currentTarget.value) })}
              className="h-7 w-20 rounded border border-input bg-background px-1"
            >
              {MULTIPV_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </label>
        </div>
        <p className="text-xs text-muted-foreground">
          参数含义：
          <br />
          <span className="font-semibold">线程</span>：引擎用多少个 CPU 核心并行计算，数值越大越快。
          <br />
          <span className="font-semibold">哈希</span>
          ：分配给引擎思考的内存缓存，数值越大能记住更多局面。
          <br />
          <span className="font-semibold">深度</span>
          ：引擎搜索到第几层为止（实时分析始终持续，不受此限制）。
          <br />
          <span className="font-semibold">MultiPV</span>：同时显示几条最佳变化，数值越大越慢。
          <br />
          参数保存到本地，重启后自动恢复。
        </p>
      </div>

      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">快捷键</span>
        <ul data-testid="settings-shortcuts" className="flex flex-col gap-0.5 text-xs">
          {SHORTCUTS.map((s) => (
            <li key={`${s.action}-${s.label}`} className="flex items-center justify-between">
              <span>{s.description}</span>
              <kbd className="rounded border border-input bg-muted px-1.5 py-0.5 font-mono text-[10px]">
                {s.label}
              </kbd>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
