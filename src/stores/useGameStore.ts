import { create } from "zustand";
import type { BoardApi } from "../lib/board/api";
import { moveFromUci, sideToColor, squareToUci } from "../lib/board/notation";
import type {
  BoardView,
  PositionSnapshot,
  Square,
  Tool,
  ValidationResult,
} from "../lib/board/types";
import type { GameApi } from "../lib/game/api";
import type { GameSnapshot } from "../lib/game/types";

interface GameState {
  api: GameApi | null;
  boardApi: BoardApi | null;
  snapshot: GameSnapshot | null;
  /** 编辑模式下的独立摆棋缓冲（M1）；非编辑状态以 snapshot.position 为准。 */
  editPosition: PositionSnapshot | null;
  validation: ValidationResult | null;
  selected: Square | null;
  legalTargets: Square[];
  editing: boolean;
  tool: Tool | null;
  view: BoardView;
  message: string | null;
  expandedVariations: number[];

  init: (api: GameApi, boardApi: BoardApi) => Promise<void>;
  handleSquareClick: (sq: Square) => Promise<void>;
  toggleEditing: () => Promise<void>;
  setTool: (tool: Tool) => void;
  clearAll: () => Promise<void>;
  toggleSide: () => Promise<void>;
  rotateView: () => void;
  mirrorView: () => void;
  loadFen: (fen: string) => Promise<void>;
  navigate: (nodeId: number) => Promise<void>;
  previous: () => Promise<void>;
  next: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  goToStart: () => Promise<void>;
  goToEnd: () => Promise<void>;
  deleteVariation: (nodeId: number) => Promise<void>;
  promoteVariation: (nodeId: number) => Promise<void>;
  reorderVariation: (parentId: number, from: number, to: number) => Promise<void>;
  setComment: (nodeId: number, comment: string) => Promise<void>;
  setNag: (nodeId: number, nag: string, add: boolean) => Promise<void>;
  toggleVariation: (nodeId: number) => void;
}

/** 当前展示的局面：编辑模式用 editPosition，否则由 snapshot.position 派生（M1）。 */
export function selectDisplayPosition(state: GameState): PositionSnapshot | null {
  return state.editing ? state.editPosition : (state.snapshot?.position ?? null);
}

export const useGameStore = create<GameState>((set, get) => {
  function applySnapshot(snapshot: GameSnapshot) {
    set({
      snapshot,
      selected: null,
      legalTargets: [],
      message: null,
    });
  }

  return {
    api: null,
    boardApi: null,
    snapshot: null,
    editPosition: null,
    validation: null,
    selected: null,
    legalTargets: [],
    editing: false,
    tool: null,
    view: { flipVertical: false, flipHorizontal: false },
    message: null,
    expandedVariations: [],

    async init(api, boardApi) {
      set({ api, boardApi, message: null });
      try {
        const snapshot = await api.snapshot();
        applySnapshot(snapshot);
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async handleSquareClick(sq) {
      const { api, boardApi, editing, selected, legalTargets, tool } = get();
      const position = selectDisplayPosition(get());
      if (!api || !boardApi || !position) {
        return;
      }
      if (editing) {
        try {
          const result =
            tool === "eraser"
              ? await boardApi.clearSquare(position.fen, squareToUci(sq))
              : tool
                ? await boardApi.setPiece(position.fen, squareToUci(sq), tool.color, tool.kind)
                : null;
          if (result) {
            set({ editPosition: result.position, validation: result.validation, message: null });
          }
        } catch (error) {
          set({ message: String(error) });
        }
        return;
      }

      const ownColor = sideToColor(position.sideToMove);
      const piece = position.board[sq.rank][sq.file];

      if (selected && legalTargets.some((t) => t.rank === sq.rank && t.file === sq.file)) {
        try {
          const snapshot = await api.insertMove(squareToUci(selected) + squareToUci(sq));
          applySnapshot(snapshot);
        } catch (error) {
          set({ message: String(error) });
        }
        return;
      }

      if (piece && piece.color === ownColor) {
        set({ selected: sq, legalTargets: [], message: null });
        try {
          const moves = await boardApi.legalMoves(position.fen);
          const targets = moves
            .map((mv) => moveFromUci(mv))
            .filter(
              (mv): mv is NonNullable<typeof mv> =>
                mv !== null && mv.from.rank === sq.rank && mv.from.file === sq.file,
            )
            .map((mv) => mv.to);
          set({ legalTargets: targets });
        } catch (error) {
          set({ message: String(error) });
        }
        return;
      }

      set({ selected: null, legalTargets: [], message: null });
    },

    async toggleEditing() {
      const { api, editing, snapshot } = get();
      if (editing && api && snapshot) {
        // 退出编辑：以编辑后的局面作为新棋谱树根
        const editPosition = get().editPosition ?? snapshot.position;
        try {
          const next = await api.newGame(editPosition.fen);
          set({
            snapshot: next,
            editPosition: null,
            editing: false,
            selected: null,
            legalTargets: [],
            validation: null,
            message: null,
          });
        } catch (error) {
          set({ message: String(error) });
        }
        return;
      }
      if (snapshot) {
        set({
          editing: true,
          editPosition: snapshot.position,
          selected: null,
          legalTargets: [],
          validation: null,
        });
      }
    },

    setTool: (tool) => set({ tool }),

    async clearAll() {
      const { boardApi, editing } = get();
      if (!boardApi || !editing) return;
      try {
        const result = await boardApi.clearAll();
        set({ editPosition: result.position, validation: result.validation, message: null });
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async toggleSide() {
      const { boardApi, editing } = get();
      const position = selectDisplayPosition(get());
      if (!boardApi || !editing || !position) return;
      try {
        const result = await boardApi.setSide(
          position.fen,
          position.sideToMove === "w" ? "b" : "w",
        );
        set({ editPosition: result.position, validation: result.validation, message: null });
      } catch (error) {
        set({ message: String(error) });
      }
    },

    rotateView: () =>
      set((state) => ({ view: { ...state.view, flipVertical: !state.view.flipVertical } })),

    mirrorView: () =>
      set((state) => ({ view: { ...state.view, flipHorizontal: !state.view.flipHorizontal } })),

    async loadFen(fen) {
      const { api } = get();
      if (!api) return;
      try {
        const snapshot = await api.newGame(fen);
        applySnapshot(snapshot);
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async navigate(nodeId) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.navigate(nodeId));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async previous() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.previous());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async next() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.next());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async undo() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.undo());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async redo() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.redo());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async goToStart() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.goToStart());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async goToEnd() {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.goToEnd());
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async deleteVariation(nodeId) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.deleteVariation(nodeId));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async promoteVariation(nodeId) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.promoteVariation(nodeId));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async reorderVariation(parentId, from, to) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.reorderVariation(parentId, from, to));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async setComment(nodeId, comment) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.setComment(nodeId, comment));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async setNag(nodeId, nag, add) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.setNag(nodeId, nag, add));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    toggleVariation: (nodeId) =>
      set((state) => ({
        expandedVariations: state.expandedVariations.includes(nodeId)
          ? state.expandedVariations.filter((id) => id !== nodeId)
          : [...state.expandedVariations, nodeId],
      })),
  };
});
