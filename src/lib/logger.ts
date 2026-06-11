/**
 * Frontend logging pipeline: forwards `console.*` to the host (`@tauri-apps/plugin-log`),
 * mirrors Rust logs into the devtools console (`attachConsole`), and exposes scoped helpers.
 */
import { env } from "../config/env";

function stringifyArg(value: unknown): string {
  if (value instanceof Error) {
    return value.stack ?? `${value.name}: ${value.message}`;
  }
  if (typeof value === "string") {
    return value;
  }
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

/** Initialize host logging and optional devtools mirroring. Safe to call outside Tauri (no-op). */
export async function initLoggingPipeline(): Promise<void> {
  try {
    const log = await import("@tauri-apps/plugin-log");
    if (env.isDev) {
      await log.attachConsole();
    }
    if (env.forwardConsoleToHost) {
      wireConsoleToHost(log);
    }
  } catch {
    /* Web / Vitest / early boot without IPC */
  }
}

function wireConsoleToHost(log: typeof import("@tauri-apps/plugin-log")): void {
  const wrap = (
    fnName: "log" | "debug" | "info" | "warn" | "error",
    emit: (message: string) => Promise<void>,
  ): void => {
    const original = console[fnName].bind(console) as (
      ...args: unknown[]
    ) => void;
    console[fnName] = (...args: unknown[]) => {
      original(...args);
      const message = args.map(stringifyArg).join(" ");
      void emit(message).catch(() => undefined);
    };
  };

  wrap("log", log.trace);
  wrap("debug", log.debug);
  wrap("info", log.info);
  wrap("warn", log.warn);
  wrap("error", log.error);
}

type LogFn = (message: string) => Promise<void>;

/** Minimal scoped logger — messages go to host log files and terminal via the plugin. */
export function createScopedLogger(scope: string): {
  trace: LogFn;
  debug: LogFn;
  info: LogFn;
  warn: LogFn;
  error: LogFn;
} {
  const prefix = `[${scope}]`;
  return {
    trace: async (message: string) => {
      const { trace } = await import("@tauri-apps/plugin-log");
      await trace(`${prefix} ${message}`);
    },
    debug: async (message: string) => {
      const { debug } = await import("@tauri-apps/plugin-log");
      await debug(`${prefix} ${message}`);
    },
    info: async (message: string) => {
      const { info } = await import("@tauri-apps/plugin-log");
      await info(`${prefix} ${message}`);
    },
    warn: async (message: string) => {
      const { warn } = await import("@tauri-apps/plugin-log");
      await warn(`${prefix} ${message}`);
    },
    error: async (message: string) => {
      const { error } = await import("@tauri-apps/plugin-log");
      await error(`${prefix} ${message}`);
    },
  };
}
