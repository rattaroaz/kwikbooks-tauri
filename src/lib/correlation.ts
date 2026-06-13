/** Short request id for correlating webview IPC log lines (no shared state with the Rust host `seq`). */

let lastRequestId: string | null = null;

export function createRequestId(): string {
  if (typeof globalThis.crypto?.randomUUID === "function") {
    return globalThis.crypto.randomUUID().replace(/-/g, "").slice(0, 12);
  }
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

/** Most recent IPC request id from `api/invoke.ts` (for UI-layer error correlation). */
export function getLastRequestId(): string | null {
  return lastRequestId;
}

export function setLastRequestId(rid: string): void {
  lastRequestId = rid;
}

/** Test-only reset. */
export function resetLastRequestId(): void {
  lastRequestId = null;
}
