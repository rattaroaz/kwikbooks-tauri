//! Import QuickBooks-compatible export files into the live SQLite database.

use std::path::Path;

use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::import;
use tauri::State;

fn read_import_file(path: &Path) -> Result<String, DbCommandError> {
    let bytes = std::fs::read(path)?;
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let tail = &bytes[2..];
        if tail.len() % 2 != 0 {
            return Err(DbCommandError::Validation {
                message: "UTF-16 file has odd byte length after BOM.".into(),
            });
        }
        let u16s: Vec<u16> = tail
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return Ok(String::from_utf16_lossy(&u16s).to_string());
    }
    String::from_utf8(bytes).map_err(|e| DbCommandError::Validation {
        message: format!("File is not valid UTF-8 (try re-exporting as UTF-8 CSV/IIF): {}", e),
    })
}

#[tauri::command]
pub fn import_quickbooks_file(
    state: State<'_, DbState>,
    path: String,
) -> Result<import::ImportSummary, DbCommandError> {
    timed_ipc("import_quickbooks_file", || {
        let p = Path::new(path.trim());
        let content = read_import_file(p)?;
        let hint = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let mut conn = open_sqlite(&state.db_path)?;
        let summary = import::run_import(&mut conn, &content, &hint)?;
        log::info!(
            target: "kwikbooks_lib::ipc::import",
            "import_quickbooks_file format={} accounts={} customers={} vendors={} items={}",
            summary.format_detected,
            summary.accounts_created,
            summary.customers_created,
            summary.vendors_created,
            summary.items_created
        );
        Ok(summary)
    })
}
