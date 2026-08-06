import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 约定：dev 时监听 1420 端口，且对 IP6 :: 严格，故显式 host
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "0.0.0.0",
    hmr: { protocol: "ws", host: "localhost", port: 1421 },
    watch: { ignored: ["**/src-tauri/**", "**/sidecar/**"] },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
});
