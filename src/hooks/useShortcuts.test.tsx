import { fireEvent, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useShortcuts } from "./useShortcuts";
import type { ShortcutAction } from "../lib/shortcuts";

describe("useShortcuts", () => {
  it("dispatches matched actions and ignores unknown keys", () => {
    const actions: ShortcutAction[] = [];
    renderHook(() => useShortcuts({ onAction: (a) => actions.push(a) }));

    fireEvent.keyDown(window, { key: "ArrowRight" });
    fireEvent.keyDown(window, { key: "f" });
    fireEvent.keyDown(window, { key: " ", ctrlKey: true });
    fireEvent.keyDown(window, { key: "Q" });

    expect(actions).toEqual(["next", "flip"]);
  });

  it("respects shouldIgnore", () => {
    const onAction = vi.fn();
    renderHook(() =>
      useShortcuts({
        onAction,
        shouldIgnore: (event) => event.key === "ArrowRight",
      }),
    );

    fireEvent.keyDown(window, { key: "ArrowRight" });
    fireEvent.keyDown(window, { key: "ArrowLeft" });

    expect(onAction).toHaveBeenCalledTimes(1);
    expect(onAction).toHaveBeenCalledWith("previous");
  });

  it("unsubscribes on unmount", () => {
    const onAction = vi.fn();
    const { unmount } = renderHook(() => useShortcuts({ onAction }));
    unmount();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(onAction).not.toHaveBeenCalled();
  });
});
