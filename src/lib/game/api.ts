// 棋谱树访问接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri } from "../ipc";
import { parseFen, START_FEN } from "../board/notation";
import type { GameSnapshot } from "./types";

export interface GameApi {
  newGame(fen?: string): Promise<GameSnapshot>;
  snapshot(): Promise<GameSnapshot>;
  insertMove(mv: string): Promise<GameSnapshot>;
  navigate(nodeId: number): Promise<GameSnapshot>;
  previous(): Promise<GameSnapshot>;
  next(): Promise<GameSnapshot>;
  undo(): Promise<GameSnapshot>;
  redo(): Promise<GameSnapshot>;
  goToStart(): Promise<GameSnapshot>;
  goToEnd(): Promise<GameSnapshot>;
  deleteVariation(nodeId: number): Promise<GameSnapshot>;
  promoteVariation(nodeId: number): Promise<GameSnapshot>;
  reorderVariation(parentId: number, from: number, to: number): Promise<GameSnapshot>;
  /** H1：修改注释必须显式携带 nodeId，避免依赖 Rust 全局 current。 */
  setComment(nodeId: number, comment: string): Promise<GameSnapshot>;
  setNag(nodeId: number, nag: string, add: boolean): Promise<GameSnapshot>;
}

export const tauriGameApi: GameApi = {
  newGame: (fen) => invokeCommand<GameSnapshot>("game_new", { fen: fen ?? "" }),
  snapshot: () => invokeCommand<GameSnapshot>("game_snapshot"),
  insertMove: (mv) => invokeCommand<GameSnapshot>("game_insert_move", { mv }),
  navigate: (nodeId) => invokeCommand<GameSnapshot>("game_navigate", { nodeId }),
  previous: () => invokeCommand<GameSnapshot>("game_previous"),
  next: () => invokeCommand<GameSnapshot>("game_next"),
  undo: () => invokeCommand<GameSnapshot>("game_undo"),
  redo: () => invokeCommand<GameSnapshot>("game_redo"),
  goToStart: () => invokeCommand<GameSnapshot>("game_go_to_start"),
  goToEnd: () => invokeCommand<GameSnapshot>("game_go_to_end"),
  deleteVariation: (nodeId) => invokeCommand<GameSnapshot>("game_delete_variation", { nodeId }),
  promoteVariation: (nodeId) => invokeCommand<GameSnapshot>("game_promote_variation", { nodeId }),
  reorderVariation: (parentId, from, to) =>
    invokeCommand<GameSnapshot>("game_reorder_variation", { parentId, from, to }),
  setComment: (nodeId, comment) =>
    invokeCommand<GameSnapshot>("game_set_comment", { nodeId, comment }),
  setNag: (nodeId, nag, add) => invokeCommand<GameSnapshot>("game_set_nag", { nodeId, nag, add }),
};

/** 浏览器开发预览回退：仅展示起始局面（规则单一事实来源在 Rust）。 */
export const memoryGameApi: GameApi = {
  newGame: async () => rootOnlySnapshot(),
  snapshot: async () => rootOnlySnapshot(),
  insertMove: async () => {
    throw new Error("走子需要 Tauri 环境（Rust 棋谱树核心）");
  },
  navigate: async () => rootOnlySnapshot(),
  previous: async () => rootOnlySnapshot(),
  next: async () => rootOnlySnapshot(),
  undo: async () => rootOnlySnapshot(),
  redo: async () => rootOnlySnapshot(),
  goToStart: async () => rootOnlySnapshot(),
  goToEnd: async () => rootOnlySnapshot(),
  deleteVariation: async () => rootOnlySnapshot(),
  promoteVariation: async () => rootOnlySnapshot(),
  reorderVariation: async () => rootOnlySnapshot(),
  setComment: async () => rootOnlySnapshot(),
  setNag: async () => rootOnlySnapshot(),
};

function rootOnlySnapshot(): GameSnapshot {
  const position = parseFen(START_FEN);
  return {
    tree: {
      id: 0,
      mv: null,
      moveNumber: 0,
      isRed: true,
      comment: "",
      nags: [],
      children: [],
      isVariation: false,
    },
    currentId: 0,
    currentFen: START_FEN,
    position,
    comment: "",
    nags: [],
    hasParent: false,
    previousId: null,
    nextMainId: null,
    undoAvailable: false,
    redoAvailable: false,
  };
}

export function getDefaultGameApi(): GameApi {
  return isTauri() ? tauriGameApi : memoryGameApi;
}
