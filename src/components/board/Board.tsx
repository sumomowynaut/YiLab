import type { ReactElement } from "react";
import {
  NUM_FILES,
  NUM_RANKS,
  pieceGlyph,
  screenFileOf,
  screenRankOf,
} from "../../lib/board/notation";
import type { BoardArrow, BoardView, PositionSnapshot, Square } from "../../lib/board/types";

const CELL = 56;
const PAD = 52;

/** 红方纵线号：file 0(a, 红左)=九 … file 8(i, 红右)=一（从红方视角右→左）。 */
const RED_FILE_LABELS = ["九", "八", "七", "六", "五", "四", "三", "二", "一"];
/** 黑方纵线号：file 0(黑右)=1 … file 8(黑左)=9（从黑方视角右→左）。 */
const BLACK_FILE_LABELS = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

export interface BoardProps {
  position: PositionSnapshot;
  selected: Square | null;
  legalTargets: Square[];
  view: BoardView;
  onSquareClick: (sq: Square) => void;
  /** 分析着法箭头（MultiPV 提示）。 */
  arrows?: BoardArrow[];
}

/** 计算一条箭头（含箭头头部）的 SVG 路径数据。 */
function arrowGeometry(sx: number, sy: number, tx: number, ty: number) {
  const dx = tx - sx;
  const dy = ty - sy;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len;
  const uy = dy / len;
  const back = CELL * 0.16;
  const head = CELL * 0.3;
  const endX = tx - ux * back;
  const endY = ty - uy * back;
  const baseX = endX - ux * head;
  const baseY = endY - uy * head;
  const px = -uy;
  const py = ux;
  const w = head * 0.4;
  return {
    line: `M${sx.toFixed(1)} ${sy.toFixed(1)} L${endX.toFixed(1)} ${endY.toFixed(1)}`,
    head: `${endX.toFixed(1)},${endY.toFixed(1)} ${(baseX + px * w).toFixed(1)},${(baseY + py * w).toFixed(1)} ${(baseX - px * w).toFixed(1)},${(baseY - py * w).toFixed(1)}`,
  };
}

