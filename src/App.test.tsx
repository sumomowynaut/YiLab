import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { useGameStore } from "./stores/useGameStore";

beforeEach(() => {
  useGameStore.setState({
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
  });
});

describe("App", () => {
  it("renders the board with the start position and empty move tree", async () => {
    render(<App />);
    expect(await screen.findByText("PikaXiangqi")).toBeInTheDocument();
    expect(screen.getByText("帅")).toBeInTheDocument();
    expect(screen.getByText("红方")).toBeInTheDocument();
    expect(screen.getByTestId("move-tree-empty")).toBeInTheDocument();
  });

  it("toggles the position editor and shows the palette", async () => {
    render(<App />);
    await screen.findByText("帅");
    fireEvent.click(screen.getByRole("button", { name: "编辑局面" }));
    expect(screen.getByTestId("palette")).toBeInTheDocument();
    expect(screen.getByTestId("palette-red-king")).toBeInTheDocument();
    expect(screen.getByTestId("palette-eraser")).toBeInTheDocument();
  });

  it("toggles the dark theme via the header button", async () => {
    document.documentElement.classList.remove("dark");
    render(<App />);
    await screen.findByText("帅");
    const button = screen.getByTestId("theme-toggle");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
    fireEvent.click(button);
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("switches between feature tabs", async () => {
    render(<App />);
    await screen.findByText("帅");
    // 默认「棋谱」标签显示棋谱树
    expect(screen.getByTestId("move-tree-empty")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tab-analysis"));
    expect(screen.getByTestId("analysis-panel")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tab-book"));
    expect(screen.getByTestId("book-panel")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tab-io"));
    expect(screen.getByTestId("game-codec")).toBeInTheDocument();
    expect(screen.getByTestId("ocr-panel")).toBeInTheDocument();
    expect(screen.getByTestId("gif-panel")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tab-settings"));
    expect(screen.getByTestId("settings-panel")).toBeInTheDocument();
    // 设置页显示快捷键说明
    expect(screen.getByTestId("settings-shortcuts")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tab-game"));
    expect(screen.getByTestId("move-tree-empty")).toBeInTheDocument();
  });
});
