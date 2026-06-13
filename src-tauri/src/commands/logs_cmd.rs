use crate::db::DbCommandError;
use crate::ipc_log::timed_ipc;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub source: String,
    pub level: String,
    pub line: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsReadResponse {
    pub log_dir: String,
    pub lines: Vec<LogLine>,
}

const DEFAULT_MAX_LINES: usize = 500;
const MAX_ALLOWED_LINES: usize = 5_000;
const TAIL_READ_BYTES: u64 = 512_000;

/// Sort key from `[YYYY-MM-DD][HH:MM:SS]` plugin-log prefix (lexicographic = chronological).
fn parse_timestamp_sort_key(line: &str) -> Option<&str> {
    if line.len() < 22 {
        return None;
    }
    let b = line.as_bytes();
    if b[0] != b'['
        || b[5] != b'-'
        || b[8] != b'-'
        || b[11] != b']'
        || b[12] != b'['
        || b[15] != b':'
        || b[18] != b':'
        || b[21] != b']'
    {
        return None;
    }
    Some(&line[..22])
}

fn compare_log_lines(a: &LogLine, b: &LogLine) -> std::cmp::Ordering {
    match (
        parse_timestamp_sort_key(&a.line),
        parse_timestamp_sort_key(&b.line),
    ) {
        (Some(ta), Some(tb)) => ta.cmp(tb),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.line.cmp(&b.line),
    }
}

fn parse_level(line: &str) -> String {
    for level in ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"] {
        if line.contains(&format!("[{level}]")) {
            return level.to_ascii_lowercase();
        }
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(level) = value.get("level").and_then(|v| v.as_str()) {
            return level.trim().to_ascii_lowercase();
        }
    }
    "unknown".into()
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn tail_lines(path: &Path, max_lines: usize) -> std::io::Result<Vec<String>> {
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(TAIL_READ_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if start > 0 {
        if let Some(idx) = buf.find('\n') {
            buf = buf[idx + 1..].to_string();
        } else {
            buf.clear();
        }
    }
    let mut lines: Vec<String> = buf
        .lines()
        .map(|l| strip_ansi(l).trim_end().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    Ok(lines)
}

fn log_files_with_prefix(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == prefix || name.starts_with(&format!("{prefix}.")) {
            files.push(entry.path());
        }
    }
    files.sort();
    files
}

fn read_source_lines(dir: &Path, prefix: &str, source: &str, max_lines: usize) -> Vec<LogLine> {
    let mut merged = Vec::new();
    for path in log_files_with_prefix(dir, prefix) {
        if let Ok(lines) = tail_lines(&path, max_lines) {
            for line in lines {
                merged.push(LogLine {
                    level: parse_level(&line),
                    source: source.into(),
                    line,
                });
            }
        }
    }
    merged.sort_by(compare_log_lines);
    if merged.len() > max_lines {
        merged = merged.split_off(merged.len() - max_lines);
    }
    merged
}

/// Returns recent application log lines from the OS log directory (native + webview).
#[tauri::command]
pub fn logs_read(
    app: tauri::AppHandle,
    max_lines: Option<u32>,
) -> Result<LogsReadResponse, DbCommandError> {
    timed_ipc("logs_read", || {
        let max_lines = max_lines
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_MAX_LINES)
            .clamp(1, MAX_ALLOWED_LINES);
        let log_dir = app.path().app_log_dir().map_err(|e| DbCommandError::PathResolution {
            message: format!("log directory unavailable: {e}"),
        })?;
        let mut lines = read_source_lines(&log_dir, "kwikbooks", "app", max_lines);
        lines.extend(read_source_lines(&log_dir, "webview", "webview", max_lines));
        lines.sort_by(compare_log_lines);
        if lines.len() > max_lines {
            lines = lines.split_off(lines.len() - max_lines);
        }
        Ok(LogsReadResponse {
            log_dir: log_dir.to_string_lossy().into_owned(),
            lines,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timestamp_sort_key_reads_bracketed_prefix() {
        let line = "[2026-01-01][12:00:00][INFO] invoke_ok seq=1";
        assert_eq!(
            parse_timestamp_sort_key(line),
            Some("[2026-01-01][12:00:00]")
        );
    }

    #[test]
    fn compare_log_lines_orders_by_timestamp() {
        let early = LogLine {
            source: "app".into(),
            level: "info".into(),
            line: "[2026-01-01][12:00:00][INFO] first".into(),
        };
        let late = LogLine {
            source: "app".into(),
            level: "info".into(),
            line: "[2026-01-01][12:00:01][INFO] second".into(),
        };
        assert_eq!(compare_log_lines(&early, &late), std::cmp::Ordering::Less);
    }

    #[test]
    fn parse_level_reads_bracketed_level() {
        let line = "[2026-01-01][12:00:00][WARN] something happened";
        assert_eq!(parse_level(line), "warn");
    }

    #[test]
    fn parse_level_reads_json_level() {
        let line = r#"{"level":"ERROR","message":"boom"}"#;
        assert_eq!(parse_level(line), "error");
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        let raw = "\u{1b}[31merror\u{1b}[0m line";
        assert_eq!(strip_ansi(raw), "error line");
    }

    #[test]
    fn tail_lines_returns_last_entries() {
        let dir = std::env::temp_dir().join(format!("kwikbooks-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("kwikbooks.log");
        let body = (0..20)
            .map(|i| format!("line-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, body).unwrap();
        let lines = tail_lines(&path, 3).unwrap();
        assert_eq!(lines, vec!["line-17", "line-18", "line-19"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
