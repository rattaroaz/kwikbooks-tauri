use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;
use crate::domain::ids::{
    account_exists, account_has_type, account_id_by_code, require_bank_cash_account,
};
use crate::domain::journal::{insert_journal, DraftJournalLine};
use crate::domain::money::line_total_minor;

type InvoicePostRow = (String, Option<i64>, i64, i64, i64, i64, String);
type BillPostRow = (String, Option<i64>, i64, String, Option<i64>);
type CustomerPaymentPostRow = (Option<i64>, i64, i64, i64, String, Option<i64>);
type VendorPaymentPostRow = (Option<i64>, i64, i64, String, i64, Option<i64>);

pub fn post_invoice(conn: &mut Connection, invoice_id: i64) -> Result<i64, DbCommandError> {
    let row: Option<InvoicePostRow> = conn
        .query_row(
            r#"SELECT status, journal_id, customer_id, subtotal_minor, tax_minor, total_minor, issue_date
               FROM invoice WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![invoice_id, COMPANY_ID],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .optional()?;

    let Some((status, journal_id, _cust, subtotal, tax, total, issue_date)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "invoice".into(),
            id: invoice_id,
        });
    };

    if journal_id.is_some() {
        return Err(DbCommandError::Conflict {
            message: "invoice already posted".into(),
        });
    }

    if status != "sent" {
        return Err(DbCommandError::Validation {
            message: "only sent invoices can be posted (use invoice_set_status)".into(),
        });
    }

    let ar_id = account_id_by_code(conn, COMPANY_ID, "1100")?;
    let sales_id = account_id_by_code(conn, COMPANY_ID, "4000")?;
    let tax_payable_id = account_id_by_code(conn, COMPANY_ID, "2100")?;

    let collected: Vec<(i32, String, i64, i64)> = {
        let mut stmt = conn.prepare(
            r#"SELECT line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id, item_id
               FROM invoice_line WHERE invoice_id = ?1 ORDER BY line_number"#,
        )?;

        let lines_iter = stmt.query_map(rusqlite::params![invoice_id], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })?;

        let mut sum_lines: i64 = 0;
        let mut collected = Vec::new();
        for row in lines_iter {
            let (num, desc, qty, unit_p, line_tot, inc_acc, _item_id) = row?;
            let computed = line_total_minor(qty, unit_p);
            if computed != line_tot {
                return Err(DbCommandError::Validation {
                    message: format!("invoice line {num}: stored total does not match qty × unit"),
                });
            }
            sum_lines = sum_lines.saturating_add(line_tot);
            let income_acct = inc_acc.unwrap_or(sales_id);
            if !account_exists(conn, COMPANY_ID, income_acct)? {
                return Err(DbCommandError::Validation {
                    message: format!("unknown income account on line {num}"),
                });
            }
            if !account_has_type(conn, COMPANY_ID, income_acct, "income")? {
                return Err(DbCommandError::Validation {
                    message: format!("line {num} income account must have account_type=income"),
                });
            }
            collected.push((num, desc, line_tot, income_acct));
        }

        if sum_lines != subtotal {
            return Err(DbCommandError::Validation {
                message: "invoice subtotal does not equal sum of line totals".into(),
            });
        }

        collected
    };

    if subtotal.saturating_add(tax) != total {
        return Err(DbCommandError::Validation {
            message: "invoice total must equal subtotal + tax".into(),
        });
    }

    let mut jl: Vec<DraftJournalLine> = Vec::new();
    jl.push(DraftJournalLine {
        account_id: ar_id,
        line_number: 1,
        description: Some("Accounts receivable".into()),
        debit_minor: total,
        credit_minor: 0,
    });

    let mut n = 2;
    for (_ln, desc, line_tot, inc_acct) in &collected {
        jl.push(DraftJournalLine {
            account_id: *inc_acct,
            line_number: n,
            description: Some(desc.clone()),
            debit_minor: 0,
            credit_minor: *line_tot,
        });
        n += 1;
    }

    if tax < 0 {
        return Err(DbCommandError::Validation {
            message: "invoice tax must be zero or positive".into(),
        });
    }
    if tax > 0 {
        jl.push(DraftJournalLine {
            account_id: tax_payable_id,
            line_number: n,
            description: Some("Sales tax".into()),
            debit_minor: 0,
            credit_minor: tax,
        });
    }

    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // Re-check inside the write lock so void cannot win after validation.
    let (status_now, journal_now): (String, Option<i64>) = tx.query_row(
        "SELECT status, journal_id FROM invoice WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if journal_now.is_some() {
        return Err(DbCommandError::Conflict {
            message: "invoice already posted".into(),
        });
    }
    if status_now != "sent" {
        return Err(DbCommandError::Validation {
            message: "only sent invoices can be posted (use invoice_set_status)".into(),
        });
    }

    let jid = insert_journal(
        &tx,
        COMPANY_ID,
        &issue_date,
        Some(&format!("Invoice #{invoice_id}")),
        "invoice",
        invoice_id,
        &jl,
    )?;

    let n = tx.execute(
        r#"UPDATE invoice SET journal_id = ?1, updated_at = datetime('now')
           WHERE id = ?2 AND company_id = ?3 AND status = 'sent' AND journal_id IS NULL"#,
        rusqlite::params![jid, invoice_id, COMPANY_ID],
    )?;
    if n == 0 {
        return Err(DbCommandError::Conflict {
            message: "invoice status changed concurrently; post aborted".into(),
        });
    }
    tx.commit()?;
    log::info!(
        target: "kwikbooks_lib::domain::posting",
        "invoice_posted invoice_id={} journal_id={}",
        invoice_id,
        jid
    );
    Ok(jid)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::db::{open_sqlite, run_all_on_connection};
    use tempfile::tempdir;

    fn seed_sent_invoice(conn: &mut Connection) -> i64 {
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'Acme Labs')",
            [],
        )
        .expect("customer");
        let cust_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, 'INV-9001', 'sent', '2026-01-15', 5000, 0, 5000)"#,
            [cust_id],
        )
        .expect("invoice");
        let inv_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice_line (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id)
               VALUES (?1, 1, 'Consulting', 1, 5000, 5000,
                (SELECT id FROM account WHERE company_id = 1 AND code = '4000'))"#,
            [inv_id],
        )
        .expect("line");
        inv_id
    }

    fn seed_customer_payment(conn: &Connection, bank_account_id: i64, amount_minor: i64) -> i64 {
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'Paying Customer')",
            [],
        )
        .expect("customer");
        let customer_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor)
               VALUES (1, ?1, ?2, '2026-03-01', ?3)"#,
            rusqlite::params![customer_id, bank_account_id, amount_minor],
        )
        .expect("customer payment");
        conn.last_insert_rowid()
    }

    fn seed_vendor_payment(conn: &Connection, bank_account_id: i64, amount_minor: i64) -> i64 {
        conn.execute(
            "INSERT INTO vendor (company_id, display_name) VALUES (1, 'Paid Vendor')",
            [],
        )
        .expect("vendor");
        let vendor_id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO vendor_payment
               (company_id, vendor_id, bank_account_id, payment_date, amount_minor)
               VALUES (1, ?1, ?2, '2026-03-02', ?3)"#,
            rusqlite::params![vendor_id, bank_account_id, amount_minor],
        )
        .expect("vendor payment");
        conn.last_insert_rowid()
    }

    #[test]
    fn post_invoice_once_and_journal_balances() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("post.sqlite");
        let inv_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            seed_sent_invoice(&mut c)
        };

        let mut conn = open_sqlite(&p).expect("reopen");
        let jid = post_invoice(&mut conn, inv_id).expect("post");
        assert!(jid > 0);
        assert!(post_invoice(&mut conn, inv_id).is_err());

        let (dr, cr): (i64, i64) = conn
            .query_row(
                r#"SELECT COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0)
                   FROM journal_line WHERE journal_id = ?1"#,
                [jid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("sums");
        assert_eq!(dr, cr);
        assert_eq!(dr, 5000);
    }

    #[test]
    fn post_invoice_credits_tax_to_sales_tax_payable() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("post_tax.sqlite");
        let inv_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            c.execute(
                "INSERT INTO customer (company_id, display_name) VALUES (1, 'Taxed Co')",
                [],
            )
            .unwrap();
            let cust = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO invoice (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
                   VALUES (1, ?1, 'INV-TAX', 'sent', '2026-01-20', 1000, 80, 1080)"#,
                [cust],
            )
            .unwrap();
            let inv = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO invoice_line (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id)
                   VALUES (?1, 1, 'Item', 1, 1000, 1000,
                    (SELECT id FROM account WHERE company_id = 1 AND code = '4000'))"#,
                [inv],
            )
            .unwrap();
            inv
        };

        let mut conn = open_sqlite(&p).expect("reopen");
        let jid = post_invoice(&mut conn, inv_id).expect("post");
        let tax_credit: i64 = conn
            .query_row(
                r#"SELECT jl.credit_minor FROM journal_line jl
                   JOIN account a ON a.id = jl.account_id
                   WHERE jl.journal_id = ?1 AND a.code = '2100'"#,
                [jid],
                |row| row.get(0),
            )
            .expect("tax payable credit");
        assert_eq!(tax_credit, 80);
        let sales_credit: i64 = conn
            .query_row(
                r#"SELECT jl.credit_minor FROM journal_line jl
                   JOIN account a ON a.id = jl.account_id
                   WHERE jl.journal_id = ?1 AND a.code = '4000'"#,
                [jid],
                |row| row.get(0),
            )
            .expect("sales credit");
        assert_eq!(sales_credit, 1000);
    }

    #[test]
    fn post_invoice_fails_when_draft() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("draft.sqlite");
        let inv_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            c.execute(
                "INSERT INTO customer (company_id, display_name) VALUES (1, 'Solo')",
                [],
            )
            .unwrap();
            let cust = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO invoice (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
                   VALUES (1, ?1, 'D-1', 'draft', '2026-01-01', 100, 0, 100)"#,
                [cust],
            )
            .unwrap();
            let inv = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO invoice_line (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor)
                   VALUES (?1, 1, 'X', 1, 100, 100)"#,
                [inv],
            )
            .unwrap();
            inv
        };
        let mut conn = open_sqlite(&p).expect("open");
        let err = post_invoice(&mut conn, inv_id);
        assert!(err.is_err());
    }

    #[test]
    fn post_open_bill_creates_payable_credit() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("bill.sqlite");
        let bill_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            let exp5000: i64 = c
                .query_row(
                    "SELECT id FROM account WHERE company_id = 1 AND code = '5000'",
                    [],
                    |row| row.get(0),
                )
                .expect("exp account");
            c.execute(
                "INSERT INTO vendor (company_id, display_name) VALUES (1, 'Bill Vendor')",
                [],
            )
            .expect("vendor");
            let vendor_id = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO bill (company_id, vendor_id, number, status, issue_date, total_minor)
                   VALUES (1, ?1, 'B-77', 'open', '2026-02-01', 1200)"#,
                [vendor_id],
            )
            .expect("bill");
            let bid = c.last_insert_rowid();
            c.execute(
                r#"INSERT INTO bill_line (bill_id, line_number, description, amount_minor, expense_account_id)
                   VALUES (?1, 1, 'Supplies', 1200, ?2)"#,
                rusqlite::params![bid, exp5000],
            )
            .expect("bill line");
            bid
        };
        let mut conn = open_sqlite(&p).expect("open");
        let jid = post_bill(&mut conn, bill_id).expect("post bill");
        let (dr, cr): (i64, i64) = conn
            .query_row(
                r#"SELECT COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0)
                   FROM journal_line WHERE journal_id = ?1"#,
                [jid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("sums");
        assert_eq!(dr, cr);
        assert_eq!(cr, 1200);
    }

    #[test]
    fn post_customer_payment_rejects_missing_payment_id() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("customer_pay_notfound.sqlite");
        let mut c = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut c).expect("migrate");
        let mut conn = open_sqlite(&p).expect("open");
        let err = post_customer_payment(&mut conn, 999_999);
        assert!(err.is_err());
    }

    #[test]
    fn post_customer_payment_is_idempotent() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("customer_pay_ok.sqlite");
        let payment_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            let cash_id: i64 = c
                .query_row(
                    "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                    [],
                    |row| row.get(0),
                )
                .expect("cash");
            seed_customer_payment(&c, cash_id, 700)
        };
        let mut conn = open_sqlite(&p).expect("open");
        let jid = post_customer_payment(&mut conn, payment_id).expect("post once");
        assert!(jid > 0);
        assert!(post_customer_payment(&mut conn, payment_id).is_err());
    }

    #[test]
    fn post_vendor_payment_rejects_missing_payment_id() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("vendor_pay_notfound.sqlite");
        let mut c = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut c).expect("migrate");
        let mut conn = open_sqlite(&p).expect("open");
        let err = post_vendor_payment(&mut conn, 999_999);
        assert!(err.is_err());
    }

    #[test]
    fn post_vendor_payment_balances_journal() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("vendor_pay_ok.sqlite");
        let payment_id = {
            let mut c = open_sqlite(&p).expect("open");
            run_all_on_connection(&mut c).expect("migrate");
            let cash_id: i64 = c
                .query_row(
                    "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                    [],
                    |row| row.get(0),
                )
                .expect("cash");
            seed_vendor_payment(&c, cash_id, 350)
        };
        let mut conn = open_sqlite(&p).expect("open");
        let jid = post_vendor_payment(&mut conn, payment_id).expect("post vendor payment");
        let (dr, cr): (i64, i64) = conn
            .query_row(
                r#"SELECT COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0)
                   FROM journal_line WHERE journal_id = ?1"#,
                [jid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("sums");
        assert_eq!(dr, cr);
        assert_eq!(dr, 350);
    }
}

