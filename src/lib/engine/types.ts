// 引擎分析相关类型（与 Rust `engine::types` 对应，JSON 均为 camelCase）。

export type EngineStatus = "stopped" | "ready" | "searching" | "crashed";

export type Score = { cp: number } | { mate: number };

export interface InfoLineDto {
  depth: number | null;
  seldepth: number | null;
  multipv: number;
  score: Score | null;
  nodes: number | null;
  nps: number | null;
  timeMs: number | null;
  pv: string[];
  lowerbound: boolean;
  upperbound: boolean;
}

export interface BestMoveDto {
  mv: string;
  ponder: string | null;
}

export interface EngineStatusDto {
  status: EngineStatus;
  engineId: string | null;
}

/** `go` 参数（camelCase）。 */
export interface EngineParamsDto {
  infinite: boolean;
  depth: number | null;
  movetimeMs: number | null;
  nodes: number | null;
}

/** Tauri 事件原始载荷（Rust `EngineEvent` 外部标签序列化 + camelCase）。 */
export type EngineEventDto =
  | { info: InfoLineDto }
  | { infoString: string }
  | { bestMove: BestMoveDto }
  | { optionSet: { name: string; value: string | null } }
  | { error: string }
  | { crashed: { code: number | null } }
  | "searching"
  | "ready"
  | "started"
  | "stopped";

/** 归一化后的引擎事件（store 使用）。 */
export type EngineEvent =
  | { type: "info"; info: InfoLineDto }
  | { type: "infoString"; message: string }
  | { type: "bestmove"; bestMove: BestMoveDto }
  | { type: "optionSet"; name: string; value: string | null }
  | { type: "error"; message: string }
  | { type: "crashed"; code: number | null }
  | { type: "searching" }
  | { type: "ready" }
  | { type: "started" }
  | { type: "stopped" };

export function normalizeEngineEvent(ev: EngineEventDto): EngineEvent {
  if (typeof ev === "string") {
    return { type: ev };
  }
  if ("info" in ev) return { type: "info", info: ev.info };
  if ("infoString" in ev) return { type: "infoString", message: ev.infoString };
  if ("bestMove" in ev) return { type: "bestmove", bestMove: ev.bestMove };
  if ("optionSet" in ev)
    return { type: "optionSet", name: ev.optionSet.name, value: ev.optionSet.value };
  if ("error" in ev) return { type: "error", message: ev.error };
  if ("crashed" in ev) return { type: "crashed", code: ev.crashed.code };
  return { type: "ready" };
}
