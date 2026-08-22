// 导入导出接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri } from "../ipc";
import type { GameSnapshot } from "../game/types";
import type { IoFormat } from "./types";

export interface IoApi {
  /** 从文本导入棋谱；format 传空串时由 Rust 按内容嗅探。 */
  importText(format: IoFormat | "", text: string): Promise<GameSnapshot>;
  /** 导出当前棋谱为文本。 */
  exportText(format: IoFormat): Promise<string>;
}

export const tauriIoApi: IoApi = {
  importText: (format, text) => invokeCommand<GameSnapshot>("io_import", { format, text }),
  exportText: (format) => invokeCommand<string>("io_export", { format }),
};

/** 浏览器开发预览回退：导入导出需要 Rust 核心。 */
export const memoryIoApi: IoApi = {
  importText: async () => {
    throw new Error("导入需要 Tauri 环境（Rust 核心）");
  },
  exportText: async () => {
    throw new Error("导出需要 Tauri 环境（Rust 核心）");
  },
};

export function getDefaultIoApi(): IoApi {
  return isTauri() ? tauriIoApi : memoryIoApi;
}
