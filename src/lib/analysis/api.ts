// 自动复盘接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri, listenEvent } from "../ipc";
import type { AnalysisEvent, AnalysisStatusDto } from "./types";

export interface AnalysisApi {
  start(depth: number | null, movetimeMs: number | null): Promise<AnalysisStatusDto>;
  stop(): Promise<void>;
  continue(): Promise<AnalysisStatusDto>;
  status(): Promise<AnalysisStatusDto>;
  subscribe(cb: (ev: AnalysisEvent) => void): () => void;
}

export const tauriAnalysisApi: AnalysisApi = {
  start: (depth, movetimeMs) =>
    invokeCommand<AnalysisStatusDto>("analysis_start", { depth, movetimeMs }),
  stop: () => invokeCommand<void>("analysis_stop"),
  continue: () => invokeCommand<AnalysisStatusDto>("analysis_continue"),
  status: () => invokeCommand<AnalysisStatusDto>("analysis_status"),
  subscribe: (cb) => {
    let cancel = () => {};
    void listenEvent<AnalysisEvent>("analysis://event", (ev) => cb(ev)).then((unsub) => {
      cancel = unsub;
    });
    return () => cancel();
  },
};

export const memoryAnalysisApi: AnalysisApi = {
  start: async () => {
    throw new Error("自动复盘需要 Tauri 环境（Rust 引擎核心）");
  },
  stop: async () => {},
  continue: async () => {
    throw new Error("自动复盘需要 Tauri 环境（Rust 引擎核心）");
  },
  status: async () => ({ status: "idle", progress: 0, total: 0, assessments: [] }),
  subscribe: () => () => {},
};

export function getDefaultAnalysisApi(): AnalysisApi {
  return isTauri() ? tauriAnalysisApi : memoryAnalysisApi;
}
