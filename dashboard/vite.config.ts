/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuration Vite + Vitest du dashboard
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        // Recharts et ses dépendances (d3) pèsent l'essentiel du bundle : chunk séparé, mis en cache
        manualChunks: { charts: ["recharts"], vendor: ["react", "react-dom", "react-router-dom"] },
      },
    },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
