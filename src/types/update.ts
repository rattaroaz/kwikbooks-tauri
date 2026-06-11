export type UpdateDialogPhase =
  | "idle"
  | "checking"
  | "up_to_date"
  | "downloading"
  | "installing"
  | "error";

export type UpdateStatus = "up_to_date" | "update_available" | "error";
