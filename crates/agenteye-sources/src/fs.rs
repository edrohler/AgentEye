//! Filesystem discovery shared by the adapters.

use std::ffi::OsStr;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::{fs::Metadata, ops::RangeInclusive};

use agenteye_core::{AgentKind, MemoryFile, SessionFile};
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
            let modified_ms = modified_ms(&metadata);
            Some(SessionFile {
                agent,
                path: entry.into_path(),
                size_bytes: metadata.len(),
                modified_ms,
            })
        })
        .collect()
}

/// Finds Markdown memory files: `*.md` directly inside a directory named `memory`, within
/// `depths` of `root` and whose name starts with `name_prefix`.
///
/// For Claude Code (`root` = `<config>/projects`, depths `3..=3`) that is
/// `<project>/memory/*.md`; non-`.md` files and anything nested deeper are skipped.
/// `project` is the first path component below `root`.
///
/// A missing root yields an empty list. Entries that cannot be read (for example because a file
/// vanished mid-walk) are skipped.
pub(crate) fn discover_md(
    agent: AgentKind,
    root: &Path,
    depths: RangeInclusive<usize>,
    name_prefix: &str,
) -> Vec<MemoryFile> {
    WalkDir::new(root)
        .min_depth(*depths.start())
        .max_depth(*depths.end())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy();
            name.starts_with(name_prefix) && name.ends_with(".md")
        })
        .filter(|entry| {
            entry.path().parent().and_then(Path::file_name) == Some(OsStr::new("memory"))
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified_ms = modified_ms(&metadata);
            let project = entry
                .path()
                .strip_prefix(root)
                .ok()?
                .components()
                .next()?
                .as_os_str()
                .to_string_lossy()
                .into_owned();

            Some(MemoryFile {
                agent,
                project,
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry.into_path(),
                size_bytes: metadata.len(),
                modified_ms,
            })
        })
        .collect()
}

fn modified_ms(metadata: &Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}