/** 中国象棋棋盘（SVG 渲染，10×9，可翻转/镜像视图；纵线号采用传统记谱法）。 */
export function Board({
  position,
  selected,
  legalTargets,
  view,
  onSquareClick,
  arrows = [],
}: BoardProps) {
  const width = PAD * 2 + (NUM_FILES - 1) * CELL;
  const height = PAD * 2 + (NUM_RANKS - 1) * CELL;
  const x = (file: number) => PAD + file * CELL;
  const y = (rank: number) => PAD + rank * CELL;

  const targetKeys = new Set(legalTargets.map((t) => `${t.file}${t.rank}`));
  const selectedKey = selected ? `${selected.file}${selected.rank}` : null;

  // 竖线：左右边框连续；中间竖线在「楚河汉界」处断开（河界为空白带）。
  const verticals: ReactElement[] = [];
  for (let file = 0; file < NUM_FILES; file++) {
    if (file === 0 || file === NUM_FILES - 1) {
      verticals.push(
        <line
          key={`v${file}`}
          x1={x(file)}
          y1={y(0)}
          x2={x(file)}
          y2={y(NUM_RANKS - 1)}
          stroke="#5b3a1e"
          strokeWidth={1.5}
        />,
      );
    } else {
      verticals.push(
        <line
          key={`v${file}-top`}
          x1={x(file)}
          y1={y(0)}
          x2={x(file)}
          y2={y(4)}
          stroke="#5b3a1e"
          strokeWidth={1.5}
        />,
        <line
          key={`v${file}-bottom`}
          x1={x(file)}
          y1={y(5)}
          x2={x(file)}
          y2={y(NUM_RANKS - 1)}
          stroke="#5b3a1e"
          strokeWidth={1.5}
        />,
      );
    }
  }

  const horizontals = Array.from({ length: NUM_RANKS }, (_, rank) => (
    <line
      key={`h${rank}`}
      x1={x(0)}
      y1={y(rank)}
      x2={x(NUM_FILES - 1)}
      y2={y(rank)}
      stroke="#5b3a1e"
      strokeWidth={1.5}
    />
  ));

  const palaceDiagonals: ReactElement[] = [];
  for (const [lo, hi] of [
    [0, 2],
    [7, 9],
  ] as const) {
    palaceDiagonals.push(
      <line
        key={`pd-${lo}`}
        x1={x(screenFileOf(3, view))}
        y1={y(screenRankOf(lo, view))}
        x2={x(screenFileOf(5, view))}
        y2={y(screenRankOf(hi, view))}
        stroke="#5b3a1e"
        strokeWidth={1.5}
      />,
      <line
        key={`pd2-${lo}`}
        x1={x(screenFileOf(5, view))}
        y1={y(screenRankOf(lo, view))}
        x2={x(screenFileOf(3, view))}
        y2={y(screenRankOf(hi, view))}
        stroke="#5b3a1e"
        strokeWidth={1.5}
      />,
    );
  }

  const squares: ReactElement[] = [];
  for (let rank = 0; rank < NUM_RANKS; rank++) {
    for (let file = 0; file < NUM_FILES; file++) {
      const sRank = screenRankOf(rank, view);
      const sFile = screenFileOf(file, view);
      const key = `${file}${rank}`;
      const piece = position.board[rank][file];
      const isSelected = selectedKey === key;
      const isTarget = targetKeys.has(key);
      squares.push(
        <g
          key={key}
          data-testid={`sq-${key}`}
          onClick={() => onSquareClick({ rank, file })}
          style={{ cursor: "pointer" }}
        >
          <rect
            x={x(sFile) - CELL / 2}
            y={y(sRank) - CELL / 2}
            width={CELL}
            height={CELL}
            fill="transparent"
          />
          {isSelected && (
            <circle cx={x(sFile)} cy={y(sRank)} r={CELL * 0.48} fill="rgba(250, 204, 21, 0.45)" />
          )}
          {isTarget && !piece && <circle cx={x(sFile)} cy={y(sRank)} r={7} fill="#16a34a" />}
          {isTarget && piece && (
            <circle
              cx={x(sFile)}
              cy={y(sRank)}
              r={CELL * 0.48}
              fill="none"
              stroke="#dc2626"
              strokeWidth={4}
            />
          )}
          {piece && (
            <g data-testid={`piece-${key}`}>
              <circle
                cx={x(sFile)}
                cy={y(sRank)}
                r={CELL * 0.44}
                fill={piece.color === "red" ? "#fde8d7" : "#e5e7eb"}
                stroke={piece.color === "red" ? "#b91c1c" : "#374151"}
                strokeWidth={2}
              />
              <text
                x={x(sFile)}
                y={y(sRank)}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={26}
                fontWeight={700}
                fill={piece.color === "red" ? "#b91c1c" : "#111827"}
              >
                {pieceGlyph(piece)}
              </text>
            </g>
          )}
        </g>,
      );
    }
  }

  // 传统记谱法纵线号：红方在红方一侧、黑方在黑方一侧，各从本方视角「从右到左」。
  const redScreenRank = screenRankOf(0, view);
  const blackScreenRank = screenRankOf(NUM_RANKS - 1, view);
  const redLabelY = redScreenRank === 0 ? 18 : height - 16;
  const blackLabelY = blackScreenRank === 0 ? 18 : height - 16;

  const redFileLabels = Array.from({ length: NUM_FILES }, (_, file) => (
    <text
      key={`red-file-${file}`}
      x={x(screenFileOf(file, view))}
      y={redLabelY}
      textAnchor="middle"
      fontSize={12}
      fill="#7a4a1e"
    >
      {RED_FILE_LABELS[file]}
    </text>
  ));

  const blackFileLabels = Array.from({ length: NUM_FILES }, (_, file) => (
    <text
      key={`black-file-${file}`}
      x={x(screenFileOf(file, view))}
      y={blackLabelY}
      textAnchor="middle"
      fontSize={12}
      fill="#7a4a1e"
    >
      {BLACK_FILE_LABELS[file]}
    </text>
  ));

  // 分析箭头：从起点到终点，带箭头头部；可选标注（MultiPV 序号）。
  const arrowEls = arrows.map((a, i) => {
    const sx = x(screenFileOf(a.from.file, view));
    const sy = y(screenRankOf(a.from.rank, view));
    const tx = x(screenFileOf(a.to.file, view));
    const ty = y(screenRankOf(a.to.rank, view));
    const g = arrowGeometry(sx, sy, tx, ty);
    return (
      <g key={`arrow-${i}`} data-testid={`arrow-${i}`}>
        <path
          d={g.line}
          stroke={a.color}
          strokeWidth={5}
          fill="none"
          opacity={0.55}
          strokeLinecap="round"
        />
        <path d={`M${g.head} Z`} fill={a.color} opacity={0.85} />
        {a.label && (
          <text
            x={sx + (tx - sx) * 0.28}
            y={sy + (ty - sy) * 0.28 - 8}
            textAnchor="middle"
            fontSize={15}
            fontWeight={800}
            fill={a.color}
            stroke="#fff"
            strokeWidth={2}
            paintOrder="stroke"
          >
            {a.label}
          </text>
        )}
      </g>
    );
  });

  const riverY = (y(4) + y(5)) / 2;

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      data-testid="board"
      role="img"
      aria-label="中国象棋棋盘"
      className="block h-auto w-full max-w-[560px] select-none"
    >
      <rect x={0} y={0} width={width} height={height} rx={8} fill="#e8c39e" />
      {verticals}
      {horizontals}
      {palaceDiagonals}
      <text x={width / 2 - 46} y={riverY} textAnchor="middle" fontSize={22} fill="#8a6a3b">
        楚 河
      </text>
      <text x={width / 2 + 46} y={riverY} textAnchor="middle" fontSize={22} fill="#8a6a3b">
        汉 界
      </text>
      {squares}
      {arrowEls}
      {redFileLabels}
      {blackFileLabels}
    </svg>
  );
}
