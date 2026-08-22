// 棋谱树（Game Tree）的 TypeScript 类型（与 Rust `game::dto` 对应）。

import type { PositionSnapshot } from "../board/types";

export interface TreeNodeDto {
  id: number;
  /** 本节点着法（UCI），根节点为 null。 */
  mv: string | null;
  /** 显示回合数（红方 N.，黑方 N…）。 */
  moveNumber: number;
  isRed: boolean;
  comment: string;
  nags: string[];
  children: TreeNodeDto[];
  isVariation: boolean;
}

export interface GameSnapshot {
  tree: TreeNodeDto;
  currentId: number;
  currentFen: string;
  position: PositionSnapshot;
  comment: string;
  nags: string[];
  hasParent: boolean;
  previousId: number | null;
  nextMainId: number | null;
  undoAvailable: boolean;
  redoAvailable: boolean;
}
