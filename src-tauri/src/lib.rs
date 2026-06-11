mod commands;
mod db;
mod domain;
mod ipc_context;
mod ipc_log;
mod logging;

use crate::ipc_log::timed_ipc;
use db::{
    current_version, open_sqlite, resolve_db_path, run_all_on_connection, DbCommandError, DbState,
};
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbInitResponse {
    pub db_path: String,
    pub migration_version: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateResponse {
    pub migration_version_before: i32,
    pub migration_version_after: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub ok: bool,
    pub sqlite_ok: bool,
    pub migration_version: i32,
}

/// Ensures the database exists and all migrations are applied (idempotent).
#[tauri::command]
fn db_init(state: tauri::State<'_, DbState>) -> Result<DbInitResponse, DbCommandError> {
    timed_ipc("db_init", || {
        let mut conn = open_sqlite(&state.db_path)?;
        run_all_on_connection(&mut conn)?;
        let migration_version = current_version(&conn)?;
        log::debug!(
            target: "kwikbooks_lib::ipc",
            "db_init_detail migration_version={}",
            migration_version
        );
        Ok(DbInitResponse {
            db_path: state.db_path.to_string_lossy().into_owned(),
            migration_version,
        })
    })
}

/// Applies any pending migrations and returns the version range touched.
#[tauri::command]
fn db_migrate(state: tauri::State<'_, DbState>) -> Result<MigrateResponse, DbCommandError> {
    timed_ipc("db_migrate", || {
        let mut conn = open_sqlite(&state.db_path)?;
        let migration_version_before = current_version(&conn)?;
        run_all_on_connection(&mut conn)?;
        let migration_version_after = current_version(&conn)?;
        if migration_version_before != migration_version_after {
            log::info!(
                target: "kwikbooks_lib::ipc",
                "db_migrate_applied before={} after={}",
                migration_version_before,
                migration_version_after
            );
        }
        Ok(MigrateResponse {
            migration_version_before,
            migration_version_after,
        })
    })
}

/// Lightweight diagnostics: SQLite ping + schema migration head.
#[tauri::command]
fn health_ping(state: tauri::State<'_, DbState>) -> Result<HealthResponse, DbCommandError> {
    timed_ipc("health_ping", || {
        let conn = open_sqlite(&state.db_path)?;
        conn.query_row("SELECT 1", [], |_| Ok(()))?;
        let migration_version = current_version(&conn)?;
        Ok(HealthResponse {
            ok: true,
            sqlite_ok: true,
            migration_version,
        })
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(logging::log_plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            log::info!(
                target: "kwikbooks_lib::startup",
                "Kwikbooks starting version={} (log_level={:?})",
                env!("CARGO_PKG_VERSION"),
                logging::max_level_from_env()
            );
            let handle = app.handle().clone();
            let db_path = resolve_db_path(&handle)?;
            log::info!(
                target: "kwikbooks_lib::startup",
                "database path = {}",
                db_path.display()
            );
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            db::run_all(&db_path)?;
            app.manage(DbState { db_path });
            log::info!(target: "kwikbooks_lib::startup", "database ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc_context::ipc_context_set,
            db_init,
            db_migrate,
            health_ping,
            commands::db_backup_vacuum,
            commands::db_restore_validate,
            commands::db_restore_apply,
            commands::account_list,
            commands::account_get,
            commands::account_create,
            commands::account_update,
            commands::account_deactivate,
            commands::customer_create,
            commands::vendor_create,
            commands::invoice_create,
            commands::bill_create,
            commands::customer_payment_create,
            commands::vendor_payment_create,
            commands::invoice_post,
            commands::bill_post,
            commands::customer_payment_post,
            commands::vendor_payment_post,
            commands::invoice_set_status,
            commands::bill_set_status,
            commands::report_trial_balance,
            commands::report_general_ledger,
            commands::report_ar_open,
            commands::report_ap_open,
            commands::report_profit_loss,
            commands::report_balance_sheet,
            commands::company_get,
            commands::company_update,
            commands::list_customers,
            commands::list_vendors,
            commands::list_invoices,
            commands::list_bills,
            commands::list_journals,
            commands::get_invoice,
            commands::get_bill,
            commands::import_quickbooks_file,
            commands::global_search,
            commands::logs_read,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
