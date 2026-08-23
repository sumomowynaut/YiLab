import type { BestMoveDto, EngineStatus, InfoLineDto } from "../../lib/engine/types";

export interface AnalysisPanelProps {
  status: EngineStatus;
  engineId: string | null;
  lines: Record<number, InfoLineDto>;
  bestMove: BestMoveDto | null;
  message: string | null;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
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

/** 引擎分析面板：评价 / 深度 / 节点 / NPS / 时间 / MultiPV / PV，支持预览。 */
export function AnalysisPanel({
  status,
  engineId,
  lines,
  bestMove,
  message,
  onStart,
  onStop,
  onRestart,
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
