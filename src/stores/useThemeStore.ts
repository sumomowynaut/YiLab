// 主题（深浅色）状态：localStorage 持久化 + 应用到 documentElement.dark 类。

import { create } from "zustand";

export type Theme = "light" | "dark";

const STORAGE_KEY = "pikaxiangqi-theme";

function systemPrefersDark(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const mq = window.matchMedia?.("(prefers-color-scheme: dark)");
  return mq?.matches ?? false;
}

function applyTheme(theme: Theme): void {
  document.documentElement.classList.toggle("dark", theme === "dark");
}

interface ThemeState {
  theme: Theme;
  /** 启动时读取 localStorage（无记录时跟随系统偏好）。 */
  initTheme: () => void;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: "light",

  initTheme: () => {
    const stored = localStorage.getItem(STORAGE_KEY);
    const theme: Theme =
      stored === "dark" || stored === "light" ? stored : systemPrefersDark() ? "dark" : "light";
    applyTheme(theme);
    set({ theme });
  },

  setTheme: (theme) => {
    localStorage.setItem(STORAGE_KEY, theme);
    applyTheme(theme);
    set({ theme });
  },

  toggleTheme: () => get().setTheme(get().theme === "dark" ? "light" : "dark"),
}));
