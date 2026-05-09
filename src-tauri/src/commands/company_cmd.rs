use serde::Deserialize;

use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::constants::COMPANY_ID;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyUpdateInput {
    pub name: Option<String>,
    pub legal_name: Option<String>,
    pub fiscal_year_start_month: Option<i64>,
    pub base_currency_code: Option<String>,
    pub next_invoice_number: Option<i64>,
    pub next_bill_number: Option<i64>,
}

#[tauri::command]
pub fn company_get(state: State<'_, DbState>) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("company_get", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.query_row(
            r#"SELECT id, name, legal_name, fiscal_year_start_month, base_currency_code,
                  next_invoice_number, next_bill_number, created_at, updated_at
           FROM company WHERE id = ?1"#,
            [COMPANY_ID],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "legalName": row.get::<_, Option<String>>(2)?,
                    "fiscalYearStartMonth": row.get::<_, i64>(3)?,
                    "baseCurrencyCode": row.get::<_, String>(4)?,
                    "nextInvoiceNumber": row.get::<_, i64>(5)?,
                    "nextBillNumber": row.get::<_, i64>(6)?,
                    "createdAt": row.get::<_, String>(7)?,
                    "updatedAt": row.get::<_, String>(8)?,
                }))
            },
        )
        .map_err(|_| DbCommandError::NotFound {
            entity: "company".into(),
            id: COMPANY_ID,
        })
    })
}

#[tauri::command]
pub fn company_update(
    state: State<'_, DbState>,
    input: CompanyUpdateInput,
) -> Result<(), DbCommandError> {
    timed_ipc("company_update", || {
        let conn = open_sqlite(&state.db_path)?;
        let cur = conn.query_row(
            r#"SELECT name, legal_name, fiscal_year_start_month, base_currency_code,
                  next_invoice_number, next_bill_number
           FROM company WHERE id = ?1"#,
            [COMPANY_ID],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .map_err(|_| DbCommandError::NotFound {
            entity: "company".into(),
            id: COMPANY_ID,
        })?;

        let (
            c_name,
            c_legal,
            c_fiscal,
            c_curr,
            c_inv,
            c_bill,
        ) = cur;

        let name = input.name.unwrap_or(c_name);
        let legal_name = input.legal_name.or(c_legal);
        let fiscal = input.fiscal_year_start_month.unwrap_or(c_fiscal);
        let curr = input.base_currency_code.unwrap_or(c_curr);
        let next_inv = input.next_invoice_number.unwrap_or(c_inv);
        let next_bill = input.next_bill_number.unwrap_or(c_bill);

        conn.execute(
            r#"UPDATE company SET
             name = ?1,
             legal_name = ?2,
             fiscal_year_start_month = ?3,
             base_currency_code = ?4,
             next_invoice_number = ?5,
             next_bill_number = ?6,
             updated_at = datetime('now')
           WHERE id = ?7"#,
            rusqlite::params![
                name,
                legal_name,
                fiscal,
                curr,
                next_inv,
                next_bill,
                COMPANY_ID,
            ],
        )?;
        log::debug!(
            target: "kwikbooks_lib::ipc::company",
            "company_updated company_id={}",
            COMPANY_ID
        );
        Ok(())
    })
}
