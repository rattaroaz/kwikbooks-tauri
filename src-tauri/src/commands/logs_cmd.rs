use crate::db::DbCommandError;
use crate::ipc_log::{slow_threshold_ms, timed_ipc};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsExportResponse {
    pub path: String,
    pub bytes_written: u64,
    pub line_count: usize,
}

const DEFAULT_MAX_LINES: usize = 500;
const MAX_ALLOWED_LINES: usize = 5_000;
const EXPORT_MAX_LINES: usize = 5_000;
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

/// Redact absolute home / profile paths for support bundles shared off-machine.
pub(crate) fn redact_paths(text: &str) -> String {
    let mut out = text.to_string();
    let mut needles: Vec<String> = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        if !home.is_empty() {
            needles.push(home);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            needles.push(home);
        }
    }
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        if user.len() >= 2 {
            needles.push(format!(r"\Users\{user}"));
            needles.push(format!("/Users/{user}"));
            needles.push(format!("/home/{user}"));
        }
    }
    needles.sort_by_key(|a| std::cmp::Reverse(a.len()));
    needles.dedup();
    for n in needles {
        if n.len() < 3 {
            continue;
        }
        out = out.replace(&n, "<REDACTED_HOME>");
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
        if name == prefix
            || name.starts_with(&format!("{prefix}."))
            || name == format!("{prefix}.log")
            || (name.starts_with(prefix) && name.ends_with(".log"))
        {
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

fn collect_lines(log_dir: &Path, max_lines: usize) -> Vec<LogLine> {
    let mut lines = read_source_lines(log_dir, "kwikbooks", "app", max_lines);
    lines.extend(read_source_lines(log_dir, "webview", "webview", max_lines));

    // Include panic.log as a dedicated source if present.
    let panic_path = log_dir.join("panic.log");
    if panic_path.is_file() {
        if let Ok(extra) = tail_lines(&panic_path, max_lines.min(200)) {
            for line in extra {
                lines.push(LogLine {
                    level: "error".into(),
                    source: "panic".into(),
                    line,
                });
            }
        }
    }

    lines.sort_by(compare_log_lines);
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines
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
        let lines = collect_lines(&log_dir, max_lines);
        Ok(LogsReadResponse {
            log_dir: log_dir.to_string_lossy().into_owned(),
            lines,
        })
    })
}

/// Write a redacted offline support bundle (text) to `destination_path`.
/// Optional `extra_context` is appended (e.g. UI breadcrumbs from the webview).
#[tauri::command]
pub fn logs_export_support_bundle(
    app: tauri::AppHandle,
    destination_path: String,
    max_lines: Option<u32>,
    extra_context: Option<String>,
) -> Result<LogsExportResponse, DbCommandError> {
    timed_ipc("logs_export_support_bundle", || {
        let max_lines = max_lines
            .map(|n| n as usize)
            .unwrap_or(EXPORT_MAX_LINES)
            .clamp(1, MAX_ALLOWED_LINES);
        let log_dir = app.path().app_log_dir().map_err(|e| DbCommandError::PathResolution {
            message: format!("log directory unavailable: {e}"),
        })?;
        let lines = collect_lines(&log_dir, max_lines);
        let dest = PathBuf::from(&destination_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut body = String::new();
        body.push_str("# Kwikbooks offline support bundle\n");
        body.push_str("# Paths and usernames are redacted. Nothing was transmitted off-device.\n");
        body.push_str(&format!(
            "generated_at={}\napp_version={}\nos={}\narch={}\nlog_level={:?}\nslow_ms={}\nlog_dir={}\nline_count={}\n\n",
            chrono_like_now(),
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            crate::logging::max_level_from_env(),
            slow_threshold_ms(),
            redact_paths(&log_dir.to_string_lossy()),
            lines.len(),
        ));
        if let Some(ctx) = extra_context.as_ref().filter(|s| !s.trim().is_empty()) {
            body.push_str("## Client context (breadcrumbs / meta)\n");
            body.push_str(&redact_paths(ctx));
            if !ctx.ends_with('\n') {
                body.push('\n');
            }
            body.push('\n');
        }
        body.push_str("## Log lines\n");
        for entry in &lines {
            let redacted = redact_paths(&entry.line);
            body.push_str(&format!(
                "[{}][{}] {}\n",
                entry.source, entry.level, redacted
            ));
        }

        let mut file = std::fs::File::create(&dest)?;
        file.write_all(body.as_bytes())?;
        Ok(LogsExportResponse {
            path: dest.to_string_lossy().into_owned(),
            bytes_written: body.len() as u64,
            line_count: lines.len(),
        })
    })
}

fn chrono_like_now() -> String {
    // Avoid adding chrono crate — RFC3339-ish via SystemTime
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix_secs={secs}")
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

    #[test]
    fn redact_paths_masks_userprofile() {
        std::env::set_var("USERPROFILE", r"C:\Users\example");
        let raw = r"database path = C:\Users\example\AppData\kwikbooks.sqlite";
        let out = redact_paths(raw);
        assert!(out.contains("<REDACTED_HOME>"));
        assert!(!out.contains(r"C:\Users\example"));
        std::env::remove_var("USERPROFILE");
    }
}
