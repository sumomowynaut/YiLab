// 自动复盘状态：订阅分析事件 + 控制命令。

import { create } from "zustand";
import type { AnalysisApi } from "../lib/analysis/api";
import type { AnalysisStatusName, MoveAssessmentDto } from "../lib/analysis/types";

interface AnalysisState {
  api: AnalysisApi | null;
  status: AnalysisStatusName;
  progress: number;
  total: number;
  assessments: MoveAssessmentDto[];
  /** 订阅事件并拉取初始状态。 */
  init: (api: AnalysisApi) => void;
  start: (depth: number | null, movetimeMs: number | null) => Promise<void>;
  stop: () => Promise<void>;
  continue: () => Promise<void>;
  refresh: () => Promise<void>;
}

export const useAnalysisStore = create<AnalysisState>((set, get) => ({
  api: null,
  status: "idle",
  progress: 0,
  total: 0,
  assessments: [],

  init(api) {
    set({ api });
    api.subscribe((ev) => {
      switch (ev.type) {
        case "statusChanged":
          set({ status: ev.status });
          break;
        case "progress":
          set({ progress: ev.done, total: ev.total });
          break;
        case "assessment":
          if (ev.assessment) {
            set((s) => {
              const exists = s.assessments.some((a) => a.nodeId === ev.assessment.nodeId);
              return {
                assessments: exists
                  ? s.assessments.map((a) =>
                      a.nodeId === ev.assessment.nodeId ? ev.assessment : a,
                    )
                  : [...s.assessments, ev.assessment],
              };
            });
          }
          break;
        case "finished":
          set({
            assessments: Array.isArray(ev.assessments) ? ev.assessments : [],
            status: "done",
          });
          break;
      }
    });
    void get().refresh();
  },

  async start(depth, movetimeMs) {
    const { api } = get();
    if (!api) return;
    try {
      const dto = await api.start(depth, movetimeMs);
      set({
        status: dto.status,
        progress: dto.progress,
        total: dto.total,
        assessments: dto.assessments,
      });
    } catch (error) {
      console.error(error);
    }
  },

  async stop() {
    const { api } = get();
    if (!api) return;
    try {
      await api.stop();
      set({ status: "paused" });
    } catch (error) {
      console.error(error);
    }
  },

  async continue() {
    const { api } = get();
    if (!api) return;
    try {
      const dto = await api.continue();
      set({ status: dto.status, progress: dto.progress, total: dto.total });
    } catch (error) {
      console.error(error);
    }
  },

  async refresh() {
    const { api } = get();
    if (!api) return;
    try {
      const dto = await api.status();
      set({
        status: dto.status,
        progress: dto.progress,
        total: dto.total,
        assessments: dto.assessments,
      });
    } catch (error) {
      console.error(error);
    }
  },
}));
