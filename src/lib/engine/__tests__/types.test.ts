import { describe, expect, it } from "vitest";
import { normalizeEngineEvent, type EngineEventDto } from "../types";

describe("normalizeEngineEvent", () => {
  it("normalizes object variants", () => {
    const ev: EngineEventDto = {
      info: {
        depth: 8,
        seldepth: 10,
        multipv: 2,
        score: { cp: 35 },
        nodes: 123,
        nps: 456,
        timeMs: 12,
        pv: ["h2e2"],
        lowerbound: false,
        upperbound: false,
      },
    };
    expect(normalizeEngineEvent(ev)).toEqual({ type: "info", info: ev.info });
    expect(normalizeEngineEvent({ bestMove: { mv: "h2e2", ponder: null } })).toEqual({
      type: "bestmove",
      bestMove: { mv: "h2e2", ponder: null },
    });
    expect(normalizeEngineEvent({ crashed: { code: 1 } })).toEqual({
      type: "crashed",
      code: 1,
    });
  });

  it("normalizes string variants", () => {
    expect(normalizeEngineEvent("searching")).toEqual({ type: "searching" });
    expect(normalizeEngineEvent("ready")).toEqual({ type: "ready" });
    expect(normalizeEngineEvent("stopped")).toEqual({ type: "stopped" });
  });
});
