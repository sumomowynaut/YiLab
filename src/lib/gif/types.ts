// GIF 导出类型（与 Rust `gif_export` 对应）。

export type GifSource = "current" | "mainline" | "variation";

export interface GifExportOptions {
  /** 帧间隔（毫秒）。 */
  frameDelayMs: number;
  /** 棋盘格子像素边长。 */
  cellSize: number;
  /** 是否显示坐标（a-i / 0-9）。 */
  showCoordinates: boolean;
  /** 是否显示棋步（最后一步高亮 + 标注）。 */
  showMoves: boolean;
}

export interface VariationOption {
  nodeId: number;
  label: string;
}
