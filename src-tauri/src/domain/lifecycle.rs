use rusqlite::{Connection, OptionalExtension};

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

pub fn set_invoice_status(
    conn: &Connection,
    id: i64,
    new_status: &str,
) -> Result<(), DbCommandError> {
    let row: Option<(String, Option<i64>, i64)> = conn
        .query_row(
            "SELECT status, journal_id, total_minor FROM invoice WHERE id = ?1 AND company_id = ?2",
            rusqlite::params![id, COMPANY_ID],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    let Some((current, journal_id, total)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "invoice".into(),
            id,
        });
    };

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

    if new_status == "void" && journal_id.is_some() {
        return Err(DbCommandError::Validation {
            message: "cannot void a posted invoice".into(),
        });
    }

    if new_status == "paid" {
        let applied: i64 = conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM customer_payment
               WHERE invoice_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
            rusqlite::params![id, COMPANY_ID],
            |row| row.get(0),
        )?;
        if applied < total {
            return Err(DbCommandError::Validation {
                message: "cannot mark invoice paid until posted payments cover the total".into(),
            });
        }
    }

    conn.execute(
        "UPDATE invoice SET status = ?1, updated_at = datetime('now') WHERE id = ?2 AND company_id = ?3",
        rusqlite::params![new_status, id, COMPANY_ID],
    )?;
    Ok(())
}

pub fn set_bill_status(conn: &Connection, id: i64, new_status: &str) -> Result<(), DbCommandError> {
    let row: Option<(String, Option<i64>, i64)> = conn
        .query_row(
            "SELECT status, journal_id, total_minor FROM bill WHERE id = ?1 AND company_id = ?2",
            rusqlite::params![id, COMPANY_ID],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    let Some((current, journal_id, total)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "bill".into(),
            id,
        });
    };

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

    if new_status == "void" && journal_id.is_some() {
        return Err(DbCommandError::Validation {
            message: "cannot void a posted bill".into(),
        });
    }

    if new_status == "paid" {
        let applied: i64 = conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM vendor_payment
               WHERE bill_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
            rusqlite::params![id, COMPANY_ID],
            |row| row.get(0),
        )?;
        if applied < total {
            return Err(DbCommandError::Validation {
                message: "cannot mark bill paid until posted payments cover the total".into(),
            });
        }
    }

    conn.execute(
        "UPDATE bill SET status = ?1, updated_at = datetime('now') WHERE id = ?2 AND company_id = ?3",
        rusqlite::params![new_status, id, COMPANY_ID],
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
    fn invoice_status_allows_draft_to_sent() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let inv = seed_invoice(&conn, "draft", "LIF-1");

        set_invoice_status(&conn, inv, "sent").expect("draft->sent");
        let err = set_invoice_status(&conn, inv, "paid");
        assert!(err.is_err(), "sent->paid without payment should be rejected");

        let status: String = conn
            .query_row("SELECT status FROM invoice WHERE id = ?1", [inv], |row| row.get(0))
            .expect("status");
        assert_eq!(status, "sent");
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
    fn invoice_status_rejects_void_when_posted() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice_void.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let inv = seed_invoice(&conn, "sent", "LIF-3");
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-01', 'invoice', ?1)",
            [inv],
        )
        .expect("journal");
        let jid = conn.last_insert_rowid();
        conn.execute(
            "UPDATE invoice SET journal_id = ?1 WHERE id = ?2",
            rusqlite::params![jid, inv],
        )
        .expect("link");

        let err = set_invoice_status(&conn, inv, "void");
        assert!(err.is_err(), "posted invoice cannot be voided");
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
