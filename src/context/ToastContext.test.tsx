// @vitest-environment happy-dom
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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
  const { pushApiError, push } = useToast();
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
    </div>
  );
}

describe("ToastContext", () => {
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
});
