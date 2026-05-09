mod backup;
mod connection;
mod error;
mod migrate;
mod path;

pub use backup::{restore_database_from_path, validate_kwkb_backup, vacuum_backup_database};
pub use connection::open_sqlite;
pub use error::DbCommandError;
pub use migrate::{current_version, run_all, run_all_on_connection};
pub use path::resolve_db_path;

use std::path::PathBuf;

/// Shared app state: absolute path to the SQLite file.
#[derive(Clone)]
pub struct DbState {
    pub db_path: PathBuf,
}
