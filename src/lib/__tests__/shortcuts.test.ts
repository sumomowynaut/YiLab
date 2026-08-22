import { describe, expect, it } from "vitest";
import { resolveShortcut, SHORTCUTS } from "../shortcuts";

function ev(key: string, opts: { ctrl?: boolean; meta?: boolean; shift?: boolean } = {}) {
  return {
    key,
    ctrlKey: opts.ctrl ?? false,
    metaKey: opts.meta ?? false,
    shiftKey: opts.shift ?? false,
  };
}

describe("resolveShortcut", () => {
  it("maps navigation keys", () => {
    expect(resolveShortcut(ev("ArrowLeft"))).toBe("previous");
    expect(resolveShortcut(ev("ArrowRight"))).toBe("next");
    expect(resolveShortcut(ev("Home"))).toBe("goToStart");
    expect(resolveShortcut(ev("End"))).toBe("goToEnd");
  });

  it("maps board view keys (case-insensitive)", () => {
    expect(resolveShortcut(ev("f"))).toBe("flip");
    expect(resolveShortcut(ev("F"))).toBe("flip");
    expect(resolveShortcut(ev("m"))).toBe("mirror");
  });

  it("maps analysis toggle to space", () => {
    expect(resolveShortcut(ev(" "))).toBe("toggleAnalysis");
  });

  it("maps undo/redo with modifiers", () => {
    expect(resolveShortcut(ev("z", { ctrl: true }))).toBe("undo");
    expect(resolveShortcut(ev("Z", { ctrl: true }))).toBe("undo");
    expect(resolveShortcut(ev("z", { ctrl: true, shift: true }))).toBe("redo");
    expect(resolveShortcut(ev("y", { ctrl: true }))).toBe("redo");
    expect(resolveShortcut(ev("y", { meta: true }))).toBe("redo"); // macOS
  });

  it("does not match plain keys for ctrl shortcuts", () => {
    expect(resolveShortcut(ev("z"))).toBeNull();
    expect(resolveShortcut(ev("y"))).toBeNull();
  });

  it("returns null for unknown keys", () => {
    expect(resolveShortcut(ev("Q"))).toBeNull();
    expect(resolveShortcut(ev("Escape"))).toBeNull();
  });

  it("exposes a non-empty configurable list", () => {
    expect(SHORTCUTS.length).toBeGreaterThanOrEqual(8);
    for (const def of SHORTCUTS) {
      expect(def.action).toBeTruthy();
      expect(def.label).toBeTruthy();
      expect(def.description).toBeTruthy();
    }
  });
});
