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
            "SELECT 1 FROM account WHERE company_id = ?1 AND id = ?2",
            rusqlite::params![company_id, id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(n.is_some())
}
