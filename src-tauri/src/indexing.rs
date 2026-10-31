//! Runs the index sync on a background thread and tells the UI how it is going.

use agenteye_engine::{Error, IndexStatus};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

/// Name of the event the UI listens to; its payload is an [`IndexStatus`].
const STATUS_EVENT: &str = "index-status";

/// Starts a sync in the background and returns immediately.
///
/// Does nothing if a sync is already running, so it is safe to call repeatedly.
pub fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || run(&app));
}

fn run(app: &AppHandle) {
    let state = app.state::<AppState>();
    let engine = state.engine();

    let final_status = match engine.sync(|progress| publish(app, &state, progress.into())) {
        Ok(_) => IndexStatus::Idle,
        // Another sync owns the status; do not overwrite what it is reporting.
        Err(Error::AlreadySyncing) => return,
        Err(error) => IndexStatus::Failed { message: error.to_string() },
    };
    publish(app, &state, final_status);
}

/// Records the status (so a UI that connects late can ask for it) and notifies listeners.
fn publish(app: &AppHandle, state: &AppState, status: IndexStatus) {
    // Emitting can fail only if the payload cannot be serialized or no window exists; the
    // recorded status remains available through the `index_status` command either way.
    let _ = app.emit(STATUS_EVENT, &status);
    state.set_status(status);
}
