// @vitest-environment happy-dom
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "./ErrorBoundary";

const { captureMock } = vi.hoisted(() => ({
  captureMock: vi.fn(),
}));

vi.mock("../config/telemetry", () => ({
  captureException: captureMock,
}));

function Boom(): null {
  throw new Error("render boom");
}

describe("ErrorBoundary", () => {
  it("renders fallback when child throws", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(
      <ErrorBoundary>
        <Boom />
      </ErrorBoundary>,
    );
    expect(screen.getByRole("alert")).toBeDefined();
    expect(screen.getByText("render boom")).toBeDefined();
    expect(captureMock).toHaveBeenCalled();
    spy.mockRestore();
  });
});
