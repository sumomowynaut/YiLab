// 自动复盘报告：控制（开始/停止/继续/重新分析）+ 可点击评价曲线 + 汇总。

import type { AnalysisStatusName, MoveAssessmentDto } from "../../lib/analysis/types";
import {
  CATEGORY_LABEL,
  CHART_H,
  CHART_W,
  chartPath,
  chartPoints,
} from "../../lib/analysis/format";
import { Button } from "../ui/button";

export interface AnalysisReportProps {
  status: AnalysisStatusName;
  progress: number;
  total: number;
  assessments: MoveAssessmentDto[];
  onStart: () => void;
  onStop: () => void;
  onContinue: () => void;
  onRestart: () => void;
  /** 点击评价曲线跳转到对应棋步。 */
  onNavigate: (nodeId: number) => void;
}

const STATUS_LABEL: Record<AnalysisStatusName, string> = {
  idle: "未开始",
  running: "分析中",
  paused: "已暂停",
  done: "已完成",
  failed: "失败",
};

function catClass(category: MoveAssessmentDto["category"]): string {
  switch (category) {
    case "best":
      return "text-green-600";
    case "excellent":
      return "text-emerald-500";
    case "good":
      return "text-blue-600";
    case "inaccuracy":
      return "text-amber-600";
    case "mistake":
      return "text-orange-600";
    case "blunder":
      return "text-red-600";
  }
}

/** 自动复盘报告面板。 */
export function AnalysisReport({
  status,
  progress,
  total,
  assessments,
  onStart,
  onStop,
  onContinue,
  onRestart,
  onNavigate,
}: AnalysisReportProps) {
  const points = chartPoints(assessments);
  const path = chartPath(points);
  const mistakes = assessments.filter((a) => a.category === "mistake" || a.category === "blunder");
  const bestMoves = assessments.filter((a) => a.category === "best");

  return (
    <div data-testid="analysis-report" className="flex flex-col gap-2 rounded border p-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">自动复盘</span>
        <span data-testid="analysis-status" className="text-xs text-muted-foreground">
          {STATUS_LABEL[status]}
          {(status === "running" || status === "paused") && `（${progress}/${total}）`}
        </span>
      </div>

      <div className="flex flex-wrap gap-2">
        {(status === "idle" || status === "done" || status === "failed") && (
          <Button type="button" size="sm" data-testid="analysis-start" onClick={onStart}>
            开始分析
          </Button>
        )}
        {status === "running" && (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            data-testid="analysis-stop"
            onClick={onStop}
          >
            停止
          </Button>
        )}
        {status === "paused" && (
          <Button type="button" size="sm" data-testid="analysis-continue" onClick={onContinue}>
            继续
          </Button>
        )}
        {(status === "done" || status === "paused" || status === "failed") && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid="analysis-restart"
            onClick={onRestart}
          >
            重新分析
          </Button>
        )}
      </div>

      {/* 评价曲线（点击跳转棋步） */}
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">评价曲线（红方视角 cp，点击跳转）</span>
        {path ? (
          <svg
            data-testid="analysis-chart"
            viewBox={`0 0 ${CHART_W} ${CHART_H}`}
            className="h-24 w-full"
            role="img"
            aria-label="自动复盘评价曲线"
          >
            <line
              x1={6}
              y1={CHART_H / 2}
              x2={CHART_W - 6}
              y2={CHART_H / 2}
              stroke="currentColor"
              strokeOpacity={0.3}
              strokeWidth={1}
            />
            <polyline points={path} fill="none" stroke="currentColor" strokeWidth={2} />
            {points.map((p) => {
              const y =
                CHART_H / 2 -
                (Math.max(-500, Math.min(500, p.evalCp)) / 500) * ((CHART_H - 12) / 2);
              const x = 6 + (p.index * (CHART_W - 12)) / Math.max(1, points.length - 1);
              return (
                <circle
                  key={p.index}
                  data-testid={`eval-point-${p.index}`}
                  cx={x}
                  cy={y}
                  r={3.5}
                  fill="currentColor"
                  onClick={() => onNavigate(p.nodeId)}
                  style={{ cursor: "pointer" }}
                  role="button"
                  aria-label={`跳转到第 ${p.index + 1} 步`}
                />
              );
            })}
          </svg>
        ) : (
          <p data-testid="analysis-chart-empty" className="text-xs text-muted-foreground">
            暂无数据——点击「开始分析」逐局面分析整盘棋。
          </p>
        )}
      </div>

      {assessments.length > 0 && (
        <>
          <div className="flex flex-col gap-1 text-xs">
            <span className="font-semibold">关键失误（失误/大漏）</span>
            {mistakes.length === 0 ? (
              <p className="text-muted-foreground">无</p>
            ) : (
              <ul data-testid="analysis-mistakes" className="list-inside list-disc text-orange-600">
                {mistakes.map((a, i) => (
                  <li key={a.nodeId}>
                    第 {i + 1} 步 {a.mvCn || a.mv}（损失 {a.lossCp} cp ·{" "}
                    {CATEGORY_LABEL[a.category]}）
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="flex flex-col gap-1 text-xs">
            <span className="font-semibold">最佳着法</span>
            {bestMoves.length === 0 ? (
              <p className="text-muted-foreground">无</p>
            ) : (
              <ul data-testid="analysis-best" className="list-inside list-disc text-green-600">
                {bestMoves.map((a) => (
                  <li key={a.nodeId}>
                    第 {a.nodeId + 1} 步 {a.mvCn || a.mv}
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="overflow-x-auto">
            <table data-testid="analysis-table" className="w-full text-left text-xs">
              <thead>
                <tr className="border-b text-muted-foreground">
                  <th className="px-1 py-0.5">着法</th>
                  <th className="px-1 py-0.5">最佳</th>
                  <th className="px-1 py-0.5">走前→走后</th>
                  <th className="px-1 py-0.5">损失</th>
                  <th className="px-1 py-0.5">深度</th>
                  <th className="px-1 py-0.5">类别</th>
                  <th className="px-1 py-0.5">PV</th>
                </tr>
              </thead>
              <tbody>
                {assessments.map((a) => (
                  <tr key={a.nodeId} className="border-b border-dashed">
                    <td className="px-1 py-0.5 font-mono">{a.mvCn || a.mv}</td>
                    <td className="px-1 py-0.5 font-mono">{a.bestMoveCn || a.bestMove}</td>
                    <td className="px-1 py-0.5 font-mono">
                      {a.evalBeforeCp} → {a.evalAfterCp}
                    </td>
                    <td className="px-1 py-0.5">{a.lossCp}</td>
                    <td className="px-1 py-0.5">{a.depth}</td>
                    <td className={`px-1 py-0.5 ${catClass(a.category)}`}>
                      {CATEGORY_LABEL[a.category]}
                    </td>
                    <td
                      className="max-w-40 truncate px-1 py-0.5 font-mono"
                      title={(a.pvCn && a.pvCn.length ? a.pvCn : a.pv).join(" ")}
                    >
                      {(a.pvCn && a.pvCn.length ? a.pvCn : a.pv).join(" ")}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="text-xs text-muted-foreground">共 {assessments.length} 步</p>
        </>
      )}
    </div>
  );
}
