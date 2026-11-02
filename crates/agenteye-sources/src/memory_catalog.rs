use crate::ClaudeSource;
use agenteye_core::{AgentKind, Error, MemoryFile, MemorySource, Result};

use tracing::warn;

/// All configured agent memory sources, queried together.
///
/// Separate from [`crate::Catalog`] on purpose: only agents that keep memory files are
/// registered here, so an agent without them (Codex) is simply absent.
pub struct MemoryCatalog {
    sources: Vec<Box<dyn MemorySource>>,
}

impl MemoryCatalog {
    pub fn new(sources: Vec<Box<dyn MemorySource>>) -> Self {
        Self { sources }
    }

    /// Every agent with memory support, located via its standard environment variable or home
    /// directory. Currently Claude Code only.
    pub fn from_env() -> Self {
        let mut sources: Vec<Box<dyn MemorySource>> = Vec::new();
        if let Some(source) = ClaudeSource::from_env() {
            sources.push(Box::new(source));
        }
        Self::new(sources)
    }

    /// The source registered for `kind`, if any.
    pub fn source(&self, kind: AgentKind) -> Option<&dyn MemorySource> {
        self.sources.iter().find(|s| s.kind() == kind).map(Box::as_ref)
    }

    /// Memory files from every source, newest first. A failing source is logged and skipped.
    pub fn discover(&self) -> Vec<MemoryFile> {
        let mut files: Vec<MemoryFile> = self
            .sources
            .iter()
            .flat_map(|source| {
                source.discover().unwrap_or_else(|error| {
                    warn!(agent = %source.kind(), %error, "memory discovery failed");
                    Vec::new()
                })
            })
            .collect();
        files.sort_by_key(|file| std::cmp::Reverse(file.modified_ms));
        files
    }

    /// Whole content of `file`, read through the source for its agent.
    ///
    /// # Errors
    /// [`Error::MemorySourceNotFound`] if no source is registered for `file.agent`; otherwise
    /// whatever the source returns, for example [`Error::PathOutsideRoot`] or [`Error::Io`].
    pub fn read(&self, file: &MemoryFile) -> Result<String> {
        self.source(file.agent).ok_or(Error::MemorySourceNotFound(file.agent))?.read(file)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    /// A canned source: lists `files`, or fails discovery when `files` is `None`.
    struct FakeSource {
        kind: AgentKind,
        files: Option<Vec<MemoryFile>>,
    }

    impl MemorySource for FakeSource {
        fn kind(&self) -> AgentKind {
            self.kind
        }

        fn discover(&self) -> Result<Vec<MemoryFile>> {
            self.files.clone().ok_or_else(|| Error::UnknownAgent("boom".to_owned()))
        }

        fn read(&self, file: &MemoryFile) -> Result<String> {
            Ok(format!("fake:{}", file.name))
        }
    }

    fn memory_file(agent: AgentKind, name: &str, modified_ms: i64) -> MemoryFile {
        MemoryFile {
            agent,
            project: "proj".to_owned(),
            name: name.to_owned(),
            path: PathBuf::from(name),
            size_bytes: 0,
            modified_ms,
        }
    }

    fn fake(kind: AgentKind, files: Option<Vec<MemoryFile>>) -> Box<dyn MemorySource> {
        Box::new(FakeSource { kind, files })
    }

    #[test]
    fn empty_catalog_discovers_nothing() {
        assert_eq!(MemoryCatalog::new(Vec::new()).discover(), Vec::new());
    }

    #[test]
    fn source_is_found_by_kind() {
        let catalog = MemoryCatalog::new(vec![fake(AgentKind::Claude, Some(Vec::new()))]);

        assert_eq!(
            catalog.source(AgentKind::Claude).map(MemorySource::kind),
            Some(AgentKind::Claude)
        );
        assert!(catalog.source(AgentKind::Codex).is_none());
    }

    #[test]
    fn failing_source_is_skipped() {
        let good = memory_file(AgentKind::Claude, "a.md", 1);
        let catalog = MemoryCatalog::new(vec![
            fake(AgentKind::Codex, None),
            fake(AgentKind::Claude, Some(vec![good.clone()])),
        ]);

        assert_eq!(catalog.discover(), vec![good]);
    }

    #[test]
    fn discover_lists_newest_first_across_sources() {
        let old = memory_file(AgentKind::Claude, "old.md", 10);
        let new = memory_file(AgentKind::Codex, "new.md", 30);
        let mid = memory_file(AgentKind::Claude, "mid.md", 20);
        let catalog = MemoryCatalog::new(vec![
            fake(AgentKind::Claude, Some(vec![old.clone(), mid.clone()])),
            fake(AgentKind::Codex, Some(vec![new.clone()])),
        ]);

        assert_eq!(catalog.discover(), vec![new, mid, old]);
    }

    #[test]
    fn read_dispatches_by_agent() {
        let catalog = MemoryCatalog::new(vec![
            fake(AgentKind::Claude, Some(Vec::new())),
            fake(AgentKind::Codex, Some(Vec::new())),
        ]);
        let file = memory_file(AgentKind::Codex, "x.md", 0);

        assert_eq!(catalog.read(&file).unwrap(), "fake:x.md");
    }

    #[test]
    fn read_without_matching_source_is_an_error() {
        let catalog = MemoryCatalog::new(vec![fake(AgentKind::Claude, Some(Vec::new()))]);
        let file = memory_file(AgentKind::Codex, "x.md", 0);

        let err = catalog.read(&file).unwrap_err();
        assert!(matches!(err, Error::MemorySourceNotFound(AgentKind::Codex)));
        assert_eq!(err.to_string(), "no memory source found for agent `codex`");
    }

    #[test]
    fn claude_source_round_trips_through_the_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let memory = dir.path().join("proj").join("memory");
        fs::create_dir_all(&memory).unwrap();
        fs::write(memory.join("a.md"), "---\nname: a\n---\nbody\n").unwrap();
        let catalog = MemoryCatalog::new(vec![Box::new(ClaudeSource::new(dir.path()))]);

        let files = catalog.discover();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].project, "proj");
        assert_eq!(catalog.read(&files[0]).unwrap(), "---\nname: a\n---\nbody\n");
    }

    #[test]
    fn missing_claude_root_is_empty_through_the_catalog() {
        let catalog = MemoryCatalog::new(vec![Box::new(ClaudeSource::new("does/not/exist"))]);

        assert_eq!(catalog.discover(), Vec::new());
    }

    #[test]
    fn vanished_file_is_an_io_error_through_the_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let memory = dir.path().join("proj").join("memory");
        fs::create_dir_all(&memory).unwrap();
        fs::write(memory.join("a.md"), "x").unwrap();
        let catalog = MemoryCatalog::new(vec![Box::new(ClaudeSource::new(dir.path()))]);
        let file = catalog.discover().remove(0);
        fs::remove_file(&file.path).unwrap();

        assert!(matches!(catalog.read(&file).unwrap_err(), Error::Io { .. }));
    }
}
