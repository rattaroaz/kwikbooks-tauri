/** Shape of `DbCommandError` from the Rust side (`tag = "code"`). */
export type AppCommandError =
  | { code: "path_resolution"; message: string }
  | { code: "database_open"; message: string }
  | { code: "migration"; version: number; message: string }
  | { code: "sql"; message: string }
  | { code: "validation"; message: string }
  | { code: "invariant"; message: string }
  | { code: "conflict"; message: string }
  | { code: "not_found"; entity: string; id: number };

const COMMAND_ERROR_CODES = new Set([
  "path_resolution",
  "database_open",
  "migration",
  "sql",
  "validation",
  "invariant",
  "conflict",
  "not_found",
]);

/** Parse a Tauri invoke rejection into a typed command error when possible. */
export function parseAppCommandError(err: unknown): AppCommandError | null {
  if (!err || typeof err !== "object") {
    return null;
  }
  const o = err as Record<string, unknown>;
  const code = o.code;
  if (typeof code !== "string" || !COMMAND_ERROR_CODES.has(code)) {
    return null;
  }
  switch (code) {
    case "path_resolution":
    case "database_open":
    case "sql":
    case "validation":
    case "invariant":
    case "conflict":
      if (typeof o.message === "string") {
        return { code, message: o.message };
      }
      return null;
    case "migration":
      if (typeof o.message === "string" && typeof o.version === "number") {
        return { code, message: o.message, version: o.version };
      }
      return null;
    case "not_found":
      if (
        typeof o.entity === "string" &&
        typeof o.id === "number" &&
        Number.isFinite(o.id)
      ) {
        return { code, entity: o.entity, id: o.id };
      }
      return null;
    default:
      return null;
  }
}

/** User-facing message for a parsed command error. */
export function formatAppCommandError(err: AppCommandError): string {
  switch (err.code) {
    case "not_found":
      return `${err.entity.replace(/_/g, " ")} #${err.id} was not found.`;
    case "conflict":
      if (/already posted/i.test(err.message)) {
        return "This document is already posted to the general ledger.";
      }
      return err.message;
    case "validation":
      return err.message;
    case "migration":
      return `Database migration failed (version ${err.version}): ${err.message}`;
    case "path_resolution":
      return `Could not resolve data path: ${err.message}`;
    case "database_open":
      return `Could not open database: ${err.message}`;
    case "sql":
      return `Database error: ${err.message}`;
    case "invariant":
      return err.message;
  }
}

export function errorMessage(err: unknown): string {
  const parsed = parseAppCommandError(err);
  if (parsed) {
    return formatAppCommandError(parsed);
  }
  if (typeof err === "string") {
    return err;
  }
  if (err && typeof err === "object" && "message" in err) {
    const m = (err as { message?: string }).message;
    if (typeof m === "string" && m.length > 0) {
      return m;
    }
  }
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
