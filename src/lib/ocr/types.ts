// 截图识别结果类型（与 Rust `ocr::dto` 对应）。
export interface OcrPieceDto {
  color: "red" | "black";
  kind: "king" | "advisor" | "elephant" | "horse" | "rook" | "cannon" | "pawn";
}

export interface OcrCellDto {
  rank: number;
  file: number;
  piece: OcrPieceDto | null;
  confidence: number;
  uncertain: boolean;
}

export interface OcrResultDto {
  cells: OcrCellDto[];
  /** normal / flipped180 */
  orientation: string;
  sideToMove: string | null;
  fen: string;
  confidence: number;
  valid: boolean;
  issues: string[];
}
