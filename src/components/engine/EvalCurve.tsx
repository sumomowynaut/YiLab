import { useState, type MouseEvent as ReactMouseEvent } from "react";
import type { CurvePoint } from "../../lib/engine/curve";
import { Button } from "../ui/button";

export interface EvalCurveProps {
  points: CurvePoint[];
  onClear: () => void;
}

const W = 400;
const H = 160;
const PAD_LEFT = 40;
const PAD_RIGHT = 12;
const PAD_TOP = 14;
const PAD_BOTTOM = 24;
const MAX_ABS = 500;

function clamp(v: number): number {
  return Math.max(-MAX_ABS, Math.min(MAX_ABS, v));
}

/** 用 Catmull-Rom 生成平滑曲线路径（点数 < 3 时退回折线）。 */
function buildSmoothPath(
  points: CurvePoint[],
  xs: (i: number) => number,
  ys: (score: number) => number,
): string {
  const n = points.length;
  if (n === 0) {
    return "";
  }
  const xi = (i: number) => xs(Math.max(0, Math.min(n - 1, i)));
  const yi = (i: number) => ys(points[Math.max(0, Math.min(n - 1, i))].scoreCp);
  if (n < 3) {
    return points
      .map((p, i) => `${i === 0 ? "M" : "L"}${xs(i).toFixed(1)} ${ys(p.scoreCp).toFixed(1)}`)
      .join(" ");
  }
  let d = `M${xs(0).toFixed(1)} ${ys(points[0].scoreCp).toFixed(1)}`;
  for (let i = 0; i < n - 1; i++) {
    const c1x = xi(i) + (xi(i + 1) - xi(i - 1)) / 6;
    const c1y = yi(i) + (yi(i + 1) - yi(i - 1)) / 6;
    const c2x = xi(i + 1) - (xi(i + 2) - xi(i)) / 6;
    const c2y = yi(i + 1) - (yi(i + 2) - yi(i)) / 6;
    d += ` C${c1x.toFixed(1)} ${c1y.toFixed(1)} ${c2x.toFixed(1)} ${c2y.toFixed(1)} ${xs(i + 1).toFixed(1)} ${ys(points[i + 1].scoreCp).toFixed(1)}`;
  }
  return d;
}

