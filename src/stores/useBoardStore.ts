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

interface BoardState {
  api: BoardApi | null;
  position: PositionSnapshot | null;
  validation: ValidationResult | null;
  selected: Square | null;
  legalTargets: Square[];
  editing: boolean;
  tool: Tool | null;
  view: BoardView;
  message: string | null;
  loading: boolean;

  init: (api: BoardApi) => Promise<void>;
  handleSquareClick: (sq: Square) => Promise<void>;
  toggleEditing: () => void;
  setTool: (tool: Tool) => void;
  clearAll: () => Promise<void>;
  toggleSide: () => Promise<void>;
  rotateView: () => void;
  mirrorView: () => void;
  loadFen: (fen: string) => Promise<void>;
}

export const useBoardStore = create<BoardState>((set, get) => {
  async function applyPosition(
    api: BoardApi,
    position: PositionSnapshot,
    message: string | null = null,
  ) {
    const validation = await api.validate(position.fen);
    set({ position, validation, selected: null, legalTargets: [], message });
  }

  return {
    api: null,
    position: null,
    validation: null,
    selected: null,
    legalTargets: [],
    editing: false,
    tool: null,
    view: { flipVertical: false, flipHorizontal: false },
    message: null,
    loading: false,

    async init(api) {
      set({ api, loading: true, message: null });
      try {
        const position = await api.startPosition();
        await applyPosition(api, position);
      } catch (error) {
        set({ message: String(error) });
      } finally {
        set({ loading: false });
      }
    },

    async handleSquareClick(sq) {
      const { api, position, selected, legalTargets, editing, tool } = get();
      if (!api || !position) {
        return;
      }
      if (editing) {
        try {
          const result =
            tool === "eraser"
              ? await api.clearSquare(position.fen, squareToUci(sq))
              : tool
                ? await api.setPiece(position.fen, squareToUci(sq), tool.color, tool.kind)
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
          const next = await api.makeMove(position.fen, squareToUci(selected) + squareToUci(sq));
          await applyPosition(api, next);
        } catch (error) {
          set({ message: String(error) });
        }
        return;
      }

      if (piece && piece.color === ownColor) {
        set({ selected: sq, legalTargets: [], message: null });
        try {
          const moves = await api.legalMoves(position.fen);
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

    toggleEditing: () =>
      set((state) => ({ editing: !state.editing, selected: null, legalTargets: [] })),

    setTool: (tool) => set({ tool }),

    async clearAll() {
      const { api } = get();
      if (!api) return;
      try {
        const { position, validation } = await api.clearAll();
        set({ position, validation, selected: null, legalTargets: [], message: null });
      } catch (error) {
        set({ message: String(error) });
      }
    },

    async toggleSide() {
      const { api, position } = get();
      if (!api || !position) return;
      try {
        const { position: next, validation } = await api.setSide(
          position.fen,
          position.sideToMove === "w" ? "b" : "w",
        );
        set({ position: next, validation, selected: null, legalTargets: [], message: null });
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
        const { position, validation } = await api.fromFen(fen);
        set({ position, validation, selected: null, legalTargets: [], message: null });
      } catch (error) {
        set({ message: String(error) });
      }
    },
  };
});
