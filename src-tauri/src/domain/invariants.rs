//! Regression suite for bookkeeping invariants.
//!
//! These tests encode rules that must hold regardless of how posting or reports
//! are implemented. If a change makes the books disagree with themselves, fail
//! here rather than in a UI screenshot.

use rusqlite::Connection;
use tempfile::TempDir;

use crate::db::{open_sqlite, run_all_on_connection};
use crate::domain::lifecycle::{set_bill_status, set_invoice_status};
use crate::domain::posting::{
    post_bill, post_customer_payment, post_invoice, post_vendor_payment,
};
use crate::domain::reports::{
    ap_open_by_vendor, ar_open_by_customer, balance_sheet, profit_and_loss, trial_balance,
};

struct Books {
    _dir: TempDir,
    path: std::path::PathBuf,
}

impl Books {
    fn new() -> Self {
        let dir = TempDir::new().expect("tmp");
        let path = dir.path().join("invariants.sqlite");
        let mut conn = open_sqlite(&path).expect("open");
        run_all_on_connection(&mut conn).expect("migrate");
        Self { _dir: dir, path }
    }

    fn conn(&self) -> Connection {
        open_sqlite(&self.path).expect("reopen")
    }

    fn acct(&self, code: &str) -> i64 {
        self.conn()
            .query_row(
                "SELECT id FROM account WHERE company_id = 1 AND code = ?1",
                [code],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| panic!("missing account {code}"))
    }

    fn customer(&self, name: &str) -> i64 {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO customer (company_id, display_name) VALUES (1, ?1)",
            [name],
        )
        .expect("customer");
        conn.last_insert_rowid()
    }

    fn vendor(&self, name: &str) -> i64 {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO vendor (company_id, display_name) VALUES (1, ?1)",
            [name],
        )
        .expect("vendor");
        conn.last_insert_rowid()
    }

    fn posted_invoice(
        &self,
        customer_id: i64,
        number: &str,
        issue_date: &str,
        subtotal: i64,
        tax: i64,
    ) -> i64 {
        let sales = self.acct("4000");
        let total = subtotal + tax;
        let conn = self.conn();
        conn.execute(
            r#"INSERT INTO invoice
               (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
               VALUES (1, ?1, ?2, 'sent', ?3, ?4, ?5, ?6)"#,
            rusqlite::params![customer_id, number, issue_date, subtotal, tax, total],
        )
        .expect("invoice");
        let id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO invoice_line
               (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id)
               VALUES (?1, 1, 'Work', 1, ?2, ?2, ?3)"#,
            rusqlite::params![id, subtotal, sales],
        )
        .expect("line");
        drop(conn);
        let mut c = self.conn();
        post_invoice(&mut c, id).expect("post invoice");
        id
    }

    fn posted_bill(
        &self,
        vendor_id: i64,
        number: &str,
        issue_date: &str,
        amount: i64,
    ) -> i64 {
        let exp = self.acct("5000");
        let conn = self.conn();
        conn.execute(
            r#"INSERT INTO bill
               (company_id, vendor_id, number, status, issue_date, total_minor)
               VALUES (1, ?1, ?2, 'open', ?3, ?4)"#,
            rusqlite::params![vendor_id, number, issue_date, amount],
        )
        .expect("bill");
        let id = conn.last_insert_rowid();
        conn.execute(
            r#"INSERT INTO bill_line (bill_id, line_number, description, amount_minor, expense_account_id)
               VALUES (?1, 1, 'Expense', ?2, ?3)"#,
            rusqlite::params![id, amount, exp],
        )
        .expect("bill line");
        drop(conn);
        let mut c = self.conn();
        post_bill(&mut c, id).expect("post bill");
        id
    }

    fn insert_customer_payment(
        &self,
        customer_id: i64,
        bank_id: i64,
        amount: i64,
        date: &str,
        invoice_id: Option<i64>,
    ) -> i64 {
        let conn = self.conn();
        conn.execute(
            r#"INSERT INTO customer_payment
               (company_id, customer_id, bank_account_id, payment_date, amount_minor, invoice_id)
               VALUES (1, ?1, ?2, ?3, ?4, ?5)"#,
            rusqlite::params![customer_id, bank_id, date, amount, invoice_id],
        )
        .expect("customer payment");
        conn.last_insert_rowid()
    }

    fn insert_vendor_payment(
        &self,
        vendor_id: i64,
        bank_id: i64,
        amount: i64,
        date: &str,
        bill_id: Option<i64>,
    ) -> i64 {
        let conn = self.conn();
        conn.execute(
            r#"INSERT INTO vendor_payment
               (company_id, vendor_id, bank_account_id, payment_date, amount_minor, bill_id)
               VALUES (1, ?1, ?2, ?3, ?4, ?5)"#,
            rusqlite::params![vendor_id, bank_id, date, amount, bill_id],
        )
        .expect("vendor payment");
        conn.last_insert_rowid()
    }
}

