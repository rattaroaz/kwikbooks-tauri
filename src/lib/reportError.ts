import { captureException } from "../config/telemetry";
import { errorMessage } from "../types/errors";
import { createScopedLogger } from "./logger";

/**
 * Log an API/IPC failure to the host, then notify the user via toast callback.
 * Use for caught exceptions — not validation messages.
 */
export function reportError(
  context: string,
  error: unknown,
  notify: (message: string) => void,
): string {
  const msg = errorMessage(error);
  const scope = context.includes(".") ? context.split(".")[0]! : context;
  void createScopedLogger(scope).warn(`${context}: ${msg}`);
  captureException(error, context);
  notify(msg);
  return msg;
}
