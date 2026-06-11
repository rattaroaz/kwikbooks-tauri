function parseBool(value: string | undefined, defaultValue: boolean): boolean {
  if (value === undefined || value === "") return defaultValue;
  const v = value.toLowerCase();
  return v === "1" || v === "true" || v === "yes" || v === "on";
}

/** @internal Exported for unit tests. */
export function parsePositiveMs(
  value: string | undefined,
  defaultMs: number,
): number {
  if (value === undefined || value === "") return defaultMs;
  const n = Number(String(value).trim());
  return Number.isFinite(n) && n > 0 ? Math.floor(n) : defaultMs;
}

/** Vite/Tauri frontend environment (import.meta.env). */
export const env = {
  isDev: import.meta.env.DEV,
  isProd: import.meta.env.PROD,
  mode: import.meta.env.MODE,
  /**
   * Opt-in remote telemetry (not wired in v1). `captureException` always logs to
   * the host via plugin-log regardless of this flag.
   */
  telemetry: parseBool(import.meta.env.VITE_TELEMETRY, false),
  /**
   * Log each IPC call timing/detail via `@tauri-apps/plugin-log` (default: on in dev, off in prod).
   */
  verboseIpc: parseBool(import.meta.env.VITE_VERBOSE_IPC, import.meta.env.DEV),
  /**
   * Forward browser `console.*` to the host log pipeline (default: on in dev, off in prod).
   * Disable in release builds to avoid leaking arbitrary console payloads into log files.
   */
  forwardConsoleToHost: parseBool(
    import.meta.env.VITE_LOG_CONSOLE_FORWARD,
    import.meta.env.DEV,
  ),
  /**
   * Warn when an IPC round-trip exceeds this many ms (default **1500**). Mirrors `KWIKBOOKS_SLOW_MS` on the host.
   */
  slowIpcMs: parsePositiveMs(import.meta.env.VITE_SLOW_IPC_MS, 1500),
  parseBool,
} as const;
