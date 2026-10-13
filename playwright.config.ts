import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1,
  use: { baseURL: "http://127.0.0.1:1421", headless: true, channel: "msedge" },
  webServer: {
    command: "npm run dev -- --port 1421",
    url: "http://127.0.0.1:1421",
    reuseExistingServer: true,
  },
  reporter: "list",
});
