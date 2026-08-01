import { defineConfig, devices } from "@playwright/test";

// The server is started externally (see readme / the CI e2e job) and its URL
// passed via E2E_BASE_URL; defaults to the local dev port. We don't let
// Playwright manage the Rust binary because it needs Postgres + schema + seed
// wired up first, which the harness (dev-db.sh / the CI job) already does.
export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL: process.env.E2E_BASE_URL || "http://localhost:8130",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
