use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;
use crate::domain::dates::require_iso_date;

fn require_date_range(date_from: &str, date_to: &str) -> Result<(), DbCommandError> {
    require_iso_date("from date", date_from)?;
    require_iso_date("to date", date_to)?;
    if date_from > date_to {
        return Err(DbCommandError::Validation {
            message: "from date must be on or before to date".into(),
        });
    }
    Ok(())
}

pub fn trial_balance(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    require_date_range(date_from, date_to)?;
    let mut stmt = conn.prepare(
        r#"SELECT a.id, a.code, a.name, a.account_type,
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL
                     AND j.entry_date >= ?1 AND j.entry_date <= ?2
                    THEN jl.debit_minor ELSE 0 END), 0) AS dr,
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL
                     AND j.entry_date >= ?1 AND j.entry_date <= ?2
                    THEN jl.credit_minor ELSE 0 END), 0) AS cr
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
           WHERE a.company_id = ?3
           GROUP BY a.id
           ORDER BY a.sort_order, a.code"#,
    )?;

    let rows = stmt.query_map(rusqlite::params![date_from, date_to, COMPANY_ID], |row| {
        let dr: i64 = row.get(4)?;
        let cr: i64 = row.get(5)?;
        Ok(serde_json::json!({
            "accountId": row.get::<_, i64>(0)?,
            "code": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "accountType": row.get::<_, String>(3)?,
            "debitMinor": dr,
            "creditMinor": cr,
            "netMinor": dr - cr,
        }))
    })?;

    let mut out = Vec::new();
    for r in rows {
        let v = r?;
        let dr = v.get("debitMinor").and_then(|x| x.as_i64()).unwrap_or(0);
        let cr = v.get("creditMinor").and_then(|x| x.as_i64()).unwrap_or(0);
        if dr != 0 || cr != 0 {
            out.push(v);
        }
    }
    Ok(out)
}

