use crate::ipc_log::timed_ipc;
use crate::db::{open_sqlite, DbCommandError, DbState};
use crate::domain::search;
use tauri::State;

#[tauri::command]
pub fn global_search(
    state: State<'_, DbState>,
    query: String,
    limit: Option<i64>,
) -> Result<search::GlobalSearchResponse, DbCommandError> {
    timed_ipc("global_search", || {
        let trimmed = query.trim();
        if trimmed.len() > 200 {
            return Err(DbCommandError::Validation {
                message: "Search query is too long (max 200 characters).".into(),
            });
        }
        let conn = open_sqlite(&state.db_path)?;
        let lim = limit.unwrap_or(12);
        let out = search::global_search(&conn, trimmed, lim)?;
        log::debug!(
            target: "kwikbooks_lib::ipc::search",
            "global_search hits={}",
            out.hits.len()
        );
        Ok(out)
    })
}
