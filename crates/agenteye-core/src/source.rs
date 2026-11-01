use std::path::Path;

use crate::{AgentKind, MemoryFile, MessageRecord, Result, SessionFile, SessionSummary};

/// Adapter for one agent's on-disk session format.
///
/// Implementations must be read-only and must never hold a file open longer than a call.
pub trait SessionSource: Send + Sync {
    fn kind(&self) -> AgentKind;

    /// Lists session files. A missing root directory is not an error: it yields an empty list.
    fn discover(&self) -> Result<Vec<SessionFile>>;

    /// Builds a [`SessionSummary`] from file metadata plus the head and tail of the file.
    fn summarize(&self, file: &SessionFile) -> Result<SessionSummary>;

    /// Streams message text starting at `from_offset` into `sink`.
    ///
    /// Only complete lines are consumed; a partially written trailing line is left for the next
    /// call. Returns the offset after the last consumed line, which is the `from_offset` to use
    /// to resume. Lines that fail to parse are skipped.
    fn scan_messages(
        &self,
        path: &Path,
        from_offset: u64,
        sink: &mut dyn FnMut(MessageRecord),
    ) -> Result<u64>;
}

/// Adapter for one agent's on-disk memory format.
///
/// Implementations must be read-only and must never hold a file open longer than a call.
pub trait MemorySource: Send + Sync {
    fn kind(&self) -> AgentKind;
    /// A missing root yields an empty list, not an error.
    fn discover(&self) -> Result<Vec<MemoryFile>>;
    /// Whole file content. Errors if `file` is outside this source's root.
    fn read(&self, file: &MemoryFile) -> Result<String>;
}
