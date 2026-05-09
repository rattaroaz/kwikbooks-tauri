use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

pub fn trial_balance(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT a.id, a.code, a.name, a.account_type,
                  COALESCE(SUM(jl.debit_minor), 0) AS dr,
                  COALESCE(SUM(jl.credit_minor), 0) AS cr
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
             AND j.entry_date >= ?1 AND j.entry_date <= ?2
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
        out.push(r?);
    }
    Ok(out)
}

pub fn general_ledger(
    conn: &Connection,
    account_id: i64,
    date_from: &str,
    date_to: &str,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
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

/// Open AR: invoices in `sent` status (unpaid recognition for MVP).
pub fn ar_open_by_customer(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT c.id, c.display_name,
                  COALESCE(SUM(i.total_minor), 0)
           FROM customer c
           INNER JOIN invoice i ON i.customer_id = c.id
             AND i.company_id = c.company_id
             AND i.status = 'sent'
           WHERE c.company_id = ?1
           GROUP BY c.id
           ORDER BY c.display_name"#,
    )?;

    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "customerId": row.get::<_, i64>(0)?,
            "displayName": row.get::<_, String>(1)?,
            "openMinor": row.get::<_, i64>(2)?,
        }))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Open AP: bills in `open` status.
pub fn ap_open_by_vendor(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT v.id, v.display_name,
                  COALESCE(SUM(b.total_minor), 0)
           FROM vendor v
           INNER JOIN bill b ON b.vendor_id = v.id
             AND b.company_id = v.company_id
             AND b.status = 'open'
           WHERE v.company_id = ?1
           GROUP BY v.id
           ORDER BY v.display_name"#,
    )?;

    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "vendorId": row.get::<_, i64>(0)?,
            "displayName": row.get::<_, String>(1)?,
            "openMinor": row.get::<_, i64>(2)?,
        }))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Income statement: revenue (credit-normal) minus expenses for the date range.
pub fn profit_and_loss(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
) -> Result<serde_json::Value, DbCommandError> {
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
                  COALESCE(SUM(jl.debit_minor), 0),
                  COALESCE(SUM(jl.credit_minor), 0)
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
             AND j.entry_date >= ?1 AND j.entry_date <= ?2
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
        out.push(r?);
    }
    Ok(out)
}

/// Balance sheet balances as of date (inclusive): cumulative posting through `as_of_date`.
pub fn balance_sheet(conn: &Connection, as_of_date: &str) -> Result<serde_json::Value, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT a.id, a.code, a.name, a.account_type,
                  COALESCE(SUM(jl.debit_minor), 0),
                  COALESCE(SUM(jl.credit_minor), 0)
           FROM account a
           LEFT JOIN journal_line jl ON jl.account_id = a.id
           LEFT JOIN journal j ON j.id = jl.journal_id
             AND j.company_id = a.company_id
             AND j.entry_date <= ?1
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

    Ok(serde_json::json!({
        "asOfDate": as_of_date,
        "assets": assets,
        "liabilities": liabilities,
        "equity": equity,
        "totalAssetsMinor": ta,
        "totalLiabilitiesMinor": tl,
        "totalEquityMinor": te,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all_on_connection};
    use tempfile::tempdir;

    fn seed_ledger(conn: &Connection) {
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

        // Journal #1: invoice posting effect (Dr AR 1000, Cr Sales 1000)
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-10', 'Inv', 'invoice', 1)",
            [],
        )
        .expect("journal1");
        let j1 = conn.last_insert_rowid();
        let ar: i64 = conn
            .query_row("SELECT id FROM account WHERE company_id = 1 AND code = '1100'", [], |r| r.get(0))
            .expect("ar");
        let sales: i64 = conn
            .query_row("SELECT id FROM account WHERE company_id = 1 AND code = '4000'", [], |r| r.get(0))
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

        // Journal #2: bill posting effect (Dr Expense 250, Cr AP 250)
        conn.execute(
            "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (1, '2026-01-11', 'Bill', 'bill', 1)",
            [],
        )
        .expect("journal2");
        let j2 = conn.last_insert_rowid();
        let expense: i64 = conn
            .query_row("SELECT id FROM account WHERE company_id = 1 AND code = '5000'", [], |r| r.get(0))
            .expect("expense");
        let ap: i64 = conn
            .query_row("SELECT id FROM account WHERE company_id = 1 AND code = '2000'", [], |r| r.get(0))
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
        assert_eq!(bs["totalEquityMinor"].as_i64(), Some(0));
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
        // Golden expectations for the seeded minimal ledger
        let ar = tb.iter().find(|r| r["code"].as_str() == Some("1100")).expect("AR");
        let sales = tb.iter().find(|r| r["code"].as_str() == Some("4000")).expect("Sales");

        assert_eq!(ar["netMinor"].as_i64(), Some(1000));
        assert_eq!(sales["netMinor"].as_i64(), Some(-1000));
        assert_eq!(tb.len() >= 4, true, "expected at least the core accounts");
    }
}
