// 引擎访问接口：Tauri 实现（真实）与内存回退（浏览器开发预览）。

import { invokeCommand, isTauri, listenEvent } from "../ipc";
import type { EngineEventDto, EngineParamsDto, EngineStatusDto } from "./types";
import { normalizeEngineEvent, type EngineEvent } from "./types";

export interface EngineApi {
  start(program: string): Promise<string>;
  status(): Promise<EngineStatusDto>;
  setOption(name: string, value: string | null): Promise<void>;
  setPositionAndGo(fen: string, moves: string[], params: EngineParamsDto): Promise<void>;
  stop(): Promise<void>;
  restart(): Promise<void>;
  quit(): Promise<void>;
  subscribe(cb: (ev: EngineEvent) => void): () => void;
  /** 扫描常见目录中的 Pikafish 可执行文件（供引擎选择）。 */
  discover(): Promise<string[]>;
}

export const tauriEngineApi: EngineApi = {
  start: (program) => invokeCommand<string>("engine_start", { program }),
  status: () => invokeCommand<EngineStatusDto>("engine_status"),
  setOption: (name, value) => invokeCommand<void>("engine_set_option", { name, value }),
  setPositionAndGo: (fen, moves, params) =>
    invokeCommand<void>("engine_set_position_and_go", { fen, moves, params }),
  stop: () => invokeCommand<void>("engine_stop"),
  restart: () => invokeCommand<void>("engine_restart"),
  quit: () => invokeCommand<void>("engine_quit"),
  discover: () => invokeCommand<string[]>("engine_discover_binaries"),
  subscribe: (cb) => {
    let unlisten: (() => void) | null = null;
    void listenEvent<EngineEventDto>("engine://event", (payload) =>
      cb(normalizeEngineEvent(payload)),
    ).then((u) => {
      unlisten = u;
    });
    return () => {
      unlisten?.();
    };
  },
};

/** 浏览器开发预览回退：无引擎，接口保持可用但操作无效果/报错。 */
export const memoryEngineApi: EngineApi = {
  start: async () => {
    throw new Error("引擎需要 Tauri 环境（Rust Engine Manager）");
  },
  status: async () => ({ status: "stopped", engineId: null }),
  setOption: async () => undefined,
  setPositionAndGo: async () => undefined,
  stop: async () => undefined,
  restart: async () => undefined,
  quit: async () => undefined,
  discover: async () => [],
  subscribe: () => () => undefined,
};

export function getDefaultEngineApi(): EngineApi {
  return isTauri() ? tauriEngineApi : memoryEngineApi;
}
