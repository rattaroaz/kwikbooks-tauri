use serde::{Deserialize, Serialize};

use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::accounts::{list_accounts, AccountFilter};
use crate::domain::constants::COMPANY_ID;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCreateInput {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub is_bank_cash: bool,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateInput {
    pub id: i64,
    pub code: Option<String>,
    pub name: Option<String>,
    pub account_type: Option<String>,
    /// When `true`, clears `parent_id`.
    #[serde(default)]
    pub clear_parent: bool,
    pub parent_id: Option<i64>,
    pub is_bank_cash: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowsAffected {
    pub rows_affected: usize,
}

#[tauri::command]
pub fn account_list(
    state: State<'_, DbState>,
    filter: AccountFilter,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    timed_ipc("account_list", || {
        let conn = open_sqlite(&state.db_path)?;
        let v = list_accounts(&conn, &filter)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::accounts",
            "account_list rows={}",
            v.len()
        );
        Ok(v)
    })
}

#[tauri::command]
pub fn account_get(
    state: State<'_, DbState>,
    id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("account_get", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.query_row(
            r#"SELECT id, company_id, code, name, account_type, parent_id, is_bank_cash, is_active, sort_order
           FROM account WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![id, COMPANY_ID],
            |row| {
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
            },
        )
        .map_err(|_| DbCommandError::NotFound {
            entity: "account".into(),
            id,
        })
    })
}

#[tauri::command]
pub fn account_create(
    state: State<'_, DbState>,
    input: AccountCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("account_create", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.execute(
            r#"INSERT INTO account (company_id, code, name, account_type, parent_id, is_bank_cash, sort_order)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            rusqlite::params![
                COMPANY_ID,
                input.code,
                input.name,
                input.account_type,
                input.parent_id,
                input.is_bank_cash as i64,
                input.sort_order,
            ],
        )?;
        let id = conn.last_insert_rowid();
        log::debug!(
            target: "kwikbooks_lib::ipc::accounts",
            "account_created id={} code={}",
            id,
            input.code
        );
        Ok(id)
    })
}

#[tauri::command]
pub fn account_update(
    state: State<'_, DbState>,
    input: AccountUpdateInput,
) -> Result<RowsAffected, DbCommandError> {
    timed_ipc("account_update", || {
        let conn = open_sqlite(&state.db_path)?;
        let cur = conn.query_row(
            r#"SELECT code, name, account_type, parent_id, is_bank_cash, is_active, sort_order
           FROM account WHERE id = ?1 AND company_id = ?2"#,
            rusqlite::params![input.id, COMPANY_ID],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i64>(4)? == 1,
                    row.get::<_, i64>(5)? == 1,
                    row.get::<_, i64>(6)?,
                ))
            },
        );

        let Ok((c_code, c_name, c_type, c_parent, c_bank, c_active, c_sort)) = cur else {
            return Err(DbCommandError::NotFound {
                entity: "account".into(),
                id: input.id,
            });
        };

        let parent_id = if input.clear_parent {
            None
        } else if let Some(p) = input.parent_id {
            Some(p)
        } else {
            c_parent
        };

        let code = input.code.unwrap_or(c_code);
        let name = input.name.unwrap_or(c_name);
        let account_type = input.account_type.unwrap_or(c_type);
        let is_bank_cash = input.is_bank_cash.unwrap_or(c_bank);
        let is_active = input.is_active.unwrap_or(c_active);
        let sort_order = input.sort_order.unwrap_or(c_sort);

        let n = conn.execute(
            r#"UPDATE account SET
             code = ?1,
             name = ?2,
             account_type = ?3,
             parent_id = ?4,
             is_bank_cash = ?5,
             is_active = ?6,
             sort_order = ?7
           WHERE id = ?8 AND company_id = ?9"#,
            rusqlite::params![
                code,
                name,
                account_type,
                parent_id,
                is_bank_cash as i64,
                is_active as i64,
                sort_order,
                input.id,
                COMPANY_ID,
            ],
        )?;
        log::debug!(
            target: "kwikbooks_lib::ipc::accounts",
            "account_updated id={} rows={}",
            input.id,
            n
        );
        Ok(RowsAffected { rows_affected: n })
    })
}

#[tauri::command]
pub fn account_deactivate(
    state: State<'_, DbState>,
    id: i64,
) -> Result<RowsAffected, DbCommandError> {
    timed_ipc("account_deactivate", || {
        let conn = open_sqlite(&state.db_path)?;
        let n = conn.execute(
            "UPDATE account SET is_active = 0 WHERE id = ?1 AND company_id = ?2",
            rusqlite::params![id, COMPANY_ID],
        )?;
        if n == 0 {
            return Err(DbCommandError::NotFound {
                entity: "account".into(),
                id,
            });
        }
        log::debug!(
            target: "kwikbooks_lib::ipc::accounts",
            "account_deactivated id={}",
            id
        );
        Ok(RowsAffected { rows_affected: n })
    })
}
