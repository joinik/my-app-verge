import React, { useState, useEffect, useCallback, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WindowContext, type WindowContextType } from "./window-context";
import { isTauri } from "@/utils/tauri-env";

export const WindowProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  // 仅在 Tauri 环境中创建 window 实例
  const currentWindow = useMemo(() => {
    if (!isTauri()) return null;
    return getCurrentWindow();
  }, []);

  const [maximized, setMaximized] = useState<boolean | null>(null);
  const [decorated, setDecorated] = useState<boolean | null>(null);

  const checkMaximized = useCallback(async () => {
    if (!currentWindow) return;
    const isMaximized = await currentWindow.isMaximized();
    setMaximized(isMaximized);
  }, [currentWindow]);

  const minimize = useCallback(async () => {
    if (!currentWindow) return;
    // Delay one frame so the UI can clear :hover before the window hides.
    await new Promise((resolve) => setTimeout(resolve, 20));
    await currentWindow.minimize();
  }, [currentWindow]);

  const close = useCallback(async () => {
    if (!currentWindow) return;
    // Delay one frame so the UI can clear :hover before the window hides.
    await new Promise((resolve) => setTimeout(resolve, 20));
    await currentWindow.close();
  }, [currentWindow]);

  const toggleMaximize = useCallback(async () => {
    if (!currentWindow) return;
    if (await currentWindow.isMaximized()) {
      await currentWindow.unmaximize();
      setMaximized(false);
    } else {
      await currentWindow.maximize();
      setMaximized(true);
    }
  }, [currentWindow]);

  const toggleFullscreen = useCallback(async () => {
    if (!currentWindow) return;
    await currentWindow.setFullscreen(!(await currentWindow.isFullscreen()));
  }, [currentWindow]);

  const refreshDecorated = useCallback(async () => {
    if (!currentWindow) return null;
    const val = await currentWindow.isDecorated();
    setDecorated(val);
    return val;
  }, [currentWindow]);

  const toggleDecorations = useCallback(async () => {
    if (!currentWindow) return;
    const currentVal = await currentWindow.isDecorated();
    await currentWindow.setDecorations(!currentVal);
    setDecorated(!currentVal);
  }, [currentWindow]);

  useEffect(() => {
    // 仅在 Tauri 环境中执行
    if (!currentWindow) return;

    // Initial checks on mount
    checkMaximized();
    refreshDecorated();
    currentWindow.setMinimizable?.(true);

    // Setup listener for window resize to update maximized state
    const unlistenPromise = currentWindow.onResized(() => {
      checkMaximized();
    });

    return () => {
      unlistenPromise
        .then((unlisten: () => void) => unlisten())
        .catch((err: unknown) =>
          console.warn("[WindowProvider] 清理监听器失败:", err),
        );
    };
  }, [currentWindow, checkMaximized, refreshDecorated]);

  const contextValue: WindowContextType = {
    decorated,
    maximized,
    toggleDecorations,
    refreshDecorated,
    minimize,
    close,
    toggleMaximize,
    toggleFullscreen,
    currentWindow,
  };

  return (
    <WindowContext.Provider value={contextValue}>
      {children}
    </WindowContext.Provider>
  );
};
