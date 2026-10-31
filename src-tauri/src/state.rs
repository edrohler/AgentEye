use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use agenteye_engine::{Engine, IndexStatus, Result};
use agenteye_sources::Catalog;

/// Shared application state, registered with Tauri once at startup.
pub struct AppState {
    engine: Arc<Engine>,
    status: Mutex<IndexStatus>,
}

impl AppState {
    /// Creates the engine, and the index database at `db_path` if it does not exist yet.
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let engine = Engine::new(Catalog::from_env(), db_path)?;
        Ok(Self { engine: Arc::new(engine), status: Mutex::new(IndexStatus::default()) })
    }

    /// A handle that can be moved onto a background thread.
    pub fn engine(&self) -> Arc<Engine> {
        Arc::clone(&self.engine)
    }

    pub fn status(&self) -> IndexStatus {
        self.status.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn set_status(&self, status: IndexStatus) {
        *self.status.lock().unwrap_or_else(PoisonError::into_inner) = status;
    }
}
