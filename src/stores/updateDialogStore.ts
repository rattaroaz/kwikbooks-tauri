import type { UpdateDialogPhase } from "../types/update";

type UpdateDialogState = {
  show: boolean;
  phase: UpdateDialogPhase;
  message: string;
};

let state: UpdateDialogState = {
  show: false,
  phase: "idle",
  message: "",
};

const listeners = new Set<() => void>();

function emit(): void {
  for (const listener of listeners) {
    listener();
  }
}

export function subscribeUpdateDialog(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function getUpdateDialogSnapshot(): UpdateDialogState {
  return state;
}

export function openUpdateDialog(): void {
  state = {
    show: true,
    phase: "checking",
    message: "Checking for updates…",
  };
  emit();
}

export function closeUpdateDialog(): void {
  state = { show: false, phase: "idle", message: "" };
  emit();
}

export function setUpdateDialog(partial: {
  phase: UpdateDialogPhase;
  message: string;
}): void {
  state = { ...state, ...partial };
  emit();
}
