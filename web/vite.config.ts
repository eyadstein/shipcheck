import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// The dev server forwards API calls, so the browser sees a single origin and no
// CORS setup is needed while developing.
const api = "http://127.0.0.1:8787";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: { "/api": api, "/health": api },
  },
  test: {
    include: ["src/**/*.test.{ts,tsx}"],
    environment: "node",
  },
});
