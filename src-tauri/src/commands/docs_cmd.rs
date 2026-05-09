use serde::Deserialize;

use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::constants::COMPANY_ID;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerCreateInput {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(default = "default_terms")]
    pub terms_days: i64,
}

fn default_terms() -> i64 {
    30
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorCreateInput {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineInput {
    pub description: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
    pub income_account_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceCreateInput {
    pub customer_id: i64,
    pub number: String,
    pub issue_date: String,
    pub due_date: Option<String>,
    #[serde(default)]
    pub tax_minor: i64,
    pub memo: Option<String>,
    pub lines: Vec<InvoiceLineInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillLineInput {
    pub description: String,
    pub amount_minor: i64,
    pub expense_account_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillCreateInput {
    pub vendor_id: Option<i64>,
    pub payee_name: Option<String>,
    pub number: String,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub memo: Option<String>,
    pub lines: Vec<BillLineInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerPaymentCreateInput {
    pub customer_id: i64,
    pub bank_account_id: i64,
    pub payment_date: String,
    pub amount_minor: i64,
    pub memo: Option<String>,
    pub invoice_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorPaymentCreateInput {
    pub vendor_id: i64,
    pub bank_account_id: i64,
    pub payment_date: String,
    pub amount_minor: i64,
    pub memo: Option<String>,
    pub bill_id: Option<i64>,
}

fn line_total_minor(qty: f64, unit_minor: i64) -> i64 {
    ((qty * unit_minor as f64).round() as i64).max(0)
}

#[tauri::command]
pub fn customer_create(
    state: State<'_, DbState>,
    input: CustomerCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("customer_create", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.execute(
            r#"INSERT INTO customer (company_id, display_name, email, phone, terms_days)
           VALUES (?1, ?2, ?3, ?4, ?5)"#,
            rusqlite::params![
                COMPANY_ID,
                input.display_name,
                input.email,
                input.phone,
                input.terms_days,
            ],
        )?;
        let id = conn.last_insert_rowid();
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "customer_created id={}",
            id
        );
        Ok(id)
    })
}

#[tauri::command]
pub fn vendor_create(
    state: State<'_, DbState>,
    input: VendorCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("vendor_create", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.execute(
            r#"INSERT INTO vendor (company_id, display_name, email, phone)
           VALUES (?1, ?2, ?3, ?4)"#,
            rusqlite::params![COMPANY_ID, input.display_name, input.email, input.phone,],
        )?;
        let id = conn.last_insert_rowid();
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "vendor_created id={}",
            id
        );
        Ok(id)
    })
}

#[tauri::command]
pub fn invoice_create(
    state: State<'_, DbState>,
    input: InvoiceCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("invoice_create", || {
        let mut subtotal: i64 = 0;
        let mut line_payload: Vec<(i32, String, f64, i64, i64, Option<i64>)> = Vec::new();

        for (idx, line) in input.lines.iter().enumerate() {
            let ln = (idx + 1) as i32;
            let lt = line_total_minor(line.quantity, line.unit_price_minor);
            subtotal = subtotal.saturating_add(lt);
            line_payload.push((
                ln,
                line.description.clone(),
                line.quantity,
                line.unit_price_minor,
                lt,
                line.income_account_id,
            ));
        }

        let total = subtotal.saturating_add(input.tax_minor);
        let cust = input.customer_id;
        let inv_no = input.number.clone();
        let line_count = input.lines.len();
        let mut conn = open_sqlite(&state.db_path)?;
        let tx = conn.transaction()?;

        tx.execute(
            r#"INSERT INTO invoice
           (company_id, customer_id, number, status, issue_date, due_date, memo,
            subtotal_minor, tax_minor, total_minor)
           VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, ?8, ?9)"#,
            rusqlite::params![
                COMPANY_ID,
                input.customer_id,
                input.number,
                input.issue_date,
                input.due_date,
                input.memo,
                subtotal,
                input.tax_minor,
                total,
            ],
        )?;
        let invoice_id = tx.last_insert_rowid();

        for (ln, desc, qty, unit_p, lt, inc) in line_payload {
            tx.execute(
                r#"INSERT INTO invoice_line
               (invoice_id, line_number, description, quantity, unit_price_minor, line_total_minor, income_account_id)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                rusqlite::params![invoice_id, ln, desc, qty, unit_p, lt, inc],
            )?;
        }

        tx.commit()?;
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "invoice_created_draft invoice_id={} customer_id={} number={} lines={} total_minor={}",
            invoice_id,
            cust,
            inv_no,
            line_count,
            total
        );
        Ok(invoice_id)
    })
}

#[tauri::command]
pub fn bill_create(
    state: State<'_, DbState>,
    input: BillCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("bill_create", || {
        let mut total_minor: i64 = 0;
        let mut rows: Vec<(i32, String, i64, i64)> = Vec::new();

        for (idx, line) in input.lines.iter().enumerate() {
            total_minor = total_minor.saturating_add(line.amount_minor);
            rows.push((
                (idx + 1) as i32,
                line.description.clone(),
                line.amount_minor,
                line.expense_account_id,
            ));
        }

        let num = input.number.clone();
        let n_lines = input.lines.len();
        let mut conn = open_sqlite(&state.db_path)?;
        let tx = conn.transaction()?;

        tx.execute(
            r#"INSERT INTO bill
           (company_id, vendor_id, payee_name, number, status, issue_date, due_date, memo, total_minor)
           VALUES (?1, ?2, ?3, ?4, 'draft', ?5, ?6, ?7, ?8)"#,
            rusqlite::params![
                COMPANY_ID,
                input.vendor_id,
                input.payee_name,
                input.number,
                input.issue_date,
                input.due_date,
                input.memo,
                total_minor,
            ],
        )?;
        let bill_id = tx.last_insert_rowid();

        for (ln, desc, amt, exp) in rows {
            tx.execute(
                r#"INSERT INTO bill_line (bill_id, line_number, description, amount_minor, expense_account_id)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
                rusqlite::params![bill_id, ln, desc, amt, exp],
            )?;
        }

        tx.commit()?;
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "bill_created_draft bill_id={} number={} lines={} total_minor={}",
            bill_id,
            num,
            n_lines,
            total_minor
        );
        Ok(bill_id)
    })
}

