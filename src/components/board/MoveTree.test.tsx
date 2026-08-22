import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MoveTree } from "./MoveTree";
import type { TreeNodeDto } from "../../lib/game/types";

function makeTree(): TreeNodeDto {
  return {
    id: 0,
    mv: null,
    moveNumber: 0,
    isRed: true,
    comment: "",
    nags: [],
    isVariation: false,
    children: [
      {
        id: 1,
        mv: "h2e2",
        moveNumber: 1,
        isRed: true,
        comment: "中炮开局",
        nags: ["!"],
        isVariation: false,
        children: [
          {
            id: 2,
            mv: "h7e7",
            moveNumber: 1,
            isRed: false,
            comment: "",
            nags: [],
            isVariation: false,
            children: [],
          },
        ],
      },
      {
        id: 3,
        mv: "b0c2",
        moveNumber: 1,
        isRed: true,
        comment: "",
        nags: [],
        isVariation: true,
        children: [
          {
            id: 4,
            mv: "b9c7",
            moveNumber: 1,
            isRed: false,
            comment: "",
            nags: [],
            isVariation: false,
            children: [],
          },
        ],
      },
      {
        id: 5,
        mv: "a0a1",
        moveNumber: 1,
        isRed: true,
        comment: "",
        nags: [],
        isVariation: true,
        children: [],
      },
    ],
  };
}

function renderTree(overrides: Partial<Parameters<typeof MoveTree>[0]> = {}) {
  const props = {
    tree: makeTree(),
    currentId: 2,
    expanded: [],
    onNavigate: vi.fn(),
    onToggleVariation: vi.fn(),
    onDeleteVariation: vi.fn(),
    onPromoteVariation: vi.fn(),
    onReorderVariation: vi.fn(),
    ...overrides,
  };
  render(
    <MoveTree
      tree={props.tree}
      currentId={props.currentId}
      expanded={props.expanded}
      onNavigate={props.onNavigate}
      onToggleVariation={props.onToggleVariation}
      onDeleteVariation={props.onDeleteVariation}
      onPromoteVariation={props.onPromoteVariation}
      onReorderVariation={props.onReorderVariation}
    />,
  );
  return props;
}

describe("MoveTree", () => {
  it("renders the main line and variation toggle", () => {
    renderTree();
    expect(screen.getByTestId("move-1")).toHaveTextContent("1.");
    expect(screen.getByTestId("move-1")).toHaveTextContent("h2e2");
    expect(screen.getByTestId("move-2")).toHaveTextContent("1…");
    expect(screen.getByTestId("variation-toggle-0")).toHaveTextContent("变例 2");
  });

  it("highlights the current move", () => {
    renderTree({ currentId: 2 });
    expect(screen.getByTestId("move-2").className).toContain("bg-primary");
    expect(screen.getByTestId("move-1").className).not.toContain("bg-primary");
  });

  it("shows NAG and comment markers", () => {
    renderTree();
    const chip = screen.getByTestId("move-1");
    expect(chip).toHaveTextContent("!");
    expect(chip).toHaveTextContent("*");
  });

  it("navigates on move click", () => {
    const props = renderTree();
    fireEvent.click(screen.getByTestId("move-1"));
    expect(props.onNavigate).toHaveBeenCalledWith(1);
  });

  it("expands variations and deletes them", () => {
    const props = renderTree();
    fireEvent.click(screen.getByTestId("variation-toggle-0"));
    expect(props.onToggleVariation).toHaveBeenCalledWith(0);
  });

  it("renders expanded variation with delete/promote/reorder buttons", () => {
    const props = renderTree({ expanded: [0] });
    expect(screen.getByTestId("variation-3")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("delete-variation-3"));
    expect(props.onDeleteVariation).toHaveBeenCalledWith(3);
    fireEvent.click(screen.getByTestId("promote-variation-3"));
    expect(props.onPromoteVariation).toHaveBeenCalledWith(3);
  });

  it("reorders variations up and down", () => {
    const props = renderTree({ expanded: [0] });
    // 变例 3 是第一条变例（index 0）：上移禁用，下移 → (0, 1, 2)
    expect(screen.getByTestId("reorder-variation-3-up")).toBeDisabled();
    fireEvent.click(screen.getByTestId("reorder-variation-3-down"));
    expect(props.onReorderVariation).toHaveBeenCalledWith(0, 1, 2);
    // 变例 5 是第二条变例（index 1）：上移 → (0, 2, 1)，下移禁用
    fireEvent.click(screen.getByTestId("reorder-variation-5-up"));
    expect(props.onReorderVariation).toHaveBeenCalledWith(0, 2, 1);
    expect(screen.getByTestId("reorder-variation-5-down")).toBeDisabled();
  });

  it("shows empty state when no moves", () => {
    renderTree({
      tree: {
        id: 0,
        mv: null,
        moveNumber: 0,
        isRed: true,
        comment: "",
        nags: [],
        isVariation: false,
        children: [],
      },
    });
    expect(screen.getByTestId("move-tree-empty")).toBeInTheDocument();
  });
});
