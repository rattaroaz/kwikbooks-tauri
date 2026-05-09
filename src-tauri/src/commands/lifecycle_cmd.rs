use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::lifecycle::{set_bill_status, set_invoice_status};
use tauri::State;

fn invoice_set_status_impl(
    db_path: &std::path::Path,
    invoice_id: i64,
    status: &str,
) -> Result<(), DbCommandError> {
    let conn = open_sqlite(db_path)?;
    set_invoice_status(&conn, invoice_id, status)
}

fn bill_set_status_impl(
    db_path: &std::path::Path,
    bill_id: i64,
    status: &str,
) -> Result<(), DbCommandError> {
    let conn = open_sqlite(db_path)?;
    set_bill_status(&conn, bill_id, status)
}

#[tauri::command]
pub fn invoice_set_status(
    state: State<'_, DbState>,
    invoice_id: i64,
    status: String,
) -> Result<(), DbCommandError> {
    timed_ipc("invoice_set_status", || {
        log::debug!(
            target: "kwikbooks_lib::ipc::lifecycle",
            "invoice_set_status invoice_id={} status={}",
            invoice_id,
            status
        );
        invoice_set_status_impl(&state.db_path, invoice_id, &status)
    })
}

#[tauri::command]
pub fn bill_set_status(
    state: State<'_, DbState>,
    bill_id: i64,
    status: String,
) -> Result<(), DbCommandError> {
    timed_ipc("bill_set_status", || {
        log::debug!(
            target: "kwikbooks_lib::ipc::lifecycle",
            "bill_set_status bill_id={} status={}",
            bill_id,
            status
        );
        bill_set_status_impl(&state.db_path, bill_id, &status)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{run_all, DbCommandError};
    use tempfile::tempdir;

    #[test]
    fn invoice_set_status_impl_returns_validation_for_invalid_transition() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("lifecycle_invoice_cmd.sqlite");
        run_all(&db_path).expect("migrate");
        let conn = open_sqlite(&db_path).expect("open");
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'Cmd Customer')",
            [],
        )
        .expect("customer");
        let customer_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice
               (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, 'INV-CMD-1', 'draft', '2026-02-01', 1000, 0, 1000)"#,
            [customer_id],
        )
        .expect("invoice");
        let invoice_id = conn.last_insert_rowid();

        let err = invoice_set_status_impl(&db_path, invoice_id, "paid").expect_err("must fail");
        match err {
            DbCommandError::Validation { message } => {
                assert!(message.contains("invalid invoice transition"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn bill_set_status_impl_returns_conflict_after_void() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("lifecycle_bill_cmd.sqlite");
        run_all(&db_path).expect("migrate");
        let conn = open_sqlite(&db_path).expect("open");
        conn.execute(
            r#"INSERT INTO bill
               (company_id, number, status, issue_date, total_minor)
               VALUES (1, 'B-CMD-1', 'void', '2026-02-01', 1000)"#,
            [],
        )
        .expect("bill");
        let bill_id = conn.last_insert_rowid();

        let err = bill_set_status_impl(&db_path, bill_id, "paid").expect_err("must fail");
        match err {
            DbCommandError::Conflict { message } => {
                assert!(message.contains("already void"));
            }
            other => panic!("expected conflict error, got {other:?}"),
        }
    }
}
