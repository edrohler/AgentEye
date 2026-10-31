//! Commands the webview can call. Each one validates input, calls the engine, and maps errors
//! to strings; none contains domain logic.

use agenteye_core::SessionSummary;
use agenteye_engine::{IndexStatus, SearchHit, DEFAULT_SEARCH_LIMIT};
use tauri::{AppHandle, State};

use crate::indexing;
use crate::state::AppState;

/// Tier 1: every known session, newest first.
///
/// Discovery does blocking file I/O, so it runs on the blocking pool rather than the async
/// runtime.
#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, String> {
    let engine = state.engine();
    tauri::async_runtime::spawn_blocking(move || engine.list_sessions())
        .await
        .map_err(|error| error.to_string())
}

/// Full-text search over indexed messages. `limit` defaults to a sensible page size.
#[tauri::command]
pub async fn search(
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchHit>, String> {
    let engine = state.engine();
    let limit = limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
    tauri::async_runtime::spawn_blocking(move || engine.search(&query, limit))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

/// The indexer's current status, for a UI that connected after the last status event.
#[tauri::command]
pub fn index_status(state: State<'_, AppState>) -> IndexStatus {
    state.status()
}

/// Starts (or re-runs) background indexing. A no-op while a sync is already running.
#[tauri::command]
pub fn start_indexing(app: AppHandle) {
    indexing::start(&app);
}
