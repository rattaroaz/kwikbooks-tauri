use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::posting::{post_bill, post_customer_payment, post_invoice, post_vendor_payment};
use tauri::State;

fn invoice_post_impl(db_path: &std::path::Path, invoice_id: i64) -> Result<i64, DbCommandError> {
    let mut conn = open_sqlite(db_path)?;
    post_invoice(&mut conn, invoice_id)
}

fn bill_post_impl(db_path: &std::path::Path, bill_id: i64) -> Result<i64, DbCommandError> {
    let mut conn = open_sqlite(db_path)?;
    post_bill(&mut conn, bill_id)
}

fn customer_payment_post_impl(db_path: &std::path::Path, payment_id: i64) -> Result<i64, DbCommandError> {
    let mut conn = open_sqlite(db_path)?;
    post_customer_payment(&mut conn, payment_id)
}

fn vendor_payment_post_impl(db_path: &std::path::Path, payment_id: i64) -> Result<i64, DbCommandError> {
    let mut conn = open_sqlite(db_path)?;
    post_vendor_payment(&mut conn, payment_id)
}

#[tauri::command]
pub fn invoice_post(state: State<'_, DbState>, invoice_id: i64) -> Result<i64, DbCommandError> {
    timed_ipc("invoice_post", || invoice_post_impl(&state.db_path, invoice_id))
}

#[tauri::command]
pub fn bill_post(state: State<'_, DbState>, bill_id: i64) -> Result<i64, DbCommandError> {
    timed_ipc("bill_post", || bill_post_impl(&state.db_path, bill_id))
}

#[tauri::command]
pub fn customer_payment_post(
    state: State<'_, DbState>,
    payment_id: i64,
) -> Result<i64, DbCommandError> {
    timed_ipc("customer_payment_post", || {
        customer_payment_post_impl(&state.db_path, payment_id)
    })
}

#[tauri::command]
pub fn vendor_payment_post(
    state: State<'_, DbState>,
    payment_id: i64,
) -> Result<i64, DbCommandError> {
    timed_ipc("vendor_payment_post", || vendor_payment_post_impl(&state.db_path, payment_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{run_all, DbCommandError};
    use tempfile::tempdir;

    #[test]
    fn invoice_post_impl_maps_not_found_error() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("post_invoice_cmd.sqlite");
        run_all(&db_path).expect("migrate");

        let err = invoice_post_impl(&db_path, 404).expect_err("must fail");
        match err {
            DbCommandError::NotFound { entity, id } => {
                assert_eq!(entity, "invoice");
                assert_eq!(id, 404);
            }
            other => panic!("expected not_found error, got {other:?}"),
        }
    }

    #[test]
    fn customer_payment_post_impl_maps_not_found_error() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("post_customer_payment_cmd.sqlite");
        run_all(&db_path).expect("migrate");

        let err = customer_payment_post_impl(&db_path, 777).expect_err("must fail");
        match err {
            DbCommandError::NotFound { entity, id } => {
                assert_eq!(entity, "customer_payment");
                assert_eq!(id, 777);
            }
            other => panic!("expected not_found error, got {other:?}"),
        }
    }
}
