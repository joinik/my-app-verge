/// <reference types="vite/client" />
/// <reference types="vite-plugin-svgr/client" />
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { ComposeContextProvider } from 'foxact/compose-context-provider'
import { RouterProvider } from 'react-router'
import { router } from './pages/_routers'

const contexts: React.ReactElement<unknown, string | React.JSXElementConstructor<any>>[] = [
  // <LoadingCacheProvider key="loading" />, // 加载状态提供者
  // <UpdateStateProvider key="update" />, // 更新状态提供者
]

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ComposeContextProvider contexts={contexts}>
      <RouterProvider router={router} />
      {/* <App /> */}
    </ComposeContextProvider>
  </React.StrictMode>
)
