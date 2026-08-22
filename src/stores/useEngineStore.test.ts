import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BoardApi } from "../lib/board/api";
import type { EngineApi } from "../lib/engine/api";
import type { EngineEvent, InfoLineDto } from "../lib/engine/types";
import { useEngineStore } from "./useEngineStore";

function makeInfo(multipv: number, ...pv: string[]): InfoLineDto {
  return {
    depth: 8,
    seldepth: 10,
    multipv,
    score: { cp: 35 },
    nodes: 100,
    nps: 1000,
    timeMs: 10,
    pv,
    lowerbound: false,
    upperbound: false,
  };
}

function makeEngineApi(): EngineApi & { emit: (ev: EngineEvent) => void } {
  const listeners: ((ev: EngineEvent) => void)[] = [];
  return {
    start: vi.fn(async () => "mock-engine"),
    status: vi.fn(async () => ({ status: "ready", engineId: "mock-engine" })),
    setOption: vi.fn(async () => undefined),
    setPositionAndGo: vi.fn(async () => undefined),
    stop: vi.fn(async () => undefined),
    restart: vi.fn(async () => undefined),
    quit: vi.fn(async () => undefined),
    subscribe: vi.fn((cb: (ev: EngineEvent) => void) => {
      listeners.push(cb);
      return () => undefined;
    }),
    emit: (ev: EngineEvent) => {
      for (const cb of listeners) cb(ev);
    },
  } as never;
}

function makeBoardApi(): BoardApi {
  return {
    startPosition: vi.fn(async () => undefined as never),
    fromFen: vi.fn(async () => undefined as never),
    legalMoves: vi.fn(async () => []),
    makeMove: vi.fn(async () => undefined as never),
    applyMoves: vi.fn(async () => undefined as never),
    validate: vi.fn(async () => undefined as never),
    rotate: vi.fn(async () => undefined as never),
    setPiece: vi.fn(async () => undefined as never),
    clearSquare: vi.fn(async () => undefined as never),
    setSide: vi.fn(async () => undefined as never),
    clearAll: vi.fn(async () => undefined as never),
  };
}

beforeEach(() => {
  useEngineStore.setState({
    api: null,
    boardApi: null,
    status: "stopped",
    engineId: null,
    message: null,
    lines: {},
    bestMove: null,
    analysisEnabled: false,
    settings: { programPath: "", threads: 1, hash: 16, depth: null, multipv: 1 },
    preview: null,
    epoch: 0,
    pending: false,
  });
});

describe("useEngineStore", () => {
  it("starts, stops and restarts the engine", async () => {
    const api = makeEngineApi();
    useEngineStore.getState().init(api, makeBoardApi());
    await useEngineStore.getState().start();
    expect(api.start).toHaveBeenCalledWith("");
    expect(useEngineStore.getState().analysisEnabled).toBe(true);
    await useEngineStore.getState().stop();
    expect(api.stop).toHaveBeenCalled();
    expect(useEngineStore.getState().analysisEnabled).toBe(false);
    await useEngineStore.getState().restart();
    expect(api.restart).toHaveBeenCalled();
  });

  it("discards stale analysis after switching positions (race)", async () => {
    const api = makeEngineApi();
    useEngineStore.getState().init(api, makeBoardApi());
    // 分析局面 A
    await useEngineStore.getState().startAnalysis("fenA");
    api.emit({ type: "info", info: makeInfo(1, "a0a1") }); // pending 期间的旧事件
    expect(useEngineStore.getState().lines).toEqual({});
    api.emit({ type: "searching" });
    api.emit({ type: "info", info: makeInfo(1, "h2e2") });
    expect(useEngineStore.getState().lines[1]?.pv).toEqual(["h2e2"]);

    // 快速切换到局面 B
    await useEngineStore.getState().startAnalysis("fenB");
    api.emit({ type: "info", info: makeInfo(1, "b0c2") }); // 旧分析的迟到事件
    expect(useEngineStore.getState().lines[1]).toBeUndefined();
    api.emit({ type: "searching" });
    api.emit({ type: "info", info: makeInfo(1, "h2e2") });
    expect(useEngineStore.getState().lines[1]?.pv).toEqual(["h2e2"]);
    expect(useEngineStore.getState().lines[1]?.pv).not.toContain("b0c2");
  });

  it("applies settings and uses depth in analysis params", async () => {
    const api = makeEngineApi();
    useEngineStore.getState().init(api, makeBoardApi());
    await useEngineStore.getState().applySettings({ threads: 4, hash: 64, multipv: 3, depth: 10 });
    expect(api.setOption).toHaveBeenCalledWith("Threads", "4");
    expect(api.setOption).toHaveBeenCalledWith("Hash", "64");
    expect(api.setOption).toHaveBeenCalledWith("MultiPV", "3");
    await useEngineStore.getState().startAnalysis("fen");
    expect(api.setPositionAndGo).toHaveBeenCalledWith("fen", [], {
      infinite: false,
      depth: 10,
      movetimeMs: null,
      nodes: null,
    });
  });

  it("previews a PV on the board via applyMoves", async () => {
    const api = makeEngineApi();
    const boardApi = makeBoardApi();
    boardApi.applyMoves = vi.fn(async () => ({ fen: "fen2" }) as never);
    useEngineStore.getState().init(api, boardApi);
    await useEngineStore.getState().previewPv(["h2e2", "h7e7"], "fen1");
    expect(boardApi.applyMoves).toHaveBeenCalledWith("fen1", ["h2e2", "h7e7"]);
    expect(useEngineStore.getState().preview?.moves).toEqual(["h2e2", "h7e7"]);
    useEngineStore.getState().clearPreview();
    expect(useEngineStore.getState().preview).toBeNull();
  });

  it("reports crash via event", async () => {
    const api = makeEngineApi();
    useEngineStore.getState().init(api, makeBoardApi());
    api.emit({ type: "crashed", code: 1 });
    expect(useEngineStore.getState().status).toBe("crashed");
    expect(useEngineStore.getState().message).toContain("崩溃");
  });
});
