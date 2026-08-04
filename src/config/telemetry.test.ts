import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const debugMock = vi.fn();
const envMock = {
  diagnostics: false,
  isDev: false,
};

const { hostErrorMock } = vi.hoisted(() => ({
  hostErrorMock: vi.fn<(message: string) => Promise<void>>(() =>
    Promise.resolve(),
  ),
}));

vi.mock("@tauri-apps/plugin-log", () => ({
  error: hostErrorMock,
}));

vi.mock("./env", () => ({
  env: envMock,
}));

describe("telemetry / offline diagnostics", () => {
  beforeEach(async () => {
    envMock.diagnostics = false;
    envMock.isDev = false;
    debugMock.mockReset();
    hostErrorMock.mockReset();
    vi.spyOn(console, "debug").mockImplementation(debugMock);
    const { resetGlobalErrorHandlersForTests } =
      await import("../lib/diagnostics");
    resetGlobalErrorHandlersForTests();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("diagnosticsEnabled reflects env.diagnostics", async () => {
    const { diagnosticsEnabled, telemetryEnabled } =
      await import("./telemetry");
    envMock.diagnostics = false;
    expect(diagnosticsEnabled()).toBe(false);
    expect(telemetryEnabled()).toBe(false);
    envMock.diagnostics = true;
    expect(diagnosticsEnabled()).toBe(true);
  });

  it("captureException always forwards to host log", async () => {
    const { captureException } = await import("./telemetry");
    envMock.diagnostics = false;
    envMock.isDev = true;
    captureException(new Error("boom"), "DbGate.init");
    expect(hostErrorMock).toHaveBeenCalledTimes(1);
    expect(debugMock).toHaveBeenCalledTimes(1);
  });

  it("captureException includes breadcrumbs when diagnostics on", async () => {
    const { recordBreadcrumb } = await import("../lib/diagnostics");
    const { captureException } = await import("./telemetry");
    envMock.diagnostics = true;
    envMock.isDev = false;
    recordBreadcrumb("ipc", "→ health_ping");
    captureException(new Error("boom"), "DbGate.init");
    const firstArg = hostErrorMock.mock.calls[0]?.[0];
    const msg = String(firstArg ?? "");
    expect(msg).toContain("breadcrumbs:");
    expect(msg).toContain("health_ping");
    expect(msg).toContain("meta ");
  });

  it("formats non-Error values without context", async () => {
    const { captureException } = await import("./telemetry");
    captureException("plain failure");
    expect(hostErrorMock).toHaveBeenCalled();
    const firstArg = hostErrorMock.mock.calls[0]?.[0];
    expect(String(firstArg ?? "")).toContain("plain failure");
  });
});
