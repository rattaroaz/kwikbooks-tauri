// @vitest-environment happy-dom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { LogViewerPanel } from "./LogViewerPanel";

const { logsReadMock, reportErrorMock } = vi.hoisted(() => ({
  logsReadMock: vi.fn(),
  reportErrorMock: vi.fn(),
}));

vi.mock("../api/tauri", () => ({
  logsRead: logsReadMock,
}));

vi.mock("../lib/reportError", () => ({
  reportError: reportErrorMock,
}));

describe("LogViewerPanel", () => {
  it("reports load failures to the host log pipeline", async () => {
    logsReadMock.mockRejectedValue(new Error("read failed"));
    reportErrorMock.mockImplementation((_ctx, _err, notify) => {
      notify("read failed");
    });

    render(<LogViewerPanel onClose={vi.fn()} />);

    await waitFor(() => {
      expect(reportErrorMock).toHaveBeenCalledWith(
        "LogViewerPanel.load",
        expect.any(Error),
        expect.any(Function),
      );
    });
    expect(screen.getByTestId("logs-error")).toHaveTextContent("read failed");
  });

  it("renders log lines from the host", async () => {
    logsReadMock.mockResolvedValue({
      logDir: "C:\\logs",
      lines: [
        {
          source: "app",
          level: "info",
          line: "[INFO] startup complete",
        },
      ],
    });

    render(<LogViewerPanel onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId("logs-body")).toHaveTextContent(
        "startup complete",
      );
    });
  });

  it("filters by log level", async () => {
    logsReadMock.mockResolvedValue({
      logDir: "C:\\logs",
      lines: [
        { source: "app", level: "info", line: "[INFO] ok" },
        { source: "app", level: "warn", line: "[WARN] slow" },
      ],
    });

    render(<LogViewerPanel onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId("logs-body")).toHaveTextContent("[WARN] slow");
    });

    fireEvent.change(screen.getByTestId("logs-level-select"), {
      target: { value: "info" },
    });

    expect(screen.getByTestId("logs-body")).toHaveTextContent("[INFO] ok");
    expect(screen.getByTestId("logs-body")).not.toHaveTextContent(
      "[WARN] slow",
    );
  });

  it("sorts lines by parsed timestamp", async () => {
    logsReadMock.mockResolvedValue({
      logDir: "C:\\logs",
      lines: [
        {
          source: "app",
          level: "info",
          line: "[2026-01-01][12:00:01][INFO] later",
        },
        {
          source: "app",
          level: "info",
          line: "[2026-01-01][12:00:00][INFO] earlier",
        },
      ],
    });

    render(<LogViewerPanel onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId("logs-body")).toHaveTextContent("earlier");
    });

    const text = screen.getByTestId("logs-body").textContent ?? "";
    expect(text.indexOf("earlier")).toBeLessThan(text.indexOf("later"));
  });

  it("filters by seq or rid search", async () => {
    logsReadMock.mockResolvedValue({
      logDir: "C:\\logs",
      lines: [
        {
          source: "app",
          level: "info",
          line: "[2026-01-01][12:00:00][INFO] invoke_ok seq=1 rid=aaa",
        },
        {
          source: "app",
          level: "info",
          line: "[2026-01-01][12:00:01][INFO] invoke_ok seq=2 rid=bbb",
        },
      ],
    });

    render(<LogViewerPanel onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId("logs-body")).toHaveTextContent("rid=bbb");
    });

    fireEvent.change(screen.getByTestId("logs-search"), {
      target: { value: "aaa" },
    });

    expect(screen.getByTestId("logs-body")).toHaveTextContent("rid=aaa");
    expect(screen.getByTestId("logs-body")).not.toHaveTextContent("rid=bbb");
  });
});
