//! AgentEye desktop shell: a thin Tauri layer over the `agenteye-*` crates.
//!
//! Commands here only translate between the webview and the engine; domain logic lives in the
//! crates so it can be tested and benchmarked without a window.

mod commands;
mod indexing;
mod state;

use tauri::Manager;

use state::AppState;

/// File name of the search index inside the app's data directory.
const INDEX_DB_FILE: &str = "index.db";

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            app.manage(AppState::new(data_dir.join(INDEX_DB_FILE))?);

            // Index in the background so the window opens immediately.
            indexing::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_sessions,
            commands::search,
            commands::index_status,
            commands::start_indexing,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the AgentEye application");
}
