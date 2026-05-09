//! Cold backup / restore helpers. Prefer `VACUUM INTO` for backups (consistent snapshot).
//!
//! Paths come from UI file dialogs — still validate uniqueness and canonical equality.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use super::error::DbCommandError;
use super::migrate::{current_version, run_all_on_connection};
use super::open_sqlite;

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
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    conn.execute(
        "VACUUM INTO ?1",
        rusqlite::params![destination.to_string_lossy().as_ref()],
    )?;
    log::info!(
        target: "kwikbooks_lib::db",
        "vacuum_backup_completed dest={}",
        destination.display()
    );
    Ok(())
}

pub fn restore_database_from_path(backup_path: &Path, live_path: &Path) -> Result<(), DbCommandError> {
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

    if live_path.exists() {
        {
            let live_conn = open_sqlite(live_path)?;
            live_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        for tag in ["-wal", "-shm"] {
            let _ = fs::remove_file(wal_sidecar(live_path, tag));
        }
    } else if let Some(parent) = live_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(backup_path, live_path).map_err(|e| DbCommandError::PathResolution {
        message: e.to_string(),
    })?;

    let mut conn = open_sqlite(live_path)?;
    run_all_on_connection(&mut conn)?;
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
