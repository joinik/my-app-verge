/// <reference types="vite/client" />
/// <reference types="vite-plugin-svgr/client" />
import React from "react";
import ReactDOM from "react-dom/client";
import { ComposeContextProvider } from "foxact/compose-context-provider";
import { RouterProvider } from "react-router";
import { router } from "./pages/_routers";
import {
  LoadingCacheProvider,
  ThemeModeProvider,
  UpdateStateProvider,
} from "./services/states";
import { WindowProvider } from "@/providers/window";

const AppContent = () => <RouterProvider router={router} />;

const contexts: React.ReactElement<
  unknown,
  string | React.JSXElementConstructor<any>
>[] = [
  <ThemeModeProvider key="theme" initialState="light" />,
  <LoadingCacheProvider key="loading" />,
  <UpdateStateProvider key="update" />,
  <WindowProvider key="window">
    <AppContent />
  </WindowProvider>,
];

// 等待 Tauri 环境初始化（如果是 Tauri 应用）
const initializeApp = async () => {
  // 如果在 Tauri 环境中，等待 Tauri 对象注入
  if (typeof window !== "undefined") {
    let retries = 0;
    const maxRetries = 50; // 最多等待 5 秒（50 * 100ms）

    // Tauri v2 使用 __TAURI_INTERNALS__, v1 使用 __TAURI__
    const hasTauri = () =>
      (window as any).__TAURI_INTERNALS__ !== undefined ||
      (window as any).__TAURI__ !== undefined;

    while (!hasTauri() && retries < maxRetries) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      retries++;
    }
  }

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ComposeContextProvider contexts={contexts}>
        <AppContent />
      </ComposeContextProvider>
    </React.StrictMode>,
  );
};

initializeApp();
