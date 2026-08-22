import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GameCodec } from "./GameCodec";
import type { IoApi } from "../../lib/io/api";
import type { GameSnapshot } from "../../lib/game/types";
import { parseFen, START_FEN } from "../../lib/board/notation";

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

function makeIoApi(overrides: Partial<IoApi> = {}): IoApi {
  return {
    importText: vi.fn(async () => makeSnapshot()),
    exportText: vi.fn(async () => '[Event "test"]\n\n1. h2e2 *'),
    ...overrides,
  };
}

describe("GameCodec", () => {
  beforeEach(() => {
    // 允许 jsdom 中模拟文件下载
    if (!URL.createObjectURL) {
      URL.createObjectURL = vi.fn(() => "blob:mock") as unknown as typeof URL.createObjectURL;
    }
    URL.revokeObjectURL = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
  });

  it("imports pasted text with auto format and adopts the snapshot", async () => {
    const onImported = vi.fn();
    const ioApi = makeIoApi();
    render(<GameCodec ioApi={ioApi} onImported={onImported} />);

    fireEvent.change(screen.getByTestId("codec-import"), {
      target: { value: "1. h2e2 h7e7" },
    });
    fireEvent.click(screen.getByTestId("codec-import-button"));

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("导入成功");
    expect(ioApi.importText).toHaveBeenCalledWith("", "1. h2e2 h7e7");
    expect(onImported).toHaveBeenCalledTimes(1);
  });

  it("imports with explicit PGN format", async () => {
    const ioApi = makeIoApi();
    render(<GameCodec ioApi={ioApi} onImported={vi.fn()} />);

    fireEvent.change(screen.getByTestId("codec-import-format"), { target: { value: "pgn" } });
    fireEvent.change(screen.getByTestId("codec-import"), {
      target: { value: '[Event "x"]\n1. h2e2' },
    });
    fireEvent.click(screen.getByTestId("codec-import-button"));

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("导入成功");
    expect(ioApi.importText).toHaveBeenCalledWith("pgn", '[Event "x"]\n1. h2e2');
  });

  it("shows an error message when import fails", async () => {
    const ioApi = makeIoApi({
      importText: vi.fn(async () => {
        throw new Error("坏 FEN");
      }),
    });
    render(<GameCodec ioApi={ioApi} onImported={vi.fn()} />);

    fireEvent.change(screen.getByTestId("codec-import"), { target: { value: "garbage" } });
    fireEvent.click(screen.getByTestId("codec-import-button"));

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("导入失败");
  });

  it("exports and copies to clipboard", async () => {
    const ioApi = makeIoApi();
    render(<GameCodec ioApi={ioApi} onImported={vi.fn()} />);

    fireEvent.click(screen.getByTestId("codec-copy"));

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("已复制");
    expect(ioApi.exportText).toHaveBeenCalledWith("pgn");
  });

  it("exports and downloads a file", async () => {
    const ioApi = makeIoApi();
    render(<GameCodec ioApi={ioApi} onImported={vi.fn()} />);

    fireEvent.click(screen.getByTestId("codec-download"));

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("已导出");
    expect(ioApi.exportText).toHaveBeenCalledWith("pgn");
    expect(HTMLAnchorElement.prototype.click).toHaveBeenCalled();
  });

  it("imports a local file with content sniffing", async () => {
    const ioApi = makeIoApi();
    render(<GameCodec ioApi={ioApi} onImported={vi.fn()} />);

    const file = new File(["1. h2e2 h7e7"], "game.pgn", { type: "text/plain" });
    fireEvent.change(screen.getByTestId("codec-file"), { target: { files: [file] } });

    expect(await screen.findByTestId("codec-message")).toHaveTextContent("导入成功");
    expect(ioApi.importText).toHaveBeenCalledWith("", "1. h2e2 h7e7");
  });
});
