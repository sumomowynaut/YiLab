import type { EngineSettings } from "../../stores/useEngineStore";
import type { BestMoveDto, EngineStatus, InfoLineDto } from "../../lib/engine/types";

export interface AnalysisPanelProps {
  status: EngineStatus;
  engineId: string | null;
  lines: Record<number, InfoLineDto>;
  bestMove: BestMoveDto | null;
  settings: EngineSettings;
  message: string | null;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  onApplySettings: (patch: Partial<EngineSettings>) => void;
  onPreview: (pv: string[]) => void;
}

function formatScore(score: InfoLineDto["score"]): string {
  if (!score) return "—";
  if ("cp" in score) {
    const v = score.cp;
    return `${v > 0 ? "+" : ""}${v}`;
  }
  return `绝杀 ${score.mate}`;
}

function formatNum(v: number | null): string {
  return v == null ? "—" : v.toLocaleString("en-US");
}

const MULTIPV_OPTIONS = [1, 2, 3, 5, 10];

function StatusBadge({ status }: { status: EngineStatus }) {
  const map: Record<EngineStatus, string> = {
    stopped: "未启动",
    ready: "就绪",
    searching: "分析中",
    crashed: "崩溃",
  };
  const cls: Record<EngineStatus, string> = {
    stopped: "bg-muted text-muted-foreground",
    ready: "bg-green-100 text-green-700",
    searching: "bg-amber-100 text-amber-700",
    crashed: "bg-red-100 text-red-700",
  };
  return (
    <span
      className={`rounded px-1.5 py-0.5 text-xs font-medium ${cls[status]}`}
      data-testid="engine-status"
    >
      {map[status]}
    </span>
  );
}

/** 引擎分析面板：评价 / 深度 / 节点 / NPS / 时间 / MultiPV / PV，支持预览与参数设置。 */
export function AnalysisPanel({
  status,
  engineId,
  lines,
  bestMove,
  settings,
  message,
  onStart,
  onStop,
  onRestart,
  onApplySettings,
  onPreview,
}: AnalysisPanelProps) {
  const sorted = Object.values(lines).sort((a, b) => a.multipv - b.multipv);
  const running = status === "searching";

  return (
    <div className="flex flex-col gap-2 rounded border p-3" data-testid="analysis-panel">
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold">引擎分析</span>
        <StatusBadge status={status} />
        {engineId && <span className="truncate text-xs text-muted-foreground">{engineId}</span>}
        <div className="ml-auto flex gap-1">
          <button
            type="button"
            data-testid="engine-start"
            onClick={onStart}
            disabled={running}
            className="rounded border border-input bg-background px-2 py-0.5 text-xs hover:bg-accent disabled:opacity-40"
          >
            开始
          </button>
          <button
            type="button"
            data-testid="engine-stop"
            onClick={onStop}
            disabled={!running}
            className="rounded border border-input bg-background px-2 py-0.5 text-xs hover:bg-accent disabled:opacity-40"
          >
            停止
          </button>
          <button
            type="button"
            data-testid="engine-restart"
            onClick={onRestart}
            disabled={status === "stopped"}
            className="rounded border border-input bg-background px-2 py-0.5 text-xs hover:bg-accent disabled:opacity-40"
          >
            重启
          </button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
        <span>引擎路径</span>
        <input
          data-testid="engine-program"
          value={settings.programPath}
          onChange={(e) => onApplySettings({ programPath: e.currentTarget.value })}
          placeholder="留空使用 PIKAFISH_BIN"
          className="h-6 w-40 rounded border border-input bg-background px-1.5 text-xs"
        />
      </div>

      <div className="grid grid-cols-2 gap-1 text-xs">
        <label className="flex items-center gap-1">
          线程
          <input
            type="number"
            min={1}
            data-testid="setting-threads"
            value={settings.threads}
            onChange={(e) => onApplySettings({ threads: Number(e.currentTarget.value) })}
            className="h-6 w-14 rounded border border-input bg-background px-1"
          />
        </label>
        <label className="flex items-center gap-1">
          哈希(MB)
          <input
            type="number"
            min={1}
            data-testid="setting-hash"
            value={settings.hash}
            onChange={(e) => onApplySettings({ hash: Number(e.currentTarget.value) })}
            className="h-6 w-14 rounded border border-input bg-background px-1"
          />
        </label>
        <label className="flex items-center gap-1">
          深度(0=无限)
          <input
            type="number"
            min={0}
            data-testid="setting-depth"
            value={settings.depth ?? 0}
            onChange={(e) =>
              onApplySettings({
                depth: Number(e.currentTarget.value) > 0 ? Number(e.currentTarget.value) : null,
              })
            }
            className="h-6 w-14 rounded border border-input bg-background px-1"
          />
        </label>
        <label className="flex items-center gap-1">
          MultiPV
          <select
            data-testid="setting-multipv"
            value={settings.multipv}
            onChange={(e) => onApplySettings({ multipv: Number(e.currentTarget.value) })}
            className="h-6 rounded border border-input bg-background px-1"
          >
            {MULTIPV_OPTIONS.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
      </div>

      {message && (
        <p className="text-xs text-amber-600" data-testid="engine-message">
          {message}
        </p>
      )}

      {bestMove && (
        <p className="text-xs text-muted-foreground" data-testid="engine-bestmove">
          最佳着法：<span className="font-mono text-foreground">{bestMove.mv}</span>
          {bestMove.ponder ? ` （应着 ${bestMove.ponder}）` : ""}
        </p>
      )}

      {sorted.length === 0 && !running && (
        <p className="text-xs text-muted-foreground" data-testid="engine-empty">
          点击「开始」后，切换棋步即可分析当前局面。
        </p>
      )}

      <div className="flex flex-col gap-1" data-testid="engine-lines">
        {sorted.map((info) => (
          <button
            key={info.multipv}
            type="button"
            data-testid={`engine-line-${info.multipv}`}
            onClick={() => onPreview(info.pv)}
            title="点击预览该变化"
            className="rounded border border-input bg-background px-1.5 py-1 text-left text-xs hover:bg-accent"
          >
            <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
              <span className="w-4 text-muted-foreground">{info.multipv}</span>
              <span className="w-14 font-semibold" data-testid={`engine-eval-${info.multipv}`}>
                {formatScore(info.score)}
              </span>
              <span className="w-8">d{info.depth ?? "—"}</span>
              <span className="w-16 text-right">{formatNum(info.nodes)}</span>
              <span className="w-16 text-right">{formatNum(info.nps)}nps</span>
              <span className="w-12 text-right">
                {info.timeMs == null ? "—" : `${info.timeMs}ms`}
              </span>
            </div>
            <div
              className="flex flex-wrap gap-0.5 pt-0.5 font-mono"
              data-testid={`engine-pv-${info.multipv}`}
            >
              {info.pv.length === 0 ? (
                <span className="text-muted-foreground">（暂无 PV）</span>
              ) : (
                info.pv.map((mv, i) => (
                  <span key={`${mv}-${i}`} className="rounded bg-muted px-0.5">
                    {mv}
                  </span>
                ))
              )}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
