import type { ReactElement } from "react";
import {
  NUM_FILES,
  NUM_RANKS,
  pieceGlyph,
  screenFileOf,
  screenRankOf,
} from "../../lib/board/notation";
import type { BoardView, PositionSnapshot, Square } from "../../lib/board/types";

const CELL = 56;
const PAD = 40;

export interface BoardProps {
  position: PositionSnapshot;
  selected: Square | null;
  legalTargets: Square[];
  view: BoardView;
  onSquareClick: (sq: Square) => void;
}

/** 中国象棋棋盘（SVG 渲染，10×9，可翻转/镜像视图）。 */
export function Board({ position, selected, legalTargets, view, onSquareClick }: BoardProps) {
  const width = PAD * 2 + (NUM_FILES - 1) * CELL;
  const height = PAD * 2 + (NUM_RANKS - 1) * CELL;
  const x = (file: number) => PAD + file * CELL;
  const y = (rank: number) => PAD + rank * CELL;

  const targetKeys = new Set(legalTargets.map((t) => `${t.file}${t.rank}`));
  const selectedKey = selected ? `${selected.file}${selected.rank}` : null;

  const verticals = Array.from({ length: NUM_FILES }, (_, file) => (
    <line
      key={`v${file}`}
      x1={x(file)}
      y1={y(0)}
      x2={x(file)}
      y2={y(NUM_RANKS - 1)}
      stroke="#5b3a1e"
      strokeWidth={1.5}
    />
  ));

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

  const fileLabels = Array.from({ length: NUM_FILES }, (_, file) => (
    <text
      key={`fl${file}`}
      x={x(file)}
      y={height - 12}
      textAnchor="middle"
      fontSize={12}
      fill="#8a6a3b"
    >
      {String.fromCharCode(97 + file)}
    </text>
  ));

  const rankLabels = Array.from({ length: NUM_RANKS }, (_, rank) => (
    <text key={`rl${rank}`} x={12} y={y(rank) + 4} textAnchor="middle" fontSize={12} fill="#8a6a3b">
      {rank}
    </text>
  ));

  const riverY = (y(4) + y(5)) / 2;

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      data-testid="board"
      role="img"
      aria-label="中国象棋棋盘"
      className="select-none"
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
      {fileLabels}
      {rankLabels}
    </svg>
  );
}
