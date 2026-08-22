// Tauri IPC 的类型化封装。

/** 是否运行在 Tauri 环境中。 */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 调用一个 Tauri 命令。 */
export async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}
/** 订阅一个 Tauri 事件，返回取消函数。 */
export async function listenEvent<T>(event: string, cb: (payload: T) => void): Promise<() => void> {
  const { listen } = await import("@tauri-apps/api/event");
  return listen<T>(event, (e) => cb(e.payload));
}
