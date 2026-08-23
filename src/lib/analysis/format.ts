// 自动复盘展示辅助（纯函数，可独立测试）。

import type { AnalysisCategory, MoveAssessmentDto } from "./types";

export const CATEGORY_LABEL: Record<AnalysisCategory, string> = {
  best: "最佳",
  excellent: "优秀",
  good: "好棋",
  inaccuracy: "不精确",
  mistake: "失误",
  blunder: "大漏",
};

export interface ChartPoint {
  index: number;
  nodeId: number;
  /** 红方视角走前评价（厘兵）。 */
  evalCp: number;
}

export const CHART_W = 300;
export const CHART_H = 90;
const PAD = 6;
const MAX_ABS = 500;

export function clampEval(v: number): number {
  return Math.max(-MAX_ABS, Math.min(MAX_ABS, v));
}

/** 折线坐标：x 等分按步序，y 映射评价（上正下负）。 */
export function chartPath(points: ChartPoint[]): string | null {
  if (points.length === 0) {
    return null;
  }
  const innerW = CHART_W - PAD * 2;
  const innerH = CHART_H - PAD * 2;
  const mid = PAD + innerH / 2;
  const y = (score: number) => mid - (clampEval(score) / MAX_ABS) * (innerH / 2);
  const step = points.length === 1 ? 0 : innerW / (points.length - 1);
  return points
    .map((p, i) => {
      const x = PAD + i * step;
      const cmd = i === 0 ? "M" : "L";
      return `${cmd}${x.toFixed(1)} ${y(p.evalCp).toFixed(1)}`;
    })
    .join(" ");
}

/** 评估序列（用于图表与汇总）。 */
export function chartPoints(assessments: MoveAssessmentDto[]): ChartPoint[] {
  return assessments.map((a, i) => ({
    index: i,
    nodeId: a.nodeId,
    evalCp: a.evalBeforeCp,
  }));
}
