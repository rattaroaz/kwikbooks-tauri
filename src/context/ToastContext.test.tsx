// @vitest-environment happy-dom
import { act, render, screen } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { ToastProvider, useToast } from "./ToastContext";

const { captureMock, warnMock } = vi.hoisted(() => ({
  captureMock: vi.fn(),
  warnMock: vi.fn(() => Promise.resolve()),
}));

vi.mock("../lib/logger", () => ({
  createScopedLogger: () => ({ warn: warnMock }),
}));

vi.mock("../config/telemetry", () => ({
  captureException: captureMock,
}));

function Probe() {
  const { pushApiError, push, dismiss, toasts } = useToast();
  return (
    <div>
      <button
        type="button"
        onClick={() => pushApiError(new Error("fail"), "Probe")}
      >
        api-fail
      </button>
      <button type="button" onClick={() => push("error", "validation")}>
        validation
      </button>
      <button type="button" onClick={() => push("info", "hello")}>
        info
      </button>
      {toasts.map((t) => (
        <button key={t.id} type="button" onClick={() => dismiss(t.id)}>
          dismiss-{t.id}
        </button>
      ))}
    </div>
  );
}

function OutsideProbe() {
  useToast();
  return null;
}

describe("ToastContext", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("pushApiError logs and captures exception", () => {
    render(
      <ToastProvider>
        <Probe />
      </ToastProvider>,
    );
    screen.getByRole("button", { name: "api-fail" }).click();
    expect(warnMock).toHaveBeenCalled();
    expect(captureMock).toHaveBeenCalledWith(expect.any(Error), "Probe");
  });

  it("push error without api helper does not capture exception", () => {
    captureMock.mockClear();
    render(
      <ToastProvider>
        <Probe />
      </ToastProvider>,
    );
    screen.getByRole("button", { name: "validation" }).click();
    expect(captureMock).not.toHaveBeenCalled();
  });

  it("dismiss removes a toast", () => {
    render(
      <ToastProvider>
        <Probe />
      </ToastProvider>,
    );
    act(() => {
      screen.getByRole("button", { name: "info" }).click();
    });
    expect(screen.getByRole("button", { name: /^dismiss-/ })).toBeDefined();
    act(() => {
      screen.getByRole("button", { name: /^dismiss-/ }).click();
    });
    expect(screen.queryByRole("button", { name: /^dismiss-/ })).toBeNull();
  });

  it("auto-dismisses toasts after timeout", () => {
    render(
      <ToastProvider>
        <Probe />
      </ToastProvider>,
    );
    act(() => {
      screen.getByRole("button", { name: "info" }).click();
    });
    expect(screen.getByRole("button", { name: /^dismiss-/ })).toBeDefined();
    act(() => {
      vi.advanceTimersByTime(6000);
    });
    expect(screen.queryByRole("button", { name: /^dismiss-/ })).toBeNull();
  });

  it("throws when useToast is used outside provider", () => {
    expect(() => render(<OutsideProbe />)).toThrow(
      "useToast outside ToastProvider",
    );
  });
});
