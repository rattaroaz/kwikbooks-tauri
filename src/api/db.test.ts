import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { dbInit, dbMigrate, healthPing } from "./db";

describe("db command wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("maps db init/migrate/health command names", async () => {
    await dbInit();
    await dbMigrate();
    await healthPing();
    expect(invokeMock).toHaveBeenNthCalledWith(1, "db_init");
    expect(invokeMock).toHaveBeenNthCalledWith(2, "db_migrate");
    expect(invokeMock).toHaveBeenNthCalledWith(3, "health_ping");
  });
});
