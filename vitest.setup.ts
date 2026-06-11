import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

afterEach(() => {
  cleanup();
});

vi.mock("@tauri-apps/plugin-log", () => ({
  debug: vi.fn(() => Promise.resolve()),
  info: vi.fn(() => Promise.resolve()),
  warn: vi.fn(() => Promise.resolve()),
  trace: vi.fn(() => Promise.resolve()),
  error: vi.fn(() => Promise.resolve()),
  attachConsole: vi.fn(() => Promise.resolve(() => undefined)),
}));
