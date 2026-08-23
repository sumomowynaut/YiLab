// 自动复盘类型（与 Rust `analysis` 模块对应）。

export type AnalysisCategory = "best" | "excellent" | "good" | "inaccuracy" | "mistake" | "blunder";

export interface MoveAssessmentDto {
  nodeId: number;
  /** 实际着法（UCI）。 */
  mv: string;
  /** 最佳着法（UCI）。 */
  bestMove: string;
  /** 走前评价（红方视角，厘兵）。 */
  evalBeforeCp: number;
  /** 走后评价（红方视角，厘兵）。 */
  evalAfterCp: number;
  /** 评价损失（走子方视角，厘兵）。 */
  lossCp: number;
  depth: number;
  pv: string[];
  category: AnalysisCategory;
}

export type AnalysisStatusName = "idle" | "running" | "paused" | "done" | "failed";

export interface AnalysisStatusDto {
  status: AnalysisStatusName;
  progress: number;
  total: number;
  assessments: MoveAssessmentDto[];
}

export type AnalysisEvent =
  | { type: "statusChanged"; status: AnalysisStatusName }
  | { type: "progress"; done: number; total: number; currentNode: number | null }
  | { type: "assessment"; assessment: MoveAssessmentDto }
  | { type: "finished"; assessments: MoveAssessmentDto[] };
