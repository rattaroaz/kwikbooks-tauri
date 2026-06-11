/**
 * Telemetry / crash pipeline is **disabled by default** (local desktop app).
 * `captureException` always writes to host logs; set `VITE_TELEMETRY=true` only
 * after wiring a remote collector (not implemented in v1).
 */
import { error as hostError } from "@tauri-apps/plugin-log";
import { env } from "./env";

export function telemetryEnabled(): boolean {
  return env.telemetry;
}

function formatUnknown(error: unknown): string {
  if (error instanceof Error) {
    return error.stack ?? `${error.name}: ${error.message}`;
  }
  return String(error);
}

/** Reserved for opt-in error reporting — does not transmit anything in v1. */
export function captureException(error: unknown, context?: string): void {
  const formatted = formatUnknown(error);
  const message =
    context !== undefined ? `${context}: ${formatted}` : formatted;

  void hostError(message).catch(() => undefined);

  if (!telemetryEnabled() && env.isDev && context !== undefined) {
    console.debug(`[telemetry off] ${context}`, error);
  }
}
