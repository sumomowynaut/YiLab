import { create } from "zustand";

interface AppState {
  counter: number;
  increment: () => void;
  reset: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  counter: 0,
  increment: () => set((state) => ({ counter: state.counter + 1 })),
  reset: () => set({ counter: 0 }),
}));
