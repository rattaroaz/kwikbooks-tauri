import { defineConfig, devices } from "@playwright/test";

const isCI = !!process.env.CI;

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 45_000,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL: "http://127.0.0.1:1420",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run dev -- --host 127.0.0.1",
    port: 1420,
    reuseExistingServer: !isCI,
    timeout: 120_000,
    env: {
      ...process.env,
      VITE_E2E: "true",
    },
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "firefox",
      use: { ...devices["Desktop Firefox"] },
    },
  ],
});
