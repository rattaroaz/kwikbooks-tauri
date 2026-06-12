import { beforeEach, describe, expect, it, vi } from "vitest";

const { isTauriMock, outerSizeMock, scaleFactorMock, setSizeMock } =
  vi.hoisted(() => ({
    isTauriMock: vi.fn(() => false),
    outerSizeMock: vi.fn(),
    scaleFactorMock: vi.fn(),
    setSizeMock: vi.fn(() => Promise.resolve()),
  }));

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => isTauriMock(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    outerSize: outerSizeMock,
    scaleFactor: scaleFactorMock,
    setSize: setSizeMock,
  }),
}));

vi.mock("@tauri-apps/api/dpi", () => ({
  PhysicalSize: class PhysicalSize {
    width: number;
    height: number;
    constructor(width: number, height: number) {
      this.width = width;
      this.height = height;
    }
  },
}));

describe("logPanelWindow", () => {
  beforeEach(async () => {
    vi.resetModules();
    isTauriMock.mockReturnValue(false);
    outerSizeMock.mockReset();
    scaleFactorMock.mockReset();
    setSizeMock.mockReset();
    setSizeMock.mockResolvedValue(undefined);
    const mod = await import("./logPanelWindow");
    await mod.restoreWindowAfterLogPanel();
  });

  it("no-ops expand and restore outside Tauri", async () => {
    const { expandWindowForLogPanel, restoreWindowAfterLogPanel } =
      await import("./logPanelWindow");
    await expandWindowForLogPanel();
    await restoreWindowAfterLogPanel();
    expect(setSizeMock).not.toHaveBeenCalled();
  });

  it("widens then restores window in Tauri", async () => {
    isTauriMock.mockReturnValue(true);
    outerSizeMock
      .mockResolvedValueOnce({ width: 1024, height: 720 })
      .mockResolvedValueOnce({ width: 1444, height: 720 });
    scaleFactorMock.mockResolvedValue(1);

    const { LOG_PANEL_WIDTH, expandWindowForLogPanel, restoreWindowAfterLogPanel } =
      await import("./logPanelWindow");
    await expandWindowForLogPanel();
    expect(setSizeMock).toHaveBeenCalledWith(
      expect.objectContaining({
        width: 1024 + LOG_PANEL_WIDTH,
        height: 720,
      }),
    );

    await restoreWindowAfterLogPanel();
    expect(setSizeMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ width: 1024, height: 720 }),
    );
  });

  it("ignores window API failures", async () => {
    isTauriMock.mockReturnValue(true);
    outerSizeMock.mockRejectedValue(new Error("no window"));

    const { expandWindowForLogPanel, restoreWindowAfterLogPanel } =
      await import("./logPanelWindow");
    await expect(expandWindowForLogPanel()).resolves.toBeUndefined();
    await expect(restoreWindowAfterLogPanel()).resolves.toBeUndefined();
  });
});
