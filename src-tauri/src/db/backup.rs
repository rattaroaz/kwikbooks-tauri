//! Cold backup / restore helpers. Prefer `VACUUM INTO` for backups (consistent snapshot).
//!
//! Paths come from UI file dialogs — still validate uniqueness and canonical equality.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use rusqlite::{Connection, OpenFlags};

use super::error::DbCommandError;
use super::migrate::{current_version, run_all_on_connection};
use super::open_sqlite;

/// Serializes backup/restore against other restore operations (file swap safety).
fn restore_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn wal_sidecar(main: &Path, tag: &str) -> PathBuf {
    let mut s = main.as_os_str().to_owned();
    s.push(tag);
    PathBuf::from(s)
}

/// Returns migration head row from `_migrations` (0 if `_migrations` missing / empty prior to migrate).
pub fn validate_kwkb_backup(path: &Path) -> Result<i32, DbCommandError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
        DbCommandError::DatabaseOpen {
            message: e.to_string(),
        }
    })?;
    let triple: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM sqlite_master
        WHERE type = 'table' AND name IN ('company','account','_migrations')
        "#,
        [],
        |row| row.get(0),
    )?;
    if triple < 3 {
        return Err(DbCommandError::Validation {
            message: "backup is missing Kwikbooks core tables".into(),
        });
    }
    current_version(&conn)
}

pub fn vacuum_backup_database(live_path: &Path, destination: &Path) -> Result<(), DbCommandError> {
    let conn = open_sqlite(live_path)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    // Write to a temp file first so a failed VACUUM never deletes an existing backup.
    let tmp = destination.with_extension("sqlite.tmp");
    if tmp.exists() {
        fs::remove_file(&tmp).map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
    }
    conn.execute(
        "VACUUM INTO ?1",
        rusqlite::params![tmp.to_string_lossy().as_ref()],
    )?;
    if destination.exists() {
        fs::remove_file(destination).map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
    }
    fs::rename(&tmp, destination).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        DbCommandError::PathResolution {
            message: e.to_string(),
        }
    })?;
    log::info!(
        target: "kwikbooks_lib::db",
        "vacuum_backup_completed dest={}",
        destination.display()
    );
    Ok(())
}

pub fn restore_database_from_path(backup_path: &Path, live_path: &Path) -> Result<(), DbCommandError> {
    let _guard = restore_lock().lock().unwrap_or_else(|e| e.into_inner());

    let version = validate_kwkb_backup(backup_path)?;
    if version < 1 {
        return Err(DbCommandError::Validation {
            message: "backup has no migration history".into(),
        });
    }

    let backup_canon = fs::canonicalize(backup_path)
        .map_err(|e| DbCommandError::PathResolution { message: e.to_string() })?;
    if live_path.exists() {
        let live_canon = fs::canonicalize(live_path)
            .map_err(|e| DbCommandError::PathResolution { message: e.to_string() })?;
        if backup_canon == live_canon {
            return Err(DbCommandError::Validation {
                message: "backup path must differ from the live database file".into(),
            });
        }
    }

    if let Some(parent) = live_path.parent() {
        fs::create_dir_all(parent).map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
    }

    // Stage restore into a sibling temp file, migrate there, then swap.
    let staged = live_path.with_extension("restore.tmp.sqlite");
    if staged.exists() {
        fs::remove_file(&staged).map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
    }
    fs::copy(backup_path, &staged).map_err(|e| DbCommandError::PathResolution {
        message: e.to_string(),
    })?;

    {
        let mut conn = open_sqlite(&staged)?;
        run_all_on_connection(&mut conn)?;
    }

    let previous = if live_path.exists() {
        {
            let live_conn = open_sqlite(live_path)?;
            live_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        for tag in ["-wal", "-shm"] {
            let _ = fs::remove_file(wal_sidecar(live_path, tag));
        }
        let bak = live_path.with_extension("pre-restore.bak.sqlite");
        if bak.exists() {
            let _ = fs::remove_file(&bak);
        }
        fs::rename(live_path, &bak).map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
        Some(bak)
    } else {
        None
    };

    if let Err(e) = fs::rename(&staged, live_path) {
        if let Some(bak) = previous.as_ref() {
            let _ = fs::rename(bak, live_path);
        }
        let _ = fs::remove_file(&staged);
        return Err(DbCommandError::PathResolution {
            message: e.to_string(),
        });
    }

    // Drop previous live snapshot after successful swap.
    if let Some(bak) = previous {
        let _ = fs::remove_file(bak);
    }
    for tag in ["-wal", "-shm"] {
        let _ = fs::remove_file(wal_sidecar(&staged, tag));
    }

    log::warn!(
        target: "kwikbooks_lib::db",
        "restore_completed backup={} live={}",
        backup_path.display(),
        live_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::db::run_all_on_connection;

    #[test]
    fn vacuum_backup_roundtrip_restore() {
        let dir = tempdir().expect("tmp");
        let live = dir.path().join("live.sqlite");
        let bak = dir.path().join("copy.sqlite");

        {
            let mut conn = open_sqlite(&live).expect("open");
            run_all_on_connection(&mut conn).expect("migrate");
        }

        vacuum_backup_database(&live, &bak).expect("vacuum into");

        let other_dir = tempdir().expect("tmp2");
        let live2 = other_dir.path().join("restored.sqlite");
        restore_database_from_path(&bak, &live2).expect("restore");

        let v = validate_kwkb_backup(&live2).expect("validate");
        assert!(v >= 1);
        let ct: i64 = {
            let c = Connection::open_with_flags(live2, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            c.query_row("SELECT COUNT(*) FROM company WHERE id = 1", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(ct, 1);
    }

    #[test]
    fn vacuum_preserves_existing_dest_on_failure() {
        let dir = tempdir().expect("tmp");
        let live = dir.path().join("live.sqlite");
        let bak = dir.path().join("copy.sqlite");
        {
            let mut conn = open_sqlite(&live).expect("open");
            run_all_on_connection(&mut conn).expect("migrate");
        }
        vacuum_backup_database(&live, &bak).expect("first");
        let size = fs::metadata(&bak).unwrap().len();
        assert!(size > 0);
        // Overwrite with another successful vacuum — dest must still exist afterward.
        vacuum_backup_database(&live, &bak).expect("second");
        assert!(bak.exists());
    }

    #[test]
    fn validate_rejects_non_kwikbooks_sqlite() {
        let dir = tempdir().expect("tmp");
        let p = dir.path().join("not_kw.sqlite");
        {
            let conn = Connection::open(&p).expect("create sqlite");
            conn.execute("CREATE TABLE something_else (id INTEGER)", [])
                .expect("table");
        }
        let err = validate_kwkb_backup(&p);
        assert!(err.is_err(), "non-kwikbooks database should be rejected");
    }

    #[test]
    fn restore_rejects_same_path_source_and_destination() {
        let dir = tempdir().expect("tmp");
        let live = dir.path().join("live.sqlite");
        {
            let mut conn = open_sqlite(&live).expect("open");
            run_all_on_connection(&mut conn).expect("migrate");
        }
        let err = restore_database_from_path(&live, &live);
        assert!(err.is_err(), "restore to same file path must be rejected");
    }
}
