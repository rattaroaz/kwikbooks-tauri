use rusqlite::Connection;

use crate::db::DbCommandError;

pub fn set_invoice_status(
    conn: &Connection,
    id: i64,
    new_status: &str,
) -> Result<(), DbCommandError> {
    let current: String =
        conn.query_row("SELECT status FROM invoice WHERE id = ?1", [id], |row| {
            row.get(0)
        })?;

    if matches!(current.as_str(), "paid" | "void") {
        return Err(DbCommandError::Conflict {
            message: format!("invoice {id} is already {current}"),
        });
    }

    let allowed = matches!(
        (current.as_str(), new_status),
        ("draft", "sent") | ("draft", "void") | ("sent", "paid") | ("sent", "void")
    );

    if !allowed {
        return Err(DbCommandError::Validation {
            message: format!("invalid invoice transition {current} -> {new_status}"),
        });
    }

    conn.execute(
        "UPDATE invoice SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_status, id],
    )?;
    Ok(())
}

pub fn set_bill_status(conn: &Connection, id: i64, new_status: &str) -> Result<(), DbCommandError> {
    let current: String = conn.query_row("SELECT status FROM bill WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;

    if matches!(current.as_str(), "paid" | "void") {
        return Err(DbCommandError::Conflict {
            message: format!("bill {id} is already {current}"),
        });
    }

    let allowed = matches!(
        (current.as_str(), new_status),
        ("draft", "open") | ("draft", "void") | ("open", "paid") | ("open", "void")
    );

    if !allowed {
        return Err(DbCommandError::Validation {
            message: format!("invalid bill transition {current} -> {new_status}"),
        });
    }

    conn.execute(
        "UPDATE bill SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_status, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all_on_connection};
    use tempfile::tempdir;

    fn seed_invoice(conn: &Connection, status: &str, number: &str) -> i64 {
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'Lifecycle Cust')",
            [],
        )
        .expect("customer");
        let customer_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice
               (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, ?2, ?3, '2026-01-01', 100, 0, 100)"#,
            rusqlite::params![customer_id, number, status],
        )
        .expect("invoice");
        conn.last_insert_rowid()
    }

    fn seed_bill(conn: &Connection, status: &str, number: &str) -> i64 {
        conn.execute(
            r#"INSERT INTO bill
               (company_id, number, status, issue_date, total_minor)
               VALUES (1, ?1, ?2, '2026-01-01', 100)"#,
            rusqlite::params![number, status],
        )
        .expect("bill");
        conn.last_insert_rowid()
    }

    #[test]
    fn invoice_status_allows_draft_to_sent_then_paid() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let inv = seed_invoice(&conn, "draft", "LIF-1");

        set_invoice_status(&conn, inv, "sent").expect("draft->sent");
        set_invoice_status(&conn, inv, "paid").expect("sent->paid");

        let status: String = conn
            .query_row("SELECT status FROM invoice WHERE id = ?1", [inv], |row| row.get(0))
            .expect("status");
        assert_eq!(status, "paid");
    }

    #[test]
    fn invoice_status_rejects_invalid_transition() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice_invalid.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let inv = seed_invoice(&conn, "draft", "LIF-2");

        let err = set_invoice_status(&conn, inv, "paid");
        assert!(err.is_err(), "draft->paid should be rejected");
    }

    #[test]
    fn bill_status_rejects_changes_after_void() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_bill.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let bill = seed_bill(&conn, "draft", "BL-1");

        set_bill_status(&conn, bill, "void").expect("draft->void");
        let err = set_bill_status(&conn, bill, "paid");
        assert!(err.is_err(), "void->paid should be conflict");
    }
}
