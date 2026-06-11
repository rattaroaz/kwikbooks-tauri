export const LOG_LEVELS = [
  "trace",
  "debug",
  "info",
  "warn",
  "error",
  "unknown",
] as const;

export type LogLevel = (typeof LOG_LEVELS)[number];

export type LogLevelFilter = "all" | LogLevel;

export const LOG_LEVEL_LABELS: Record<LogLevel, string> = {
  trace: "Trace",
  debug: "Debug",
  info: "Info",
  warn: "Warn",
  error: "Error",
  unknown: "Other",
};

/** Kept for HMR safety if an older bundle still references the toggle-button UI. */
export function defaultEnabledLevels(): Set<LogLevel> {
  return new Set(LOG_LEVELS);
}

export function filterLogLines<T extends { level: string }>(
  lines: T[],
  levelFilter: LogLevelFilter,
): T[] {
  if (levelFilter === "all") {
    return lines;
  }
  return lines.filter((line) => line.level === levelFilter);
}
