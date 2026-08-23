import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { BookPanel } from "./BookPanel";
import type { BookApi } from "../../lib/book/api";
import { useBookStore } from "../../stores/useBookStore";
import { parseFen, START_FEN } from "../../lib/board/notation";
import type { GameSnapshot } from "../../lib/game/types";

function makeSnapshot(): GameSnapshot {
  return {
    tree: {
      id: 0,
      mv: null,
      moveNumber: 0,
      isRed: true,
      comment: "",
      nags: [],
      children: [],
      isVariation: false,
    },
    currentId: 0,
    currentFen: START_FEN,
    position: parseFen(START_FEN),
    comment: "",
    nags: [],
    hasParent: false,
    previousId: null,
    nextMainId: null,
    undoAvailable: false,
    redoAvailable: false,
  };
}

function makeBookApi(overrides: Partial<BookApi> = {}): BookApi {
  return {
    lookup: vi.fn(async () => [
      { mv: "h2e2", count: 100, wins: 60, draws: 20, losses: 20, score: 0.7, hasStats: true },
      { mv: "b0c2", count: 10, wins: 9, draws: 0, losses: 1, score: 0.9, hasStats: true },
    ]),
    recommend: vi.fn(async () => ({
      mv: "b0c2",
      count: 10,
      wins: 9,
      draws: 0,
      losses: 1,
      score: 0.9,
      hasStats: true,
    })),
    autoMove: vi.fn(async () => ({ applied: "b0c2", snapshot: makeSnapshot() })),
    ...overrides,
  };
}

beforeEach(() => {
  useBookStore.setState({
    api: null,
    strategy: "best_score",
    candidates: [],
    recommended: null,
    status: "idle",
    message: null,
  });
});

describe("BookPanel", () => {
  it("shows candidates and recommended move on hit", async () => {
    render(<BookPanel bookApi={makeBookApi()} currentFen={START_FEN} onAutoMove={vi.fn()} />);
    expect(await screen.findByText("命中 2 条")).toBeInTheDocument();
    expect(screen.getByTestId("book-recommended")).toHaveTextContent("b0c2");
    expect(screen.getByTestId("book-candidates").children.length).toBe(2);
  });

  it("shows empty state when the book misses", async () => {
    const api = makeBookApi({
      lookup: vi.fn(async () => []),
      recommend: vi.fn(async () => null),
    });
    render(<BookPanel bookApi={api} currentFen={START_FEN} onAutoMove={vi.fn()} />);
    expect(await screen.findByTestId("book-empty")).toBeInTheDocument();
    expect(screen.getByTestId("book-status")).toHaveTextContent("未命中");
    expect(screen.getByTestId("book-automove")).toBeDisabled();
  });

  it("auto-plays the recommended move and applies the snapshot", async () => {
    const api = makeBookApi();
    const onAutoMove = vi.fn();
    render(<BookPanel bookApi={api} currentFen={START_FEN} onAutoMove={onAutoMove} />);
    await screen.findByTestId("book-status");
    fireEvent.click(screen.getByTestId("book-automove"));

    expect(await screen.findByTestId("book-message")).toHaveTextContent("已走库：b0c2");
    expect(api.autoMove).toHaveBeenCalledWith("best_score");
    expect(onAutoMove).toHaveBeenCalledWith(expect.objectContaining({ currentFen: START_FEN }));
  });

  it("shows an error state on lookup failure", async () => {
    const api = makeBookApi({
      lookup: vi.fn(async () => {
        throw new Error("查询失败");
      }),
    });
    render(<BookPanel bookApi={api} currentFen={START_FEN} onAutoMove={vi.fn()} />);
    expect(await screen.findByTestId("book-error")).toHaveTextContent("查询失败");
    expect(screen.getByTestId("book-status")).toHaveTextContent("查询失败");
  });
});
