// 开局库类型（与 Rust `book::dto` 对应）。

import type { GameSnapshot } from "../game/types";

export type BookStrategy = "best_score" | "most_popular" | "first";

export interface BookMoveDto {
  /** 着法（UCI）。 */
  mv: string;
  /** 出现次数。 */
  count: number;
  /** 胜/和/负（数据源提供时才有）。 */
  wins: number | null;
  draws: number | null;
  losses: number | null;
  /** 推荐分 [0,1]。 */
  score: number;
  hasStats: boolean;
}

export interface BookAutoMoveDto {
  /** 实际走出的着法；开局库未命中时为 null。 */
  applied: string | null;
  snapshot: GameSnapshot;
}
