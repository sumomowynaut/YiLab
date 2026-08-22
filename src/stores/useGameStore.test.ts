import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BoardApi } from "../lib/board/api";
import type { GameApi } from "../lib/game/api";
import { parseFen, START_FEN } from "../lib/board/notation";
import type { GameSnapshot } from "../lib/game/types";
import { useGameStore } from "./useGameStore";

function makeSnapshot(overrides: Partial<GameSnapshot> = {}): GameSnapshot {
  return {
    tree: {
      id: 0,
      mv: null,
      moveNumber: 0,
      isRed: true,
      comment: "",
      nags: [],
      isVariation: false,
      children: [],
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
    ...overrides,
  };
}

let gameApi: GameApi;
let boardApi: BoardApi;

beforeEach(() => {
  useGameStore.setState({
    api: null,
    boardApi: null,
    snapshot: null,
    position: null,
    validation: null,
    selected: null,
    legalTargets: [],
    editing: false,
    tool: null,
    view: { flipVertical: false, flipHorizontal: false },
    message: null,
    expandedVariations: [],
  });
  gameApi = {
    newGame: vi.fn(async () => makeSnapshot()),
    snapshot: vi.fn(async () => makeSnapshot()),
    insertMove: vi.fn(async () => makeSnapshot()),
    navigate: vi.fn(async () => makeSnapshot({ currentId: 5 })),
    previous: vi.fn(async () => makeSnapshot()),
    next: vi.fn(async () => makeSnapshot()),
    undo: vi.fn(async () => makeSnapshot()),
    redo: vi.fn(async () => makeSnapshot()),
    goToStart: vi.fn(async () => makeSnapshot()),
    goToEnd: vi.fn(async () => makeSnapshot()),
    deleteVariation: vi.fn(async () => makeSnapshot()),
    setComment: vi.fn(async () => makeSnapshot({ comment: "注释" })),
    setNag: vi.fn(async () => makeSnapshot({ nags: ["!"] })),
  };
  boardApi = {
    startPosition: vi.fn(async () => parseFen(START_FEN)),
    fromFen: vi.fn(async () => ({
      position: parseFen(START_FEN),
      validation: { ok: true, issues: [] },
    })),
    legalMoves: vi.fn(async () => ["h2e2"]),
    makeMove: vi.fn(async () => parseFen(START_FEN)),
    validate: vi.fn(async () => ({ ok: true, issues: [] })),
    rotate: vi.fn(async () => parseFen(START_FEN)),
    setPiece: vi.fn(async () => ({
      position: parseFen(START_FEN),
      validation: { ok: true, issues: [] },
    })),
    clearSquare: vi.fn(async () => ({
      position: parseFen(START_FEN),
      validation: { ok: true, issues: [] },
    })),
    setSide: vi.fn(async () => ({
      position: parseFen(START_FEN),
      validation: { ok: true, issues: [] },
    })),
    clearAll: vi.fn(async () => ({
      position: parseFen(START_FEN),
      validation: { ok: true, issues: [] },
    })),
  };
});

describe("useGameStore", () => {
  it("loads the initial snapshot", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    const state = useGameStore.getState();
    expect(state.snapshot).not.toBeNull();
    expect(state.position).toEqual(state.snapshot?.position);
  });

  it("navigates and updates the current position", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    await useGameStore.getState().navigate(5);
    expect(gameApi.navigate).toHaveBeenCalledWith(5);
    expect(useGameStore.getState().snapshot?.currentId).toBe(5);
  });

  it("undoes and redoes", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    await useGameStore.getState().undo();
    expect(gameApi.undo).toHaveBeenCalled();
    await useGameStore.getState().redo();
    expect(gameApi.redo).toHaveBeenCalled();
  });

  it("plays a move by selecting a piece then a target", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    // 选中红炮 h2（rank 2 file 7）
    await useGameStore.getState().handleSquareClick({ rank: 2, file: 7 });
    expect(boardApi.legalMoves).toHaveBeenCalled();
    expect(useGameStore.getState().legalTargets).toEqual([{ rank: 2, file: 4 }]);
    // 点击目标 e2 → 插入着法
    await useGameStore.getState().handleSquareClick({ rank: 2, file: 4 });
    expect(gameApi.insertMove).toHaveBeenCalledWith("h2e2");
  });

  it("deletes a variation", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    await useGameStore.getState().deleteVariation(3);
    expect(gameApi.deleteVariation).toHaveBeenCalledWith(3);
  });

  it("rebases the tree when leaving edit mode", async () => {
    await useGameStore.getState().init(gameApi, boardApi);
    await useGameStore.getState().toggleEditing();
    expect(useGameStore.getState().editing).toBe(true);
    await useGameStore.getState().toggleEditing();
    expect(gameApi.newGame).toHaveBeenCalled();
    expect(useGameStore.getState().editing).toBe(false);
  });
});
