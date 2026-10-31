use std::sync::Arc;

use agenteye_sources::Catalog;

/// Shared, read-only application state.
pub struct AppState {
    catalog: Arc<Catalog>,
}

impl AppState {
    pub fn new() -> Self {
        Self { catalog: Arc::new(Catalog::from_env()) }
    }

    /// A handle that can be moved onto a blocking worker thread.
    pub fn catalog(&self) -> Arc<Catalog> {
        Arc::clone(&self.catalog)
    }
}
