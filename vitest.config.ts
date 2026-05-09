import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    setupFiles: ["./vitest.setup.ts"],
    environment: "node",
    include: ["src/**/*.test.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: ["src/lib/**/*.ts", "src/api/**/*.ts", "src/config/**/*.ts"],
      exclude: [
        "src/**/*.d.ts",
        /** Bootstraps Tauri plugin + console forwarding; covered by E2E / manual smoke. */
        "src/lib/logger.ts",
      ],
      thresholds: {
        lines: 85,
        functions: 85,
        branches: 80,
        statements: 85,
      },
    },
  },
});
