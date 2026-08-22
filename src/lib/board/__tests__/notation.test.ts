import { describe, expect, it } from "vitest";
import {
  moveFromUci,
  moveToUci,
  parseFen,
  pieceGlyph,
  screenFileOf,
  screenRankOf,
  squareFromUci,
  squareToUci,
  START_FEN,
} from "../notation";
import type { Piece } from "../types";

describe("square notation", () => {
  it("round-trips all 90 squares", () => {
    for (let rank = 0; rank < 10; rank++) {
      for (let file = 0; file < 9; file++) {
        const sq = { rank, file };
        expect(squareFromUci(squareToUci(sq))).toEqual(sq);
      }
    }
  });

  it("rejects invalid squares", () => {
    expect(squareFromUci("j0")).toBeNull();
    expect(squareFromUci("a10")).toBeNull();
    expect(squareFromUci("aa")).toBeNull();
    expect(squareFromUci("")).toBeNull();
  });

  it("round-trips moves", () => {
    expect(moveToUci({ from: { rank: 2, file: 7 }, to: { rank: 2, file: 4 } })).toBe("h2e2");
    expect(moveFromUci("h2e2")).toEqual({
      from: { rank: 2, file: 7 },
      to: { rank: 2, file: 4 },
    });
    expect(moveFromUci("h2")).toBeNull();
  });
});

describe("FEN parsing (browser fallback)", () => {
  it("parses the start position", () => {
    const snapshot = parseFen(START_FEN);
    expect(snapshot.board).toHaveLength(10);
    expect(snapshot.board[0]).toHaveLength(9);
    expect(snapshot.sideToMove).toBe("w");
    const pieceCount = snapshot.board.flat().filter(Boolean).length;
    expect(pieceCount).toBe(32);
    // 红帅在 e0（rank 0 file 4）
    const king: Piece | null = snapshot.board[0][4];
    expect(king).toEqual({ color: "red", kind: "king" });
  });

  it("rejects malformed FEN", () => {
    expect(() => parseFen("9/9/9/9/9/9/9/9/9 w - - 0 1")).toThrow();
    expect(() => parseFen("9/9/9/9/9/9/9/9/9/x w - - 0 1")).toThrow();
    expect(() => parseFen("9/9/9/9/9/9/9/9/9/9 z - - 0 1")).toThrow();
  });
});

describe("piece glyphs and view transforms", () => {
  it("maps pieces to Chinese glyphs", () => {
    expect(pieceGlyph({ color: "red", kind: "king" })).toBe("帅");
    expect(pieceGlyph({ color: "black", kind: "king" })).toBe("将");
    expect(pieceGlyph({ color: "red", kind: "pawn" })).toBe("兵");
    expect(pieceGlyph({ color: "black", kind: "pawn" })).toBe("卒");
  });

  it("flips vertical and horizontal views", () => {
    expect(screenRankOf(0, { flipVertical: false, flipHorizontal: false })).toBe(0);
    expect(screenRankOf(0, { flipVertical: true, flipHorizontal: false })).toBe(9);
    expect(screenFileOf(0, { flipVertical: false, flipHorizontal: true })).toBe(8);
  });
});
