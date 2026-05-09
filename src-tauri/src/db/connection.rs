use std::path::Path;

use rusqlite::Connection;

use super::error::DbCommandError;

/// Opens SQLite with settings required for data integrity and desktop performance.
pub fn open_sqlite(path: &Path) -> Result<Connection, DbCommandError> {
    let conn = Connection::open(path).map_err(|e| {
        log::warn!(
            target: "kwikbooks_lib::db",
            "sqlite_open_failed path={} error={}",
            path.display(),
            e
        );
        DbCommandError::Sql {
            message: e.to_string(),
        }
    })?;
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        "#,
    )
    .map_err(|e| {
        log::warn!(
            target: "kwikbooks_lib::db",
            "sqlite_pragma_failed path={} error={}",
            path.display(),
            e
        );
        DbCommandError::Sql {
            message: e.to_string(),
        }
    })?;
    Ok(conn)
}
