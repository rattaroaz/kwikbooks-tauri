/**
 * Offline-only local diagnostics: breadcrumb ring + global error hooks.
 * Nothing is transmitted off-device.
 */
import { APP_VERSION } from "./constants";

const MAX_BREADCRUMBS = 80;

export type Breadcrumb = {
  ts: number;
  category: string;
  message: string;
};

const breadcrumbs: Breadcrumb[] = [];

export function recordBreadcrumb(category: string, message: string): void {
  breadcrumbs.push({
    ts: Date.now(),
    category,
    message: message.slice(0, 500),
  });
  while (breadcrumbs.length > MAX_BREADCRUMBS) {
    breadcrumbs.shift();
  }
}

/** Snapshot of recent breadcrumbs (oldest → newest). */
export function getBreadcrumbs(): readonly Breadcrumb[] {
  return breadcrumbs.slice();
}

export function clearBreadcrumbs(): void {
  breadcrumbs.length = 0;
}

export function formatBreadcrumbsForLog(
  list: readonly Breadcrumb[] = breadcrumbs,
): string {
  if (list.length === 0) {
    return "(none)";
  }
  return list
    .map((b) => {
      const iso = new Date(b.ts).toISOString();
      return `${iso} [${b.category}] ${b.message}`;
    })
    .join("\n");
}

export function diagnosticsHeader(): string {
  return [
    `kwikbooks_version=${APP_VERSION}`,
    `user_agent=${typeof navigator !== "undefined" ? navigator.userAgent : "n/a"}`,
    `href=${typeof location !== "undefined" ? location.href : "n/a"}`,
  ].join(" ");
}

type CaptureFn = (error: unknown, context?: string) => void;

let handlersInstalled = false;

/** Install once: window.onerror + unhandledrejection → captureException. */
export function installGlobalErrorHandlers(capture: CaptureFn): void {
  if (handlersInstalled || typeof window === "undefined") {
    return;
  }
  handlersInstalled = true;

  window.addEventListener("error", (event) => {
    recordBreadcrumb("window.error", event.message || "error");
    const err =
      event.error instanceof Error
        ? event.error
        : new Error(event.message || "window.error");
    capture(err, "window.onerror");
  });

  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    recordBreadcrumb(
      "unhandledrejection",
      reason instanceof Error ? reason.message : String(reason),
    );
    capture(reason, "unhandledrejection");
  });
}

/** @internal test helper */
export function resetGlobalErrorHandlersForTests(): void {
  handlersInstalled = false;
  clearBreadcrumbs();
}
