use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::dates::require_iso_date;
use crate::domain::reports::{
    ap_open_by_vendor, ar_open_by_customer, balance_sheet, general_ledger, profit_and_loss,
    trial_balance,
};
use tauri::State;

#[tauri::command]
pub fn report_trial_balance(
    state: State<'_, DbState>,
    date_from: String,
    date_to: String,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("report_trial_balance", || {
        require_iso_date("from date", &date_from)?;
        require_iso_date("to date", &date_to)?;
        let conn = open_sqlite(&state.db_path)?;
        let v = trial_balance(&conn, &date_from, &date_to)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "trial_balance rows={} date_from={} date_to={}",
            v.len(),
            date_from,
            date_to
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn report_general_ledger(
    state: State<'_, DbState>,
    account_id: i64,
    date_from: String,
    date_to: String,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("report_general_ledger", || {
        require_iso_date("from date", &date_from)?;
        require_iso_date("to date", &date_to)?;
        let conn = open_sqlite(&state.db_path)?;
        let v = general_ledger(&conn, account_id, &date_from, &date_to)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "general_ledger rows={} account_id={} date_from={} date_to={}",
            v.len(),
            account_id,
            date_from,
            date_to
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn report_ar_open(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("report_ar_open", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = ar_open_by_customer(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "ar_open rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn report_ap_open(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("report_ap_open", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = ap_open_by_vendor(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "ap_open rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn report_profit_loss(
    state: State<'_, DbState>,
    date_from: String,
    date_to: String,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("report_profit_loss", || {
        require_iso_date("from date", &date_from)?;
        require_iso_date("to date", &date_to)?;
        let conn = open_sqlite(&state.db_path)?;
        let j = profit_and_loss(&conn, &date_from, &date_to)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "profit_loss date_from={} date_to={}",
            date_from,
            date_to
        );
        Ok(j)
    })
}

#[tauri::command]
pub fn report_balance_sheet(
    state: State<'_, DbState>,
    as_of_date: String,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("report_balance_sheet", || {
        require_iso_date("as-of date", &as_of_date)?;
        let conn = open_sqlite(&state.db_path)?;
        let j = balance_sheet(&conn, &as_of_date)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::reports",
            "balance_sheet as_of_date={}",
            as_of_date
        );
        Ok(j)
    })
}
