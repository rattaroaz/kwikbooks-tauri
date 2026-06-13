/** Bracketed timestamp from Tauri plugin-log, e.g. `[2026-01-01][12:00:00][INFO] …`. */
const BRACKET_TS =
  /\[(\d{4}-\d{2}-\d{2})\]\[(\d{2}:\d{2}:\d{2})(?:\.\d+)?\]/;

const SEQ_VALUE = /\bseq=(\d+)/i;
const RID_VALUE = /\brid=([^\s]+)/i;

/** Parse epoch ms from a log line, or `null` when no timestamp is present. */
export function parseLogTimestamp(line: string): number | null {
  const bracket = line.match(BRACKET_TS);
  if (bracket) {
    const ms = Date.parse(`${bracket[1]}T${bracket[2]}`);
    return Number.isNaN(ms) ? null : ms;
  }

  const trimmed = line.trim();
  if (trimmed.startsWith("{")) {
    try {
      const value = JSON.parse(trimmed) as Record<string, unknown>;
      for (const key of ["timestamp", "time", "ts"]) {
        const raw = value[key];
        if (typeof raw === "string") {
          const ms = Date.parse(raw);
          if (!Number.isNaN(ms)) {
            return ms;
          }
        }
      }
    } catch {
      /* not JSON */
    }
  }

  return null;
}

/** Sort log lines chronologically; lines without timestamps keep their relative order. */
export function sortLogLinesByTimestamp<T extends { line: string }>(
  lines: T[],
): T[] {
  return lines
    .map((entry, index) => ({
      entry,
      index,
      ts: parseLogTimestamp(entry.line),
    }))
    .sort((a, b) => {
      if (a.ts !== null && b.ts !== null) {
        return a.ts - b.ts;
      }
      if (a.ts !== null) {
        return -1;
      }
      if (b.ts !== null) {
        return 1;
      }
      return a.index - b.index;
    })
    .map(({ entry }) => entry);
}

/** Match IPC correlation fields (`seq`, `rid`) in invoke / webview log lines. */
export function lineMatchesCorrelationSearch(
  line: string,
  query: string,
): boolean {
  const q = query.trim();
  if (q === "") {
    return true;
  }
  const lower = line.toLowerCase();
  const ql = q.toLowerCase();

  if (lower.includes(`seq=${ql}`) || lower.includes(`rid=${ql}`)) {
    return true;
  }

  const seq = line.match(SEQ_VALUE)?.[1];
  if (seq?.includes(q)) {
    return true;
  }

  const rid = line.match(RID_VALUE)?.[1];
  if (rid?.toLowerCase().includes(ql)) {
    return true;
  }

  return false;
}

export function filterLogLinesBySearch<T extends { line: string }>(
  lines: T[],
  query: string,
): T[] {
  const q = query.trim();
  if (q === "") {
    return lines;
  }
  return lines.filter((entry) => lineMatchesCorrelationSearch(entry.line, q));
}
