import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invokeCoreMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeCoreMock,
}));

const debugMock = vi.fn(() => Promise.resolve());
const warnMock = vi.fn(() => Promise.resolve());

vi.mock("@tauri-apps/plugin-log", () => ({
  debug: debugMock,
  warn: warnMock,
}));

describe("invoke wrapper", () => {
  beforeEach(() => {
    invokeCoreMock.mockReset();
    debugMock.mockReset();
    warnMock.mockReset();
    invokeCoreMock.mockResolvedValue(42);
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it("warns with rid on failure and truncates long messages", async () => {
    vi.stubEnv("VITE_VERBOSE_IPC", "false");
    vi.stubEnv("VITE_SLOW_IPC_MS", "60000");
    vi.resetModules();
    invokeCoreMock.mockRejectedValueOnce(new Error("x".repeat(600)));
    const { invoke } = await import("./invoke");
    await expect(invoke("fail_cmd")).rejects.toThrow();
    expect(warnMock).toHaveBeenCalledTimes(1);
    const first = (warnMock.mock.calls as unknown[][])[0]?.[0];
    expect(first).toBeDefined();
    const msg = String(first);
    expect(msg).toContain("rid=");
    expect(msg).toContain("fail_cmd");
    expect(msg.length).toBeLessThan(900);
  });

  it("logs slow calls when duration exceeds threshold", async () => {
    vi.stubEnv("VITE_VERBOSE_IPC", "false");
    vi.stubEnv("VITE_SLOW_IPC_MS", "15");
    vi.resetModules();
    invokeCoreMock.mockImplementation(
      () =>
        new Promise<number>((resolve) => {
          setTimeout(() => resolve(1), 40);
        }),
    );
    const { invoke } = await import("./invoke");
    await invoke("slow_cmd");
    expect(warnMock).toHaveBeenCalled();
    const slowLine = (warnMock.mock.calls as unknown[][]).find((c) =>
      String(c[0]).includes("slow"),
    );
    expect(slowLine?.[0]).toBeDefined();
  });

  it("includes redacted args in debug when verbose", async () => {
    vi.stubEnv("VITE_VERBOSE_IPC", "true");
    vi.stubEnv("VITE_SLOW_IPC_MS", "60000");
    vi.resetModules();
    const { invoke } = await import("./invoke");
    await invoke("x", { memo: "secret", n: 1 });
    expect(debugMock).toHaveBeenCalled();
    const line = (debugMock.mock.calls as unknown[][])
      .map((c) => String(c[0] ?? ""))
      .join("\n");
    expect(line).toContain("[redacted]");
    expect(line).not.toContain("secret");
  });
});
