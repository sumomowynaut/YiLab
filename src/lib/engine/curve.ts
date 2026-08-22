// 评价曲线纯函数（可独立测试，避免与 React 组件同文件导出冲突）。

export interface CurvePoint {
  /** 局面 FEN（去重键）。 */
  fen: string;
  /** 红方视角分数（centipawns）。 */
  scoreCp: number;
}

export const WIDTH = 280;
export const HEIGHT = 80;
export const PAD = 6;
/** 显示分数上限（clamp），避免 mate 或极端值压扁曲线。 */
const MAX_ABS = 500;

function clampScore(v: number): number {
  return Math.max(-MAX_ABS, Math.min(MAX_ABS, v));
}

/** 折线坐标：x 等分按顺序，y 映射分数（上正下负）；空序列返回 null。 */
export function curvePath(points: CurvePoint[]): string | null {
  if (points.length === 0) {
    return null;
  }
  const innerW = WIDTH - PAD * 2;
  const innerH = HEIGHT - PAD * 2;
  const mid = PAD + innerH / 2;
  const y = (score: number) => mid - (clampScore(score) / MAX_ABS) * (innerH / 2);
  const step = points.length === 1 ? 0 : innerW / (points.length - 1);
  return points
    .map((p, i) => {
      const x = PAD + i * step;
      const cmd = i === 0 ? "M" : "L";
      return `${cmd}${x.toFixed(1)} ${y(p.scoreCp).toFixed(1)}`;
    })
    .join(" ");
}
