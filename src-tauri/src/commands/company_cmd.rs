use serde::Deserialize;

use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::constants::COMPANY_ID;
use crate::ipc_log::timed_ipc;
use tauri::State;

fn valid_check_style(s: &str) -> bool {
    matches!(
        s,
        "voucher_top" | "voucher_middle" | "voucher_bottom" | "generic"
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyUpdateInput {
    pub name: Option<String>,
    pub legal_name: Option<String>,
    pub fiscal_year_start_month: Option<i64>,
    pub base_currency_code: Option<String>,
    pub next_invoice_number: Option<i64>,
    pub next_bill_number: Option<i64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub next_check_number: Option<i64>,
    pub default_check_style: Option<String>,
}

#[tauri::command]
pub fn company_get(state: State<'_, DbState>) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("company_get", || company_get_impl(&state.db_path))
}

fn company_get_impl(db_path: &std::path::Path) -> Result<serde_json::Value, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    conn.query_row(
        r#"SELECT id, name, legal_name, fiscal_year_start_month, base_currency_code,
                  next_invoice_number, next_bill_number,
                  address_line1, address_line2, city, region, postal_code,
                  next_check_number, default_check_style,
                  created_at, updated_at
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
                "addressLine1": row.get::<_, Option<String>>(7)?,
                "addressLine2": row.get::<_, Option<String>>(8)?,
                "city": row.get::<_, Option<String>>(9)?,
                "region": row.get::<_, Option<String>>(10)?,
                "postalCode": row.get::<_, Option<String>>(11)?,
                "nextCheckNumber": row.get::<_, i64>(12)?,
                "defaultCheckStyle": row.get::<_, String>(13)?,
                "createdAt": row.get::<_, String>(14)?,
                "updatedAt": row.get::<_, String>(15)?,
            }))
        },
    )
    .map_err(|_| DbCommandError::NotFound {
        entity: "company".into(),
        id: COMPANY_ID,
    })
}

#[tauri::command]
pub fn company_update(
    state: State<'_, DbState>,
    input: CompanyUpdateInput,
) -> Result<(), DbCommandError> {
    timed_ipc("company_update", || company_update_impl(&state.db_path, input))
}

