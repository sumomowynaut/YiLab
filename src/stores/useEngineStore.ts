import { create } from "zustand";
import type { BoardApi } from "../lib/board/api";
import type { PositionSnapshot } from "../lib/board/types";
import type { EngineApi } from "../lib/engine/api";
import type { EngineEvent } from "../lib/engine/types";
import type { BestMoveDto, EngineStatus, InfoLineDto } from "../lib/engine/types";

export interface EngineSettings {
  programPath: string;
  threads: number;
  hash: number;
  depth: number | null; // null = 无限分析
  multipv: number;
}

let cleanupSubscription: (() => void) | null = null;

const SETTINGS_KEY = "pikaxiangqi-engine-settings";

const DEFAULT_SETTINGS: EngineSettings = {
  programPath: "",
  threads: 1,
  hash: 16,
  depth: null,
  multipv: 1,
};

const MULTIPV_OPTIONS = [1, 2, 3, 5, 10];

function loadSettings(): EngineSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      return DEFAULT_SETTINGS;
    }
    const p = JSON.parse(raw) as Partial<EngineSettings>;
    return {
      programPath: typeof p.programPath === "string" ? p.programPath : DEFAULT_SETTINGS.programPath,
      threads:
        typeof p.threads === "number" && p.threads > 0 ? p.threads : DEFAULT_SETTINGS.threads,
      hash: typeof p.hash === "number" && p.hash > 0 ? p.hash : DEFAULT_SETTINGS.hash,
      depth: p.depth === null || typeof p.depth === "number" ? p.depth : DEFAULT_SETTINGS.depth,
      multipv:
        typeof p.multipv === "number" && MULTIPV_OPTIONS.includes(p.multipv)
          ? p.multipv
          : DEFAULT_SETTINGS.multipv,
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function saveSettings(settings: EngineSettings): void {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // localStorage 不可用时静默忽略
  }
}

interface EngineState {
  api: EngineApi | null;
  boardApi: BoardApi | null;
  status: EngineStatus;
  engineId: string | null;
  message: string | null;
  /** multipv -> 该主变最新 info。 */
  lines: Record<number, InfoLineDto>;
  bestMove: BestMoveDto | null;
  analysisEnabled: boolean;
  settings: EngineSettings;
  /** 棋盘 PV 预览。 */
  preview: { fen: string; moves: string[]; position: PositionSnapshot } | null;
  /** 分析代次：每次换局面递增；`pending` 期间忽略旧事件（竞态防护）。 */
  epoch: number;
  pending: boolean;

  init: (api: EngineApi, boardApi: BoardApi) => void;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  restart: () => Promise<void>;
  setProgramPath: (path: string) => void;
  applySettings: (patch: Partial<EngineSettings>) => Promise<void>;
  startAnalysis: (fen: string) => Promise<void>;
  previewPv: (pv: string[], fen: string) => Promise<void>;
  clearPreview: () => void;
}

function handleEvent(state: EngineState, ev: EngineEvent): Partial<EngineState> | null {
  switch (ev.type) {
    case "searching":
      return { status: "searching", pending: false };
    case "ready":
    case "started":
      return { status: "ready" };
    case "stopped":
      return { status: "ready" };
    case "info":
      // 竞态防护：pending（新分析尚未开始）期间的旧事件一律忽略
      if (state.pending) {
        return null;
      }
      return {
        lines: { ...state.lines, [ev.info.multipv]: ev.info },
        status: "searching",
      };
    case "bestmove":
      if (state.pending) {
        return null;
      }
      return { bestMove: ev.bestMove, status: "ready", pending: false };
    case "crashed":
      return { status: "crashed", pending: false, message: `引擎崩溃（code=${ev.code ?? "?"}）` };
    case "error":
      return { message: ev.message };
    default:
      return null;
  }
}

export const useEngineStore = create<EngineState>((set, get) => ({
  api: null,
  boardApi: null,
  status: "stopped",
  engineId: null,
  message: null,
  lines: {},
  bestMove: null,
  analysisEnabled: false,
  settings: DEFAULT_SETTINGS,
  preview: null,
  epoch: 0,
  pending: false,

  init(api, boardApi) {
    cleanupSubscription?.();
    cleanupSubscription = api.subscribe((ev) => {
      const patch = handleEvent(get(), ev);
      if (patch) {
        set(patch);
      }
    });
    set({ api, boardApi, settings: loadSettings() });
  },

  async start() {
    const { api, settings } = get();
    if (!api) return;
    set({ message: null });
    try {
      const engineId = await api.start(settings.programPath);
      set({ engineId, analysisEnabled: true, status: "ready", message: null });
    } catch (error) {
      set({ message: String(error), analysisEnabled: false });
    }
  },

  async stop() {
    const { api } = get();
    if (!api) return;
    try {
      await api.stop();
      set({ analysisEnabled: false, pending: false });
    } catch (error) {
      set({ message: String(error) });
    }
  },

  async restart() {
    const { api } = get();
    if (!api) return;
    set({ message: null });
    try {
      await api.restart();
      set({ analysisEnabled: true, status: "ready", message: null });
    } catch (error) {
      set({ message: String(error), analysisEnabled: false });
    }
  },

  setProgramPath: (programPath) => set({ settings: { ...get().settings, programPath } }),

  async applySettings(patch) {
    const { api, settings } = get();
    const next = { ...settings, ...patch };
    set({ settings: next });
    saveSettings(next);
    if (!api) return;
    try {
      await api.setOption("Threads", String(next.threads));
      await api.setOption("Hash", String(next.hash));
      await api.setOption("MultiPV", String(next.multipv));
    } catch (error) {
      set({ message: String(error) });
    }
  },

  async startAnalysis(fen) {
    const { api, settings } = get();
    if (!api || !settings) return;
    // 新代次：清空旧显示，进入 pending，直到引擎发出 Searching 边界事件
    set((s) => ({
      epoch: s.epoch + 1,
      pending: true,
      lines: {},
      bestMove: null,
    }));
    const params = {
      infinite: settings.depth == null,
      depth: settings.depth,
      movetimeMs: null,
      nodes: null,
    };
    try {
      await api.setPositionAndGo(fen, [], params);
    } catch (error) {
      set({ message: String(error), pending: false });
    }
  },

  async previewPv(pv, fen) {
    const { boardApi } = get();
    if (!boardApi || pv.length === 0) return;
    try {
      const position = await boardApi.applyMoves(fen, pv);
      set({ preview: { fen, moves: pv, position }, message: null });
    } catch (error) {
      set({ message: String(error) });
    }
  },

  clearPreview: () => set({ preview: null }),
}));
