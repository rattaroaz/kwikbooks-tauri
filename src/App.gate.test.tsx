// @vitest-environment happy-dom
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { ToastProvider } from "./context/ToastContext";

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => false),
}));

vi.mock("./api/db", () => ({
  dbInit: vi.fn(() => Promise.resolve({ dbPath: "/x", migrationVersion: 1 })),
  healthPing: vi.fn(() =>
    Promise.resolve({ ok: true, sqliteOk: true, migrationVersion: 1 }),
  ),
}));

describe("TauriGate", () => {
  it("shows browser-only message when not in Tauri", async () => {
    const App = (await import("./App")).default;
    render(
      <MemoryRouter>
        <ToastProvider>
          <App />
        </ToastProvider>
      </MemoryRouter>,
    );
    expect(
      screen.getByRole("heading", { name: /open the desktop app/i }),
    ).toBeDefined();
  });
});
