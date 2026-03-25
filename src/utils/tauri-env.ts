/**
 * 检查是否在 Tauri 环境中运行
 * Tauri v2 使用 __TAURI_INTERNALS__ 而不是 __TAURI__
 * @returns boolean - 如果在 Tauri 环境中返回 true，否则返回 false
 */
export const isTauri = (): boolean => {
  // Tauri v2 检测方式
  const hasTauriInternals =
    typeof window !== "undefined" &&
    (window as any).__TAURI_INTERNALS__ !== undefined;

  // Tauri v1 兼容检测
  const hasTauriV1 =
    typeof window !== "undefined" && window.__TAURI__ !== undefined;

  const result = hasTauriInternals || hasTauriV1;

  return result;
};

/**
 * 安全地调用 Tauri invoke 函数
 * @param cmd - Tauri 命令名称
 * @param args - 命令参数
 * @returns 如果不在 Tauri 环境中返回 null，否则返回 invoke 结果
 */
export const safeInvoke = async <T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | null> => {
  if (!isTauri()) {
    return null;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return await invoke<T>(cmd, args);
};
