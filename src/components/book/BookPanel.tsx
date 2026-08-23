// 开局库面板：展示当前局面候选/推荐，支持自动走库与手动走库。

import { useEffect } from "react";
import type { BookApi } from "../../lib/book/api";
import type { BookStrategy } from "../../lib/book/types";
import type { GameSnapshot } from "../../lib/game/types";
import { useBookStore } from "../../stores/useBookStore";
import { Button } from "../ui/button";

export interface BookPanelProps {
  bookApi: BookApi;
  /** 当前局面 FEN（变化时重新查询）。 */
  currentFen: string | null;
  /** 自动走库后把新棋谱快照应用到游戏。 */
  onAutoMove: (snapshot: GameSnapshot) => void;
}

const STRATEGY_LABEL: Record<BookStrategy, string> = {
  best_score: "最高胜率",
  most_popular: "出现最多",
  first: "首条",
};

/** 开局库面板（状态清晰：命中/未命中/加载/错误）。 */
export function BookPanel({ bookApi, currentFen, onAutoMove }: BookPanelProps) {
  const status = useBookStore((state) => state.status);
  const strategy = useBookStore((state) => state.strategy);
  const candidates = useBookStore((state) => state.candidates);
  const recommended = useBookStore((state) => state.recommended);
  const message = useBookStore((state) => state.message);
  const init = useBookStore((state) => state.init);
  const setStrategy = useBookStore((state) => state.setStrategy);
  const refresh = useBookStore((state) => state.refresh);
  const autoMove = useBookStore((state) => state.autoMove);

  // 开局库独立于引擎；每次局面变化重新查询
  useEffect(() => {
    init(bookApi);
  }, [bookApi, init]);

  useEffect(() => {
    if (currentFen) {
      void refresh();
    }
  }, [currentFen, refresh]);

  return (
    <div data-testid="book-panel" className="flex flex-col gap-2 rounded border p-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">开局库</span>
        {status === "loading" && (
          <span data-testid="book-status" className="text-xs text-muted-foreground">
            查询中…
          </span>
        )}
        {status === "empty" && (
          <span data-testid="book-status" className="text-xs text-muted-foreground">
            未命中
          </span>
        )}
        {status === "error" && (
          <span data-testid="book-status" className="text-xs text-red-600">
            查询失败
          </span>
        )}
        {status === "hit" && (
          <span data-testid="book-status" className="text-xs text-green-700">
            命中 {candidates.length} 条
          </span>
        )}
      </div>

      <div className="flex items-center gap-2 text-xs">
        <label className="text-muted-foreground">推荐策略</label>
        <select
          data-testid="book-strategy"
          value={strategy}
          onChange={(event) => setStrategy(event.currentTarget.value as BookStrategy)}
          className="h-7 rounded-md border border-input bg-background px-2 text-xs"
        >
          {(Object.keys(STRATEGY_LABEL) as BookStrategy[]).map((s) => (
            <option key={s} value={s}>
              {STRATEGY_LABEL[s]}
            </option>
          ))}
        </select>
        <Button
          type="button"
          variant="outline"
          size="sm"
          data-testid="book-refresh"
          onClick={() => void refresh()}
        >
          重新查询
        </Button>
        <Button
          type="button"
          size="sm"
          data-testid="book-automove"
          disabled={status !== "hit"}
          onClick={() => void autoMove(onAutoMove)}
        >
          自动走库
        </Button>
      </div>

      {status === "empty" && (
        <p data-testid="book-empty" className="text-xs text-muted-foreground">
          当前局面未命中开局库——可继续走子后再次查询，或使用引擎分析。
        </p>
      )}
      {status === "error" && message && (
        <p data-testid="book-error" className="text-xs text-red-600">
          {message}
        </p>
      )}

      {recommended && (
        <p data-testid="book-recommended" className="text-xs">
          推荐：<span className="font-mono font-semibold">{recommended.mv}</span>（
          {STRATEGY_LABEL[strategy]}）
        </p>
      )}

      {candidates.length > 0 && (
        <ul data-testid="book-candidates" className="flex flex-col gap-0.5 text-xs">
          {candidates.map((c) => (
            <li key={c.mv} className="flex items-center gap-2">
              <span className="w-14 font-mono">{c.mv}</span>
              <span className="text-muted-foreground">×{c.count}</span>
              {c.hasStats && c.wins != null && (
                <span className="text-muted-foreground">
                  胜/和/负 {c.wins}/{c.draws}/{c.losses} · 得分 {(c.score * 100).toFixed(0)}%
                </span>
              )}
            </li>
          ))}
        </ul>
      )}

      {message && status !== "error" && (
        <p data-testid="book-message" className="text-xs text-muted-foreground">
          {message}
        </p>
      )}
    </div>
  );
}
