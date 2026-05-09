import { vi } from "vitest";

vi.mock("@tauri-apps/plugin-log", () => ({
  debug: vi.fn(() => Promise.resolve()),
  info: vi.fn(() => Promise.resolve()),
  warn: vi.fn(() => Promise.resolve()),
  trace: vi.fn(() => Promise.resolve()),
  error: vi.fn(() => Promise.resolve()),
  attachConsole: vi.fn(() => Promise.resolve(() => undefined)),
}));
