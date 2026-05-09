use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use super::error::DbCommandError;

/// Canonical DB location: `<app_data_dir>/com.kwikbooks.app/data/kwikbooks.sqlite`.
pub fn resolve_db_path(handle: &AppHandle) -> Result<PathBuf, DbCommandError> {
    let mut dir = handle
        .path()
        .app_data_dir()
        .map_err(|e| DbCommandError::PathResolution {
            message: e.to_string(),
        })?;
    dir.push("com.kwikbooks.app");
    dir.push("data");
    Ok(dir.join("kwikbooks.sqlite"))
}
