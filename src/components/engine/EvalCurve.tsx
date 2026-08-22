// 评价曲线图：SVG 折线，展示主变分数随分析局面的变化（红方视角 centipawns）。

import type { CurvePoint } from "../../lib/engine/curve";
import { curvePath, HEIGHT, PAD, WIDTH } from "../../lib/engine/curve";
import { Button } from "../ui/button";

export interface EvalCurveProps {
  points: CurvePoint[];
  onClear: () => void;
}

/** 评价曲线图（会话内）。 */
export function EvalCurve({ points, onClear }: EvalCurveProps) {
  const path = curvePath(points);
  return (
    <div data-testid="eval-curve" className="flex flex-col gap-1 rounded border p-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-muted-foreground">评价曲线（红方视角 cp）</span>
        <Button variant="outline" size="sm" data-testid="eval-curve-clear" onClick={onClear}>
          清空
        </Button>
      </div>
      {path ? (
        <svg
          data-testid="eval-curve-svg"
          viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
          className="h-20 w-full"
          role="img"
          aria-label="评价曲线"
        >
          <line
            x1={PAD}
            y1={HEIGHT / 2}
            x2={WIDTH - PAD}
            y2={HEIGHT / 2}
            stroke="currentColor"
            strokeOpacity={0.3}
            strokeWidth={1}
          />
          <polyline points={path} fill="none" stroke="currentColor" strokeWidth={2} />
        </svg>
      ) : (
        <p data-testid="eval-curve-empty" className="text-xs text-muted-foreground">
          暂无数据——开启分析后逐局面产生曲线。
        </p>
      )}
    </div>
  );
}
