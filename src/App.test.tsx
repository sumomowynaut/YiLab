import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { useGameStore } from "./stores/useGameStore";

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
});
