use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::lists::{
    bill_get, bills_list, customers_list, invoice_get, invoices_list, journals_list,
    vendors_list,
};
use tauri::State;

#[tauri::command]
pub fn list_customers(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_customers", || list_customers_impl(&state.db_path))
}

fn list_customers_impl(
    db_path: &std::path::Path,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let v = customers_list(&conn)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "list_customers rows={}",
        v.len()
    );
    Ok(v)
}

#[tauri::command]
pub fn list_vendors(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_vendors", || list_vendors_impl(&state.db_path))
}

fn list_vendors_impl(db_path: &std::path::Path) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let v = vendors_list(&conn)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "list_vendors rows={}",
        v.len()
    );
    Ok(v)
}

#[tauri::command]
pub fn list_invoices(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_invoices", || list_invoices_impl(&state.db_path))
}

fn list_invoices_impl(
    db_path: &std::path::Path,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let v = invoices_list(&conn)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "list_invoices rows={}",
        v.len()
    );
    Ok(v)
}

#[tauri::command]
pub fn list_bills(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_bills", || list_bills_impl(&state.db_path))
}

fn list_bills_impl(db_path: &std::path::Path) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let v = bills_list(&conn)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "list_bills rows={}",
        v.len()
    );
    Ok(v)
}

#[tauri::command]
pub fn list_journals(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("list_journals", || list_journals_impl(&state.db_path, limit))
}

fn list_journals_impl(
    db_path: &std::path::Path,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let lim = limit.unwrap_or(500);
    let v = journals_list(&conn, lim)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "list_journals rows={} limit={}",
        v.len(),
        lim
    );
    Ok(v)
}

#[tauri::command]
pub fn get_invoice(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("get_invoice", || get_invoice_impl(&state.db_path, invoice_id))
}

fn get_invoice_impl(
    db_path: &std::path::Path,
    invoice_id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "get_invoice invoice_id={}",
        invoice_id
    );
    invoice_get(&conn, invoice_id)
}

#[tauri::command]
pub fn get_bill(state: State<'_, DbState>, bill_id: i64) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("get_bill", || get_bill_impl(&state.db_path, bill_id))
}

fn get_bill_impl(
    db_path: &std::path::Path,
    bill_id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::lists",
        "get_bill bill_id={}",
        bill_id
    );
    bill_get(&conn, bill_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all, DbCommandError};
    use tempfile::tempdir;

    fn test_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("list_cmd.sqlite");
        run_all(&db_path).expect("migrate");
        (dir, db_path)
    }

    fn seed_customer(db_path: &std::path::Path) -> i64 {
        let conn = open_sqlite(db_path).expect("open");
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'List Customer')",
            [],
        )
        .expect("customer");
        conn.last_insert_rowid()
    }

    fn seed_invoice(db_path: &std::path::Path, customer_id: i64) -> i64 {
        let conn = open_sqlite(db_path).expect("open");
        conn.execute(
            r#"INSERT INTO invoice
               (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, 'INV-LIST-1', 'draft', '2026-03-01', 5000, 0, 5000)"#,
            [customer_id],
        )
        .expect("invoice");
        let invoice_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice_line
               (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor)
               VALUES (?1, 1, 'Service', 1, 5000, 5000)"#,
            [invoice_id],
        )
        .expect("invoice line");
        invoice_id
    }

    #[test]
    fn list_customers_impl_returns_seeded_row() {
        let (_dir, db_path) = test_db();
        let customer_id = seed_customer(&db_path);

        let rows = list_customers_impl(&db_path).expect("list customers");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], customer_id);
        assert_eq!(rows[0]["displayName"], "List Customer");
    }

    #[test]
    fn list_invoices_and_get_invoice_impl_return_header_and_lines() {
        let (_dir, db_path) = test_db();
        let customer_id = seed_customer(&db_path);
        let invoice_id = seed_invoice(&db_path, customer_id);

        let listed = list_invoices_impl(&db_path).expect("list invoices");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["number"], "INV-LIST-1");
        assert_eq!(listed[0]["customerName"], "List Customer");

        let detail = get_invoice_impl(&db_path, invoice_id).expect("get invoice");
        assert_eq!(detail["header"]["number"], "INV-LIST-1");
        assert_eq!(detail["lines"].as_array().map(|l| l.len()), Some(1));
        assert_eq!(detail["lines"][0]["description"], "Service");
    }

    #[test]
    fn get_invoice_impl_maps_missing_invoice_to_not_found() {
        let (_dir, db_path) = test_db();

        let err = get_invoice_impl(&db_path, 404).expect_err("must fail");
        match err {
            DbCommandError::NotFound { entity, id } => {
                assert_eq!(entity, "invoice");
                assert_eq!(id, 404);
            }
            other => panic!("expected not_found error, got {other:?}"),
        }
    }

    #[test]
    fn list_journals_impl_defaults_limit_to_500() {
        let (_dir, db_path) = test_db();
        let conn = open_sqlite(&db_path).expect("open");
        for i in 0..3 {
            conn.execute(
                r#"INSERT INTO journal (company_id, entry_date, memo)
                   VALUES (1, ?1, ?2)"#,
                rusqlite::params![
                    format!("2026-01-{:02}", i + 1),
                    format!("memo {i}")
                ],
            )
            .expect("journal");
        }

        let rows = list_journals_impl(&db_path, None).expect("list journals");
        assert_eq!(rows.len(), 3);
    }
}
