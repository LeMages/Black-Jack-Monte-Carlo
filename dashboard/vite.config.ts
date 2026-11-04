/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuration Vite + Vitest du dashboard
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
