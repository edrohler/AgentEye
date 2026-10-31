//! Read-only adapters that turn each agent's on-disk session logs into AgentEye's model.

mod catalog;
mod claude;
mod codex;
mod content;
mod fs;
mod jsonl;
mod text;

pub use catalog::Catalog;
pub use claude::ClaudeSource;
pub use codex::CodexSource;