#[tauri::command]
pub fn customer_payment_create(
    state: State<'_, DbState>,
    input: CustomerPaymentCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("customer_payment_create", || {
        let amt = input.amount_minor;
        let cid = input.customer_id;
        let conn = open_sqlite(&state.db_path)?;
        conn.execute(
            r#"INSERT INTO customer_payment
           (company_id, customer_id, bank_account_id, payment_date, amount_minor, memo, invoice_id)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            rusqlite::params![
                COMPANY_ID,
                input.customer_id,
                input.bank_account_id,
                input.payment_date,
                input.amount_minor,
                input.memo,
                input.invoice_id,
            ],
        )?;
        let id = conn.last_insert_rowid();
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "customer_payment_created id={} customer_id={} amount_minor={}",
            id,
            cid,
            amt
        );
        Ok(id)
    })
}

#[tauri::command]
pub fn vendor_payment_create(
    state: State<'_, DbState>,
    input: VendorPaymentCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("vendor_payment_create", || {
        let amt = input.amount_minor;
        let vid = input.vendor_id;
        let conn = open_sqlite(&state.db_path)?;
        conn.execute(
            r#"INSERT INTO vendor_payment
           (company_id, vendor_id, bank_account_id, payment_date, amount_minor, memo, bill_id)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            rusqlite::params![
                COMPANY_ID,
                input.vendor_id,
                input.bank_account_id,
                input.payment_date,
                input.amount_minor,
                input.memo,
                input.bill_id,
            ],
        )?;
        let id = conn.last_insert_rowid();
        log::debug!(
            target: "kwikbooks_lib::ipc::docs",
            "vendor_payment_created id={} vendor_id={} amount_minor={}",
            id,
            vid,
            amt
        );
        Ok(id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_create_input_deserializes_camel_case_payload() {
        let payload = serde_json::json!({
            "customerId": 42,
            "number": "INV-42",
            "issueDate": "2026-03-12",
            "dueDate": "2026-04-11",
            "taxMinor": 125,
            "memo": "memo",
            "lines": [
                {
                    "description": "Service",
                    "quantity": 2.0,
                    "unitPriceMinor": 5000,
                    "incomeAccountId": 9
                }
            ]
        });
        let parsed: InvoiceCreateInput =
            serde_json::from_value(payload).expect("deserialize invoice input");
        assert_eq!(parsed.customer_id, 42);
        assert_eq!(parsed.tax_minor, 125);
        assert_eq!(parsed.lines.len(), 1);
        assert_eq!(parsed.lines[0].unit_price_minor, 5000);
        assert_eq!(parsed.lines[0].income_account_id, Some(9));
    }

    #[test]
    fn customer_create_defaults_terms_days_to_30() {
        let payload = serde_json::json!({
            "displayName": "Default Terms Customer",
            "email": null,
            "phone": null
        });
        let parsed: CustomerCreateInput =
            serde_json::from_value(payload).expect("deserialize customer input");
        assert_eq!(parsed.terms_days, 30);
    }

    #[test]
    fn bill_create_input_supports_vendor_or_payee_shape() {
        let payload = serde_json::json!({
            "vendorId": null,
            "payeeName": "Independent Payee",
            "number": "B-22",
            "issueDate": "2026-03-13",
            "dueDate": null,
            "memo": null,
            "lines": [
                {
                    "description": "Expense",
                    "amountMinor": 900,
                    "expenseAccountId": 5
                }
            ]
        });
        let parsed: BillCreateInput =
            serde_json::from_value(payload).expect("deserialize bill input");
        assert_eq!(parsed.vendor_id, None);
        assert_eq!(parsed.payee_name.as_deref(), Some("Independent Payee"));
        assert_eq!(parsed.lines[0].amount_minor, 900);
    }
}
