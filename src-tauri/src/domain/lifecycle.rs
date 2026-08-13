use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

/// Posted payments linked to this invoice (minor units). Used by AR and mark-paid.
pub fn invoice_linked_paid(conn: &Connection, invoice_id: i64) -> Result<i64, DbCommandError> {
    conn.query_row(
        r#"SELECT COALESCE(SUM(amount_minor), 0) FROM customer_payment
           WHERE invoice_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

/// total − linked posted payments (negative ⇒ overpaid).
pub fn invoice_linked_remaining(conn: &Connection, invoice_id: i64) -> Result<i64, DbCommandError> {
    let total: i64 = conn.query_row(
        "SELECT total_minor FROM invoice WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(total - invoice_linked_paid(conn, invoice_id)?)
}

/// Linked payments including unposted drafts (optionally excluding one payment id).
pub fn invoice_linked_applied(
    conn: &Connection,
    invoice_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<i64, DbCommandError> {
    if let Some(pid) = exclude_payment_id {
        conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM customer_payment
               WHERE invoice_id = ?1 AND company_id = ?2 AND id != ?3"#,
            rusqlite::params![invoice_id, COMPANY_ID, pid],
            |row| row.get(0),
        )
        .map_err(Into::into)
    } else {
        conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM customer_payment
               WHERE invoice_id = ?1 AND company_id = ?2"#,
            rusqlite::params![invoice_id, COMPANY_ID],
            |row| row.get(0),
        )
        .map_err(Into::into)
    }
}

/// Open balance for applying a new/posting payment (counts draft links).
pub fn invoice_reservable_remaining(
    conn: &Connection,
    invoice_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<i64, DbCommandError> {
    let total: i64 = conn.query_row(
        "SELECT total_minor FROM invoice WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(total - invoice_linked_applied(conn, invoice_id, exclude_payment_id)?)
}

pub fn bill_linked_paid(conn: &Connection, bill_id: i64) -> Result<i64, DbCommandError> {
    conn.query_row(
        r#"SELECT COALESCE(SUM(amount_minor), 0) FROM vendor_payment
           WHERE bill_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
        rusqlite::params![bill_id, COMPANY_ID],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

pub fn bill_linked_remaining(conn: &Connection, bill_id: i64) -> Result<i64, DbCommandError> {
    let total: i64 = conn.query_row(
        "SELECT total_minor FROM bill WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![bill_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(total - bill_linked_paid(conn, bill_id)?)
}

pub fn bill_linked_applied(
    conn: &Connection,
    bill_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<i64, DbCommandError> {
    if let Some(pid) = exclude_payment_id {
        conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM vendor_payment
               WHERE bill_id = ?1 AND company_id = ?2 AND id != ?3"#,
            rusqlite::params![bill_id, COMPANY_ID, pid],
            |row| row.get(0),
        )
        .map_err(Into::into)
    } else {
        conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM vendor_payment
               WHERE bill_id = ?1 AND company_id = ?2"#,
            rusqlite::params![bill_id, COMPANY_ID],
            |row| row.get(0),
        )
        .map_err(Into::into)
    }
}

pub fn bill_reservable_remaining(
    conn: &Connection,
    bill_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<i64, DbCommandError> {
    let total: i64 = conn.query_row(
        "SELECT total_minor FROM bill WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![bill_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(total - bill_linked_applied(conn, bill_id, exclude_payment_id)?)
}

/// Validate linking a customer payment to an invoice (create or post).
pub fn assert_invoice_payment_link(
    conn: &Connection,
    invoice_id: i64,
    customer_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<(), DbCommandError> {
    let row: Result<(i64, String, Option<i64>), _> = conn.query_row(
        "SELECT customer_id, status, journal_id FROM invoice WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![invoice_id, COMPANY_ID],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );
    let (inv_cust, status, journal_id) = row.map_err(|_| DbCommandError::Validation {
        message: format!("invoice {invoice_id} not found"),
    })?;
    if inv_cust != customer_id {
        return Err(DbCommandError::Validation {
            message: "invoice does not belong to this customer".into(),
        });
    }
    if matches!(status.as_str(), "draft" | "void" | "paid") {
        return Err(DbCommandError::Validation {
            message: "cannot apply payment to draft, void, or paid invoice".into(),
        });
    }
    if journal_id.is_none() {
        return Err(DbCommandError::Validation {
            message: "cannot apply payment until invoice is posted to the ledger".into(),
        });
    }
    let remaining = invoice_reservable_remaining(conn, invoice_id, exclude_payment_id)?;
    if remaining <= 0 {
        return Err(DbCommandError::Validation {
            message: "invoice is already fully applied (no open balance)".into(),
        });
    }
    Ok(())
}

/// Validate linking a vendor payment to a bill (create or post).
pub fn assert_bill_payment_link(
    conn: &Connection,
    bill_id: i64,
    vendor_id: i64,
    exclude_payment_id: Option<i64>,
) -> Result<(), DbCommandError> {
    let row: Result<(Option<i64>, String, Option<i64>), _> = conn.query_row(
        "SELECT vendor_id, status, journal_id FROM bill WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![bill_id, COMPANY_ID],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );
    let (bill_vendor, status, journal_id) = row.map_err(|_| DbCommandError::Validation {
        message: format!("bill {bill_id} not found"),
    })?;
    if matches!(status.as_str(), "draft" | "void" | "paid") {
        return Err(DbCommandError::Validation {
            message: "cannot apply payment to draft, void, or paid bill".into(),
        });
    }
    if journal_id.is_none() {
        return Err(DbCommandError::Validation {
            message: "cannot apply payment until bill is posted to the ledger".into(),
        });
    }
    let Some(bvid) = bill_vendor else {
        return Err(DbCommandError::Validation {
            message: "assign a vendor to the bill before applying a vendor payment (payee-only bills cannot be linked)".into(),
        });
    };
    if bvid != vendor_id {
        return Err(DbCommandError::Validation {
            message: "bill does not belong to this vendor".into(),
        });
    }
    let remaining = bill_reservable_remaining(conn, bill_id, exclude_payment_id)?;
    if remaining <= 0 {
        return Err(DbCommandError::Validation {
            message: "bill is already fully applied (no open balance)".into(),
        });
    }
    Ok(())
}

fn invoice_has_posted_payments(conn: &Connection, invoice_id: i64) -> Result<bool, DbCommandError> {
    let n: i64 = conn.query_row(
        r#"SELECT COUNT(*) FROM customer_payment
           WHERE invoice_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

fn bill_has_posted_payments(conn: &Connection, bill_id: i64) -> Result<bool, DbCommandError> {
    let n: i64 = conn.query_row(
        r#"SELECT COUNT(*) FROM vendor_payment
           WHERE bill_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
        rusqlite::params![bill_id, COMPANY_ID],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

pub fn set_invoice_status(
    conn: &Connection,
    id: i64,
    new_status: &str,
) -> Result<(), DbCommandError> {
    let (current, journal_id, _customer_id): (String, Option<i64>, i64) = conn.query_row(
        "SELECT status, journal_id, customer_id FROM invoice WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![id, COMPANY_ID],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

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

    if new_status == "void" {
        if journal_id.is_some() {
            return Err(DbCommandError::Conflict {
                message: "cannot void a posted invoice (journal exists); reverse the entry first"
                    .into(),
            });
        }
        if invoice_has_posted_payments(conn, id)? {
            return Err(DbCommandError::Conflict {
                message: "cannot void invoice with posted payments applied; reverse payments first"
                    .into(),
            });
        }
    }

    if new_status == "paid" {
        if journal_id.is_none() {
            return Err(DbCommandError::Validation {
                message: "cannot mark invoice paid until it is posted to the ledger".into(),
            });
        }
        // Require linked payments (posted) to cover the invoice. Unallocated
        // payments are not treated as multi-document settlement.
        let rem = invoice_linked_remaining(conn, id)?;
        if rem > 0 {
            return Err(DbCommandError::Validation {
                message: format!(
                    "cannot mark invoice paid while open balance remains ({rem} minor units after linked payments)"
                ),
            });
        }
    }

    // Conditional update closes void↔post races (must still be unposted when voiding).
    let n = if new_status == "void" {
        conn.execute(
            r#"UPDATE invoice SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4 AND journal_id IS NULL"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    } else if new_status == "paid" {
        conn.execute(
            r#"UPDATE invoice SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4 AND journal_id IS NOT NULL"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    } else {
        conn.execute(
            r#"UPDATE invoice SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    };
    if n == 0 {
        return Err(DbCommandError::Conflict {
            message: format!(
                "invoice {id} status changed concurrently (could not apply {current} -> {new_status})"
            ),
        });
    }
    Ok(())
}

pub fn set_bill_status(conn: &Connection, id: i64, new_status: &str) -> Result<(), DbCommandError> {
    let (current, journal_id, _vendor_id): (String, Option<i64>, Option<i64>) = conn.query_row(
        "SELECT status, journal_id, vendor_id FROM bill WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![id, COMPANY_ID],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

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

    if new_status == "void" {
        if journal_id.is_some() {
            return Err(DbCommandError::Conflict {
                message: "cannot void a posted bill (journal exists); reverse the entry first"
                    .into(),
            });
        }
        if bill_has_posted_payments(conn, id)? {
            return Err(DbCommandError::Conflict {
                message: "cannot void bill with posted payments applied; reverse payments first"
                    .into(),
            });
        }
    }

    if new_status == "paid" {
        if journal_id.is_none() {
            return Err(DbCommandError::Validation {
                message: "cannot mark bill paid until it is posted to the ledger".into(),
            });
        }
        let rem = bill_linked_remaining(conn, id)?;
        if rem > 0 {
            return Err(DbCommandError::Validation {
                message: format!(
                    "cannot mark bill paid while open balance remains ({rem} minor units after linked payments)"
                ),
            });
        }
    }

    let n = if new_status == "void" {
        conn.execute(
            r#"UPDATE bill SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4 AND journal_id IS NULL"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    } else if new_status == "paid" {
        conn.execute(
            r#"UPDATE bill SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4 AND journal_id IS NOT NULL"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    } else {
        conn.execute(
            r#"UPDATE bill SET status = ?1, updated_at = datetime('now')
               WHERE id = ?2 AND company_id = ?3 AND status = ?4"#,
            rusqlite::params![new_status, id, COMPANY_ID, current],
        )?
    };
    if n == 0 {
        return Err(DbCommandError::Conflict {
            message: format!(
                "bill {id} status changed concurrently (could not apply {current} -> {new_status})"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all_on_connection};
    use tempfile::tempdir;

    fn seed_invoice(conn: &Connection, status: &str, number: &str) -> (i64, i64) {
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
        (conn.last_insert_rowid(), customer_id)
    }

    fn post_invoice_journal(conn: &Connection, inv: i64) {
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-01', 'invoice', ?1)",
            [inv],
        )
        .unwrap();
        let jid = conn.last_insert_rowid();
        conn.execute(
            "UPDATE invoice SET journal_id = ?1 WHERE id = ?2",
            rusqlite::params![jid, inv],
        )
        .unwrap();
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
        let (inv, _) = seed_invoice(&conn, "draft", "LIF-1");

        set_invoice_status(&conn, inv, "sent").expect("draft->sent");
        let status: String = conn
            .query_row("SELECT status FROM invoice WHERE id = ?1", [inv], |row| {
                row.get(0)
            })
            .expect("status");
        assert_eq!(status, "sent");
    }

    #[test]
    fn invoice_paid_requires_posted_and_settled() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice_paid.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, cust) = seed_invoice(&conn, "sent", "LIF-PAID");

        assert!(set_invoice_status(&conn, inv, "paid").is_err());

        post_invoice_journal(&conn, inv);
        assert!(set_invoice_status(&conn, inv, "paid").is_err());

        let cash: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-02', 'payment_customer', 1)",
            [],
        )
        .unwrap();
        let jid = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-02', 100, ?3, ?4)"#,
            rusqlite::params![cust, cash, jid, inv],
        )
        .unwrap();

        set_invoice_status(&conn, inv, "paid").expect("paid when settled");
    }

    #[test]
    fn invoice_paid_allows_overpay_but_not_unallocated_cover() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_overpay.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, cust) = seed_invoice(&conn, "sent", "LIF-OV");
        post_invoice_journal(&conn, inv);
        let cash: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-02', 'payment_customer', 1)",
            [],
        )
        .unwrap();
        let j1 = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-02', 150, ?3, ?4)"#,
            rusqlite::params![cust, cash, j1, inv],
        )
        .unwrap();
        set_invoice_status(&conn, inv, "paid").expect("overpay => paid");

        let (inv2, cust2) = seed_invoice(&conn, "sent", "LIF-UN");
        post_invoice_journal(&conn, inv2);
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-03', 'payment_customer', 2)",
            [],
        )
        .unwrap();
        let j2 = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-03', 100, ?3, NULL)"#,
            rusqlite::params![cust2, cash, j2],
        )
        .unwrap();
        assert!(
            set_invoice_status(&conn, inv2, "paid").is_err(),
            "unallocated must not settle a different invoice"
        );
    }

    #[test]
    fn invoice_void_conditional_update_fails_if_journal_appears() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_void_race.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, _) = seed_invoice(&conn, "sent", "LIF-RACE");
        // Simulate a concurrent post after void validated an empty journal_id:
        // journal appears before the conditional UPDATE runs.
        post_invoice_journal(&conn, inv);
        let err = set_invoice_status(&conn, inv, "void");
        assert!(err.is_err(), "void must fail once journal_id is set");
        let (status, jid): (String, Option<i64>) = conn
            .query_row(
                "SELECT status, journal_id FROM invoice WHERE id = ?1",
                [inv],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "sent");
        assert!(jid.is_some());
    }

    #[test]
    fn invoice_void_blocked_when_posted_or_has_payments() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_void_posted.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, _) = seed_invoice(&conn, "sent", "LIF-VOID");
        post_invoice_journal(&conn, inv);
        assert!(set_invoice_status(&conn, inv, "void").is_err());

        let (inv2, cust2) = seed_invoice(&conn, "sent", "LIF-VOID2");
        let cash: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        // Payment on unposted invoice (legacy edge) — void must still block
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-02', 'payment_customer', 9)",
            [],
        )
        .unwrap();
        let jid = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-02', 50, ?3, ?4)"#,
            rusqlite::params![cust2, cash, jid, inv2],
        )
        .unwrap();
        assert!(set_invoice_status(&conn, inv2, "void").is_err());
    }

    #[test]
    fn invoice_status_rejects_invalid_transition() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_invoice_invalid.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, _) = seed_invoice(&conn, "draft", "LIF-2");

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

    #[test]
    fn reservable_remaining_counts_unposted_drafts() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("lifecycle_reserve.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (inv, cust) = seed_invoice(&conn, "sent", "LIF-RES");
        post_invoice_journal(&conn, inv);
        let cash: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-02', 100, ?3)"#,
            rusqlite::params![cust, cash, inv],
        )
        .unwrap();
        assert_eq!(invoice_reservable_remaining(&conn, inv, None).unwrap(), 0);
        assert_eq!(invoice_linked_remaining(&conn, inv).unwrap(), 100);
    }
}
