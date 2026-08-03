import { afterEach, describe, expect, it } from "vitest";
import {
  clearBreadcrumbs,
  formatBreadcrumbsForLog,
  getBreadcrumbs,
  recordBreadcrumb,
  resetGlobalErrorHandlersForTests,
} from "./diagnostics";

describe("diagnostics breadcrumbs", () => {
  afterEach(() => {
    resetGlobalErrorHandlersForTests();
  });

  it("records and formats breadcrumbs", () => {
    clearBreadcrumbs();
    recordBreadcrumb("ipc", "→ company_get");
    recordBreadcrumb("ui", "opened settings");
    expect(getBreadcrumbs()).toHaveLength(2);
    const formatted = formatBreadcrumbsForLog();
    expect(formatted).toMatch(/\[ipc\]/);
    expect(formatted).toMatch(/company_get/);
  });

  it("caps breadcrumb ring size", () => {
    clearBreadcrumbs();
    for (let i = 0; i < 120; i += 1) {
      recordBreadcrumb("test", `n=${i}`);
    }
    expect(getBreadcrumbs().length).toBeLessThanOrEqual(80);
    const last = getBreadcrumbs()[getBreadcrumbs().length - 1];
    expect(last?.message).toBe("n=119");
  });
});
