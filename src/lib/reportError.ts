import { captureException } from "../config/telemetry";
import { errorMessage } from "../types/errors";
import { getLastRequestId } from "./correlation";
import { createScopedLogger } from "./logger";

export { logContext } from "./logContext";

function formatContext(context: string): string {
  const rid = getLastRequestId();
  return rid ? `${context} rid=${rid}` : context;
}

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
  const formatted = formatContext(context);
  void createScopedLogger(scope).warn(`${formatted}: ${msg}`);
  captureException(error, formatted);
  notify(msg);
  return msg;
}
