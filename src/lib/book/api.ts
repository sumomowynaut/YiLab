// 开局库接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri } from "../ipc";
import type { BookAutoMoveDto, BookMoveDto, BookStrategy } from "./types";

export interface BookApi {
  /** 查询当前局面的候选着法（本地优先，未命中回退云库）。 */
  lookup(): Promise<BookMoveDto[]>;
  /** 查询当前局面的推荐着法。 */
  recommend(strategy: BookStrategy): Promise<BookMoveDto | null>;
  /** 自动走库：把推荐着法插入当前棋谱树。 */
  autoMove(strategy: BookStrategy): Promise<BookAutoMoveDto>;
}

export const tauriBookApi: BookApi = {
  lookup: () => invokeCommand<BookMoveDto[]>("book_lookup"),
  recommend: (strategy) => invokeCommand<BookMoveDto | null>("book_recommend", { strategy }),
  autoMove: (strategy) => invokeCommand<BookAutoMoveDto>("book_auto_move", { strategy }),
};

export const memoryBookApi: BookApi = {
  lookup: async () => {
    throw new Error("开局库需要 Tauri 环境（Rust 开局库核心）");
  },
  recommend: async () => {
    throw new Error("开局库需要 Tauri 环境（Rust 开局库核心）");
  },
  autoMove: async () => {
    throw new Error("开局库需要 Tauri 环境（Rust 开局库核心）");
  },
};

export function getDefaultBookApi(): BookApi {
  return isTauri() ? tauriBookApi : memoryBookApi;
}
