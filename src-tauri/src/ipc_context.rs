//! Per-invoke request id from the webview (`ipc_context_set`) for log correlation.
//!
//! Stored in thread-local for the duration of a sync IPC handler. Cleared after
//! the outer `timed_ipc` finishes so nested/overlapping handlers on the same
//! thread do not permanently lose the id.

use std::cell::RefCell;

thread_local! {
    static REQUEST_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn set_request_id(id: Option<String>) {
    REQUEST_ID.with(|c| *c.borrow_mut() = id);
}

/// Clone the current request id without clearing (safe for start + end logging).
pub fn peek_request_id() -> Option<String> {
    REQUEST_ID.with(|c| c.borrow().clone())
}

pub fn clear_request_id() {
    REQUEST_ID.with(|c| *c.borrow_mut() = None);
}

#[tauri::command]
pub fn ipc_context_set(request_id: String) {
    set_request_id(Some(request_id));
}