pub fn post_bill(conn: &mut Connection, bill_id: i64) -> Result<i64, DbCommandError> {
    let row: Option<BillPostRow> = conn
        .query_row(
            r#"SELECT status, journal_id, total_minor, issue_date, vendor_id
               FROM bill WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![bill_id, COMPANY_ID],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;

    let Some((status, journal_id, total, issue_date, vendor_id)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "bill".into(),
            id: bill_id,
        });
    };

    if journal_id.is_some() {
        return Err(DbCommandError::Conflict {
            message: "bill already posted".into(),
        });
    }

    if status != "open" {
        return Err(DbCommandError::Validation {
            message: "only open bills can be posted".into(),
        });
    }

    if vendor_id.is_none() {
        return Err(DbCommandError::Validation {
            message: "assign a vendor before posting a bill (payee-only bills cannot hit AP)"
                .into(),
        });
    }

    let ap_id = account_id_by_code(conn, COMPANY_ID, "2000")?;

    let jl: Vec<DraftJournalLine> = {
        let mut stmt = conn.prepare(
            r#"SELECT line_number, description, amount_minor, expense_account_id FROM bill_line WHERE bill_id = ?1 ORDER BY line_number"#,
        )?;
        let rows = stmt.query_map(rusqlite::params![bill_id], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        let mut sum_exp: i64 = 0;
        let mut expense_lines: Vec<(i32, String, i64, i64)> = Vec::new();

        for r in rows {
            let (ln, desc, amt, exp_acc) = r?;
            if amt <= 0 {
                return Err(DbCommandError::Validation {
                    message: format!("bill line {ln} amount must be greater than zero"),
                });
            }
            sum_exp = sum_exp.saturating_add(amt);
            if !account_exists(conn, COMPANY_ID, exp_acc)? {
                return Err(DbCommandError::Validation {
                    message: format!("unknown expense account on line {ln}"),
                });
            }
            if !account_has_type(conn, COMPANY_ID, exp_acc, "expense")? {
                return Err(DbCommandError::Validation {
                    message: format!("line {ln} expense account must have account_type=expense"),
                });
            }
            expense_lines.push((ln, desc, amt, exp_acc));
        }

        if sum_exp != total {
            return Err(DbCommandError::Validation {
                message: "bill total does not equal sum of expense lines".into(),
            });
        }

        let mut out = Vec::new();
        let mut n = 1;
        for (_ln, desc, amt, exp_acc) in expense_lines {
            out.push(DraftJournalLine {
                account_id: exp_acc,
                line_number: n,
                description: Some(desc),
                debit_minor: amt,
                credit_minor: 0,
            });
            n += 1;
        }

        out.push(DraftJournalLine {
            account_id: ap_id,
            line_number: n,
            description: Some("Accounts payable".into()),
            debit_minor: 0,
            credit_minor: total,
        });

        out
    };

    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (status_now, journal_now, vendor_now): (String, Option<i64>, Option<i64>) = tx.query_row(
        "SELECT status, journal_id, vendor_id FROM bill WHERE id = ?1 AND company_id = ?2",
        rusqlite::params![bill_id, COMPANY_ID],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    if journal_now.is_some() {
        return Err(DbCommandError::Conflict {
            message: "bill already posted".into(),
        });
    }
    if status_now != "open" {
        return Err(DbCommandError::Validation {
            message: "only open bills can be posted".into(),
        });
    }
    if vendor_now.is_none() {
        return Err(DbCommandError::Validation {
            message: "assign a vendor before posting a bill (payee-only bills cannot hit AP)"
                .into(),
        });
    }

    let jid = insert_journal(
        &tx,
        COMPANY_ID,
        &issue_date,
        Some(&format!("Bill #{bill_id}")),
        "bill",
        bill_id,
        &jl,
    )?;

    let n = tx.execute(
        r#"UPDATE bill SET journal_id = ?1, updated_at = datetime('now')
           WHERE id = ?2 AND company_id = ?3 AND status = 'open' AND journal_id IS NULL"#,
        rusqlite::params![jid, bill_id, COMPANY_ID],
    )?;
    if n == 0 {
        return Err(DbCommandError::Conflict {
            message: "bill status changed concurrently; post aborted".into(),
        });
    }
    tx.commit()?;
    log::info!(
        target: "kwikbooks_lib::domain::posting",
        "bill_posted bill_id={} journal_id={}",
        bill_id,
        jid
    );
    Ok(jid)
}

pub fn post_customer_payment(
    conn: &mut Connection,
    payment_id: i64,
) -> Result<i64, DbCommandError> {
    let row: Option<CustomerPaymentPostRow> = conn
        .query_row(
            r#"SELECT journal_id, bank_account_id, amount_minor, customer_id, payment_date, invoice_id
               FROM customer_payment WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![payment_id, COMPANY_ID],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()?;

    let Some((journal_id, bank_id, amount, cust, pay_date, invoice_id)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "customer_payment".into(),
            id: payment_id,
        });
    };

    if journal_id.is_some() {
        return Err(DbCommandError::Conflict {
            message: "payment already posted".into(),
        });
    }

    require_bank_cash_account(conn, COMPANY_ID, bank_id)?;
    if let Some(inv_id) = invoice_id {
        crate::domain::lifecycle::assert_invoice_payment_link(
            conn,
            inv_id,
            cust,
            Some(payment_id),
        )?;
    }

    let ar_id = account_id_by_code(conn, COMPANY_ID, "1100")?;

    let jl = vec![
        DraftJournalLine {
            account_id: bank_id,
            line_number: 1,
            description: Some("Customer payment".into()),
            debit_minor: amount,
            credit_minor: 0,
        },
        DraftJournalLine {
            account_id: ar_id,
            line_number: 2,
            description: Some("Apply to AR".into()),
            debit_minor: 0,
            credit_minor: amount,
        },
    ];

    let tx = conn.transaction()?;
    let jid = insert_journal(
        &tx,
        COMPANY_ID,
        &pay_date,
        Some(&format!("Customer payment #{payment_id}")),
        "payment_customer",
        payment_id,
        &jl,
    )?;
    tx.execute(
        "UPDATE customer_payment SET journal_id = ?1 WHERE id = ?2",
        rusqlite::params![jid, payment_id],
    )?;
    tx.commit()?;
    log::info!(
        target: "kwikbooks_lib::domain::posting",
        "customer_payment_posted payment_id={} journal_id={}",
        payment_id,
        jid
    );
    Ok(jid)
}

