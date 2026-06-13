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
    timed_ipc("account_list", || account_list_impl(&state.db_path, filter))
}

fn account_list_impl(
    db_path: &std::path::Path,
    filter: AccountFilter,
) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let conn = open_sqlite(db_path)?;
    let v = list_accounts(&conn, &filter)?;
    log::debug!(
        target: "kwikbooks_lib::ipc::accounts",
        "account_list rows={}",
        v.len()
    );
    Ok(v)
}

#[tauri::command]
pub fn account_get(
    state: State<'_, DbState>,
    id: i64,
) -> Result<serde_json::Value, DbCommandError> {
    timed_ipc("account_get", || account_get_impl(&state.db_path, id))
}

fn account_get_impl(db_path: &std::path::Path, id: i64) -> Result<serde_json::Value, DbCommandError> {
    let conn = open_sqlite(db_path)?;
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
}

#[tauri::command]
pub fn account_create(
    state: State<'_, DbState>,
    input: AccountCreateInput,
) -> Result<i64, DbCommandError> {
    timed_ipc("account_create", || account_create_impl(&state.db_path, input))
}

fn account_create_impl(
    db_path: &std::path::Path,
    input: AccountCreateInput,
) -> Result<i64, DbCommandError> {
    let conn = open_sqlite(db_path)?;
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
}

#[tauri::command]
pub fn account_update(
    state: State<'_, DbState>,
    input: AccountUpdateInput,
) -> Result<RowsAffected, DbCommandError> {
    timed_ipc("account_update", || account_update_impl(&state.db_path, input))
}

fn account_update_impl(
    db_path: &std::path::Path,
    input: AccountUpdateInput,
) -> Result<RowsAffected, DbCommandError> {
    let conn = open_sqlite(db_path)?;
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
}

#[tauri::command]
pub fn account_deactivate(
    state: State<'_, DbState>,
    id: i64,
) -> Result<RowsAffected, DbCommandError> {
    timed_ipc("account_deactivate", || account_deactivate_impl(&state.db_path, id))
}

fn account_deactivate_impl(
    db_path: &std::path::Path,
    id: i64,
) -> Result<RowsAffected, DbCommandError> {
    let conn = open_sqlite(db_path)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{run_all, DbCommandError};
    use tempfile::tempdir;

    fn test_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("accounts_cmd.sqlite");
        run_all(&db_path).expect("migrate");
        (dir, db_path)
    }

    #[test]
    fn account_create_input_deserializes_camel_case_payload() {
        let payload = serde_json::json!({
            "code": "6100",
            "name": "Rent",
            "accountType": "expense",
            "parentId": null,
            "isBankCash": false,
            "sortOrder": 10
        });
        let parsed: AccountCreateInput =
            serde_json::from_value(payload).expect("deserialize account create input");
        assert_eq!(parsed.code, "6100");
        assert_eq!(parsed.account_type, "expense");
        assert_eq!(parsed.sort_order, 10);
    }

    #[test]
    fn account_create_get_update_and_deactivate_round_trip() {
        let (_dir, db_path) = test_db();

        let id = account_create_impl(
            &db_path,
            AccountCreateInput {
                code: "6100".into(),
                name: "Rent".into(),
                account_type: "expense".into(),
                parent_id: None,
                is_bank_cash: false,
                sort_order: 10,
            },
        )
        .expect("create");

        let row = account_get_impl(&db_path, id).expect("get");
        assert_eq!(row["code"], "6100");
        assert_eq!(row["name"], "Rent");
        assert_eq!(row["isActive"], true);

        account_update_impl(
            &db_path,
            AccountUpdateInput {
                id,
                code: None,
                name: Some("Office rent".into()),
                account_type: None,
                clear_parent: false,
                parent_id: None,
                is_bank_cash: None,
                is_active: None,
                sort_order: None,
            },
        )
        .expect("update");

        let updated = account_get_impl(&db_path, id).expect("get after update");
        assert_eq!(updated["name"], "Office rent");

        account_deactivate_impl(&db_path, id).expect("deactivate");

        let filter = AccountFilter {
            active_only: Some(true),
            ..Default::default()
        };
        let active = account_list_impl(&db_path, filter).expect("list active");
        assert!(!active.iter().any(|a| a["id"] == id));
    }

    #[test]
    fn account_get_impl_maps_missing_account_to_not_found() {
        let (_dir, db_path) = test_db();

        let err = account_get_impl(&db_path, 9999).expect_err("must fail");
        match err {
            DbCommandError::NotFound { entity, id } => {
                assert_eq!(entity, "account");
                assert_eq!(id, 9999);
            }
            other => panic!("expected not_found error, got {other:?}"),
        }
    }
}
