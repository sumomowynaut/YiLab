import { PIECE_GLYPHS } from "../../lib/board/notation";
import type { Color, PieceKind, Tool } from "../../lib/board/types";

const ORDER: PieceKind[] = ["king", "advisor", "elephant", "horse", "rook", "cannon", "pawn"];

export interface PiecePaletteProps {
  tool: Tool | null;
  onSelect: (tool: Tool) => void;
}

/** 局面编辑器棋子选择面板。 */
export function PiecePalette({ tool, onSelect }: PiecePaletteProps) {
  const base =
    "flex h-9 w-9 items-center justify-center rounded border text-sm font-bold transition-colors";
  return (
    <div className="flex flex-wrap gap-1" data-testid="palette">
      {(["red", "black"] as Color[]).map((color) =>
        ORDER.map((kind) => {
          const key = `${color}-${kind}`;
          const selected =
            tool !== null && tool !== "eraser" && tool.color === color && tool.kind === kind;
          return (
            <button
              key={key}
              type="button"
              data-testid={`palette-${key}`}
              onClick={() => onSelect({ color, kind })}
              className={`${base} ${
                selected
                  ? "border-primary bg-primary text-primary-foreground"
                  : color === "red"
                    ? "border-input bg-background text-red-600"
                    : "border-input bg-background text-gray-800"
              }`}
            >
              {PIECE_GLYPHS[color][kind]}
            </button>
          );
        }),
      )}
      <button
        type="button"
        data-testid="palette-eraser"
        onClick={() => onSelect("eraser")}
        className={`${base} ${
          tool === "eraser"
            ? "border-primary bg-primary text-primary-foreground"
            : "border-input bg-background text-muted-foreground"
        }`}
        aria-label="橡皮"
      >
        ✕
      </button>
    </div>
  );
}
