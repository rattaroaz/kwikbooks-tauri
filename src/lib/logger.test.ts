// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { initLoggingPipeline, createScopedLogger } from "./logger";

const { logMocks, envMock } = vi.hoisted(() => ({
  logMocks: {
    attachConsole: vi.fn(() => Promise.resolve()),
    trace: vi.fn(() => Promise.resolve()),
    debug: vi.fn(() => Promise.resolve()),
    info: vi.fn(() => Promise.resolve()),
    warn: vi.fn(() => Promise.resolve()),
    error: vi.fn(() => Promise.resolve()),
  },
  envMock: {
    isDev: true,
    forwardConsoleToHost: false,
  },
}));

vi.mock("@tauri-apps/plugin-log", () => logMocks);

vi.mock("../config/env", () => ({ env: envMock }));

describe("logger", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    envMock.isDev = true;
    envMock.forwardConsoleToHost = false;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initLoggingPipeline attaches console in dev", async () => {
    await initLoggingPipeline();
    expect(logMocks.attachConsole).toHaveBeenCalledTimes(1);
  });

  it("initLoggingPipeline forwards console when enabled", async () => {
    envMock.forwardConsoleToHost = true;
    const original = console.info;
    await initLoggingPipeline();
    console.info("hello");
    expect(logMocks.info).toHaveBeenCalledWith("hello");
    console.info = original;
  });

  it("createScopedLogger prefixes messages", async () => {
    const scoped = createScopedLogger("Test");
    await scoped.warn("something failed");
    expect(logMocks.warn).toHaveBeenCalledWith("[Test] something failed");
  });

  it("createScopedLogger supports all levels", async () => {
    const scoped = createScopedLogger("Levels");
    await scoped.trace("t");
    await scoped.debug("d");
    await scoped.info("i");
    await scoped.error("e");
    expect(logMocks.trace).toHaveBeenCalledWith("[Levels] t");
    expect(logMocks.debug).toHaveBeenCalledWith("[Levels] d");
    expect(logMocks.info).toHaveBeenCalledWith("[Levels] i");
    expect(logMocks.error).toHaveBeenCalledWith("[Levels] e");
  });
});
