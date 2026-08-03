import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { APP_NAME, APP_VERSION } from "../lib/constants";
import { isVersionNewer } from "../lib/semver";
import { createScopedLogger } from "../lib/logger";
import { captureException } from "../config/telemetry";
import {
  closeUpdateDialog,
  openUpdateDialog,
  setUpdateDialog,
} from "../stores/updateDialogStore";

const log = createScopedLogger("Update");

const UPDATE_FEED_UNAVAILABLE_MESSAGE =
  "No update feed is published yet for Kwikbooks. Publish a GitHub Release " +
  "(tag vX.Y.Z) with latest.json and signed installers. " +
  "Set repository secrets TAURI_SIGNING_PRIVATE_KEY and " +
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD, then push a tag to run the Release workflow.";

const ARM64_FEED_MISSING_MESSAGE =
  "The current GitHub release feed does not include a Windows ARM64 update. " +
  "Download the *_arm64-setup.exe installer from GitHub Releases, or publish a " +
  "new release after both x64 and ARM64 Release workflow jobs succeed.";

function upToDateMessage(): string {
  return `${APP_NAME} is up to date (version ${APP_VERSION}).`;
}

function isUpdateFeedUnavailable(message: string): boolean {
  const m = message.toLowerCase();
  return (
    m.includes("could not fetch a valid release json") ||
    m.includes("failed to fetch") ||
    m.includes("404") ||
    m.includes("not found")
  );
}

function isArm64PlatformMissing(message: string): boolean {
  const m = message.toLowerCase();
  return (
    m.includes("fallback platforms") &&
    m.includes("windows-aarch64")
  );
}

function errorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}

/** Manual update check — never runs automatically on startup. */
export async function checkForUpdatesAndApply(): Promise<void> {
  if (import.meta.env.VITE_E2E === "true") {
    openUpdateDialog();
    setUpdateDialog({ phase: "up_to_date", message: upToDateMessage() });
    return;
  }

  openUpdateDialog();
  void log.info(`check started installed=${APP_VERSION}`);

  try {
    const update = await check({ allowDowngrades: false });

    if (!update || !isVersionNewer(update.version, APP_VERSION)) {
      const remote = update?.version ?? APP_VERSION;
      void log.info(`up_to_date installed=${APP_VERSION} remote=${remote}`);
      setUpdateDialog({ phase: "up_to_date", message: upToDateMessage() });
      return;
    }

    void log.info(
      `update_available installed=${APP_VERSION} remote=${update.version}`,
    );
    setUpdateDialog({
      phase: "downloading",
      message: `Downloading ${update.version}…`,
    });

    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        void log.info("download started");
      } else if (event.event === "Finished") {
        void log.info("download finished");
      }
    });

    setUpdateDialog({
      phase: "installing",
      message: "Installing update and restarting…",
    });
    void log.info("install relaunch");
    await relaunch();
  } catch (err) {
    const msg = errorMessage(err);
    if (isUpdateFeedUnavailable(msg)) {
      void log.warn("feed not published");
      setUpdateDialog({
        phase: "error",
        message: UPDATE_FEED_UNAVAILABLE_MESSAGE,
      });
      return;
    }
    if (isArm64PlatformMissing(msg)) {
      void log.warn("arm64 platform missing from release feed");
      captureException(err, "Update.arm64FeedMissing");
      setUpdateDialog({
        phase: "error",
        message: ARM64_FEED_MISSING_MESSAGE,
      });
      return;
    }
    void log.error(`check failed: ${msg}`);
    captureException(err, "Update.checkFailed");
    setUpdateDialog({ phase: "error", message: msg });
  }
}

export function dismissUpdateDialog(): void {
  closeUpdateDialog();
}
