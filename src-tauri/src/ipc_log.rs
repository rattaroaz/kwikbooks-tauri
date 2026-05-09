//! Uniform timing, correlation sequence, and outcome logs for every Tauri IPC handler (`timed_ipc`).
//!
//! Environment:
//! - `KWIKBOOKS_SLOW_MS` — log an extra `invoke_slow` warning when duration meets or exceeds this (default **1500** ms).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub const IPC_TARGET: &str = "kwikbooks_lib::ipc";

static IPC_SEQ: AtomicU64 = AtomicU64::new(0);

#[inline]
fn next_seq() -> u64 {
    IPC_SEQ.fetch_add(1, Ordering::Relaxed) + 1
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
pub fn timed_ipc<T, E: std::fmt::Debug>(
    command: &'static str,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let seq = next_seq();
    let start = Instant::now();
    log::debug!(
        target: IPC_TARGET,
        "invoke_start seq={} command={}",
        seq,
        command
    );
    let result = f();
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let slow_ms = slow_threshold_ms();
    match &result {
        Ok(_) => {
            log::info!(
                target: IPC_TARGET,
                "invoke_ok seq={} command={} elapsed_ms={}",
                seq,
                command,
                elapsed_ms
            );
            if elapsed_ms >= slow_ms {
                log::warn!(
                    target: IPC_TARGET,
                    "invoke_slow seq={} command={} elapsed_ms={} threshold_ms={}",
                    seq,
                    command,
                    elapsed_ms,
                    slow_ms
                );
            }
        }
        Err(e) => {
            log::warn!(
                target: IPC_TARGET,
                "invoke_err seq={} command={} elapsed_ms={} err={:?}",
                seq,
                command,
                elapsed_ms,
                e
            );
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_ipc_ok_returns_inner_ok() {
        let v = timed_ipc("unit_ok", || Ok::<u32, &str>(7)).expect("ok");
        assert_eq!(v, 7);
    }

    #[test]
    fn timed_ipc_err_propagates() {
        let r = timed_ipc("unit_err", || Err::<(), &str>("nope"));
        assert_eq!(r.expect_err("err"), "nope");
    }

    #[test]
    fn slow_threshold_default_positive() {
        assert!(slow_threshold_ms() >= 1);
    }
}
