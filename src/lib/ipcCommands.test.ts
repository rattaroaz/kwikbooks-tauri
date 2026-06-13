import { describe, expect, it } from "vitest";
import {
  e2eMockCommands,
  frontendInvokeCommands,
  frontendNotInRust,
  missingFromE2eMock,
  registeredRustCommands,
} from "./ipcCommands";

describe("ipcCommands contract", () => {
  it("frontend invoke commands are registered in Rust", () => {
    expect(frontendNotInRust()).toEqual([]);
  });

  it("E2E mock implements every Rust IPC command", () => {
    expect(missingFromE2eMock()).toEqual([]);
  });

  it("lists non-empty command registries", () => {
    expect(registeredRustCommands().length).toBeGreaterThan(30);
    expect(frontendInvokeCommands().length).toBeGreaterThan(30);
    expect(e2eMockCommands().length).toBeGreaterThan(30);
  });
});
