import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@bao/engine": resolve(__dirname, "../bindings/wasm/pkg/bao_engine_wasm.js"),
    },
  },
  optimizeDeps: {
    exclude: ["@bao/engine"],
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
    fs: {
      allow: [resolve(__dirname, ".."), resolve(__dirname, "../bindings/wasm")],
    },
  },
});
