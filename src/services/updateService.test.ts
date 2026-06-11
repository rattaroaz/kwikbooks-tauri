import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { APP_VERSION } from "../lib/constants";
import {
  closeUpdateDialog,
  getUpdateDialogSnapshot,
} from "../stores/updateDialogStore";

const checkMock = vi.fn();
const relaunchMock = vi.fn();

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: checkMock,
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: relaunchMock,
}));

vi.mock("../lib/logger", () => ({
  createScopedLogger: () => ({
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
    trace: vi.fn(),
  }),
}));

function bumpPatch(version: string): string {
  const parts = version.split(".").map(Number);
  const patch = (parts[2] ?? 0) + 1;
  return `${parts[0]}.${parts[1]}.${patch}`;
}

describe("checkForUpdatesAndApply", () => {
  beforeEach(() => {
    checkMock.mockReset();
    relaunchMock.mockReset();
    closeUpdateDialog();
    vi.unstubAllEnvs();
  });

  afterEach(() => {
    closeUpdateDialog();
  });

  it("shows up to date when check returns null", async () => {
    checkMock.mockResolvedValue(null);
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("up_to_date");
    expect(relaunchMock).not.toHaveBeenCalled();
  });

  it("shows up to date when remote version is not newer", async () => {
    checkMock.mockResolvedValue({
      version: APP_VERSION,
      downloadAndInstall: vi.fn(),
    });
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("up_to_date");
  });

  it("downloads and relaunches when remote is newer", async () => {
    const downloadAndInstall = vi.fn(async () => undefined);
    checkMock.mockResolvedValue({
      version: bumpPatch(APP_VERSION),
      downloadAndInstall,
    });
    relaunchMock.mockResolvedValue(undefined);
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(downloadAndInstall).toHaveBeenCalled();
    expect(relaunchMock).toHaveBeenCalled();
  });

  it("shows setup guidance when feed is missing", async () => {
    checkMock.mockRejectedValue(
      new Error("Could not fetch a valid release JSON"),
    );
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("error");
    expect(getUpdateDialogSnapshot().message).toMatch(/No update feed/i);
  });

  it("shows raw error for other failures", async () => {
    checkMock.mockRejectedValue(new Error("network down"));
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("error");
    expect(getUpdateDialogSnapshot().message).toBe("network down");
  });
});
