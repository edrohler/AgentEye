//! Tier 3: a disposable full-text search index over session messages.
//!
//! The index is a cache. It stores message text, a pointer (file path and byte offset) to where
//! it came from, and how far each file has been indexed. Deleting the database loses nothing.

mod error;
mod index;
mod query;
mod schema;

pub use error::{IndexError, Result};
pub use index::{Index, SearchHit, SyncOutcome, HIGHLIGHT_END, HIGHLIGHT_START};
