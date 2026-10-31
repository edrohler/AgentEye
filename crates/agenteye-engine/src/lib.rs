//! Application layer: one API over session discovery, search and background indexing.
//!
//! Front ends (the Tauri shell, the CLI) depend on this crate instead of wiring `sources` and
//! `index` together themselves.

mod engine;
mod error;
mod status;
mod sync;

pub use agenteye_index::{SearchHit, HIGHLIGHT_END, HIGHLIGHT_START};
pub use engine::{Engine, DEFAULT_SEARCH_LIMIT, MAX_SEARCH_LIMIT};
pub use error::{Error, Result};
pub use status::IndexStatus;
pub use sync::{SyncProgress, SyncSummary};
