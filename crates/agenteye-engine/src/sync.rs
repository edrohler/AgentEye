use std::cmp::Reverse;

use agenteye_index::Index;
use agenteye_sources::Catalog;
use serde::Serialize;
use tracing::warn;

/// Where a running sync is, reported after discovery and after every file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub files_done: usize,
    pub files_total: usize,
    pub messages_added: usize,
}

/// Totals for a finished sync.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncSummary {
    pub files_total: usize,
    /// Files that could not be indexed (unreadable, vanished mid-sync, ...). They are skipped.
    pub files_failed: usize,
    pub messages_added: usize,
    pub bytes_scanned: u64,
}

/// Brings `index` up to date with every session the catalog can see.
///
/// Newest sessions go first so recent work becomes searchable soonest. One bad file never stops
/// the run; it is logged and counted in [`SyncSummary::files_failed`].
pub(crate) fn sync_all(
    index: &mut Index,
    catalog: &Catalog,
    mut on_progress: impl FnMut(SyncProgress),
) -> SyncSummary {
    let mut files = catalog.discover();
    files.sort_by_key(|file| Reverse(file.modified_ms));

    let mut summary = SyncSummary { files_total: files.len(), ..SyncSummary::default() };
    let mut progress = SyncProgress { files_total: files.len(), ..SyncProgress::default() };
    on_progress(progress);

    for file in &files {
        let outcome = catalog.source(file.agent).ok_or(()).and_then(|source| {
            index.sync_file(source, file).map_err(|error| {
                warn!(path = %file.path.display(), %error, "indexing failed");
            })
        });
        match outcome {
            Ok(outcome) => {
                summary.messages_added += outcome.messages_added;
                summary.bytes_scanned += outcome.bytes_scanned;
            }
            Err(()) => summary.files_failed += 1,
        }

        progress.files_done += 1;
        progress.messages_added = summary.messages_added;
        on_progress(progress);
    }
    summary
}
