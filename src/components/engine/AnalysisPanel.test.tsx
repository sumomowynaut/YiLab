import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AnalysisPanel } from "./AnalysisPanel";
import type { InfoLineDto } from "../../lib/engine/types";

function makeInfo(multipv: number, ...pv: string[]): InfoLineDto {
  return {
    depth: 8,
    seldepth: 10,
    multipv,
    score: { cp: 35 },
    nodes: 1234,
    nps: 56789,
    timeMs: 12,
    pv,
    lowerbound: false,
    upperbound: false,
  };
}

function renderPanel(overrides: Partial<Parameters<typeof AnalysisPanel>[0]> = {}) {
  const props = {
    status: "searching" as const,
    engineId: "mock",
    lines: { 1: makeInfo(1, "h2e2", "h7e7"), 2: makeInfo(2, "b0c2") },
    bestMove: { mv: "h2e2", ponder: null },
    message: null,
    onStart: vi.fn(),
    onStop: vi.fn(),
    onRestart: vi.fn(),
    onPreview: vi.fn(),
    ...overrides,
  };
  render(
    <AnalysisPanel
      status={props.status}
      engineId={props.engineId}
      lines={props.lines}
      bestMove={props.bestMove}
      message={props.message}
      onStart={props.onStart}
      onStop={props.onStop}
      onRestart={props.onRestart}
      onPreview={props.onPreview}
    />,
  );
  return props;
}

describe("AnalysisPanel", () => {
  it("shows evaluation, depth, nodes, nps, time and PV per multipv", () => {
    renderPanel();
    expect(screen.getByTestId("engine-eval-1")).toHaveTextContent("+35");
    expect(screen.getByTestId("engine-eval-2")).toHaveTextContent("+35");
    expect(screen.getByTestId("engine-line-1")).toHaveTextContent("d8");
    expect(screen.getByTestId("engine-line-1")).toHaveTextContent("1,234");
    expect(screen.getByTestId("engine-line-1")).toHaveTextContent("56,789nps");
    expect(screen.getByTestId("engine-line-1")).toHaveTextContent("12ms");
    expect(screen.getByTestId("engine-pv-1")).toHaveTextContent("h2e2");
    expect(screen.getByTestId("engine-pv-1")).toHaveTextContent("h7e7");
  });

  it("shows mate scores and bestmove", () => {
    renderPanel({
      lines: { 1: { ...makeInfo(1, "h2e2"), score: { mate: 3 } } },
      bestMove: { mv: "h2e2", ponder: "h7e7" },
    });
    expect(screen.getByTestId("engine-eval-1")).toHaveTextContent("绝杀 3");
    expect(screen.getByTestId("engine-bestmove")).toHaveTextContent("h2e2");
  });

  it("previews a PV on click", () => {
    const props = renderPanel();
    fireEvent.click(screen.getByTestId("engine-line-1"));
    expect(props.onPreview).toHaveBeenCalledWith(["h2e2", "h7e7"]);
  });

  it("start/stop/restart call handlers", () => {
    const props = renderPanel();
    fireEvent.click(screen.getByTestId("engine-stop"));
    expect(props.onStop).toHaveBeenCalled();
    fireEvent.click(screen.getByTestId("engine-restart"));
    expect(props.onRestart).toHaveBeenCalled();
  });

  it("shows the status badge", () => {
    renderPanel({ status: "searching", engineId: null });
    expect(screen.getByTestId("engine-status")).toHaveTextContent("分析中");
  });
});
