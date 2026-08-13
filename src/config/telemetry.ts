/**
 * Offline exception capture — always writes to host log files via plugin-log.
 * Never transmits data off-device. `VITE_DIAGNOSTICS` (or legacy `VITE_TELEMETRY`)
 * attaches breadcrumbs + app meta to each capture for richer local support.
 */
import { error as hostError } from "@tauri-apps/plugin-log";
import { env } from "./env";
import {
  diagnosticsHeader,
  formatBreadcrumbsForLog,
  getBreadcrumbs,
  recordBreadcrumb,
} from "../lib/diagnostics";

/** True when verbose local diagnostics (breadcrumbs on capture) are enabled. */
export function diagnosticsEnabled(): boolean {
  return env.diagnostics;
}

/** @deprecated Use {@link diagnosticsEnabled} — alias for older call sites/tests. */
export function telemetryEnabled(): boolean {
  return diagnosticsEnabled();
}

function formatUnknown(error: unknown): string {
  if (error instanceof Error) {
    return error.stack ?? `${error.name}: ${error.message}`;
  }
  return String(error);
}

/** Always logs to the host. Offline-only; no remote collector. */
export function captureException(error: unknown, context?: string): void {
  const formatted = formatUnknown(error);
  const ctx = context ?? "exception";
  recordBreadcrumb("exception", `${ctx}: ${formatted.split("\n")[0] ?? ""}`);

  const parts = [`${ctx}: ${formatted}`];
  if (diagnosticsEnabled()) {
    parts.push(`meta ${diagnosticsHeader()}`);
    parts.push(`breadcrumbs:\n${formatBreadcrumbsForLog(getBreadcrumbs())}`);
  }

  const message = parts.join("\n");
  void hostError(message).catch(() => undefined);

  if (env.isDev && context !== undefined) {
    console.debug(`[diagnostics] ${context}`, error);
  }
}
