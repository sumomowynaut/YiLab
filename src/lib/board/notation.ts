// 坐标 / FEN / 棋子字形工具。
// 注意：真实应用的规则单一事实来源在 Rust；本文件的 FEN 解析仅用于
// 浏览器开发回退（memory api）与前端测试。

import type { BoardView, Color, Move, Piece, PieceKind, PositionSnapshot, Square } from "./types";

export const NUM_RANKS = 10;
export const NUM_FILES = 9;
export const START_FEN = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";

export function squareToUci(sq: Square): string {
  return String.fromCharCode(97 + sq.file) + sq.rank;
}

export function squareFromUci(s: string): Square | null {
  if (!/^[a-i][0-9]$/.test(s)) {
    return null;
  }
  const file = s.charCodeAt(0) - 97;
  const rank = Number(s[1]);
  if (rank < 0 || rank >= NUM_RANKS || file < 0 || file >= NUM_FILES) {
    return null;
  }
  return { rank, file };
}

export function moveToUci(mv: Move): string {
  return squareToUci(mv.from) + squareToUci(mv.to);
}

export function moveFromUci(s: string): Move | null {
  if (s.length !== 4) {
    return null;
  }
  const from = squareFromUci(s.slice(0, 2));
  const to = squareFromUci(s.slice(2, 4));
  return from && to ? { from, to } : null;
}

const FEN_KINDS: Record<string, PieceKind> = {
  k: "king",
  a: "advisor",
  b: "elephant",
  n: "horse",
  r: "rook",
  c: "cannon",
  p: "pawn",
};

/** 解析中国象棋 FEN 为快照（浏览器回退与测试用；应用内以 Rust 为准）。 */
export function parseFen(fen: string): PositionSnapshot {
  const fields = fen.trim().split(/\s+/);
  if (fields.length < 2) {
    throw new Error("FEN 至少需要两个字段：局面与走子方");
  }
  const rows = fields[0].split("/");
  if (rows.length !== NUM_RANKS) {
    throw new Error(`局面应为 ${NUM_RANKS} 行，实际 ${rows.length}`);
  }
  const board: (Piece | null)[][] = Array.from({ length: NUM_RANKS }, () =>
    Array.from({ length: NUM_FILES }, () => null),
  );
  rows.forEach((row, i) => {
    const rank = NUM_RANKS - 1 - i;
    let file = 0;
    for (const ch of row) {
      if (file >= NUM_FILES) {
        throw new Error(`rank ${rank} 的棋子超过 9 格`);
      }
      if (ch >= "1" && ch <= "9") {
        file += Number(ch);
        if (file > NUM_FILES) {
          throw new Error(`rank ${rank} 的格子数超过 9`);
        }
      } else {
        const kind = FEN_KINDS[ch.toLowerCase()];
        if (!kind) {
          throw new Error(`无法识别的棋子字符：${ch}`);
        }
        board[rank][file] = { kind, color: ch === ch.toUpperCase() ? "red" : "black" };
        file += 1;
      }
    }
    if (file !== NUM_FILES) {
      throw new Error(`rank ${rank} 应合计 9 格，实际 ${file}`);
    }
  });
  const sideToMove =
    fields[1] === "w"
      ? "w"
      : fields[1] === "b"
        ? "b"
        : (() => {
            throw new Error("走子方必须为 w 或 b");
          })();
  const halfmoveClock = fields[4] ? Number(fields[4]) : 0;
  const fullmoveNumber = fields[5] ? Number(fields[5]) : 1;
  return { board, sideToMove, halfmoveClock, fullmoveNumber, fen: fen.trim() };
}

/** 棋子汉字字形：红方用简体，黑方用对应汉字。 */
export const PIECE_GLYPHS: Record<Color, Record<PieceKind, string>> = {
  red: {
    king: "帅",
    advisor: "仕",
    elephant: "相",
    horse: "马",
    rook: "车",
    cannon: "炮",
    pawn: "兵",
  },
  black: {
    king: "将",
    advisor: "士",
    elephant: "象",
    horse: "马",
    rook: "车",
    cannon: "炮",
    pawn: "卒",
  },
};

export function pieceGlyph(piece: Piece): string {
  return PIECE_GLYPHS[piece.color][piece.kind];
}

/** 视图变换：逻辑 rank → 屏幕 rank。 */
export function screenRankOf(rank: number, view: BoardView): number {
  return view.flipVertical ? NUM_RANKS - 1 - rank : rank;
}

/** 视图变换：逻辑 file → 屏幕 file。 */
export function screenFileOf(file: number, view: BoardView): number {
  return view.flipHorizontal ? NUM_FILES - 1 - file : file;
}

export function sideToColor(side: "w" | "b"): Color {
  return side === "w" ? "red" : "black";
}
