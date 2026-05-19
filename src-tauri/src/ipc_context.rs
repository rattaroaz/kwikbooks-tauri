//! Per-invoke request id from the webview (`ipc_context_set`) for log correlation.

use std::cell::RefCell;

thread_local! {
    static REQUEST_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn set_request_id(id: Option<String>) {
    REQUEST_ID.with(|c| *c.borrow_mut() = id);
}

pub fn take_request_id() -> Option<String> {
    REQUEST_ID.with(|c| c.borrow_mut().take())
}

#[tauri::command]
pub fn ipc_context_set(request_id: String) {
    set_request_id(Some(request_id));
}
