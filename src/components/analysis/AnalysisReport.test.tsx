import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AnalysisReport } from "./AnalysisReport";
import type { MoveAssessmentDto } from "../../lib/analysis/types";

function makeAssessment(overrides: Partial<MoveAssessmentDto> = {}): MoveAssessmentDto {
  return {
    nodeId: 1,
    mv: "h2e2",
    bestMove: "b0c2",
    evalBeforeCp: 30,
    evalAfterCp: 10,
    lossCp: 20,
    depth: 12,
    pv: ["b0c2", "h7e7"],
    category: "good",
    ...overrides,
  };
}

function renderReport(overrides: Partial<Parameters<typeof AnalysisReport>[0]> = {}) {
  const props = {
    status: "idle" as const,
    progress: 0,
    total: 0,
    assessments: [],
    onStart: vi.fn(),
    onStop: vi.fn(),
    onContinue: vi.fn(),
    onRestart: vi.fn(),
    onNavigate: vi.fn(),
    ...overrides,
  };
  return { ...render(<AnalysisReport {...props} />), props };
}

describe("AnalysisReport", () => {
  it("shows start button when idle", () => {
    renderReport();
    expect(screen.getByTestId("analysis-start")).toBeInTheDocument();
    expect(screen.queryByTestId("analysis-stop")).not.toBeInTheDocument();
    expect(screen.getByTestId("analysis-chart-empty")).toBeInTheDocument();
  });

  it("shows stop while running and continue/restart when paused", () => {
    const { rerender } = renderReport({ status: "running", progress: 1, total: 3 });
    expect(screen.getByTestId("analysis-stop")).toBeInTheDocument();
    expect(screen.getByTestId("analysis-status")).toHaveTextContent("分析中（1/3）");

    rerender(
      <AnalysisReport
        status="paused"
        progress={1}
        total={3}
        assessments={[]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onContinue={vi.fn()}
        onRestart={vi.fn()}
        onNavigate={vi.fn()}
      />,
    );
    expect(screen.getByTestId("analysis-continue")).toBeInTheDocument();
    expect(screen.getByTestId("analysis-restart")).toBeInTheDocument();
  });

  it("renders chart, mistakes, best moves and table when done", () => {
    const assessments = [
      makeAssessment({ nodeId: 1, mv: "h2e2", category: "best", lossCp: 5 }),
      makeAssessment({ nodeId: 2, mv: "h7e7", category: "blunder", lossCp: 400 }),
      makeAssessment({ nodeId: 3, mv: "h0g2", category: "good", lossCp: 40 }),
    ];
    renderReport({ status: "done", assessments, total: 3 });
    expect(screen.getByTestId("analysis-chart")).toBeInTheDocument();
    expect(screen.getByTestId("analysis-mistakes").children.length).toBe(1);
    expect(screen.getByTestId("analysis-best").children.length).toBe(1);
    expect(screen.getByTestId("analysis-table")).toBeInTheDocument();
  });

  it("navigates when a chart point is clicked", () => {
    const onNavigate = vi.fn();
    renderReport({
      status: "done",
      assessments: [makeAssessment({ nodeId: 7, category: "best" })],
      total: 1,
      onNavigate,
    });
    fireEvent.click(screen.getByTestId("eval-point-0"));
    expect(onNavigate).toHaveBeenCalledWith(7);
  });

  it("calls start/restart handlers", () => {
    const props = renderReport({ status: "done" }).props;
    fireEvent.click(screen.getByTestId("analysis-start"));
    expect(props.onStart).toHaveBeenCalled();
    fireEvent.click(screen.getByTestId("analysis-restart"));
    expect(props.onRestart).toHaveBeenCalled();
  });
});
