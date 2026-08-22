import { beforeEach, describe, expect, it } from "vitest";
import { useThemeStore } from "./useThemeStore";

beforeEach(() => {
  localStorage.clear();
  document.documentElement.classList.remove("dark");
  useThemeStore.setState({ theme: "light" });
});

describe("useThemeStore", () => {
  it("defaults to light when nothing stored and no dark preference", () => {
    useThemeStore.getState().initTheme();
    expect(useThemeStore.getState().theme).toBe("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("reads the stored theme on init", () => {
    localStorage.setItem("pikaxiangqi-theme", "dark");
    useThemeStore.getState().initTheme();
    expect(useThemeStore.getState().theme).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("setTheme persists to localStorage and applies the dark class", () => {
    useThemeStore.getState().setTheme("dark");
    expect(useThemeStore.getState().theme).toBe("dark");
    expect(localStorage.getItem("pikaxiangqi-theme")).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);

    useThemeStore.getState().setTheme("light");
    expect(localStorage.getItem("pikaxiangqi-theme")).toBe("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("toggleTheme switches between light and dark", () => {
    useThemeStore.getState().setTheme("dark");
    useThemeStore.getState().toggleTheme();
    expect(useThemeStore.getState().theme).toBe("light");

    useThemeStore.getState().toggleTheme();
    expect(useThemeStore.getState().theme).toBe("dark");
  });
});
