import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { debug, warn } from "@tauri-apps/plugin-log";
import { env } from "../config/env";
import { createRequestId, setLastRequestId } from "../lib/correlation";
import { summarizeInvokePayload } from "../lib/logRedact";

const ERR_LOG_MAX = 500;

function truncateErr(message: string): string {
  if (message.length <= ERR_LOG_MAX) {
    return message;
  }
  return `${message.slice(0, ERR_LOG_MAX)}…`;
}

async function setIpcContext(rid: string): Promise<void> {
  try {
    await tauriInvoke("ipc_context_set", { requestId: rid });
  } catch {
    /* Web / Vitest / early boot without IPC */
  }
}

/** Every Tauri IPC call: optional verbose timing + warn on failure + slow-call warnings. */
export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const rid = createRequestId();
  setLastRequestId(rid);
  await setIpcContext(rid);
  const start = performance.now();
  const threshold = env.slowIpcMs;

  if (env.verboseIpc) {
    if (args !== undefined) {
      await debug(
        `[ipc→] rid=${rid} ${cmd} args=${summarizeInvokePayload(args)}`,
      );
    } else {
      await debug(`[ipc→] rid=${rid} ${cmd}`);
    }
  }
  try {
    const result =
      args === undefined
        ? await tauriInvoke<T>(cmd)
        : await tauriInvoke<T>(cmd, args);
    const ms = Math.round(performance.now() - start);
    if (env.verboseIpc) {
      await debug(`[ipc←] rid=${rid} ${cmd} ok ${ms}ms`);
    }
    if (ms >= threshold) {
      await warn(
        `[ipc←] rid=${rid} ${cmd} slow ${ms}ms threshold=${threshold}ms`,
      );
    }
    return result;
  } catch (e) {
    const ms = Math.round(performance.now() - start);
    const msg = truncateErr(e instanceof Error ? e.message : String(e));
    await warn(`[ipc←] rid=${rid} ${cmd} err ${ms}ms ${msg}`);
    throw e;
  }
}