pub fn general_ledger(
    conn: &Connection,
    account_id: i64,
    date_from: &str,
    date_to: &str,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    require_date_range(date_from, date_to)?;
    let mut stmt = conn.prepare(
        r#"SELECT j.entry_date, j.memo, jl.line_number, jl.description,
                  jl.debit_minor, jl.credit_minor, j.id AS journal_id, j.source_kind
           FROM journal_line jl
           JOIN journal j ON j.id = jl.journal_id
           WHERE jl.account_id = ?1 AND j.company_id = ?2
             AND j.entry_date >= ?3 AND j.entry_date <= ?4
           ORDER BY j.entry_date, j.id, jl.line_number"#,
    )?;

    let rows = stmt.query_map(
        rusqlite::params![account_id, COMPANY_ID, date_from, date_to],
        |row| {
            Ok(serde_json::json!({
                "entryDate": row.get::<_, String>(0)?,
                "memo": row.get::<_, Option<String>>(1)?,
                "lineNumber": row.get::<_, i32>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "debitMinor": row.get::<_, i64>(4)?,
                "creditMinor": row.get::<_, i64>(5)?,
                "journalId": row.get::<_, i64>(6)?,
                "sourceKind": row.get::<_, Option<String>>(7)?,
            }))
        },
    )?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Open AR by customer for **posted** invoices (signed: negative = credit balance).
/// Subtracts linked posted payments per invoice and unallocated posted payments per customer.
pub fn ar_open_by_customer(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    use std::collections::BTreeMap;

    let mut stmt = conn.prepare(
        r#"SELECT i.id, i.customer_id, c.display_name, i.total_minor
           FROM invoice i
           INNER JOIN customer c ON c.id = i.customer_id AND c.company_id = i.company_id
           WHERE i.company_id = ?1
             AND i.journal_id IS NOT NULL
             AND i.status NOT IN ('draft', 'void')
           ORDER BY c.display_name, i.id"#,
    )?;
    let invoices = stmt.query_map([COMPANY_ID], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;

    let mut open: BTreeMap<i64, (String, i64)> = BTreeMap::new();
    for row in invoices {
        let (inv_id, cust_id, name, total) = row?;
        let paid: i64 = conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM customer_payment
               WHERE invoice_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
            rusqlite::params![inv_id, COMPANY_ID],
            |r| r.get(0),
        )?;
        let entry = open.entry(cust_id).or_insert((name, 0));
        entry.1 = entry.1.saturating_add(total.saturating_sub(paid));
    }

    let mut unalloc = conn.prepare(
        r#"SELECT customer_id, COALESCE(SUM(amount_minor), 0)
           FROM customer_payment
           WHERE company_id = ?1 AND journal_id IS NOT NULL AND invoice_id IS NULL
           GROUP BY customer_id"#,
    )?;
    let unalloc_rows = unalloc.query_map([COMPANY_ID], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in unalloc_rows {
        let (cust_id, amt) = row?;
        if let Some(entry) = open.get_mut(&cust_id) {
            entry.1 = entry.1.saturating_sub(amt);
        } else {
            let name: String = conn.query_row(
                "SELECT display_name FROM customer WHERE id = ?1",
                [cust_id],
                |r| r.get(0),
            )?;
            open.insert(cust_id, (name, amt.saturating_neg()));
        }
    }

    let mut out: Vec<serde_json::Value> = open
        .into_iter()
        .filter(|(_, (_, bal))| *bal != 0)
        .map(|(customer_id, (display_name, open_minor))| {
            serde_json::json!({
                "customerId": customer_id,
                "displayName": display_name,
                "openMinor": open_minor,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a["displayName"]
            .as_str()
            .unwrap_or("")
            .cmp(b["displayName"].as_str().unwrap_or(""))
    });
    Ok(out)
}

/// Open AP for posted bills (vendors + payee-only). Signed: negative = credit.
pub fn ap_open_by_vendor(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    use std::collections::BTreeMap;

    #[derive(Ord, PartialOrd, Eq, PartialEq)]
    enum ApKey {
        Vendor(i64),
        PayeeBill(i64),
    }

    let mut stmt = conn.prepare(
        r#"SELECT b.id, b.vendor_id, b.payee_name, v.display_name, b.total_minor
           FROM bill b
           LEFT JOIN vendor v ON v.id = b.vendor_id AND v.company_id = b.company_id
           WHERE b.company_id = ?1
             AND b.journal_id IS NOT NULL
             AND b.status NOT IN ('draft', 'void')
           ORDER BY b.id"#,
    )?;
    let bills = stmt.query_map([COMPANY_ID], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<i64>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;

    let mut open: BTreeMap<ApKey, (Option<i64>, String, i64)> = BTreeMap::new();
    for row in bills {
        let (bill_id, vendor_id, payee_name, vendor_name, total) = row?;
        let paid: i64 = conn.query_row(
            r#"SELECT COALESCE(SUM(amount_minor), 0) FROM vendor_payment
               WHERE bill_id = ?1 AND company_id = ?2 AND journal_id IS NOT NULL"#,
            rusqlite::params![bill_id, COMPANY_ID],
            |r| r.get(0),
        )?;
        let remaining = total.saturating_sub(paid);
        let (key, display) = match vendor_id {
            Some(vid) => (
                ApKey::Vendor(vid),
                vendor_name.unwrap_or_else(|| format!("Vendor #{vid}")),
            ),
            None => (
                ApKey::PayeeBill(bill_id),
                payee_name
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| format!("Payee bill #{bill_id}")),
            ),
        };
        let entry = open.entry(key).or_insert((vendor_id, display, 0));
        entry.2 = entry.2.saturating_add(remaining);
    }

    let mut unalloc = conn.prepare(
        r#"SELECT vendor_id, COALESCE(SUM(amount_minor), 0)
           FROM vendor_payment
           WHERE company_id = ?1 AND journal_id IS NOT NULL AND bill_id IS NULL
           GROUP BY vendor_id"#,
    )?;
    let unalloc_rows = unalloc.query_map([COMPANY_ID], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in unalloc_rows {
        let (vendor_id, amt) = row?;
        let key = ApKey::Vendor(vendor_id);
        if let Some(entry) = open.get_mut(&key) {
            entry.2 = entry.2.saturating_sub(amt);
        } else {
            let name: String = conn.query_row(
                "SELECT display_name FROM vendor WHERE id = ?1",
                [vendor_id],
                |r| r.get(0),
            )?;
            open.insert(key, (Some(vendor_id), name, amt.saturating_neg()));
        }
    }

    let mut out: Vec<serde_json::Value> = open
        .into_iter()
        .filter(|(_, (_, _, bal))| *bal != 0)
        .map(|(_, (vendor_id, display_name, open_minor))| {
            serde_json::json!({
                "vendorId": vendor_id,
                "displayName": display_name,
                "openMinor": open_minor,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a["displayName"]
            .as_str()
            .unwrap_or("")
            .cmp(b["displayName"].as_str().unwrap_or(""))
    });
    Ok(out)
}

/// Income statement: revenue (credit-normal) minus expenses for the date range.
pub fn profit_and_loss(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
) -> Result<serde_json::Value, DbCommandError> {
    require_date_range(date_from, date_to)?;
    let income_lines = pl_section(
        conn,
        date_from,
        date_to,
        "income",
        |dr: i64, cr: i64| cr - dr,
    )?;
    let expense_lines = pl_section(
        conn,
        date_from,
        date_to,
        "expense",
        |dr: i64, cr: i64| dr - cr,
    )?;

    let total_income: i64 = income_lines
        .iter()
        .filter_map(|v| v.get("amountMinor").and_then(|x| x.as_i64()))
        .sum();
    let total_expense: i64 = expense_lines
        .iter()
        .filter_map(|v| v.get("amountMinor").and_then(|x| x.as_i64()))
        .sum();

    Ok(serde_json::json!({
        "dateFrom": date_from,
        "dateTo": date_to,
        "incomeLines": income_lines,
        "expenseLines": expense_lines,
        "totalIncomeMinor": total_income,
        "totalExpenseMinor": total_expense,
        "netIncomeMinor": total_income - total_expense,
    }))
}

fn pl_section(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
    account_type: &str,
    net_fn: fn(i64, i64) -> i64,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT a.id, a.code, a.name,
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL
                     AND j.entry_date >= ?1 AND j.entry_date <= ?2
                    THEN jl.debit_minor ELSE 0 END), 0),
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL
                     AND j.entry_date >= ?1 AND j.entry_date <= ?2
                    THEN jl.credit_minor ELSE 0 END), 0)
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
           WHERE a.company_id = ?3 AND a.account_type = ?4
           GROUP BY a.id
           ORDER BY a.sort_order, a.code"#,
    )?;

    let rows = stmt.query_map(
        rusqlite::params![date_from, date_to, COMPANY_ID, account_type],
        |row| {
            let dr: i64 = row.get(3)?;
            let cr: i64 = row.get(4)?;
            let net = net_fn(dr, cr);
            Ok(serde_json::json!({
                "accountId": row.get::<_, i64>(0)?,
                "code": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "amountMinor": net,
            }))
        },
    )?;

    let mut out = Vec::new();
    for r in rows {
        let v = r?;
        let amt = v.get("amountMinor").and_then(|x| x.as_i64()).unwrap_or(0);
        if amt != 0 {
            out.push(v);
        }
    }
    Ok(out)
}

/// Net income through `as_of_date` (inclusive): income (cr−dr) − expense (dr−cr).
fn net_income_as_of(conn: &Connection, as_of_date: &str) -> Result<i64, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT a.account_type,
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL AND j.entry_date <= ?1
                    THEN jl.debit_minor ELSE 0 END), 0),
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL AND j.entry_date <= ?1
                    THEN jl.credit_minor ELSE 0 END), 0)
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
           WHERE a.company_id = ?2 AND a.account_type IN ('income', 'expense')
           GROUP BY a.id, a.account_type"#,
    )?;

    let rows = stmt.query_map(rusqlite::params![as_of_date, COMPANY_ID], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    let mut income: i64 = 0;
    let mut expense: i64 = 0;
    for r in rows {
        let (typ, dr, cr) = r?;
        match typ.as_str() {
            "income" => income = income.saturating_add(cr.saturating_sub(dr)),
            "expense" => expense = expense.saturating_add(dr.saturating_sub(cr)),
            _ => {}
        }
    }
    Ok(income.saturating_sub(expense))
}

