use rusqlite::{Connection, OptionalExtension};

use crate::db::DbCommandError;

pub fn account_id_by_code(
    conn: &Connection,
    company_id: i64,
    code: &str,
) -> Result<i64, DbCommandError> {
    conn.query_row(
        "SELECT id FROM account WHERE company_id = ?1 AND code = ?2",
        rusqlite::params![company_id, code],
        |row| row.get(0),
    )
    .map_err(|_| DbCommandError::Invariant {
        message: format!("missing account code {code} for company {company_id}"),
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

pub fn active_account_of_type(
    conn: &Connection,
    company_id: i64,
    id: i64,
    account_type: &str,
) -> Result<bool, DbCommandError> {
    if !account_exists(conn, company_id, id)? {
        return Ok(false);
    }
    let n: Option<i64> = conn
        .query_row(
            r#"SELECT 1 FROM account
               WHERE company_id = ?1 AND id = ?2 AND is_active = 1 AND account_type = ?3"#,
            rusqlite::params![company_id, id, account_type],
            |row| row.get(0),
        )
        .optional()?;
    Ok(n.is_some())
}

pub fn active_bank_cash_asset(
    conn: &Connection,
    company_id: i64,
    id: i64,
) -> Result<bool, DbCommandError> {
    let n: Option<i64> = conn
        .query_row(
            r#"SELECT 1 FROM account
               WHERE company_id = ?1 AND id = ?2 AND is_active = 1
                 AND account_type = 'asset' AND is_bank_cash = 1"#,
            rusqlite::params![company_id, id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(n.is_some())
}