pub fn post_vendor_payment(conn: &mut Connection, payment_id: i64) -> Result<i64, DbCommandError> {
    let row: Option<VendorPaymentPostRow> = conn
        .query_row(
            r#"SELECT journal_id, bank_account_id, amount_minor, payment_date, vendor_id, bill_id
               FROM vendor_payment WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![payment_id, COMPANY_ID],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()?;

    let Some((journal_id, bank_id, amount, pay_date, vendor_id, bill_id)) = row else {
        return Err(DbCommandError::NotFound {
            entity: "vendor_payment".into(),
            id: payment_id,
        });
    };

    if journal_id.is_some() {
        return Err(DbCommandError::Conflict {
            message: "payment already posted".into(),
        });
    }

    require_bank_cash_account(conn, COMPANY_ID, bank_id)?;
    if let Some(bid) = bill_id {
        crate::domain::lifecycle::assert_bill_payment_link(conn, bid, vendor_id, Some(payment_id))?;
    }

    let ap_id = account_id_by_code(conn, COMPANY_ID, "2000")?;

    let jl = vec![
        DraftJournalLine {
            account_id: ap_id,
            line_number: 1,
            description: Some("Vendor payment".into()),
            debit_minor: amount,
            credit_minor: 0,
        },
        DraftJournalLine {
            account_id: bank_id,
            line_number: 2,
            description: Some("Pay from bank".into()),
            debit_minor: 0,
            credit_minor: amount,
        },
    ];

    let tx = conn.transaction()?;
    let jid = insert_journal(
        &tx,
        COMPANY_ID,
        &pay_date,
        Some(&format!("Vendor payment #{payment_id}")),
        "payment_vendor",
        payment_id,
        &jl,
    )?;
    tx.execute(
        "UPDATE vendor_payment SET journal_id = ?1 WHERE id = ?2",
        rusqlite::params![jid, payment_id],
    )?;
    tx.commit()?;
    log::info!(
        target: "kwikbooks_lib::domain::posting",
        "vendor_payment_posted payment_id={} journal_id={}",
        payment_id,
        jid
    );
    Ok(jid)
}
