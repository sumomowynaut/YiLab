import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { useBoardStore } from "./stores/useBoardStore";

beforeEach(() => {
  useBoardStore.setState({
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
  });
});

describe("App", () => {
  it("renders the board with the start position", async () => {
    render(<App />);
    expect(await screen.findByText("PikaXiangqi")).toBeInTheDocument();
    expect(screen.getByText("帅")).toBeInTheDocument();
    expect(screen.getByText("红方")).toBeInTheDocument();
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
