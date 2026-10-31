use agenteye_core::SessionSummary;
use tauri::State;

use crate::state::AppState;

/// Tier 1: every known session, newest first.
///
/// Session discovery does blocking file I/O, so it runs on the blocking pool rather than the
/// async runtime.
#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, String> {
    let catalog = state.catalog();
    tauri::async_runtime::spawn_blocking(move || catalog.list_sessions())
        .await
        .map_err(|error| error.to_string())
}
