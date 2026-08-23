import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { OcrPanel } from "./OcrPanel";
import type { OcrApi } from "../../lib/ocr/api";
import type { OcrResultDto } from "../../lib/ocr/types";

function makeResult(overrides: Partial<OcrResultDto> = {}): OcrResultDto {
  return {
    cells: [],
    orientation: "normal",
    sideToMove: null,
    fen: "9/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
    confidence: 0.95,
    valid: false,
    issues: ["无法从静态截图判断行棋方，已按红方先行（可在局面编辑器中切换）"],
    ...overrides,
  };
}

function makeOcrApi(overrides: Partial<OcrApi> = {}): OcrApi {
  return {
    recognize: vi.fn(async () => makeResult()),
    ...overrides,
  };
}

async function pickFile(container: HTMLElement, bytes: Uint8Array) {
  const input = container.querySelector<HTMLInputElement>("input[type=file]")!;
  const file = new File([bytes], "board.png", { type: "image/png" });
  fireEvent.change(input, { target: { files: [file] } });
}

describe("OcrPanel", () => {
  beforeEach(() => {});

  it("recognizes a picked screenshot and shows confidence and FEN", async () => {
    const onLoaded = vi.fn();
    const ocrApi = makeOcrApi();
    const { container } = render(<OcrPanel ocrApi={ocrApi} onLoaded={onLoaded} />);

    await pickFile(container, new Uint8Array([1, 2, 3]));
    expect(await screen.findByTestId("ocr-result")).toBeInTheDocument();
    expect(ocrApi.recognize).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId("ocr-confidence")).toHaveTextContent("95%");
    expect(screen.getByTestId("ocr-fen")).toHaveTextContent("4K4");
    expect(onLoaded).not.toHaveBeenCalled();
  });

  it("loads the recognized FEN into the game via onLoaded", async () => {
    const onLoaded = vi.fn();
    const { container } = render(<OcrPanel ocrApi={makeOcrApi()} onLoaded={onLoaded} />);

    await pickFile(container, new Uint8Array([1]));
    fireEvent.click(await screen.findByTestId("ocr-load"));
    expect(onLoaded).toHaveBeenCalledWith("9/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
  });

  it("lists recognition issues and flags uncertain cells", async () => {
    const ocrApi = makeOcrApi({
      recognize: vi.fn(async () =>
        makeResult({
          confidence: 0.6,
          cells: [{ rank: 0, file: 0, piece: null, confidence: 0.5, uncertain: true }],
          issues: ["0 0 格识别不确定（置信度 50%）——已置空，请手动摆棋", "规则校验：黑方缺少将/帅"],
        }),
      ),
    });
    const { container } = render(<OcrPanel ocrApi={ocrApi} onLoaded={vi.fn()} />);

    await pickFile(container, new Uint8Array([1]));
    expect(await screen.findByTestId("ocr-result")).toBeInTheDocument();
    expect(screen.getByTestId("ocr-confidence")).toHaveTextContent("60%");
    expect(screen.getByTestId("ocr-issues").children.length).toBe(2);
    expect(screen.getByText(/有 1 格识别不确定/)).toBeInTheDocument();
  });

  it("shows an error when recognition fails", async () => {
    const ocrApi = makeOcrApi({
      recognize: vi.fn(async () => {
        throw new Error("无法定位棋盘");
      }),
    });
    const { container } = render(<OcrPanel ocrApi={ocrApi} onLoaded={vi.fn()} />);

    await pickFile(container, new Uint8Array([1]));
    expect(await screen.findByTestId("ocr-error")).toHaveTextContent("无法定位棋盘");
  });
});
