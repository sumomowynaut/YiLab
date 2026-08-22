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
  position: PositionSnapshot | null;
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
  setComment: (comment: string) => Promise<void>;
  setNag: (nag: string, add: boolean) => Promise<void>;
  toggleVariation: (nodeId: number) => void;
}

export const useGameStore = create<GameState>((set, get) => {
  function applySnapshot(snapshot: GameSnapshot) {
    set({
      snapshot,
      position: snapshot.position,
      selected: null,
      legalTargets: [],
      message: null,
    });
  }

  return {
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
      const { api, boardApi, position, selected, legalTargets, editing, tool } = get();
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
            set({ position: result.position, validation: result.validation, message: null });
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
      const { api, editing, position } = get();
      if (editing && api && position) {
        // 退出编辑：以编辑后的局面作为新棋谱树根
        try {
          const snapshot = await api.newGame(position.fen);
          set({
            snapshot,
            position: snapshot.position,
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
      set({ editing: !editing, selected: null, legalTargets: [], validation: null });
    },

    setTool: (tool) => set({ tool }),

    async clearAll() {
      const { boardApi, position } = get();
      if (!boardApi || !position) return;
      try {
        const result = await boardApi.clearAll();
        set({ position: result.position, validation: result.validation, message: null });
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async toggleSide() {
      const { boardApi, position } = get();
      if (!boardApi || !position) return;
      try {
        const result = await boardApi.setSide(
          position.fen,
          position.sideToMove === "w" ? "b" : "w",
        );
        set({ position: result.position, validation: result.validation, message: null });
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

    async setComment(comment) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.setComment(comment));
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async setNag(nag, add) {
      const { api } = get();
      if (!api) return;
      try {
        applySnapshot(await api.setNag(nag, add));
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
