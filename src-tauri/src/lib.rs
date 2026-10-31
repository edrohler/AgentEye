//! AgentEye desktop shell: a thin Tauri layer over the `agenteye-*` crates.
//!
//! Commands here only translate between the webview and the core crates; domain logic lives in
//! the crates so it can be tested and benchmarked without a window.

mod commands;
mod state;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![commands::list_sessions])
        .run(tauri::generate_context!())
        .expect("failed to run the AgentEye application");
}
