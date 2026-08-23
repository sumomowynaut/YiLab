import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GifExportPanel } from "./GifExportPanel";
import type { GifApi } from "../../lib/gif/api";
import type { VariationOption } from "../../lib/gif/types";

function makeGifApi(overrides: Partial<GifApi> = {}): GifApi {
  return {
    exportCurrent: vi.fn(async () => new Uint8Array([71, 73, 70])),
    exportMainline: vi.fn(async () => new Uint8Array([71, 73, 70])),
    exportVariation: vi.fn(async () => new Uint8Array([71, 73, 70])),
    ...overrides,
  };
}

const variations: VariationOption[] = [
  { nodeId: 4, label: "2… b9c7" },
  { nodeId: 6, label: "3. b0c2" },
];

describe("GifExportPanel", () => {
  beforeEach(() => {
    if (!URL.createObjectURL) {
      URL.createObjectURL = vi.fn(() => "blob:mock") as unknown as typeof URL.createObjectURL;
    }
    URL.revokeObjectURL = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
  });

  it("exports the mainline with options", async () => {
    const api = makeGifApi();
    render(<GifExportPanel gifApi={api} variations={variations} />);
    fireEvent.change(screen.getByTestId("gif-delay"), { target: { value: "300" } });
    fireEvent.change(screen.getByTestId("gif-cell"), { target: { value: "64" } });
    fireEvent.click(screen.getByTestId("gif-coords"));
    fireEvent.click(screen.getByTestId("gif-moves"));
    fireEvent.click(screen.getByTestId("gif-export"));

    expect(await screen.findByTestId("gif-message")).toHaveTextContent("已导出");
    expect(api.exportMainline).toHaveBeenCalledWith({
      frameDelayMs: 300,
      cellSize: 64,
      showCoordinates: false,
      showMoves: false,
    });
  });

  it("exports the current position", async () => {
    const api = makeGifApi();
    render(<GifExportPanel gifApi={api} variations={variations} />);
    fireEvent.change(screen.getByTestId("gif-source"), { target: { value: "current" } });
    fireEvent.click(screen.getByTestId("gif-export"));

    expect(await screen.findByTestId("gif-message")).toHaveTextContent("已导出");
    expect(api.exportCurrent).toHaveBeenCalled();
    expect(api.exportMainline).not.toHaveBeenCalled();
  });

  it("exports a selected variation", async () => {
    const api = makeGifApi();
    render(<GifExportPanel gifApi={api} variations={variations} />);
    fireEvent.change(screen.getByTestId("gif-source"), { target: { value: "variation" } });
    fireEvent.change(screen.getByTestId("gif-variation"), { target: { value: "6" } });
    fireEvent.click(screen.getByTestId("gif-export"));

    expect(await screen.findByTestId("gif-message")).toHaveTextContent("已导出");
    expect(api.exportVariation).toHaveBeenCalledWith(6, expect.any(Object));
  });

  it("disables export when no variations are available", () => {
    const api = makeGifApi();
    render(<GifExportPanel gifApi={api} variations={[]} />);
    fireEvent.change(screen.getByTestId("gif-source"), { target: { value: "variation" } });
    expect(screen.getByTestId("gif-export")).toBeDisabled();
  });

  it("shows an error message on failure", async () => {
    const api = makeGifApi({
      exportMainline: vi.fn(async () => {
        throw new Error("渲染失败");
      }),
    });
    render(<GifExportPanel gifApi={api} variations={variations} />);
    fireEvent.click(screen.getByTestId("gif-export"));

    expect(await screen.findByTestId("gif-message")).toHaveTextContent("导出失败");
  });
});