fn company_update_impl(
    db_path: &std::path::Path,
    input: CompanyUpdateInput,
) -> Result<(), DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let cur = conn.query_row(
        r#"SELECT name, legal_name, fiscal_year_start_month, base_currency_code,
                  next_invoice_number, next_bill_number,
                  address_line1, address_line2, city, region, postal_code,
                  next_check_number, default_check_style
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
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, String>(12)?,
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
        c_a1,
        c_a2,
        c_city,
        c_region,
        c_postal,
        c_check,
        c_style,
    ) = cur;

    let name = input.name.unwrap_or(c_name);
    let legal_name = input.legal_name.or(c_legal);
    let fiscal = input.fiscal_year_start_month.unwrap_or(c_fiscal);
    if !(1..=12).contains(&fiscal) {
        return Err(DbCommandError::Validation {
            message: "fiscal year start month must be between 1 and 12".into(),
        });
    }
    let curr = input.base_currency_code.unwrap_or(c_curr);
    let next_inv = input.next_invoice_number.unwrap_or(c_inv);
    let next_bill = input.next_bill_number.unwrap_or(c_bill);
    let address_line1 = input.address_line1.or(c_a1);
    let address_line2 = input.address_line2.or(c_a2);
    let city = input.city.or(c_city);
    let region = input.region.or(c_region);
    let postal_code = input.postal_code.or(c_postal);
    let next_check = input.next_check_number.unwrap_or(c_check);
    let default_check_style = input.default_check_style.unwrap_or(c_style);

    if !valid_check_style(&default_check_style) {
        return Err(DbCommandError::Validation {
            message: "defaultCheckStyle must be voucher_top, voucher_middle, voucher_bottom, or generic"
                .into(),
        });
    }
    if next_inv < 1 || next_bill < 1 {
        return Err(DbCommandError::Validation {
            message: "next invoice and bill numbers must be at least 1".into(),
        });
    }
    if next_check < 1 {
        return Err(DbCommandError::Validation {
            message: "nextCheckNumber must be at least 1".into(),
        });
    }

    conn.execute(
        r#"UPDATE company SET
             name = ?1,
             legal_name = ?2,
             fiscal_year_start_month = ?3,
             base_currency_code = ?4,
             next_invoice_number = ?5,
             next_bill_number = ?6,
             address_line1 = ?7,
             address_line2 = ?8,
             city = ?9,
             region = ?10,
             postal_code = ?11,
             next_check_number = ?12,
             default_check_style = ?13,
             updated_at = datetime('now')
           WHERE id = ?14"#,
        rusqlite::params![
            name,
            legal_name,
            fiscal,
            curr,
            next_inv,
            next_bill,
            address_line1,
            address_line2,
            city,
            region,
            postal_code,
            next_check,
            default_check_style,
            COMPANY_ID,
        ],
    )?;
    log::debug!(
        target: "kwikbooks_lib::ipc::company",
        "company_updated company_id={}",
        COMPANY_ID
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_all;
    use tempfile::tempdir;

    fn test_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("company_cmd.sqlite");
        run_all(&db_path).expect("migrate");
        (dir, db_path)
    }

    #[test]
    fn company_get_impl_returns_seeded_company() {
        let (_dir, db_path) = test_db();

        let row = company_get_impl(&db_path).expect("get");
        assert_eq!(row["id"], 1);
        assert_eq!(row["name"], "My Company");
        assert_eq!(row["baseCurrencyCode"], "USD");
        assert_eq!(row["defaultCheckStyle"], "voucher_top");
    }

    #[test]
    fn company_update_impl_persists_partial_fields() {
        let (_dir, db_path) = test_db();

        company_update_impl(
            &db_path,
            CompanyUpdateInput {
                name: Some("Acme Books".into()),
                legal_name: Some("Acme LLC".into()),
                fiscal_year_start_month: Some(4),
                base_currency_code: None,
                next_invoice_number: Some(2000),
                next_bill_number: None,
                address_line1: Some("123 Main".into()),
                address_line2: None,
                city: Some("Portland".into()),
                region: Some("OR".into()),
                postal_code: Some("97201".into()),
                next_check_number: Some(4000),
                default_check_style: Some("voucher_middle".into()),
            },
        )
        .expect("update");

        let row = company_get_impl(&db_path).expect("get after update");
        assert_eq!(row["name"], "Acme Books");
        assert_eq!(row["legalName"], "Acme LLC");
        assert_eq!(row["fiscalYearStartMonth"], 4);
        assert_eq!(row["nextInvoiceNumber"], 2000);
        assert_eq!(row["baseCurrencyCode"], "USD");
        assert_eq!(row["addressLine1"], "123 Main");
        assert_eq!(row["city"], "Portland");
        assert_eq!(row["region"], "OR");
        assert_eq!(row["postalCode"], "97201");
        assert_eq!(row["nextCheckNumber"], 4000);
        assert_eq!(row["defaultCheckStyle"], "voucher_middle");
    }

    #[test]
    fn company_update_input_deserializes_camel_case_payload() {
        let payload = serde_json::json!({
            "name": "Renamed Co",
            "legalName": null,
            "fiscalYearStartMonth": 7,
            "baseCurrencyCode": "EUR",
            "nextInvoiceNumber": 500,
            "nextBillNumber": 600,
            "addressLine1": "123 Main",
            "addressLine2": null,
            "city": "Portland",
            "region": "OR",
            "postalCode": "97201",
            "nextCheckNumber": 700,
            "defaultCheckStyle": "generic"
        });
        let parsed: CompanyUpdateInput =
            serde_json::from_value(payload).expect("deserialize company update input");
        assert_eq!(parsed.name.as_deref(), Some("Renamed Co"));
        assert_eq!(parsed.fiscal_year_start_month, Some(7));
        assert_eq!(parsed.next_bill_number, Some(600));
        assert_eq!(parsed.address_line1.as_deref(), Some("123 Main"));
        assert_eq!(parsed.next_check_number, Some(700));
        assert_eq!(parsed.default_check_style.as_deref(), Some("generic"));
    }

    #[test]
    fn company_update_impl_rejects_invalid_fiscal_month() {
        let (_dir, db_path) = test_db();
        let err = company_update_impl(
            &db_path,
            CompanyUpdateInput {
                name: None,
                legal_name: None,
                fiscal_year_start_month: Some(13),
                base_currency_code: None,
                next_invoice_number: None,
                next_bill_number: None,
                address_line1: None,
                address_line2: None,
                city: None,
                region: None,
                postal_code: None,
                next_check_number: None,
                default_check_style: None,
            },
        )
        .expect_err("must fail");
        match err {
            DbCommandError::Validation { message } => {
                assert!(message.contains("fiscal year"));
            }
            other => panic!("expected validation, got {other:?}"),
        }
    }
}
