import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { dbInit, dbMigrate, healthPing } from "./db";

function ipcCommandCalls(): unknown[][] {
  return invokeMock.mock.calls.filter((c) => c[0] !== "ipc_context_set");
}

describe("db command wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("maps db init/migrate/health command names", async () => {
    await dbInit();
    await dbMigrate();
    await healthPing();
    const calls = ipcCommandCalls();
    expect(calls[0]).toEqual(["db_init"]);
    expect(calls[1]).toEqual(["db_migrate"]);
    expect(calls[2]).toEqual(["health_ping"]);
  });
});
