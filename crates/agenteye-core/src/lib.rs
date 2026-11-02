//! Domain types and the [`SessionSource`] contract shared by every `AgentEye` crate.
//!
//! This crate performs no I/O of its own; it only defines what an agent session looks like
//! and what an adapter must be able to do.

mod agent;
mod error;
mod memory;
mod session;
mod source;

pub use agent::AgentKind;
pub use error::{Error, Result};
pub use memory::{MemoryFile, MemoryHeader};
pub use session::{MessageRecord, Role, SessionFile, SessionSummary};
pub use source::{MemorySource, SessionSource};
