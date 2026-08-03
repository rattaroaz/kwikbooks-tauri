use rusqlite::Connection;

use super::error::DbCommandError;
use super::open_sqlite;

const M001_INITIAL: &str = include_str!("../../migrations/001_initial.sql");
const M002_DOMAIN: &str = include_str!("../../migrations/002_domain.sql");
const M003_SEQUENCES: &str = include_str!("../../migrations/003_company_sequences.sql");
const M004_INDEXES: &str = include_str!("../../migrations/004_performance_indexes.sql");
const M005_CHECKS: &str = include_str!("../../migrations/005_checks.sql");
const M006_TAX_PAYABLE: &str = include_str!("../../migrations/006_tax_payable.sql");
const M007_CHECK_UNIQUE: &str = include_str!("../../migrations/007_check_number_unique.sql");

/// Ordered migrations: `(version, SQL batch)`.
static MIGRATIONS: &[(i32, &str)] = &[
    (1, M001_INITIAL),
    (2, M002_DOMAIN),
    (3, M003_SEQUENCES),
    (4, M004_INDEXES),
    (5, M005_CHECKS),
    (6, M006_TAX_PAYABLE),
    (7, M007_CHECK_UNIQUE),
];

fn ensure_migrations_table(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )?;
    Ok(())
}

fn applied_versions(conn: &Connection) -> Result<std::collections::HashSet<i32>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT version FROM _migrations")?;
    let rows = stmt.query_map([], |row| row.get::<_, i32>(0))?;
    let mut set = std::collections::HashSet::new();
    for v in rows {
        set.insert(v?);
    }
    Ok(set)
}

/// Runs any pending migrations in order inside transactions.
pub fn run_all(db_path: &std::path::Path) -> Result<(), DbCommandError> {
    let mut conn = open_sqlite(db_path)?;
    run_all_on_connection(&mut conn)
}

pub fn run_all_on_connection(conn: &mut Connection) -> Result<(), DbCommandError> {
    ensure_migrations_table(conn)?;
    let mut applied = applied_versions(conn)?;

    for &(version, sql) in MIGRATIONS {
        if applied.contains(&version) {
            continue;
        }

        let tx = conn.transaction().map_err(|e| DbCommandError::Migration {
            version,
            message: e.to_string(),
        })?;

        if let Err(e) = tx.execute_batch(sql) {
            let _ = tx.rollback();
            return Err(DbCommandError::Migration {
                version,
                message: e.to_string(),
            });
        }

        tx.execute(
            "INSERT INTO _migrations (version) VALUES (?1)",
            rusqlite::params![version],
        )
        .map_err(|e| DbCommandError::Migration {
            version,
            message: e.to_string(),
        })?;

        tx.commit().map_err(|e| DbCommandError::Migration {
            version,
            message: e.to_string(),
        })?;

        log::info!(
            target: "kwikbooks_lib::db",
            "migration_applied version={}",
            version
        );
        applied.insert(version);
    }

    Ok(())
}

/// Highest applied migration version, or `0` if none recorded.
pub fn current_version(conn: &Connection) -> Result<i32, DbCommandError> {
    ensure_migrations_table(conn)?;
    let v: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM _migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::db::open_sqlite;

    #[test]
    fn run_migrations_twice_is_idempotent() {
        let dir = tempdir().expect("tempdir");
        let p = dir.path().join("migrate.sqlite");
        run_all(&p).expect("first run");
        let mut c = open_sqlite(&p).expect("open");
        let v1 = current_version(&c).expect("v1");
        assert!(v1 >= 6, "expected migration head including tax payable");
        run_all_on_connection(&mut c).expect("second run");
        let v2 = current_version(&c).expect("v2");
        assert_eq!(v1, v2);
    }
}
