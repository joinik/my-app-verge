import { resolve } from "node:path";
import { fileURLToPath, URL } from "node:url";

import react from "@vitejs/plugin-react";
import svgr from "vite-plugin-svgr";
import { defineConfig } from "vite";
// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  root: "src",  // 指定 src 为项目根目录
  build: {
    outDir: "../dist",  // 输出到项目根目录下的 dist
    emptyOutDir: true,
  },
  plugins: [react(), svgr()],

  clearScreen: false,
  server: {
    port: 3000,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@root": fileURLToPath(new URL(".", import.meta.url)),
    },
  },
  define: {
    OS_PLATFORM: `"${process.platform}"`,
  },
}));