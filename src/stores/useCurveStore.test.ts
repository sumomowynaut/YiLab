import { beforeEach, describe, expect, it } from "vitest";
import { useCurveStore } from "./useCurveStore";

beforeEach(() => {
  useCurveStore.setState({ points: [] });
});

describe("useCurveStore", () => {
  it("records points in order", () => {
    useCurveStore.getState().record("fenA", 20);
    useCurveStore.getState().record("fenB", -15);
    expect(useCurveStore.getState().points).toEqual([
      { fen: "fenA", scoreCp: 20 },
      { fen: "fenB", scoreCp: -15 },
    ]);
  });

  it("updates an existing fen in place without reordering", () => {
    useCurveStore.getState().record("fenA", 20);
    useCurveStore.getState().record("fenB", -15);
    useCurveStore.getState().record("fenA", 45);
    expect(useCurveStore.getState().points).toEqual([
      { fen: "fenA", scoreCp: 45 },
      { fen: "fenB", scoreCp: -15 },
    ]);
  });

  it("clears the curve", () => {
    useCurveStore.getState().record("fenA", 20);
    useCurveStore.getState().clear();
    expect(useCurveStore.getState().points).toEqual([]);
  });

  it("replaces points via setPoints", () => {
    useCurveStore.getState().setPoints([{ fen: "x", scoreCp: 1 }]);
    expect(useCurveStore.getState().points).toHaveLength(1);
  });
});
