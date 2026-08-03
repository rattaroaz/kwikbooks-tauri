//! QuickBooks-oriented list import (IIF + CSV exports). Maps chart of accounts,
//! customers, vendors, and service/non-inventory items into Kwikbooks tables.

mod iif;
mod qb_csv;

use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

#[derive(Debug, Default)]
pub(crate) struct ImportBatch {
    pub accounts: Vec<ParsedAccount>,
    pub customers: Vec<ParsedContact>,
    pub vendors: Vec<ParsedContact>,
    pub items: Vec<ParsedItem>,
}

#[derive(Debug)]
pub(crate) struct ParsedAccount {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub is_bank_cash: bool,
}

#[derive(Debug)]
pub(crate) struct ParsedContact {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ParsedItem {
    pub name: String,
    pub unit_price_minor: i64,
    pub income_account_name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub format_detected: String,
    pub accounts_created: usize,
    pub customers_created: usize,
    pub vendors_created: usize,
    pub items_created: usize,
    pub rows_skipped: usize,
    pub warnings: Vec<String>,
}

pub fn run_import(
    conn: &mut Connection,
    content: &str,
    filename_hint: &str,
) -> Result<ImportSummary, DbCommandError> {
    let lower_hint = filename_hint.to_ascii_lowercase();
    let trimmed = content.trim_start_matches('\u{feff}');

    if looks_like_ofx(trimmed, &lower_hint) {
        return Err(DbCommandError::Validation {
            message: "OFX/QFX bank downloads are not imported here — export Lists or Chart of Accounts as IIF or CSV from QuickBooks.".into(),
        });
    }

    let (format_label, batch, mut rows_skipped, mut warnings) =
        if looks_like_iif(trimmed, &lower_hint) {
            match iif::parse_iif(trimmed) {
                Ok((b, skipped, w)) => ("iif", b, skipped, w),
                Err(e) => {
                    return Err(DbCommandError::Validation { message: e });
                }
            }
        } else if looks_like_csv(trimmed, &lower_hint) {
            match qb_csv::parse_csv(trimmed) {
                Ok((b, skipped, w)) => ("csv", b, skipped, w),
                Err(e) => {
                    return Err(DbCommandError::Validation { message: e });
                }
            }
        } else {
            return Err(DbCommandError::Validation {
                message: "Could not detect a supported QuickBooks export (try .iif or .csv)."
                    .into(),
            });
        };

    if batch.accounts.is_empty()
        && batch.customers.is_empty()
        && batch.vendors.is_empty()
        && batch.items.is_empty()
    {
        return Err(DbCommandError::Validation {
            message: "No importable rows found (accounts, customers, vendors, or items)."
                .into(),
        });
    }

    let tx = conn.transaction()?;

    let mut accounts_created = 0usize;
    let mut customers_created = 0usize;
    let mut vendors_created = 0usize;
    let mut items_created = 0usize;

    let max_sort: i64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) FROM account WHERE company_id = ?1",
        [COMPANY_ID],
        |row| row.get(0),
    )?;
    let mut next_sort = max_sort;

    let mut existing_codes: std::collections::HashSet<String> = tx
        .prepare("SELECT lower(code) FROM account WHERE company_id = ?1")?
        .query_map([COMPANY_ID], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for a in batch.accounts {
        let code_lower = a.code.trim().to_lowercase();
        if code_lower.is_empty() {
            rows_skipped += 1;
            warnings.push("Skipped account with empty code.".into());
            continue;
        }
        if existing_codes.contains(&code_lower) {
            rows_skipped += 1;
            warnings.push(format!("Skipped duplicate account code \"{}\".", a.code));
            continue;
        }

        let account_type = a.account_type;
        let mut is_bank_cash = a.is_bank_cash;
        if let Err(e) =
            crate::domain::ids::require_bank_cash_flag(is_bank_cash, &account_type, a.code.trim())
        {
            warnings.push(format!(
                "Account \"{}\": {e} — imported without bank/cash.",
                a.code
            ));
            is_bank_cash = false;
        }

        next_sort += 1;
        tx.execute(
            r#"INSERT INTO account (company_id, code, name, account_type, parent_id, is_bank_cash, sort_order)
               VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)"#,
            rusqlite::params![
                COMPANY_ID,
                a.code.trim(),
                a.name.trim(),
                account_type,
                is_bank_cash as i64,
                next_sort,
            ],
        )?;
        existing_codes.insert(code_lower);
        accounts_created += 1;
    }

    let mut existing_customers: std::collections::HashSet<String> = tx
        .prepare("SELECT lower(trim(display_name)) FROM customer WHERE company_id = ?1")?
        .query_map([COMPANY_ID], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for c in batch.customers {
        let key = c.display_name.trim().to_lowercase();
        if key.is_empty() {
            rows_skipped += 1;
            continue;
        }
        if existing_customers.contains(&key) {
            rows_skipped += 1;
            warnings.push(format!(
                "Skipped duplicate customer \"{}\".",
                c.display_name
            ));
            continue;
        }
        tx.execute(
            r#"INSERT INTO customer (company_id, display_name, email, phone, terms_days)
               VALUES (?1, ?2, ?3, ?4, 30)"#,
            rusqlite::params![
                COMPANY_ID,
                c.display_name.trim(),
                c.email.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()),
                c.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()),
            ],
        )?;
        existing_customers.insert(key);
        customers_created += 1;
    }

    let mut existing_vendors: std::collections::HashSet<String> = tx
        .prepare("SELECT lower(trim(display_name)) FROM vendor WHERE company_id = ?1")?
        .query_map([COMPANY_ID], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for v in batch.vendors {
        let key = v.display_name.trim().to_lowercase();
        if key.is_empty() {
            rows_skipped += 1;
            continue;
        }
        if existing_vendors.contains(&key) {
            rows_skipped += 1;
            warnings.push(format!(
                "Skipped duplicate vendor \"{}\".",
                v.display_name
            ));
            continue;
        }
        tx.execute(
            r#"INSERT INTO vendor (company_id, display_name, email, phone)
               VALUES (?1, ?2, ?3, ?4)"#,
            rusqlite::params![
                COMPANY_ID,
                v.display_name.trim(),
                v.email.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()),
                v.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()),
            ],
        )?;
        existing_vendors.insert(key);
        vendors_created += 1;
    }

    let income_account_by_lower_name: std::collections::HashMap<String, i64> = tx
        .prepare(
            r#"SELECT lower(trim(name)), id FROM account
               WHERE company_id = ?1 AND is_active = 1 AND account_type = 'income'"#,
        )?
        .query_map([COMPANY_ID], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(Result::ok)
        .collect();

    let mut existing_items: std::collections::HashSet<String> = tx
        .prepare("SELECT lower(trim(name)) FROM item WHERE company_id = ?1")?
        .query_map([COMPANY_ID], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for it in batch.items {
        let key = it.name.trim().to_lowercase();
        if key.is_empty() {
            rows_skipped += 1;
            continue;
        }
        if existing_items.contains(&key) {
            rows_skipped += 1;
            warnings.push(format!("Skipped duplicate item \"{}\".", it.name));
            continue;
        }

        let income_name = it
            .income_account_name
            .as_ref()
            .map(|n| n.trim())
            .filter(|n| !n.is_empty());
        let income_id = income_name
            .and_then(|n| income_account_by_lower_name.get(&n.to_lowercase()).copied());
        if let Some(name) = income_name {
            if income_id.is_none() {
                warnings.push(format!(
                    "Item \"{}\": income account \"{}\" not found (or not an income account); left unset.",
                    it.name, name
                ));
            }
        }

        tx.execute(
            r#"INSERT INTO item (company_id, name, sku, unit_price_minor, default_income_account_id, default_expense_account_id, is_active)
               VALUES (?1, ?2, NULL, ?3, ?4, NULL, 1)"#,
            rusqlite::params![
                COMPANY_ID,
                it.name.trim(),
                it.unit_price_minor,
                income_id,
            ],
        )?;
        existing_items.insert(key);
        items_created += 1;
    }

    tx.commit()?;

    Ok(ImportSummary {
        format_detected: format_label.into(),
        accounts_created,
        customers_created,
        vendors_created,
        items_created,
        rows_skipped,
        warnings,
    })
}

fn looks_like_iif(content: &str, filename_lower: &str) -> bool {
    if filename_lower.ends_with(".iif") {
        return true;
    }
    let sample = content.chars().take(2048).collect::<String>();
    sample.contains("!ACCNT") || sample.contains("!CUST") || sample.contains("!VEND")
}

fn looks_like_csv(content: &str, filename_lower: &str) -> bool {
    filename_lower.ends_with(".csv")
        || filename_lower.ends_with(".txt")
        || content.lines().next().is_some_and(|l| {
            let l = l.trim_start_matches('\u{feff}');
            l.contains(',') && !l.starts_with('!')
        })
}

fn looks_like_ofx(content: &str, filename_lower: &str) -> bool {
    filename_lower.ends_with(".ofx")
        || filename_lower.ends_with(".qfx")
        || content
            .chars()
            .take(64)
            .collect::<String>()
            .to_ascii_uppercase()
            .contains("OFXHEADER")
        || trimmed_start(content).starts_with("<OFX")
}

fn trimmed_start(s: &str) -> &str {
    s.trim_start_matches('\u{feff}')
        .trim_start_matches(|c: char| c.is_ascii_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_all;
    use tempfile::tempdir;

    #[test]
    fn run_import_iif_minimal() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("imp_iif.sqlite");
        run_all(&db_path).expect("migrate");
        let mut conn = rusqlite::Connection::open(&db_path).expect("open");
        // Use 4100 — migration seeds 4000 "Sales" so 4000 would be skipped as duplicate.
        let iif = "!ACCNT\tNAME\tACCNTTYPE\tDESC\tACCNUM\nACCNT\tImport Test Income\tINC\t\t4100\n!CUST\tNAME\tEMAIL\nCUST\tAcme Co\tx@y.com\n";
        let s = run_import(&mut conn, iif, "test.iif").expect("import");
        assert_eq!(s.format_detected, "iif");
        assert_eq!(s.accounts_created, 1);
        assert_eq!(s.customers_created, 1);
    }

    #[test]
    fn run_import_csv_customers() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("imp_csv.sqlite");
        run_all(&db_path).expect("migrate");
        let mut conn = rusqlite::Connection::open(&db_path).expect("open");
        let csv = "Customer,Email,Phone\nBeta LLC,b@x.com,555\n";
        let s = run_import(&mut conn, csv, "cust.csv").expect("import");
        assert_eq!(s.format_detected, "csv");
        assert_eq!(s.customers_created, 1);
    }
}
