use agenteye_core::{AgentKind, SessionFile, SessionSource, SessionSummary};
use rayon::prelude::*;
use tracing::warn;

use crate::{ClaudeSource, CodexSource};

/// All configured agent sources, queried together.
pub struct Catalog {
    sources: Vec<Box<dyn SessionSource>>,
}

impl Catalog {
    pub fn new(sources: Vec<Box<dyn SessionSource>>) -> Self {
        Self { sources }
    }

    /// Every supported agent, located via its standard environment variable or home directory.
    pub fn from_env() -> Self {
        let mut sources: Vec<Box<dyn SessionSource>> = Vec::new();
        if let Some(source) = ClaudeSource::from_env() {
            sources.push(Box::new(source));
        }
        if let Some(source) = CodexSource::from_env() {
            sources.push(Box::new(source));
        }
        Self::new(sources)
    }

    pub fn source(&self, kind: AgentKind) -> Option<&dyn SessionSource> {
        self.sources.iter().find(|s| s.kind() == kind).map(|s| s.as_ref())
    }

    /// Tier 1: list session files from every source using file metadata only.
    pub fn discover(&self) -> Vec<SessionFile> {
        self.sources
            .iter()
            .flat_map(|source| {
                source.discover().unwrap_or_else(|error| {
                    warn!(agent = %source.kind(), %error, "session discovery failed");
                    Vec::new()
                })
            })
            .collect()
    }

    /// Summaries for all sessions, newest activity first. Unreadable sessions are skipped.
    pub fn list_sessions(&self) -> Vec<SessionSummary> {
        let mut sessions: Vec<SessionSummary> = self
            .discover()
            .par_iter()
            .filter_map(|file| {
                let source = self.source(file.agent)?;
                source
                    .summarize(file)
                    .map_err(|error| warn!(path = %file.path.display(), %error, "summarize failed"))
                    .ok()
            })
            .collect();
        sessions.sort_by(|a, b| b.last_activity_ms.cmp(&a.last_activity_ms));
        sessions
    }
}
