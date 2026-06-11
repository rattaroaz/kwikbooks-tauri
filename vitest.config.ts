import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    setupFiles: ["./vitest.setup.ts"],
    environment: "node",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: [
        "src/lib/**/*.ts",
        "src/api/**/*.ts",
        "src/config/**/*.ts",
        "src/services/**/*.ts",
        "src/components/ErrorBoundary.tsx",
        "src/components/PageLoading.tsx",
        "src/context/ToastContext.tsx",
      ],
      exclude: [
        "src/**/*.d.ts",
        "src/test/**",
        "src/components/UpdateDialog.tsx",
        "src/components/GlobalSearch.tsx",
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
