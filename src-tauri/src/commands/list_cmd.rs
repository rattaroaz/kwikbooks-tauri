use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::lists::{
    bill_get, bills_list, customers_list, invoice_get, invoices_list, journals_list,
    vendors_list,
};
use tauri::State;

#[tauri::command]
pub fn list_customers(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_customers", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = customers_list(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "list_customers rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn list_vendors(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_vendors", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = vendors_list(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "list_vendors rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn list_invoices(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_invoices", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = invoices_list(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "list_invoices rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn list_bills(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_bills", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = bills_list(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "list_bills rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn list_journals(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_journals", || {
        let conn = open_sqlite(&state.db_path)?;
        let lim = limit.unwrap_or(500);
        let v = journals_list(&conn, lim)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "list_journals rows={} limit={}",
            v.len(),
            lim
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn get_invoice(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("get_invoice", || {
        let conn = open_sqlite(&state.db_path)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "get_invoice invoice_id={}",
            invoice_id
        );
        invoice_get(&conn, invoice_id)
    })
}

#[tauri::command]
pub fn get_bill(state: State<'_, DbState>, bill_id: i64) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("get_bill", || {
        let conn = open_sqlite(&state.db_path)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::lists",
            "get_bill bill_id={}",
            bill_id
        );
        bill_get(&conn, bill_id)
    })
}
