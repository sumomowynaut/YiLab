import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { EvalCurve } from "./EvalCurve";
import { curvePath, type CurvePoint } from "../../lib/engine/curve";

describe("curvePath", () => {
  it("returns null for empty points", () => {
    expect(curvePath([])).toBeNull();
  });

  it("builds an M/L path for multiple points", () => {
    const path = curvePath([
      { fen: "a", scoreCp: 0 },
      { fen: "b", scoreCp: 100 },
      { fen: "c", scoreCp: -100 },
    ]);
    expect(path).toMatch(/^M/);
    expect(path).toContain("L");
  });
});

describe("EvalCurve", () => {
  it("shows the empty state", () => {
    render(<EvalCurve points={[]} onClear={vi.fn()} />);
    expect(screen.getByTestId("eval-curve-empty")).toBeInTheDocument();
  });

  it("renders an SVG when points exist", () => {
    const points: CurvePoint[] = [
      { fen: "a", scoreCp: 0 },
      { fen: "b", scoreCp: 120 },
    ];
    render(<EvalCurve points={points} onClear={vi.fn()} />);
    expect(screen.getByTestId("eval-curve-svg")).toBeInTheDocument();
    expect(screen.queryByTestId("eval-curve-empty")).not.toBeInTheDocument();
  });

  it("calls onClear when the clear button is clicked", () => {
    const onClear = vi.fn();
    render(<EvalCurve points={[{ fen: "a", scoreCp: 1 }]} onClear={onClear} />);
    fireEvent.click(screen.getByTestId("eval-curve-clear"));
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});
