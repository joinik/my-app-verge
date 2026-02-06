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

const contexts: React.ReactElement<
  unknown,
  string | React.JSXElementConstructor<any>
>[] = [
  <ThemeModeProvider key="theme" initialState="light" />,
  <LoadingCacheProvider key="loading" />,
  <UpdateStateProvider key="update" />,
];

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ComposeContextProvider contexts={contexts}>
      <RouterProvider router={router} />
    </ComposeContextProvider>
  </React.StrictMode>,
);
