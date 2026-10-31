//! Filesystem discovery shared by the adapters.

use std::ops::RangeInclusive;
use std::path::Path;
use std::time::UNIX_EPOCH;

use agenteye_core::{AgentKind, SessionFile};
use walkdir::WalkDir;

/// Finds `*.jsonl` files whose name starts with `name_prefix`, within `depths` of `root`.
///
/// A missing root yields an empty list. Entries that cannot be read (for example because a file
/// vanished mid-walk) are skipped.
pub(crate) fn discover_jsonl(
    agent: AgentKind,
    root: &Path,
    depths: RangeInclusive<usize>,
    name_prefix: &str,
) -> Vec<SessionFile> {
    WalkDir::new(root)
        .min_depth(*depths.start())
        .max_depth(*depths.end())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy();
            name.starts_with(name_prefix) && name.ends_with(".jsonl")
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified_ms = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
            Some(SessionFile {
                agent,
                path: entry.into_path(),
                size_bytes: metadata.len(),
                modified_ms,
            })
        })
        .collect()
}
