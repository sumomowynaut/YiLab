// GIF 导出接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri } from "../ipc";
import type { GifExportOptions } from "./types";

export interface GifApi {
  exportCurrent(options: GifExportOptions): Promise<Uint8Array>;
  exportMainline(options: GifExportOptions): Promise<Uint8Array>;
  exportVariation(nodeId: number, options: GifExportOptions): Promise<Uint8Array>;
}

function asBytes(result: number[]): Uint8Array {
  return new Uint8Array(result);
}

export const tauriGifApi: GifApi = {
  exportCurrent: (o) =>
    invokeCommand<number[]>("gif_export_current", {
      frameDelayMs: o.frameDelayMs,
      cellSize: o.cellSize,
      showCoordinates: o.showCoordinates,
      showMoves: o.showMoves,
    }).then(asBytes),
  exportMainline: (o) =>
    invokeCommand<number[]>("gif_export_mainline", {
      frameDelayMs: o.frameDelayMs,
      cellSize: o.cellSize,
      showCoordinates: o.showCoordinates,
      showMoves: o.showMoves,
    }).then(asBytes),
  exportVariation: (nodeId, o) =>
    invokeCommand<number[]>("gif_export_variation", {
      nodeId,
      frameDelayMs: o.frameDelayMs,
      cellSize: o.cellSize,
      showCoordinates: o.showCoordinates,
      showMoves: o.showMoves,
    }).then(asBytes),
};

export const memoryGifApi: GifApi = {
  exportCurrent: async () => {
    throw new Error("GIF 导出需要 Tauri 环境（Rust 渲染核心）");
  },
  exportMainline: async () => {
    throw new Error("GIF 导出需要 Tauri 环境（Rust 渲染核心）");
  },
  exportVariation: async () => {
    throw new Error("GIF 导出需要 Tauri 环境（Rust 渲染核心）");
  },
};

export function getDefaultGifApi(): GifApi {
  return isTauri() ? tauriGifApi : memoryGifApi;
}
