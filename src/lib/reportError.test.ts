import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { reportError } from "./reportError";

const { warnMock, captureMock } = vi.hoisted(() => ({
  warnMock: vi.fn(() => Promise.resolve()),
  captureMock: vi.fn(),
}));

vi.mock("./logger", () => ({
  createScopedLogger: () => ({ warn: warnMock }),
}));

vi.mock("../config/telemetry", () => ({
  captureException: captureMock,
}));

vi.mock("../types/errors", () => ({
  errorMessage: (e: unknown) =>
    e instanceof Error ? e.message : String(e),
}));

describe("reportError", () => {
  beforeEach(() => {
    warnMock.mockClear();
    captureMock.mockClear();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("logs, captures, and notifies with formatted message", () => {
    const notify = vi.fn();
    const msg = reportError("AccountsPage.load", new Error("db down"), notify);
    expect(msg).toBe("db down");
    expect(warnMock).toHaveBeenCalledWith("AccountsPage.load: db down");
    expect(captureMock).toHaveBeenCalledWith(
      expect.any(Error),
      "AccountsPage.load",
    );
    expect(notify).toHaveBeenCalledWith("db down");
  });
});
