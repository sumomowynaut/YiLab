// 截图识别接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri } from "../ipc";
import type { OcrResultDto } from "./types";

export interface OcrApi {
  /** 识别一张图片（PNG/JPEG 字节）。 */
  recognize(image: Uint8Array): Promise<OcrResultDto>;
}

export const tauriOcrApi: OcrApi = {
  recognize: (image) => invokeCommand<OcrResultDto>("ocr_recognize", { image: Array.from(image) }),
};

export const memoryOcrApi: OcrApi = {
  recognize: async () => {
    throw new Error("截图识别需要 Tauri 环境（Rust 视觉识别核心）");
  },
};

export function getDefaultOcrApi(): OcrApi {
  return isTauri() ? tauriOcrApi : memoryOcrApi;
}