/// Balance sheet balances as of date (inclusive): cumulative posting through `as_of_date`.
/// Includes a synthetic equity line for undistributed net income so A = L + E.
pub fn balance_sheet(conn: &Connection, as_of_date: &str) -> Result<serde_json::Value, DbCommandError> {
    require_iso_date("as-of date", as_of_date)?;
    let mut stmt = conn.prepare(
        r#"SELECT a.id, a.code, a.name, a.account_type,
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL AND j.entry_date <= ?1
                    THEN jl.debit_minor ELSE 0 END), 0),
                  COALESCE(SUM(CASE
                    WHEN j.id IS NOT NULL AND j.entry_date <= ?1
                    THEN jl.credit_minor ELSE 0 END), 0)
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
           WHERE a.company_id = ?2 AND a.account_type IN ('asset', 'liability', 'equity')
           GROUP BY a.id
           ORDER BY a.account_type, a.sort_order, a.code"#,
    )?;

    let rows = stmt.query_map(rusqlite::params![as_of_date, COMPANY_ID], |row| {
        let dr: i64 = row.get(4)?;
        let cr: i64 = row.get(5)?;
        let typ: String = row.get::<_, String>(3)?;
        let balance = match typ.as_str() {
            "asset" => dr - cr,
            "liability" | "equity" => cr - dr,
            _ => 0,
        };
        let js = serde_json::json!({
            "accountId": row.get::<_, i64>(0)?,
            "code": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "accountType": &typ,
            "balanceMinor": balance,
        });
        Ok((typ, js))
    })?;

    let mut assets = Vec::new();
    let mut liabilities = Vec::new();
    let mut equity = Vec::new();
    let mut ta: i64 = 0;
    let mut tl: i64 = 0;
    let mut te: i64 = 0;

    for r in rows {
        let (typ, val) = r?;
        let amt = val
            .get("balanceMinor")
            .and_then(|x| x.as_i64())
            .unwrap_or(0);
        match typ.as_str() {
            "asset" => {
                ta = ta.saturating_add(amt);
                assets.push(val);
            }
            "liability" => {
                tl = tl.saturating_add(amt);
                liabilities.push(val);
            }
            "equity" => {
                te = te.saturating_add(amt);
                equity.push(val);
            }
            _ => {}
        }
    }

    let net_income = net_income_as_of(conn, as_of_date)?;
    if net_income != 0 {
        equity.push(serde_json::json!({
            "accountId": null,
            "code": "NI",
            "name": "Net Income",
            "accountType": "equity",
            "balanceMinor": net_income,
            "synthetic": true,
        }));
        te = te.saturating_add(net_income);
    }

    Ok(serde_json::json!({
        "asOfDate": as_of_date,
        "assets": assets,
        "liabilities": liabilities,
        "equity": equity,
        "totalAssetsMinor": ta,
        "totalLiabilitiesMinor": tl,
        "totalEquityMinor": te,
        "netIncomeMinor": net_income,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all_on_connection};
    use tempfile::tempdir;

    fn seed_ledger(conn: &Connection) -> (i64, i64) {
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, 'A Customer')",
            [],
        )
        .expect("customer");
        let customer_id = conn.last_insert_rowid();

        conn.execute(
            r#"INSERT INTO invoice
               (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, 'RPT-INV-1', 'sent', '2026-01-10', 1000, 0, 1000)"#,
            [customer_id],
        )
        .expect("invoice");
        let invoice_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO vendor (company_id, display_name) VALUES (1, 'A Vendor')",
            [],
        )
        .expect("vendor");
        let vendor_id = conn.last_insert_rowid();

        conn.execute(
            r#"INSERT INTO bill
               (company_id, vendor_id, number, status, issue_date, total_minor)
               VALUES (1, ?1, 'RPT-BILL-1', 'open', '2026-01-11', 250)"#,
            [vendor_id],
        )
        .expect("bill");
        let bill_id = conn.last_insert_rowid();

        // Journal #1: invoice posting effect (Dr AR 1000, Cr Sales 1000)
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-10', 'Inv', 'invoice', 1)",
            [],
        )
        .expect("journal1");
        let j1 = conn.last_insert_rowid();
        let ar: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1100'",
                [],
                |r| r.get(0),
            )
            .expect("ar");
        let sales: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '4000'",
                [],
                |r| r.get(0),
            )
            .expect("sales");
        conn.execute(
            "INSERT INTO journal_line (journal_id, account_id, line_number, description, debit_minor, credit_minor) VALUES (?1, ?2, 1, 'AR', 1000, 0)",
            rusqlite::params![j1, ar],
        )
        .expect("j1l1");
        conn.execute(
            "INSERT INTO journal_line (journal_id, account_id, line_number, description, debit_minor, credit_minor) VALUES (?1, ?2, 2, 'Sales', 0, 1000)",
            rusqlite::params![j1, sales],
        )
        .expect("j1l2");
        conn.execute(
            "UPDATE invoice SET journal_id = ?1 WHERE id = ?2",
            rusqlite::params![j1, invoice_id],
        )
        .expect("link invoice journal");

        // Journal #2: bill posting effect (Dr Expense 250, Cr AP 250)
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-11', 'Bill', 'bill', 1)",
            [],
        )
        .expect("journal2");
        let j2 = conn.last_insert_rowid();
        let expense: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '5000'",
                [],
                |r| r.get(0),
            )
            .expect("expense");
        let ap: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '2000'",
                [],
                |r| r.get(0),
            )
            .expect("ap");
        conn.execute(
            "INSERT INTO journal_line (journal_id, account_id, line_number, description, debit_minor, credit_minor) VALUES (?1, ?2, 1, 'Exp', 250, 0)",
            rusqlite::params![j2, expense],
        )
        .expect("j2l1");
        conn.execute(
            "INSERT INTO journal_line (journal_id, account_id, line_number, description, debit_minor, credit_minor) VALUES (?1, ?2, 2, 'AP', 0, 250)",
            rusqlite::params![j2, ap],
        )
        .expect("j2l2");
        conn.execute(
            "UPDATE bill SET journal_id = ?1 WHERE id = ?2",
            rusqlite::params![j2, bill_id],
        )
        .expect("link bill journal");

        (invoice_id, bill_id)
    }

    #[test]
    fn ar_ap_open_summaries_match_document_statuses() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_ar_ap.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let ar = ar_open_by_customer(&conn).expect("ar");
        let ap = ap_open_by_vendor(&conn).expect("ap");
        assert_eq!(ar.len(), 1);
        assert_eq!(ap.len(), 1);
        assert_eq!(ar[0]["openMinor"].as_i64(), Some(1000));
        assert_eq!(ap[0]["openMinor"].as_i64(), Some(250));
    }

    #[test]
    fn ar_open_includes_unlinked_payments_and_overpay_credit() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_ar_unlinked.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (invoice_id, _) = seed_ledger(&conn);
        let bank: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let customer_id: i64 = conn
            .query_row("SELECT id FROM customer WHERE company_id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();

        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-20', 'payment_customer', 2)",
            [],
        )
        .unwrap();
        let j_unlinked = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-20', 200, ?3, NULL)"#,
            rusqlite::params![customer_id, bank, j_unlinked],
        )
        .unwrap();

        let ar = ar_open_by_customer(&conn).expect("ar");
        assert_eq!(ar[0]["openMinor"].as_i64(), Some(800)); // 1000 - 200 unlinked

        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-21', 'payment_customer', 3)",
            [],
        )
        .unwrap();
        let j_over = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-21', 1500, ?3, ?4)"#,
            rusqlite::params![customer_id, bank, j_over, invoice_id],
        )
        .unwrap();
        let ar2 = ar_open_by_customer(&conn).expect("ar over");
        // 1000 - 1500 linked - 200 unlinked = -700 credit
        assert_eq!(ar2[0]["openMinor"].as_i64(), Some(-700));
    }

    #[test]
    fn ap_open_includes_payee_only_bills() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_ap_payee.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        conn.execute(
            r#"INSERT INTO bill
               (company_id, vendor_id, payee_name, number, status, issue_date, total_minor)
               VALUES (1, NULL, 'One-off Payee', 'PAYEE-1', 'open', '2026-01-12', 300)"#,
            [],
        )
        .unwrap();
        let bill_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, source_kind, source_id) VALUES (1, '2026-01-12', 'bill', 2)",
            [],
        )
        .unwrap();
        let jid = conn.last_insert_rowid();
        conn.execute(
            "UPDATE bill SET journal_id = ?1 WHERE id = ?2",
            rusqlite::params![jid, bill_id],
        )
        .unwrap();

        let ap = ap_open_by_vendor(&conn).expect("ap");
        assert!(
            ap.iter()
                .any(|r| r["displayName"].as_str() == Some("One-off Payee")
                    && r["openMinor"].as_i64() == Some(300)),
            "payee-only bill should appear: {ap:?}"
        );
    }

    #[test]
    fn ar_ap_open_subtract_posted_payments() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_ar_ap_pay.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        let (invoice_id, bill_id) = seed_ledger(&conn);

        let bank: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1000'",
                [],
                |r| r.get(0),
            )
            .expect("cash");
        let customer_id: i64 = conn
            .query_row("SELECT id FROM customer WHERE company_id = 1", [], |r| {
                r.get(0)
            })
            .expect("customer");
        let vendor_id: i64 = conn
            .query_row("SELECT id FROM vendor WHERE company_id = 1", [], |r| {
                r.get(0)
            })
            .expect("vendor");

        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-15', 'CPay', 'payment_customer', 1)",
            [],
        )
        .expect("jpay");
        let j_cp = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, journal_id, invoice_id)
               VALUES (1, ?1, ?2, '2026-01-15', 400, ?3, ?4)"#,
            rusqlite::params![customer_id, bank, j_cp, invoice_id],
        )
        .expect("cpay");

        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-16', 'VPay', 'payment_vendor', 1)",
            [],
        )
        .expect("jvpay");
        let j_vp = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO vendor_payment
               (company_id, vendor_id, bank_account_id, payment_date, amount_minor, journal_id, bill_id)
               VALUES (1, ?1, ?2, '2026-01-16', 100, ?3, ?4)"#,
            rusqlite::params![vendor_id, bank, j_vp, bill_id],
        )
        .expect("vpay");

        let ar = ar_open_by_customer(&conn).expect("ar");
        let ap = ap_open_by_vendor(&conn).expect("ap");
        assert_eq!(ar[0]["openMinor"].as_i64(), Some(600));
        assert_eq!(ap[0]["openMinor"].as_i64(), Some(150));
    }

    #[test]
    fn profit_and_loss_and_balance_sheet_totals_are_consistent() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_pl_bs.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let pl = profit_and_loss(&conn, "2026-01-01", "2026-01-31").expect("p/l");
        assert_eq!(pl["totalIncomeMinor"].as_i64(), Some(1000));
        assert_eq!(pl["totalExpenseMinor"].as_i64(), Some(250));
        assert_eq!(pl["netIncomeMinor"].as_i64(), Some(750));

        let bs = balance_sheet(&conn, "2026-01-31").expect("b/s");
        assert_eq!(bs["totalAssetsMinor"].as_i64(), Some(1000));
        assert_eq!(bs["totalLiabilitiesMinor"].as_i64(), Some(250));
        assert_eq!(bs["netIncomeMinor"].as_i64(), Some(750));
        assert_eq!(bs["totalEquityMinor"].as_i64(), Some(750));
        assert_eq!(
            bs["totalAssetsMinor"].as_i64(),
            Some(
                bs["totalLiabilitiesMinor"].as_i64().unwrap()
                    + bs["totalEquityMinor"].as_i64().unwrap()
            )
        );
    }

    #[test]
    fn trial_balance_and_pl_respect_date_range_filter() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_date_filter.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let tb_feb = trial_balance(&conn, "2026-02-01", "2026-02-28").expect("tb feb");
        assert!(
            tb_feb
                .iter()
                .all(|r| r["code"].as_str() != Some("1100")),
            "zero-activity AR should be omitted from period TB"
        );
        assert!(tb_feb.is_empty() || tb_feb.iter().all(|r| {
            r["debitMinor"].as_i64().unwrap_or(0) != 0
                || r["creditMinor"].as_i64().unwrap_or(0) != 0
        }));

        let pl_feb = profit_and_loss(&conn, "2026-02-01", "2026-02-28").expect("pl feb");
        assert_eq!(pl_feb["totalIncomeMinor"].as_i64(), Some(0));
        assert_eq!(pl_feb["totalExpenseMinor"].as_i64(), Some(0));
        assert_eq!(pl_feb["netIncomeMinor"].as_i64(), Some(0));
        assert!(pl_feb["incomeLines"].as_array().unwrap().is_empty());
        assert!(pl_feb["expenseLines"].as_array().unwrap().is_empty());

        let bs_dec = balance_sheet(&conn, "2025-12-31").expect("bs before");
        assert_eq!(bs_dec["totalAssetsMinor"].as_i64(), Some(0));
        assert_eq!(bs_dec["totalLiabilitiesMinor"].as_i64(), Some(0));
        assert_eq!(bs_dec["netIncomeMinor"].as_i64(), Some(0));
    }

    #[test]
    fn trial_balance_has_expected_account_net_values() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_tb.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let tb = trial_balance(&conn, "2026-01-01", "2026-01-31").expect("tb");
        let ar = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("1100"))
            .expect("AR row");
        let ap = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("2000"))
            .expect("AP row");
        let sales = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("4000"))
            .expect("Sales row");
        let expense = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("5000"))
            .expect("Expense row");

        assert_eq!(ar["netMinor"].as_i64(), Some(1000));
        assert_eq!(ap["netMinor"].as_i64(), Some(-250));
        assert_eq!(sales["netMinor"].as_i64(), Some(-1000));
        assert_eq!(expense["netMinor"].as_i64(), Some(250));
    }

    #[test]
    fn general_ledger_respects_date_range_filter() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_gl.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let ar_id: i64 = conn
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = '1100'",
                [],
                |r| r.get(0),
            )
            .expect("ar account");

        let jan_only = general_ledger(&conn, ar_id, "2026-01-10", "2026-01-10").expect("gl jan");
        assert_eq!(jan_only.len(), 1);
        assert_eq!(jan_only[0]["debitMinor"].as_i64(), Some(1000));

        let out_of_range =
            general_ledger(&conn, ar_id, "2026-02-01", "2026-02-28").expect("gl feb");
        assert!(out_of_range.is_empty());
    }

    #[test]
    fn trial_balance_golden_snapshot_matches_expected_nets() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("reports_golden.sqlite");
        let mut conn = open_sqlite(&p).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        seed_ledger(&conn);

        let tb = trial_balance(&conn, "2026-01-01", "2026-12-31").expect("trial balance");
        let ar = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("1100"))
            .expect("AR");
        let sales = tb
            .iter()
            .find(|r| r["code"].as_str() == Some("4000"))
            .expect("Sales");

        assert_eq!(ar["netMinor"].as_i64(), Some(1000));
        assert_eq!(sales["netMinor"].as_i64(), Some(-1000));
        assert!(tb.len() >= 4, "expected at least the core accounts");
    }
}
