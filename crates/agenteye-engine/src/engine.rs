use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use agenteye_core::SessionSummary;
use agenteye_index::{Index, SearchHit};
use agenteye_sources::Catalog;

use crate::sync::{sync_all, SyncProgress, SyncSummary};
use crate::{Error, Result};

pub const DEFAULT_SEARCH_LIMIT: usize = 50;
pub const MAX_SEARCH_LIMIT: usize = 200;

/// Session discovery, search and indexing behind one object that is cheap to share.
///
/// Every method takes `&self`, so an `Arc<Engine>` can be used from many threads: searches open
/// their own read-only connection, and at most one sync runs at a time.
pub struct Engine {
    catalog: Catalog,
    db_path: PathBuf,
    syncing: AtomicBool,
}

impl Engine {
    /// Creates the index database (and its schema) at `db_path` if it does not exist yet.
    pub fn new(catalog: Catalog, db_path: impl Into<PathBuf>) -> Result<Self> {
        let db_path = db_path.into();
        Index::open(&db_path)?;
        Ok(Self { catalog, db_path, syncing: AtomicBool::new(false) })
    }

    /// Tier 1: summaries of every session, newest first. Needs no index.
    pub fn list_sessions(&self) -> Vec<SessionSummary> {
        self.catalog.list_sessions()
    }

    /// Full-text search over indexed messages. `limit` is clamped to `1..=MAX_SEARCH_LIMIT`.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let index = Index::open_read_only(&self.db_path)?;
        Ok(index.search(query, limit.clamp(1, MAX_SEARCH_LIMIT))?)
    }

    /// Tier 3: brings the index up to date, reporting progress as it goes.
    ///
    /// Blocking and potentially slow; call it from a background thread. Returns
    /// [`Error::AlreadySyncing`] if another sync is in progress.
    pub fn sync(&self, on_progress: impl FnMut(SyncProgress)) -> Result<SyncSummary> {
        let _running = SyncGuard::acquire(&self.syncing)?;
        let mut index = Index::open(&self.db_path)?;
        Ok(sync_all(&mut index, &self.catalog, on_progress))
    }
}

/// Holds the "a sync is running" flag and clears it when dropped, even if the sync panics or
/// returns early. (Rust runs `Drop` automatically when a value goes out of scope, much like a
/// C++ destructor, which is why this RAII guard is the idiomatic way to release a resource.)
struct SyncGuard<'a>(&'a AtomicBool);

impl<'a> SyncGuard<'a> {
    fn acquire(flag: &'a AtomicBool) -> Result<Self> {
        // `swap` sets the flag and returns the previous value in one atomic step, so two
        // threads can never both see `false`.
        if flag.swap(true, Ordering::AcqRel) {
            Err(Error::AlreadySyncing)
        } else {
            Ok(Self(flag))
        }
    }
}

impl Drop for SyncGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use agenteye_sources::ClaudeSource;

    use super::*;

    fn engine_with_one_session(text: &str) -> (tempfile::TempDir, Engine) {
        let dir = tempfile::tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let project = sessions.join("proj");
        fs::create_dir_all(&project).unwrap();
        let line = format!(r#"{{"type":"user","message":{{"role":"user","content":"{text}"}}}}"#);
        fs::write(project.join("s1.jsonl"), line + "\n").unwrap();

        let catalog = Catalog::new(vec![Box::new(ClaudeSource::new(sessions))]);
        let engine = Engine::new(catalog, dir.path().join("index.db")).unwrap();
        (dir, engine)
    }

    #[test]
    fn sync_makes_sessions_searchable_and_reports_progress() {
        let (_dir, engine) = engine_with_one_session("rotate the api keys");
        assert_eq!(engine.search("rotate", 10).unwrap(), Vec::new());

        let mut reports = Vec::new();
        let summary = engine.sync(|progress| reports.push(progress)).unwrap();

        assert_eq!(summary.files_total, 1);
        assert_eq!(summary.files_failed, 0);
        assert_eq!(summary.messages_added, 1);
        // One report after discovery, one after the only file.
        assert_eq!(reports.len(), 2);
        assert_eq!(reports.last().unwrap().files_done, 1);
        assert_eq!(engine.search("rotate", 10).unwrap().len(), 1);
    }

    #[test]
    fn a_second_sync_while_one_runs_is_rejected_and_the_flag_is_released_after() {
        let (_dir, engine) = engine_with_one_session("hello world");

        let mut nested = None;
        engine
            .sync(|_| {
                if nested.is_none() {
                    nested = Some(engine.sync(|_| {}));
                }
            })
            .unwrap();

        assert!(matches!(nested, Some(Err(Error::AlreadySyncing))));
        // The guard was dropped, so a later sync is allowed again.
        assert!(engine.sync(|_| {}).is_ok());
    }

    #[test]
    fn search_clamps_the_limit_to_at_least_one() {
        let (_dir, engine) = engine_with_one_session("alpha beta");
        engine.sync(|_| {}).unwrap();
        assert_eq!(engine.search("alpha", 0).unwrap().len(), 1);
    }
}