/** 评价曲线图：红方视角 cp，红色面积=红方优势，蓝色面积=黑方优势。 */
export function EvalCurve({ points, onClear }: EvalCurveProps) {
  const innerW = W - PAD_LEFT - PAD_RIGHT;
  const innerH = H - PAD_TOP - PAD_BOTTOM;
  const midY = PAD_TOP + innerH / 2;

  const xs = (i: number) =>
    PAD_LEFT + (points.length <= 1 ? 0 : (i / (points.length - 1)) * innerW);
  const ys = (score: number) => midY - (clamp(score) / MAX_ABS) * (innerH / 2);

  const line = buildSmoothPath(points, xs, ys);
  const lastScore = points.length ? points[points.length - 1].scoreCp : null;
  const [hover, setHover] = useState<number | null>(null);

  function handleMove(e: ReactMouseEvent<SVGSVGElement>) {
    if (points.length === 0) {
      setHover(null);
      return;
    }
    const rect = e.currentTarget.getBoundingClientRect();
    if (!rect.width) {
      return;
    }
    const px = ((e.clientX - rect.left) / rect.width) * W;
    if (points.length === 1) {
      setHover(0);
      return;
    }
    const ratio = (px - PAD_LEFT) / innerW;
    const idx = Math.round(ratio * (points.length - 1));
    setHover(Math.max(0, Math.min(points.length - 1, idx)));
  }
  const area = points.length
    ? `${line} L${xs(points.length - 1).toFixed(1)} ${midY.toFixed(1)} L${xs(0).toFixed(1)} ${midY.toFixed(1)} Z`
    : "";

  return (
    <div data-testid="eval-curve" className="flex flex-col gap-1 rounded border p-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-muted-foreground">
          评价曲线（红方视角）
          <span className="ml-2 font-normal">
            <span className="text-red-600">■</span> 红方优势 ·{" "}
            <span className="text-blue-600">■</span> 黑方优势
          </span>
        </span>
        <Button variant="outline" size="sm" data-testid="eval-curve-clear" onClick={onClear}>
          清空
        </Button>
      </div>

      {points.length === 0 ? (
        <p data-testid="eval-curve-empty" className="text-xs text-muted-foreground">
          暂无数据——开启分析后逐局面产生曲线。
        </p>
      ) : (
        <svg
          data-testid="eval-curve-svg"
          viewBox={`0 0 ${W} ${H}`}
          className="h-36 w-full cursor-crosshair"
          role="img"
          aria-label="评价曲线"
          onMouseMove={handleMove}
          onMouseLeave={() => setHover(null)}
        >
          <defs>
            <clipPath id="eval-top">
              <rect x={0} y={0} width={W} height={midY} />
            </clipPath>
            <clipPath id="eval-bottom">
              <rect x={0} y={midY} width={W} height={H - midY} />
            </clipPath>
          </defs>

          {/* 均势中线 */}
          <line
            x1={PAD_LEFT}
            y1={midY}
            x2={W - PAD_RIGHT}
            y2={midY}
            stroke="#8a6a3b"
            strokeDasharray="4 4"
            strokeWidth={1}
          />

          {/* 面积：上红下蓝 */}
          {area && (
            <>
              <path d={area} clipPath="url(#eval-top)" fill="rgba(220,38,38,0.18)" />
              <path d={area} clipPath="url(#eval-bottom)" fill="rgba(37,99,235,0.18)" />
            </>
          )}

          {/* 平滑曲线 */}
          {line && <path d={line} fill="none" stroke="#b91c1c" strokeWidth={2} />}

          {/* 数据点 */}
          {points.map((p, i) => (
            <circle key={`${p.fen}-${i}`} cx={xs(i)} cy={ys(p.scoreCp)} r={2.2} fill="#b91c1c" />
          ))}

          {/* 悬停：竖线 + 着法/评分提示 */}
          {hover != null && points[hover] && (
            <g data-testid="eval-curve-hover">
              <line
                x1={xs(hover)}
                y1={PAD_TOP}
                x2={xs(hover)}
                y2={H - PAD_BOTTOM}
                stroke="#6b7280"
                strokeDasharray="3 3"
                strokeWidth={1}
              />
              <circle
                cx={xs(hover)}
                cy={ys(points[hover].scoreCp)}
                r={4}
                fill="#b91c1c"
                stroke="#ffffff"
                strokeWidth={1}
              />
              {(() => {
                const tx = Math.min(xs(hover) + 8, W - 112);
                const p = points[hover];
                return (
                  <g transform={`translate(${tx},${PAD_TOP + 2})`}>
                    <rect
                      width={104}
                      height={42}
                      rx={4}
                      fill="rgba(24,24,27,0.92)"
                      stroke="#d4d4d8"
                    />
                    <text x={6} y={13} fontSize={10} fill="#ffffff">
                      {p.moveLabel || "初始局面"}
                    </text>
                    <text x={6} y={25} fontSize={9} fill="#f87171">
                      {p.turnLabel || ""}
                    </text>
                    <text x={6} y={37} fontSize={10} fill="#e5e7eb">
                      评分 {p.scoreCp > 0 ? "+" : ""}
                      {p.scoreCp}
                    </text>
                  </g>
                );
              })()}
            </g>
          )}

          {/* 当前评价标注 */}
          {/* 当前评价标注 */}
          {lastScore != null && (
            <text
              x={W - PAD_RIGHT}
              y={PAD_TOP - 4}
              textAnchor="end"
              fontSize={10}
              fill="#b91c1c"
              data-testid="eval-curve-last"
            >
              当前 {lastScore > 0 ? "+" : ""}
              {lastScore}
            </text>
          )}

          {/* 刻度与标注 */}
          <text x={PAD_LEFT - 6} y={PAD_TOP + 4} textAnchor="end" fontSize={10} fill="#dc2626">
            +{MAX_ABS}
          </text>
          <text x={PAD_LEFT - 6} y={midY + 3} textAnchor="end" fontSize={10} fill="#8a6a3b">
            0
          </text>
          <text
            x={PAD_LEFT - 6}
            y={H - PAD_BOTTOM + 4}
            textAnchor="end"
            fontSize={10}
            fill="#2563eb"
          >
            -{MAX_ABS}
          </text>
          <text x={PAD_LEFT} y={PAD_TOP - 4} fontSize={10} fill="#dc2626">
            红方优势
          </text>
          <text x={PAD_LEFT} y={H - PAD_BOTTOM + 14} fontSize={10} fill="#2563eb">
            黑方优势
          </text>
        </svg>
      )}
    </div>
  );
}
