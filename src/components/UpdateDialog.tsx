import { useSyncExternalStore } from "react";
import {
  getUpdateDialogSnapshot,
  subscribeUpdateDialog,
} from "../stores/updateDialogStore";
import { dismissUpdateDialog } from "../services/updateService";
import type { UpdateDialogPhase } from "../types/update";

function titleForPhase(phase: UpdateDialogPhase): string {
  switch (phase) {
    case "checking":
      return "Checking for updates";
    case "downloading":
      return "Downloading update";
    case "installing":
      return "Installing update";
    case "up_to_date":
      return "Up to date";
    case "error":
      return "Update error";
    default:
      return "Updates";
  }
}

export function UpdateDialog() {
  const { show, phase, message } = useSyncExternalStore(
    subscribeUpdateDialog,
    getUpdateDialogSnapshot,
  );

  if (!show) {
    return null;
  }

  const busy =
    phase === "checking" || phase === "downloading" || phase === "installing";

  return (
    <div
      className="kb-modal-backdrop"
      role="presentation"
      data-testid="update-dialog-backdrop"
    >
      <div
        className="kb-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="kb-update-dialog-title"
        data-testid="update-dialog"
      >
        <h2 id="kb-update-dialog-title">{titleForPhase(phase)}</h2>
        <p data-testid="update-dialog-message">{message}</p>
        {busy ? (
          <p className="kb-muted">Please wait…</p>
        ) : (
          <button
            type="button"
            onClick={dismissUpdateDialog}
            data-testid="update-dialog-close"
          >
            Close
          </button>
        )}
      </div>
    </div>
  );
}
