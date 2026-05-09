use serde::Serialize;

use crate::ipc_log::timed_ipc;
use crate::db::{
    restore_database_from_path, validate_kwkb_backup, vacuum_backup_database, DbCommandError,
    DbState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupValidateResponse {
    pub ok: bool,
    pub migration_version: i32,
}

/// Writes a consistent snapshot of the live DB to `destination_path` (`VACUUM INTO`).
fn db_backup_vacuum_impl(
    db_path: &std::path::Path,
    destination_path: &str,
) -> Result<(), DbCommandError> {
    let dest = std::path::PathBuf::from(destination_path.trim());
    vacuum_backup_database(db_path, &dest)
}

#[tauri::command(rename_all = "camelCase")]
pub fn db_backup_vacuum(
    state: tauri::State<'_, DbState>,
    destination_path: String,
) -> Result<(), DbCommandError> {
    timed_ipc("db_backup_vacuum", || {
        log::debug!(
            target: "kwikbooks_lib::ipc::backup",
            "backup_start dest={}",
            destination_path.trim()
        );
        db_backup_vacuum_impl(&state.db_path, &destination_path)
    })
}

/// Read-only check that a file looks like a Kwikbooks SQLite file.
fn db_restore_validate_impl(source_path: &str) -> Result<BackupValidateResponse, DbCommandError> {
    let src = std::path::PathBuf::from(source_path.trim());
    let v = validate_kwkb_backup(&src)?;
    Ok(BackupValidateResponse {
        ok: true,
        migration_version: v,
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn db_restore_validate(source_path: String) -> Result<BackupValidateResponse, DbCommandError> {
    timed_ipc("db_restore_validate", || {
        log::debug!(
            target: "kwikbooks_lib::ipc::backup",
            "restore_validate src={}",
            source_path.trim()
        );
        db_restore_validate_impl(&source_path)
    })
}

/// Replace the live database file with a validated backup, then run pending migrations.
fn db_restore_apply_impl(db_path: &std::path::Path, source_path: &str) -> Result<(), DbCommandError> {
    let src = std::path::PathBuf::from(source_path.trim());
    restore_database_from_path(&src, db_path)
}

#[tauri::command(rename_all = "camelCase")]
pub fn db_restore_apply(
    state: tauri::State<'_, DbState>,
    source_path: String,
) -> Result<(), DbCommandError> {
    timed_ipc("db_restore_apply", || {
        log::warn!(
            target: "kwikbooks_lib::ipc::backup",
            "restore_apply_start src={}",
            source_path.trim()
        );
        db_restore_apply_impl(&state.db_path, &source_path)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_sqlite, run_all};
    use tempfile::tempdir;

    #[test]
    fn db_restore_validate_impl_rejects_missing_file() {
        let dir = tempdir().expect("tmp");
        let missing = dir.path().join("does-not-exist.sqlite");
        let err = db_restore_validate_impl(missing.to_string_lossy().as_ref())
            .expect_err("missing path should fail");
        match err {
            DbCommandError::DatabaseOpen { .. } => {}
            other => panic!("expected database_open error, got {other:?}"),
        }
    }

    #[test]
    fn db_restore_apply_impl_rejects_same_path() {
        let dir = tempdir().expect("tmp");
        let live = dir.path().join("live.sqlite");
        run_all(&live).expect("migrate");
        let err = db_restore_apply_impl(&live, live.to_string_lossy().as_ref()).expect_err("must fail");
        match err {
            DbCommandError::Validation { message } => {
                assert!(message.contains("must differ"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn db_backup_vacuum_impl_creates_backup_file() {
        let dir = tempdir().expect("tmp");
        let live = dir.path().join("live.sqlite");
        let backup = dir.path().join("backup.sqlite");
        run_all(&live).expect("migrate");
        let conn = open_sqlite(&live).expect("open");
        let exists: i64 = conn
            .query_row("SELECT COUNT(*) FROM company WHERE id = 1", [], |row| row.get(0))
            .expect("company");
        assert_eq!(exists, 1);

        db_backup_vacuum_impl(&live, backup.to_string_lossy().as_ref()).expect("backup");
        assert!(backup.exists(), "backup file should exist");
    }
}
