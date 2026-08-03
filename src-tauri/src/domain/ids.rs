use rusqlite::{Connection, OptionalExtension};

use crate::db::DbCommandError;

/// Seeded control accounts that must never be payment bank/cash accounts.
const NON_BANK_CONTROL_CODES: &[&str] = &["1100", "2000", "2100", "3000", "4000", "5000"];

/// Seeded accounts that posting and reports depend on — cannot be deactivated.
pub const PROTECTED_ACCOUNT_CODES: &[&str] =
    &["1000", "1100", "2000", "2100", "3000", "4000", "5000"];

pub fn account_id_by_code(
    conn: &Connection,
    company_id: i64,
    code: &str,
) -> Result<i64, DbCommandError> {
    conn.query_row(
        "SELECT id FROM account WHERE company_id = ?1 AND code = ?2 AND is_active = 1",
        rusqlite::params![company_id, code],
        |row| row.get(0),
    )
    .map_err(|_| DbCommandError::Invariant {
        message: format!("missing active account code {code} for company {company_id}"),
    })
}

pub fn account_exists(conn: &Connection, company_id: i64, id: i64) -> Result<bool, DbCommandError> {
    let n: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM account WHERE company_id = ?1 AND id = ?2 AND is_active = 1",
            rusqlite::params![company_id, id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(n.is_some())
}

pub fn account_is_bank_cash(
    conn: &Connection,
    company_id: i64,
    id: i64,
) -> Result<bool, DbCommandError> {
    let row: Option<(i64, i64, String, String)> = conn
        .query_row(
            "SELECT is_bank_cash, is_active, account_type, code FROM account WHERE company_id = ?1 AND id = ?2",
            rusqlite::params![company_id, id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let Some((is_bank, is_active, account_type, code)) = row else {
        return Ok(false);
    };
    Ok(is_bank == 1
        && is_active == 1
        && account_type == "asset"
        && !NON_BANK_CONTROL_CODES.contains(&code.as_str()))
}

pub fn account_has_type(
    conn: &Connection,
    company_id: i64,
    id: i64,
    expected: &str,
) -> Result<bool, DbCommandError> {
    let typ: Option<String> = conn
        .query_row(
            "SELECT account_type FROM account WHERE company_id = ?1 AND id = ?2 AND is_active = 1",
            rusqlite::params![company_id, id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(typ.as_deref() == Some(expected))
}

pub fn require_bank_cash_flag(
    is_bank_cash: bool,
    account_type: &str,
    code: &str,
) -> Result<(), DbCommandError> {
    if !is_bank_cash {
        return Ok(());
    }
    if account_type != "asset" {
        return Err(DbCommandError::Validation {
            message: "bank/cash accounts must have account_type=asset".into(),
        });
    }
    if NON_BANK_CONTROL_CODES.contains(&code.trim()) {
        return Err(DbCommandError::Validation {
            message: format!(
                "account code {code} is a control account and cannot be marked bank/cash"
            ),
        });
    }
    Ok(())
}

pub fn require_bank_cash_account(
    conn: &Connection,
    company_id: i64,
    id: i64,
) -> Result<(), DbCommandError> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM account WHERE company_id = ?1 AND id = ?2",
            rusqlite::params![company_id, id],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(DbCommandError::Validation {
            message: "bank account not found".into(),
        });
    }
    if !account_is_bank_cash(conn, company_id, id)? {
        return Err(DbCommandError::Validation {
            message: "bank account must be an active cash/bank asset (not AR/AP or other control accounts)"
                .into(),
        });
    }
    Ok(())
}

pub fn account_has_journal_lines(
    conn: &Connection,
    account_id: i64,
) -> Result<bool, DbCommandError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM journal_line WHERE account_id = ?1",
        [account_id],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

pub fn account_code_is_protected(code: &str) -> bool {
    PROTECTED_ACCOUNT_CODES.contains(&code.trim())
}
