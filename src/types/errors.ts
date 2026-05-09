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

export function errorMessage(err: unknown): string {
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
