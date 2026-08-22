import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Board } from "./Board";
import { parseFen, START_FEN } from "../../lib/board/notation";
import type { BoardView } from "../../lib/board/types";

const VIEW: BoardView = { flipVertical: false, flipHorizontal: false };

describe("Board", () => {
  it("renders pieces from the start position", () => {
    render(
      <Board
        position={parseFen(START_FEN)}
        selected={null}
        legalTargets={[]}
        view={VIEW}
        onSquareClick={() => undefined}
      />,
    );
    expect(screen.getByText("帅")).toBeInTheDocument(); // 红帅
    expect(screen.getByText("将")).toBeInTheDocument(); // 黑将
  });

  it("calls onSquareClick with the logical square", () => {
    const onClick = vi.fn();
    render(
      <Board
        position={parseFen(START_FEN)}
        selected={null}
        legalTargets={[]}
        view={VIEW}
        onSquareClick={onClick}
      />,
    );
    fireEvent.click(screen.getByTestId("sq-40")); // e0 = file 4 rank 0
    expect(onClick).toHaveBeenCalledWith({ rank: 0, file: 4 });
  });

  it("renders legal target markers", () => {
    render(
      <Board
        position={parseFen(START_FEN)}
        selected={{ rank: 2, file: 7 }}
        legalTargets={[{ rank: 2, file: 4 }]}
        view={VIEW}
        onSquareClick={() => undefined}
      />,
    );
    // 目标格 e2（file 4 rank 2）应渲染落点标记
    const targetSquare = screen.getByTestId("sq-42");
    expect(targetSquare.querySelector("circle")).not.toBeNull();
  });
});
