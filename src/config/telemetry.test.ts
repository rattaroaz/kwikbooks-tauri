import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const debugMock = vi.fn();
const envMock = {
  telemetry: false,
  isDev: false,
};

const { hostErrorMock } = vi.hoisted(() => ({
  hostErrorMock: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/plugin-log", () => ({
  error: hostErrorMock,
}));

vi.mock("./env", () => ({
  env: envMock,
}));

describe("telemetry", () => {
  beforeEach(() => {
    envMock.telemetry = false;
    envMock.isDev = false;
    debugMock.mockReset();
    hostErrorMock.mockReset();
    vi.spyOn(console, "debug").mockImplementation(debugMock);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("telemetryEnabled reflects env.telemetry", async () => {
    const { telemetryEnabled } = await import("./telemetry");
    envMock.telemetry = false;
    expect(telemetryEnabled()).toBe(false);
    envMock.telemetry = true;
    expect(telemetryEnabled()).toBe(true);
  });

  it("captureException forwards to host log and debug context only in dev when telemetry is off", async () => {
    const { captureException } = await import("./telemetry");
    envMock.telemetry = false;
    envMock.isDev = true;
    captureException(new Error("boom"), "DbGate.init");
    expect(hostErrorMock).toHaveBeenCalledTimes(1);
    expect(debugMock).toHaveBeenCalledTimes(1);
  });

  it("captureException forwards to host log but skips debug when telemetry is off and not dev", async () => {
    const { captureException } = await import("./telemetry");
    envMock.telemetry = false;
    envMock.isDev = false;
    captureException(new Error("boom"), "DbGate.init");
    expect(hostErrorMock).toHaveBeenCalledTimes(1);
    expect(debugMock).not.toHaveBeenCalled();
  });
});
