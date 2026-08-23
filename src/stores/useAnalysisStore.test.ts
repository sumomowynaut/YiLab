import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisApi } from "../lib/analysis/api";
import type { AnalysisEvent, MoveAssessmentDto } from "../lib/analysis/types";
import { useAnalysisStore } from "./useAnalysisStore";

type MockApi = AnalysisApi & { emit: (ev: AnalysisEvent) => void };

function makeApi(overrides: Partial<MockApi> = {}): MockApi {
  const listeners: ((ev: AnalysisEvent) => void)[] = [];
  return {
    start: vi.fn(async () => ({ status: "running", progress: 0, total: 2, assessments: [] })),
    stop: vi.fn(async () => undefined),
    continue: vi.fn(async () => ({ status: "running", progress: 1, total: 2, assessments: [] })),
    status: vi.fn(async () => ({ status: "idle", progress: 0, total: 0, assessments: [] })),
    subscribe: vi.fn((cb: (ev: AnalysisEvent) => void) => {
      listeners.push(cb);
      return () => undefined;
    }),
    emit: (ev: AnalysisEvent) => {
      for (const cb of listeners) cb(ev);
    },
    ...overrides,
  } as MockApi;
}

const assessment = (nodeId: number, lossCp: number): MoveAssessmentDto => ({
  nodeId,
  mv: "h2e2",
  bestMove: "b0c2",
  evalBeforeCp: 30,
  evalAfterCp: 20,
  lossCp,
  depth: 12,
  pv: ["b0c2"],
  category: lossCp > 200 ? "blunder" : "best",
});

beforeEach(() => {
  useAnalysisStore.setState({
    api: null,
    status: "idle",
    progress: 0,
    total: 0,
    assessments: [],
  });
});

describe("useAnalysisStore", () => {
  it("subscribes and refreshes on init", async () => {
    const api = makeApi();
    useAnalysisStore.getState().init(api);
    expect(api.subscribe).toHaveBeenCalledTimes(1);
    expect(api.status).toHaveBeenCalled();
  });

  it("appends assessment events and marks done on finished", async () => {
    const api = makeApi();
    useAnalysisStore.getState().init(api);
    api.emit({ type: "assessment", assessment: assessment(1, 10) });
    api.emit({ type: "progress", done: 1, total: 2, currentNode: 1 });
    expect(useAnalysisStore.getState().assessments).toHaveLength(1);
    expect(useAnalysisStore.getState().progress).toBe(1);

    api.emit({
      type: "finished",
      assessments: [assessment(1, 10), assessment(2, 300)],
    });
    expect(useAnalysisStore.getState().assessments).toHaveLength(2);
    expect(useAnalysisStore.getState().status).toBe("done");
  });

  it("dedupes assessment events by node id", async () => {
    const api = makeApi();
    useAnalysisStore.getState().init(api);
    api.emit({ type: "assessment", assessment: assessment(1, 10) });
    api.emit({ type: "assessment", assessment: { ...assessment(1, 99), lossCp: 99 } });
    expect(useAnalysisStore.getState().assessments).toHaveLength(1);
    expect(useAnalysisStore.getState().assessments[0].lossCp).toBe(99);
  });

  it("start/stop/continue call the api and update status", async () => {
    const api = makeApi();
    useAnalysisStore.getState().init(api);
    await useAnalysisStore.getState().start(12, null);
    expect(api.start).toHaveBeenCalledWith(12, null);
    expect(useAnalysisStore.getState().status).toBe("running");

    await useAnalysisStore.getState().stop();
    expect(api.stop).toHaveBeenCalled();
    expect(useAnalysisStore.getState().status).toBe("paused");

    await useAnalysisStore.getState().continue();
    expect(api.continue).toHaveBeenCalled();
    expect(useAnalysisStore.getState().status).toBe("running");
  });
});
