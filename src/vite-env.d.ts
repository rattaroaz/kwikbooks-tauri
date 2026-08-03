/// <reference types="vite/client" />
/// <reference types="@testing-library/jest-dom/vitest" />

interface ImportMetaEnv {
  /** `true` | `false` | `1` | `0` — controls dev-only tooling when `import.meta.env.DEV`. */
  readonly VITE_ENABLE_DEVTOOLS?: string;
  /** Gate experimental UI; default off. */
  readonly VITE_FEATURE_EXPERIMENTAL_UI?: string;
  /** Verbose local diagnostics on capture (default: on in DEV). Offline-only. */
  readonly VITE_DIAGNOSTICS?: string;
  /** @deprecated Prefer VITE_DIAGNOSTICS — same local-only meaning. */
  readonly VITE_TELEMETRY?: string;
  /** Verbose per-invoke frontend logs (default mirrors dev build). */
  readonly VITE_VERBOSE_IPC?: string;
  /** Mirror webview console to host logs (default mirrors dev build). */
  readonly VITE_LOG_CONSOLE_FORWARD?: string;
  /** Warn when IPC exceeds this duration in ms (default 1500). */
  readonly VITE_SLOW_IPC_MS?: string;
  /** Playwright E2E: mock updater and skip Tauri/DB gates. */
  readonly VITE_E2E?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
