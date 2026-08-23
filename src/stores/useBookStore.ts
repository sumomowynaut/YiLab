// 开局库状态：查询当前局面候选、推荐、自动走库。

import { create } from "zustand";
import type { BookApi } from "../lib/book/api";
import type { BookMoveDto, BookStrategy } from "../lib/book/types";
import type { GameSnapshot } from "../lib/game/types";

export type BookPanelStatus = "idle" | "loading" | "hit" | "empty" | "error";

interface BookState {
  api: BookApi | null;
  strategy: BookStrategy;
  candidates: BookMoveDto[];
  recommended: BookMoveDto | null;
  status: BookPanelStatus;
  message: string | null;

  init: (api: BookApi) => void;
  setStrategy: (s: BookStrategy) => void;
  refresh: () => Promise<void>;
  autoMove: (onApplied: (snapshot: GameSnapshot) => void) => Promise<void>;
}

export const useBookStore = create<BookState>((set, get) => ({
  api: null,
  strategy: "best_score",
  candidates: [],
  recommended: null,
  status: "idle",
  message: null,

  init(api) {
    set({ api });
    void get().refresh();
  },

  setStrategy: (strategy) => set({ strategy }),

  async refresh() {
    const { api, strategy } = get();
    if (!api) return;
    set({ status: "loading", message: null });
    try {
      const candidates = await api.lookup();
      const recommended = await api.recommend(strategy);
      if (candidates.length === 0 && recommended === null) {
        set({ candidates, recommended, status: "empty" });
      } else {
        set({ candidates, recommended, status: "hit" });
      }
    } catch (error) {
      set({ status: "error", message: String(error) });
    }
  },

  async autoMove(onApplied) {
    const { api, strategy } = get();
    if (!api) return;
    set({ status: "loading", message: null });
    try {
      const result = await api.autoMove(strategy);
      onApplied(result.snapshot);
      await get().refresh();
      set({
        message: result.applied ? `已走库：${result.applied}` : "开局库未命中，未走库",
      });
    } catch (error) {
      set({ status: "error", message: String(error) });
    }
  },
}));
