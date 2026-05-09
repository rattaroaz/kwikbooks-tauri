use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountFilter {
    pub account_type: Option<String>,
    pub active_only: Option<bool>,
    pub search: Option<String>,
}

pub fn list_accounts(
    conn: &Connection,
    filter: &AccountFilter,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut sql =
        String::from("SELECT id, company_id, code, name, account_type, parent_id, is_bank_cash, is_active, sort_order FROM account WHERE company_id = ?");

    if filter.active_only.unwrap_or(false) {
        sql.push_str(" AND is_active = 1");
    }

    if filter.account_type.is_some() {
        sql.push_str(" AND account_type = ?");
    }

    if filter.search.is_some() {
        sql.push_str(" AND (code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%')");
    }

    sql.push_str(" ORDER BY sort_order, code");

    let mut stmt = conn.prepare(&sql)?;

    let rows = match (filter.account_type.as_ref(), filter.search.as_ref()) {
        (Some(t), Some(s)) => {
            stmt.query_map(rusqlite::params![COMPANY_ID, t, s, s], map_account_row)
        }
        (Some(t), None) => stmt.query_map(rusqlite::params![COMPANY_ID, t], map_account_row),
        (None, Some(s)) => stmt.query_map(rusqlite::params![COMPANY_ID, s, s], map_account_row),
        (None, None) => stmt.query_map([COMPANY_ID], map_account_row),
    };

    let rows = rows?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn map_account_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "id": row.get::<_, i64>(0)?,
        "companyId": row.get::<_, i64>(1)?,
        "code": row.get::<_, String>(2)?,
        "name": row.get::<_, String>(3)?,
        "accountType": row.get::<_, String>(4)?,
        "parentId": row.get::<_, Option<i64>>(5)?,
        "isBankCash": row.get::<_, i64>(6)? == 1,
        "isActive": row.get::<_, i64>(7)? == 1,
        "sortOrder": row.get::<_, i64>(8)?,
    }))
}
