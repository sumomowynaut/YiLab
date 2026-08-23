import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { SettingsPanel } from "./SettingsPanel";
import { useThemeStore } from "../../stores/useThemeStore";
import { useEngineStore } from "../../stores/useEngineStore";

beforeEach(() => {
  document.documentElement.classList.remove("dark");
  useThemeStore.setState({ theme: "light" });
  useEngineStore.setState({
    settings: { programPath: "", threads: 1, hash: 16, depth: null, multipv: 1 },
  });
});

describe("SettingsPanel", () => {
  it("toggles the theme", () => {
    render(<SettingsPanel />);
    fireEvent.click(screen.getByTestId("settings-theme"));
    expect(useThemeStore.getState().theme).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("updates engine settings through the store", () => {
    render(<SettingsPanel />);
    fireEvent.change(screen.getByTestId("settings-threads"), { target: { value: "4" } });
    expect(useEngineStore.getState().settings.threads).toBe(4);
    fireEvent.change(screen.getByTestId("settings-multipv"), { target: { value: "3" } });
    expect(useEngineStore.getState().settings.multipv).toBe(3);
  });

  it("lists keyboard shortcuts", () => {
    render(<SettingsPanel />);
    const list = screen.getByTestId("settings-shortcuts");
    expect(list.children.length).toBeGreaterThanOrEqual(8);
    expect(screen.getByText("悔棋")).toBeInTheDocument();
    expect(screen.getByText("Ctrl+Z")).toBeInTheDocument();
  });
});
