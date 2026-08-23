// 评价曲线（会话内）：记录每个被分析局面的主变（multipv=1）分数。
// 分数统一为红方视角（centipawns）；持久化随 DB 阶段。

import { create } from "zustand";
import type { CurvePoint } from "../lib/engine/curve";

export type { CurvePoint } from "../lib/engine/curve";

export interface CurveMeta {
  /** 走到该局面的着法（如 炮二平五）。 */
  moveLabel?: string;
  /** 展示用回合信息（如 第 10 回合 · 红）。 */
  turnLabel?: string;
}

interface CurveState {
  points: CurvePoint[];
  /** 记录或更新某局面的分数（保持首次出现的顺序）。 */
  record: (fen: string, scoreCp: number, meta?: CurveMeta) => void;
  /** 清空曲线。 */
  clear: () => void;
  /** 整体替换（测试/导入用）。 */
  setPoints: (points: CurvePoint[]) => void;
}

export const useCurveStore = create<CurveState>((set) => ({
  points: [],

  record: (fen, scoreCp, meta) =>
    set((state) => {
      const point: CurvePoint = { fen, scoreCp, ...meta };
      const index = state.points.findIndex((p) => p.fen === fen);
      if (index >= 0) {
        const points = [...state.points];
        points[index] = { ...points[index], scoreCp, ...meta };
        return { points };
      }
      return { points: [...state.points, point] };
    }),

  clear: () => set({ points: [] }),

  setPoints: (points) => set({ points }),
}));
