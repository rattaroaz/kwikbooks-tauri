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
    const downloadAndInstall = vi.fn(async (onEvent) => {
      await onEvent({ event: "Started" });
      await onEvent({ event: "Finished" });
    });
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

  it("shows ARM64 guidance when the release feed omits windows-aarch64", async () => {
    checkMock.mockRejectedValue(
      new Error(
        'None of the fallback platforms `["windows-aarch64-msi", "windows-aarch64"]` were found in the response `platforms` object',
      ),
    );
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("error");
    expect(getUpdateDialogSnapshot().message).toMatch(/Windows ARM64 update/i);
  });

  it("shows raw error for other failures", async () => {
    checkMock.mockRejectedValue(new Error("network down"));
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("error");
    expect(getUpdateDialogSnapshot().message).toBe("network down");
  });

  it("shows raw error for non-Error failures", async () => {
    checkMock.mockRejectedValue("network down");
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(getUpdateDialogSnapshot().phase).toBe("error");
    expect(getUpdateDialogSnapshot().message).toBe("network down");
  });

  it("closes the update dialog", async () => {
    const { dismissUpdateDialog } = await import("./updateService");
    const { openUpdateDialog } = await import("../stores/updateDialogStore");
    openUpdateDialog();
    dismissUpdateDialog();
    expect(getUpdateDialogSnapshot().show).toBe(false);
  });

  it("short-circuits in E2E mode", async () => {
    vi.stubEnv("VITE_E2E", "true");
    const { checkForUpdatesAndApply } = await import("./updateService");
    await checkForUpdatesAndApply();
    expect(checkMock).not.toHaveBeenCalled();
    expect(getUpdateDialogSnapshot().phase).toBe("up_to_date");
  });
});
