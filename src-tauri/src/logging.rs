//! [`tauri-plugin-log`] wiring: terminal output, rotated files under the OS log directory,
//! separate files for webview vs native Rust targets, and filters for noisy dependencies.
//!
//! Environment:
//! - `KWIKBOOKS_LOG` / `RUST_LOG` — global level (see [`max_level_from_env`]).
//! - `KWIKBOOKS_LOG_JSON=1` — one JSON object per line (good for parsers); disables ANSI colors.
//! - `KWIKBOOKS_SLOW_MS` — host-side slow-invoke threshold (see [`crate::ipc_log::slow_threshold_ms`]).

use log::LevelFilter;
use tauri::plugin::TauriPlugin;
use tauri::Runtime;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind, TimezoneStrategy, WEBVIEW_TARGET};

/// Reads `KWIKBOOKS_LOG`, then `RUST_LOG`, then picks a sensible default (debug in dev, info in release).
pub fn max_level_from_env() -> LevelFilter {
    let raw = std::env::var("KWIKBOOKS_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_default();
    let key = raw.trim();
    if key.is_empty() {
        return if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };
    }
    // Support plain level tokens; ignore full `RUST_LOG` module specs for now.
    match key
        .split(',')
        .next()
        .unwrap_or(key)
        .split('=')
        .next_back()
        .unwrap_or(key)
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ if cfg!(debug_assertions) => LevelFilter::Debug,
        _ => LevelFilter::Info,
    }
}

fn json_logs_enabled() -> bool {
    std::env::var("KWIKBOOKS_LOG_JSON")
        .map(|v| {
            matches!(
                v.trim(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

pub fn log_plugin<R: Runtime>() -> TauriPlugin<R> {
    let mut b = Builder::new()
        .level(max_level_from_env())
        .rotation_strategy(RotationStrategy::KeepAll)
        .max_file_size(5_242_880)
        .level_for("tao", LevelFilter::Warn)
        .level_for("tauri", LevelFilter::Warn)
        .level_for("wry", LevelFilter::Warn)
        .level_for("hyper", LevelFilter::Warn)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("rusqlite", LevelFilter::Warn)
        .clear_targets()
        .target(Target::new(TargetKind::Stdout))
        .target(
            Target::new(TargetKind::LogDir {
                file_name: Some("kwikbooks".into()),
            })
            .filter(|m| !m.target().starts_with(WEBVIEW_TARGET)),
        )
        .target(
            Target::new(TargetKind::LogDir {
                file_name: Some("webview".into()),
            })
            .filter(|m| m.target().starts_with(WEBVIEW_TARGET)),
        );

    if json_logs_enabled() {
        b = b.clear_format().format(|out, message, record| {
            let ts_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let payload = serde_json::json!({
                "ts_ms": ts_ms,
                "level": record.level().to_string(),
                "target": record.target(),
                "message": format!("{}", message),
            });
            let line = serde_json::to_string(&payload)
                .unwrap_or_else(|_| format!("{{\"ts_ms\":{ts_ms},\"level\":\"{:?}\"}}", record.level()));
            out.finish(format_args!("{}", line))
        });
    } else {
        b = b
            .timezone_strategy(TimezoneStrategy::UseLocal)
            .with_colors(ColoredLevelConfig::default());
    }

    b.build()
}
