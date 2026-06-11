import { isTauri } from "@tauri-apps/api/core";
import { PhysicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const LOG_PANEL_WIDTH = 420;

let savedOuterWidth: number | null = null;

/** Widen the desktop window so the log panel does not cover app content. */
export async function expandWindowForLogPanel(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  try {
    const win = getCurrentWindow();
    const outer = await win.outerSize();
    if (savedOuterWidth === null) {
      savedOuterWidth = outer.width;
    }
    const factor = await win.scaleFactor();
    const extra = Math.round(LOG_PANEL_WIDTH * factor);
    await win.setSize(new PhysicalSize(outer.width + extra, outer.height));
  } catch {
    /* E2E mock or missing window permission — panel still opens in-layout */
  }
}

/** Restore the window width saved when the log panel was opened. */
export async function restoreWindowAfterLogPanel(): Promise<void> {
  if (!isTauri() || savedOuterWidth === null) {
    savedOuterWidth = null;
    return;
  }
  try {
    const win = getCurrentWindow();
    const outer = await win.outerSize();
    await win.setSize(new PhysicalSize(savedOuterWidth, outer.height));
  } catch {
    /* ignore */
  }
  savedOuterWidth = null;
}
