import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      "/api": {
        target: process.env.VITE_API_BASE_URL || "http://127.0.0.1:3000",
        changeOrigin: true,
      },
    },
  },
  test: {
    include: ["tests/**/*.{test,spec}.{js,ts}"],
    globals: true,
    environment: "node",
  },
});
