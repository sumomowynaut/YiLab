// 棋盘核心的 TypeScript 类型（与 Rust `board` 模块的 DTO 一一对应）。
// 单一事实来源在 Rust；这里仅用于渲染与 IPC 数据绑定。

export type Color = "red" | "black";

export type PieceKind = "king" | "advisor" | "elephant" | "horse" | "rook" | "cannon" | "pawn";

export interface Piece {
  color: Color;
  kind: PieceKind;
}

/** 棋盘格：rank 0 为红方底线，file 0 为 a 列。 */
export interface Square {
  rank: number;
  file: number;
}

export interface Move {
  from: Square;
  to: Square;
}

/** Rust `PositionDto` 的镜像。 */
export interface PositionSnapshot {
  /** [rank][file]，10×9 */
  board: (Piece | null)[][];
  sideToMove: "w" | "b";
  halfmoveClock: number;
  fullmoveNumber: number;
  fen: string;
}

export interface ValidationResult {
  ok: boolean;
  issues: string[];
}

/** 局面编辑器当前工具。 */
export type Tool = { color: Color; kind: PieceKind } | "eraser";

/** 棋盘视图变换（仅影响显示，不改变局面数据）。 */
export interface BoardView {
  flipVertical: boolean;
  flipHorizontal: boolean;
}
