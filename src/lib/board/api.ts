// 棋盘核心访问接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。
// 真实应用中规则逻辑由 Rust 提供（单一事实来源）。

import { invokeCommand, isTauri } from "../ipc";
import { parseFen, START_FEN } from "./notation";
import type { Color, PieceKind, PositionSnapshot, ValidationResult } from "./types";

export interface PositionWithValidation {
  position: PositionSnapshot;
  validation: ValidationResult;
}

export interface BoardApi {
  startPosition(): Promise<PositionSnapshot>;
  fromFen(fen: string): Promise<PositionWithValidation>;
  legalMoves(fen: string): Promise<string[]>;
  makeMove(fen: string, mv: string): Promise<PositionSnapshot>;
  /** 依序应用一串着法（PV 预览用）。 */
  applyMoves(fen: string, moves: string[]): Promise<PositionSnapshot>;
  /** 把一串着法转成中文纵线制记谱。 */
  movesToChinese(fen: string, moves: string[]): Promise<string[]>;
  validate(fen: string): Promise<ValidationResult>;
  rotate(fen: string, mode: "180" | "mirror"): Promise<PositionSnapshot>;
  setPiece(
    fen: string,
    square: string,
    color: Color,
    kind: PieceKind,
  ): Promise<PositionWithValidation>;
  clearSquare(fen: string, square: string): Promise<PositionWithValidation>;
  setSide(fen: string, side: "w" | "b"): Promise<PositionWithValidation>;
  clearAll(): Promise<PositionWithValidation>;
}

export const tauriBoardApi: BoardApi = {
  startPosition: () => invokeCommand<PositionSnapshot>("board_startpos"),
  fromFen: async (fen) => {
    const position = await invokeCommand<PositionSnapshot>("board_from_fen", { fen });
    const validation = await invokeCommand<ValidationResult>("board_validate", { fen });
    return { position, validation };
  },
  legalMoves: (fen) => invokeCommand<string[]>("board_legal_moves", { fen }),
  makeMove: (fen, mv) => invokeCommand<PositionSnapshot>("board_make_move", { fen, mv }),
  applyMoves: (fen, moves) => invokeCommand<PositionSnapshot>("board_apply_moves", { fen, moves }),
  movesToChinese: (fen, moves) => invokeCommand<string[]>("board_moves_to_chinese", { fen, moves }),
  validate: (fen) => invokeCommand<ValidationResult>("board_validate", { fen }),
  rotate: (fen, mode) => invokeCommand<PositionSnapshot>("board_rotate", { fen, mode }),
  setPiece: async (fen, square, color, kind) => {
    const position = await invokeCommand<PositionSnapshot>("board_edit_set_piece", {
      fen,
      square,
      color,
      kind,
    });
    const validation = await invokeCommand<ValidationResult>("board_validate", {
      fen: position.fen,
    });
    return { position, validation };
  },
  clearSquare: async (fen, square) => {
    const position = await invokeCommand<PositionSnapshot>("board_edit_clear", { fen, square });
    const validation = await invokeCommand<ValidationResult>("board_validate", {
      fen: position.fen,
    });
    return { position, validation };
  },
  setSide: async (fen, side) => {
    const position = await invokeCommand<PositionSnapshot>("board_edit_set_side", { fen, side });
    const validation = await invokeCommand<ValidationResult>("board_validate", {
      fen: position.fen,
    });
    return { position, validation };
  },
  clearAll: async () => {
    const position = await invokeCommand<PositionSnapshot>("board_edit_clear_all");
    const validation = await invokeCommand<ValidationResult>("board_validate", {
      fen: position.fen,
    });
    return { position, validation };
  },
};

/** 浏览器开发预览回退：仅支持展示与 FEN 解析，走子/编辑需在 Tauri 中运行。 */
export const memoryBoardApi: BoardApi = {
  startPosition: async () => parseFen(START_FEN),
  fromFen: async (fen) => {
    const position = parseFen(fen);
    return { position, validation: { ok: true, issues: [] } };
  },
  legalMoves: async () => [],
  makeMove: async () => {
    throw new Error("走子需要 Tauri 环境（Rust 规则核心）");
  },
  applyMoves: async () => {
    throw new Error("PV 预览需要 Tauri 环境（Rust 规则核心）");
  },
  movesToChinese: async () => {
    throw new Error("中文记谱需要 Tauri 环境（Rust 规则核心）");
  },
  validate: async () => ({ ok: true, issues: [] }),
  rotate: async (fen) => parseFen(fen),
  setPiece: async () => {
    throw new Error("局面编辑需要 Tauri 环境（Rust 规则核心）");
  },
  clearSquare: async () => {
    throw new Error("局面编辑需要 Tauri 环境（Rust 规则核心）");
  },
  setSide: async () => {
    throw new Error("局面编辑需要 Tauri 环境（Rust 规则核心）");
  },
  clearAll: async () => {
    throw new Error("局面编辑需要 Tauri 环境（Rust 规则核心）");
  },
};

export function getDefaultBoardApi(): BoardApi {
  return isTauri() ? tauriBoardApi : memoryBoardApi;
}
