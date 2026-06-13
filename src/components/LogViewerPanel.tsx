import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as api from "../api/tauri";
import type { LogLine } from "../api/tauri";
import {
  filterLogLines,
  LOG_LEVEL_LABELS,
  LOG_LEVELS,
  type LogLevelFilter,
} from "../lib/logLevels";
import {
  filterLogLinesBySearch,
  sortLogLinesByTimestamp,
} from "../lib/logParse";
import { reportError } from "../lib/reportError";
import { logContext } from "../lib/logContext";

type Props = {
  onClose: () => void;
};

const POLL_MS = 3_000;

function formatLine({ source, line }: LogLine): string {
  const tag = source === "webview" ? "webview" : "app";
  return `[${tag}] ${line}`;
}

export function LogViewerPanel({ onClose }: Props) {
  const [lines, setLines] = useState<LogLine[]>([]);
  const [logDir, setLogDir] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [levelFilter, setLevelFilter] = useState<LogLevelFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const bodyRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);

  const loadLogs = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await api.logsRead(500);
      setLogDir(res.logDir);
      setLines(res.lines);
    } catch (e) {
      reportError(logContext("LogViewerPanel", "load"), e, (msg) =>
        setError(msg),
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    stickToBottomRef.current = true;
    void loadLogs();
    const id = window.setInterval(() => {
      void loadLogs();
    }, POLL_MS);
    return () => window.clearInterval(id);
  }, [loadLogs]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        void onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const visibleLines = useMemo(() => {
    const byLevel = filterLogLines(lines, levelFilter);
    const bySearch = filterLogLinesBySearch(byLevel, searchQuery);
    return sortLogLinesByTimestamp(bySearch);
  }, [lines, levelFilter, searchQuery]);

  useEffect(() => {
    const el = bodyRef.current;
    if (!el || !stickToBottomRef.current) {
      return;
    }
    el.scrollTop = el.scrollHeight;
  }, [visibleLines]);

  const onScroll = () => {
    const el = bodyRef.current;
    if (!el) {
      return;
    }
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
    stickToBottomRef.current = nearBottom;
  };

  return (
    <aside
      className="kb-logs-panel"
      role="complementary"
      aria-labelledby="kb-logs-panel-title"
      data-testid="logs-panel"
    >
      <header className="kb-logs-head">
        <div>
          <h2 id="kb-logs-panel-title">Application logs</h2>
          {logDir ? (
            <p className="kb-muted kb-logs-dir" title={logDir}>
              {logDir}
            </p>
          ) : null}
        </div>
        <div className="kb-logs-actions">
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="logs-refresh"
            disabled={loading}
            onClick={() => void loadLogs()}
          >
            Refresh
          </button>
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="logs-close"
            onClick={() => void onClose()}
          >
            Close
          </button>
        </div>
      </header>

      <div className="kb-logs-filters" data-testid="logs-level-filters">
        <label className="kb-logs-level-field" htmlFor="kb-logs-level-select">
          Log level
        </label>
        <select
          id="kb-logs-level-select"
          className="kb-logs-level-select"
          data-testid="logs-level-select"
          value={levelFilter}
          onChange={(e) => setLevelFilter(e.target.value as LogLevelFilter)}
        >
          <option value="all">All levels</option>
          {LOG_LEVELS.map((level) => (
            <option key={level} value={level}>
              {LOG_LEVEL_LABELS[level]}
            </option>
          ))}
        </select>
      </div>

      <div className="kb-logs-filters" data-testid="logs-search-filters">
        <label className="kb-logs-level-field" htmlFor="kb-logs-search">
          Search seq / rid
        </label>
        <input
          id="kb-logs-search"
          type="search"
          className="kb-logs-search-input"
          data-testid="logs-search"
          placeholder="e.g. 42 or req-abc"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      {error ? (
        <p className="kb-logs-error" data-testid="logs-error">
          {error}
        </p>
      ) : null}

      <div
        ref={bodyRef}
        className="kb-logs-body"
        data-testid="logs-body"
        onScroll={onScroll}
      >
        {loading && visibleLines.length === 0 ? (
          <p className="kb-muted">Loading logs…</p>
        ) : null}
        {!loading && visibleLines.length === 0 ? (
          <p className="kb-muted">
            {searchQuery.trim()
              ? "No log entries match the search."
              : "No log entries for the selected levels."}
          </p>
        ) : null}
        {visibleLines.map((entry, index) => (
          <div
            key={`${entry.level}-${index}-${entry.line}`}
            className={`kb-logs-line kb-logs-line-${entry.level}`}
          >
            <span
              className={`kb-logs-level-tag kb-logs-level-tag-${entry.level}`}
            >
              {entry.level.toUpperCase()}
            </span>
            <span className="kb-logs-line-text">{formatLine(entry)}</span>
          </div>
        ))}
      </div>
    </aside>
  );
}