fn assert_all_journals_balance(conn: &Connection) {
    let mut stmt = conn
        .prepare(
            r#"SELECT journal_id,
                      COALESCE(SUM(debit_minor), 0),
                      COALESCE(SUM(credit_minor), 0)
               FROM journal_line
               GROUP BY journal_id
               HAVING COALESCE(SUM(debit_minor), 0) != COALESCE(SUM(credit_minor), 0)"#,
        )
        .expect("prep");
    let bad: Vec<(i64, i64, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect();
    assert!(
        bad.is_empty(),
        "unbalanced journals (id, debit, credit): {bad:?}"
    );
}

fn gl_debit_minus_credit(conn: &Connection, code: &str, as_of: &str) -> i64 {
    conn.query_row(
        r#"SELECT COALESCE(SUM(jl.debit_minor - jl.credit_minor), 0)
           FROM journal_line jl
           INNER JOIN journal j ON j.id = jl.journal_id
           INNER JOIN account a ON a.id = jl.account_id
           WHERE a.company_id = 1 AND a.code = ?1 AND j.entry_date <= ?2"#,
        rusqlite::params![code, as_of],
        |r| r.get(0),
    )
    .expect("gl net")
}

fn tb_net(conn: &Connection, code: &str, from: &str, to: &str) -> i64 {
    let rows = trial_balance(conn, from, to).expect("tb");
    rows.iter()
        .find(|r| r["code"].as_str() == Some(code))
        .and_then(|r| r["netMinor"].as_i64())
        .unwrap_or(0)
}

fn open_ar_total(conn: &Connection) -> i64 {
    ar_open_by_customer(conn)
        .expect("ar")
        .iter()
        .filter_map(|r| r["openMinor"].as_i64())
        .sum()
}

fn open_ap_total(conn: &Connection) -> i64 {
    ap_open_by_vendor(conn)
        .expect("ap")
        .iter()
        .filter_map(|r| r["openMinor"].as_i64())
        .sum()
}

fn assert_accounting_equation(conn: &Connection, as_of: &str) {
    let bs = balance_sheet(conn, as_of).expect("bs");
    let assets = bs["totalAssetsMinor"].as_i64().unwrap();
    let liab = bs["totalLiabilitiesMinor"].as_i64().unwrap();
    let equity = bs["totalEquityMinor"].as_i64().unwrap();
    assert_eq!(
        assets,
        liab + equity,
        "balance sheet does not balance as of {as_of}: assets={assets} liab={liab} equity={equity}"
    );
}

fn assert_subledger_matches_control(conn: &Connection, as_of: &str) {
    let ar_gl = gl_debit_minus_credit(conn, "1100", as_of);
    assert_eq!(
        open_ar_total(conn),
        ar_gl.max(0),
        "open AR report must match GL AR (asset 1100) as of {as_of}"
    );
    let ap_gl = -gl_debit_minus_credit(conn, "2000", as_of);
    assert_eq!(
        open_ap_total(conn),
        ap_gl.max(0),
        "open AP report must match GL AP (liability 2000) as of {as_of}"
    );
}

#[test]
fn posted_activity_keeps_journals_balanced_and_equation_true() {
    let books = Books::new();
    let cust = books.customer("Acme");
    let vend = books.vendor("Office Co");
    books.posted_invoice(cust, "INV-1", "2026-01-15", 5000, 400);
    books.posted_bill(vend, "BILL-1", "2026-01-16", 1200);

    let conn = books.conn();
    assert_all_journals_balance(&conn);
    assert_accounting_equation(&conn, "2026-01-31");
    assert_subledger_matches_control(&conn, "2026-01-31");
}

#[test]
fn sales_tax_credits_tax_payable_not_revenue() {
    let books = Books::new();
    let cust = books.customer("Taxed");
    books.posted_invoice(cust, "INV-TAX", "2026-02-01", 10000, 800);

    let conn = books.conn();
    assert_eq!(gl_debit_minus_credit(&conn, "4000", "2026-02-01"), -10000);
    assert_eq!(gl_debit_minus_credit(&conn, "2100", "2026-02-01"), -800);
    assert_eq!(gl_debit_minus_credit(&conn, "1100", "2026-02-01"), 10800);
}

#[test]
fn reports_exclude_activity_outside_the_requested_dates() {
    let books = Books::new();
    let cust = books.customer("Dated");
    books.posted_invoice(cust, "INV-JAN", "2026-01-10", 3000, 0);
    books.posted_invoice(cust, "INV-MAR", "2026-03-10", 7000, 0);

    let conn = books.conn();
    assert_eq!(tb_net(&conn, "1100", "2026-01-01", "2026-01-31"), 3000);
    assert_eq!(tb_net(&conn, "1100", "2026-02-01", "2026-02-28"), 0);
    assert_eq!(tb_net(&conn, "1100", "2026-03-01", "2026-03-31"), 7000);

    let jan_pl = profit_and_loss(&conn, "2026-01-01", "2026-01-31").expect("pl jan");
    assert_eq!(jan_pl["totalIncomeMinor"].as_i64(), Some(3000));
    let feb_pl = profit_and_loss(&conn, "2026-02-01", "2026-02-28").expect("pl feb");
    assert_eq!(feb_pl["totalIncomeMinor"].as_i64(), Some(0));

    let bs_jan = balance_sheet(&conn, "2026-01-31").expect("bs jan");
    assert_eq!(bs_jan["totalAssetsMinor"].as_i64(), Some(3000));
    let bs_feb = balance_sheet(&conn, "2026-02-28").expect("bs feb");
    assert_eq!(bs_feb["totalAssetsMinor"].as_i64(), Some(3000));
    let bs_mar = balance_sheet(&conn, "2026-03-31").expect("bs mar");
    assert_eq!(bs_mar["totalAssetsMinor"].as_i64(), Some(10000));
}

#[test]
fn customer_payment_applied_to_invoice_clears_ar_and_marks_paid() {
    let books = Books::new();
    let cust = books.customer("Payer");
    let inv = books.posted_invoice(cust, "INV-PAY", "2026-04-01", 5000, 0);
    let pay = books.insert_customer_payment(cust, books.acct("1000"), 5000, "2026-04-05", Some(inv));
    let mut conn = books.conn();
    post_customer_payment(&mut conn, pay).expect("post payment");

    let status: String = conn
        .query_row("SELECT status FROM invoice WHERE id = ?1", [inv], |r| r.get(0))
        .expect("status");
    assert_eq!(status, "paid");
    assert_eq!(open_ar_total(&conn), 0);
    assert_eq!(gl_debit_minus_credit(&conn, "1100", "2026-04-05"), 0);
    assert_eq!(gl_debit_minus_credit(&conn, "1000", "2026-04-05"), 5000);
    assert_all_journals_balance(&conn);
    assert_accounting_equation(&conn, "2026-04-05");
    assert_subledger_matches_control(&conn, "2026-04-05");
}

#[test]
fn partial_payment_reduces_open_ar_but_leaves_invoice_sent() {
    let books = Books::new();
    let cust = books.customer("Partial");
    let inv = books.posted_invoice(cust, "INV-PART", "2026-05-01", 1000, 0);
    let pay = books.insert_customer_payment(cust, books.acct("1000"), 400, "2026-05-02", Some(inv));
    let mut conn = books.conn();
    post_customer_payment(&mut conn, pay).expect("post partial");

    let status: String = conn
        .query_row("SELECT status FROM invoice WHERE id = ?1", [inv], |r| r.get(0))
        .expect("status");
    assert_eq!(status, "sent");
    assert_eq!(open_ar_total(&conn), 600);
    assert_eq!(gl_debit_minus_credit(&conn, "1100", "2026-05-02"), 600);
    assert_subledger_matches_control(&conn, "2026-05-02");
}

#[test]
fn unapplied_customer_payment_still_reduces_open_ar_to_match_gl() {
    let books = Books::new();
    let cust = books.customer("Unapplied");
    books.posted_invoice(cust, "INV-U", "2026-06-01", 800, 0);
    let pay = books.insert_customer_payment(cust, books.acct("1000"), 800, "2026-06-03", None);
    let mut conn = books.conn();
    post_customer_payment(&mut conn, pay).expect("post unapplied");
    assert_eq!(open_ar_total(&conn), 0);
    assert_eq!(gl_debit_minus_credit(&conn, "1100", "2026-06-03"), 0);
    assert_subledger_matches_control(&conn, "2026-06-03");
}

#[test]
fn vendor_payment_applied_to_bill_clears_ap() {
    let books = Books::new();
    let vend = books.vendor("Supplier");
    let bill = books.posted_bill(vend, "BILL-PAY", "2026-07-01", 2500);
    let pay = books.insert_vendor_payment(vend, books.acct("1000"), 2500, "2026-07-04", Some(bill));
    let mut conn = books.conn();
    post_vendor_payment(&mut conn, pay).expect("post vendor pay");

    let status: String = conn
        .query_row("SELECT status FROM bill WHERE id = ?1", [bill], |r| r.get(0))
        .expect("status");
    assert_eq!(status, "paid");
    assert_eq!(open_ap_total(&conn), 0);
    assert_eq!(gl_debit_minus_credit(&conn, "2000", "2026-07-04"), 0);
    assert_accounting_equation(&conn, "2026-07-04");
    assert_subledger_matches_control(&conn, "2026-07-04");
}

#[test]
fn payment_rejects_wrong_customer_invoice_and_overpay() {
    let books = Books::new();
    let a = books.customer("A");
    let b = books.customer("B");
    let inv_a = books.posted_invoice(a, "INV-A", "2026-08-01", 1000, 0);
    let inv_b = books.posted_invoice(b, "INV-B", "2026-08-01", 1000, 0);
    let cash = books.acct("1000");

    let wrong = books.insert_customer_payment(a, cash, 1000, "2026-08-02", Some(inv_b));
    let mut conn = books.conn();
    assert!(post_customer_payment(&mut conn, wrong).is_err());

    let over = books.insert_customer_payment(a, cash, 1001, "2026-08-02", Some(inv_a));
    assert!(post_customer_payment(&mut conn, over).is_err());
}

#[test]
fn payment_rejects_non_bank_and_inactive_bank_accounts() {
    let books = Books::new();
    let cust = books.customer("BankCheck");
    let sales = books.acct("4000");
    let bad = books.insert_customer_payment(cust, sales, 100, "2026-09-01", None);
    let mut conn = books.conn();
    assert!(post_customer_payment(&mut conn, bad).is_err());

    let cash = books.acct("1000");
    conn.execute(
        "UPDATE account SET is_active = 0 WHERE id = ?1",
        [cash],
    )
    .expect("deactivate");
    drop(conn);
    let inactive = books.insert_customer_payment(cust, cash, 100, "2026-09-01", None);
    let mut conn = books.conn();
    assert!(post_customer_payment(&mut conn, inactive).is_err());
}

#[test]
fn cannot_mark_sent_invoice_paid_without_covering_payment() {
    let books = Books::new();
    let cust = books.customer("NoPay");
    let inv = books.posted_invoice(cust, "INV-NP", "2026-10-01", 500, 0);
    let conn = books.conn();
    let err = set_invoice_status(&conn, inv, "paid");
    assert!(err.is_err());
}

#[test]
fn cannot_void_a_posted_invoice_or_bill() {
    let books = Books::new();
    let cust = books.customer("VoidMe");
    let vend = books.vendor("VoidBill");
    let inv = books.posted_invoice(cust, "INV-V", "2026-11-01", 500, 0);
    let bill = books.posted_bill(vend, "BILL-V", "2026-11-01", 200);
    let conn = books.conn();
    assert!(set_invoice_status(&conn, inv, "void").is_err());
    assert!(set_bill_status(&conn, bill, "void").is_err());
    assert_eq!(gl_debit_minus_credit(&conn, "1100", "2026-11-01"), 500);
    assert_eq!(gl_debit_minus_credit(&conn, "2000", "2026-11-01"), -200);
}

#[test]
fn invoice_post_rejects_expense_account_on_income_line() {
    let books = Books::new();
    let cust = books.customer("Misclass");
    let exp = books.acct("5000");
    let conn = books.conn();
    conn.execute(
        r#"INSERT INTO invoice
           (company_id, customer_id, number, status, issue_date, subtotal_minor, tax_minor, total_minor)
           VALUES (1, ?1, 'INV-BAD', 'sent', '2026-12-01', 100, 0, 100)"#,
        [cust],
    )
    .expect("invoice");
    let id = conn.last_insert_rowid();
    conn.execute(
        r#"INSERT INTO invoice_line
           (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id)
           VALUES (?1, 1, 'Oops', 1, 100, 100, ?2)"#,
        rusqlite::params![id, exp],
    )
    .expect("line");
    drop(conn);
    let mut c = books.conn();
    assert!(post_invoice(&mut c, id).is_err());
}

#[test]
fn trial_balance_and_profit_and_loss_reject_inverted_or_malformed_dates() {
    let books = Books::new();
    let conn = books.conn();
    assert!(trial_balance(&conn, "2026-02-01", "2026-01-01").is_err());
    assert!(profit_and_loss(&conn, "not-a-date", "2026-01-01").is_err());
    assert!(balance_sheet(&conn, "2026-13-40").is_err());
}
