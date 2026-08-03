//! Uniform timing, correlation sequence, and outcome logs for every Tauri IPC handler (`timed_ipc`).
//!
//! Environment:
//! - `KWIKBOOKS_SLOW_MS` — log an extra `invoke_slow` warning when duration meets or exceeds this (default **1500** ms).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::db::DbCommandError;
use crate::ipc_context::{clear_request_id, peek_request_id};

pub const IPC_TARGET: &str = "kwikbooks_lib::ipc";

static IPC_SEQ: AtomicU64 = AtomicU64::new(0);

#[inline]
fn next_seq() -> u64 {
    IPC_SEQ.fetch_add(1, Ordering::Relaxed) + 1
}

fn format_rid(rid: &Option<String>) -> String {
    rid.as_deref()
        .map(|r| format!(" rid={}", r))
        .unwrap_or_default()
}

fn log_db_error(
    command: &str,
    seq: u64,
    rid: &Option<String>,
    elapsed_ms: u64,
    err: &DbCommandError,
) {
    let rid_s = format_rid(rid);
    match err {
        DbCommandError::NotFound { entity, id } => {
            log::warn!(
                target: IPC_TARGET,
                "invoke_err seq={}{} command={} elapsed_ms={} code=not_found entity={} id={}",
                seq,
                rid_s,
                command,
                elapsed_ms,
                entity,
                id
            );
        }
        DbCommandError::Migration { version, message } => {
            log::warn!(
                target: IPC_TARGET,
                "invoke_err seq={}{} command={} elapsed_ms={} code=migration version={} message={}",
                seq,
                rid_s,
                command,
                elapsed_ms,
                version,
                message
            );
        }
        DbCommandError::PathResolution { message }
        | DbCommandError::DatabaseOpen { message }
        | DbCommandError::Sql { message }
        | DbCommandError::Validation { message }
        | DbCommandError::Invariant { message }
        | DbCommandError::Conflict { message } => {
            let code = match err {
                DbCommandError::PathResolution { .. } => "path_resolution",
                DbCommandError::DatabaseOpen { .. } => "database_open",
                DbCommandError::Sql { .. } => "sql",
                DbCommandError::Validation { .. } => "validation",
                DbCommandError::Invariant { .. } => "invariant",
                DbCommandError::Conflict { .. } => "conflict",
                _ => "unknown",
            };
            log::warn!(
                target: IPC_TARGET,
                "invoke_err seq={}{} command={} elapsed_ms={} code={} message={}",
                seq,
                rid_s,
                command,
                elapsed_ms,
                code,
                message
            );
        }
    }
}

/// Milliseconds threshold for slow-invoke warnings (`KWIKBOOKS_SLOW_MS`, default 1500).
pub fn slow_threshold_ms() -> u64 {
    std::env::var("KWIKBOOKS_SLOW_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1500)
}

/// Runs `f`, logs start at debug, success at info (with elapsed ms + monotonic `seq`), failure at warn.
/// Emits `invoke_slow` at warn when elapsed ≥ [`slow_threshold_ms`].
pub fn timed_ipc<T>(
    command: &'static str,
    f: impl FnOnce() -> Result<T, DbCommandError>,
) -> Result<T, DbCommandError> {
    let seq = next_seq();
    let rid = peek_request_id();
    let rid_s = format_rid(&rid);
    let start = Instant::now();
    log::debug!(
        target: IPC_TARGET,
        "invoke_start seq={}{} command={}",
        seq,
        rid_s,
        command
    );
    let result = f();
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let slow_ms = slow_threshold_ms();
    match &result {
        Ok(_) => {
            log::info!(
                target: IPC_TARGET,
                "invoke_ok seq={}{} command={} elapsed_ms={}",
                seq,
                rid_s,
                command,
                elapsed_ms
            );
            if elapsed_ms >= slow_ms {
                log::warn!(
                    target: IPC_TARGET,
                    "invoke_slow seq={}{} command={} elapsed_ms={} threshold_ms={}",
                    seq,
                    rid_s,
                    command,
                    elapsed_ms,
                    slow_ms
                );
            }
        }
        Err(e) => log_db_error(command, seq, &rid, elapsed_ms, e),
    }
    clear_request_id();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_ipc_ok_returns_inner_ok() {
        let v = timed_ipc("unit_ok", || Ok(7)).expect("ok");
        assert_eq!(v, 7);
    }

    #[test]
    fn timed_ipc_err_propagates() {
        let r: Result<(), DbCommandError> = timed_ipc("unit_err", || {
            Err(DbCommandError::Validation {
                message: "nope".into(),
            })
        });
        assert!(matches!(
            r.expect_err("err"),
            DbCommandError::Validation { .. }
        ));
    }

    #[test]
    fn timed_ipc_db_err_logs_structured() {
        let r: Result<(), DbCommandError> = timed_ipc("unit_db", || {
            Err(DbCommandError::NotFound {
                entity: "invoice".into(),
                id: 9,
            })
        });
        assert!(matches!(
            r.expect_err("err"),
            DbCommandError::NotFound { .. }
        ));
    }

    #[test]
    fn slow_threshold_default_positive() {
        assert!(slow_threshold_ms() >= 1);
    }
}
