use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use agenteye_core::{AgentKind, SessionFile, SessionSource};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::Serialize;

use crate::query::to_fts_query;
use crate::schema::{PRAGMAS, SCHEMA};
use crate::Result;

/// Wraps matched words in [`SearchHit::snippet`]. Control characters are used because, unlike
/// `[` or `*`, they cannot occur in real message text, so a UI can split on them unambiguously.
pub const HIGHLIGHT_START: char = '\u{2}';
pub const HIGHLIGHT_END: char = '\u{3}';

/// How long a connection waits for another connection's lock before failing.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Work done by one [`Index::sync_file`] call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncOutcome {
    pub messages_added: usize,
    pub bytes_scanned: u64,
    /// The file shrank since it was last indexed, so its old entries were dropped and re-read.
    pub rebuilt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub agent: AgentKind,
    pub path: PathBuf,
    pub role: String,
    pub timestamp: Option<String>,
    /// Byte offset of the matching line in `path`.
    pub byte_offset: u64,
    /// Match excerpt with each hit wrapped in [`HIGHLIGHT_START`] and [`HIGHLIGHT_END`].
    pub snippet: String,
}

pub struct Index {
    conn: Connection,
}

impl Index {
    /// Opens the database for reading and writing, creating it and its schema if needed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    /// Opens an existing database for searching only. Fails if it does not exist, so a mistyped
    /// path is an error instead of a silently created empty index. Safe to use while another
    /// connection is writing (WAL mode).
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(PRAGMAS)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Brings the index up to date for one file, reading only bytes appended since the last sync.
    ///
    /// Runs in a single transaction, so an interrupted sync leaves the previous state intact.
    pub fn sync_file(
        &mut self,
        source: &dyn SessionSource,
        file: &SessionFile,
    ) -> Result<SyncOutcome> {
        let path = file.path.to_string_lossy().into_owned();
        let size = to_i64(file.size_bytes);
        let tx = self.conn.transaction()?;

        let known: Option<(i64, i64, i64)> = tx
            .query_row(
                "SELECT id, indexed_offset, size_bytes FROM files WHERE path = ?1",
                [&path],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let (file_id, start, rebuilt) = match known {
            // Smaller than when it was last read, so it was replaced or truncated and what we hold is
            // stale. This compares with the stored size, not the offset: the offset stops before an
            // unfinished last line, so a shorter replacement can still be larger than the offset.
            Some((id, offset, stored_size)) if size < stored_size || offset > size => {
                tx.execute("DELETE FROM messages WHERE file_id = ?1", [id])?;
                (id, 0, true)
            }
            Some((_, offset, _)) if offset == size => return Ok(SyncOutcome::default()),
            Some((id, offset, _)) => (id, offset, false),
            None => {
                tx.execute(
                    "INSERT INTO files (path, agent, size_bytes, mtime_ms, indexed_offset)
                     VALUES (?1, ?2, ?3, ?4, 0)",
                    params![path, file.agent.as_str(), size, file.modified_ms],
                )?;
                (tx.last_insert_rowid(), 0, false)
            }
        };

        let mut messages_added = 0_usize;
        let mut insert_error: Option<rusqlite::Error> = None;
        let end = {
            let mut insert = tx.prepare_cached(
                "INSERT INTO messages (text, file_id, byte_offset, role, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            source.scan_messages(&file.path, u64::try_from(start).unwrap_or(0), &mut |record| {
                if insert_error.is_some() {
                    return;
                }
                let result = insert.execute(params![
                    record.text,
                    file_id,
                    to_i64(record.offset),
                    record.role.as_str(),
                    record.timestamp,
                ]);
                match result {
                    Ok(_) => messages_added += 1,
                    Err(error) => insert_error = Some(error),
                }
            })?
        };
        if let Some(error) = insert_error {
            return Err(error.into());
        }

        tx.execute(
            "UPDATE files SET size_bytes = ?1, mtime_ms = ?2, indexed_offset = ?3 WHERE id = ?4",
            params![size, file.modified_ms, to_i64(end), file_id],
        )?;
        tx.commit()?;

        Ok(SyncOutcome {
            messages_added,
            bytes_scanned: end.saturating_sub(u64::try_from(start).unwrap_or(0)),
            rebuilt,
        })
    }

    /// Removes entries for session files that were deleted from disk, with their messages, so
    /// search never returns text from a session that no longer exists. Returns how many files
    /// were removed.
    ///
    /// A file counts as deleted only if its folder still exists but the file does not. If the
    /// folder is gone too (an unmounted drive, a moved home directory) nothing is removed, because
    /// the files may come back and this index is only a cache.
    pub fn prune_deleted(&mut self) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let entries = {
            let mut statement = tx.prepare("SELECT id, path FROM files")?;
            let rows = statement
                .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut removed = 0;
        for (id, path) in entries {
            if is_confirmed_deleted(Path::new(&path)) {
                tx.execute("DELETE FROM messages WHERE file_id = ?1", [id])?;
                tx.execute("DELETE FROM files WHERE id = ?1", [id])?;
                removed += 1;
            }
        }
        tx.commit()?;
        Ok(removed)
    }

    /// Full-text search, best matches first. Every word in `query` must appear in a message.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let Some(fts_query) = to_fts_query(query) else {
            return Ok(Vec::new());
        };

        let mut statement = self.conn.prepare_cached(
            "SELECT f.agent, f.path, messages.role, messages.timestamp, messages.byte_offset,
                    snippet(messages, 0, char(2), char(3), '…', 16)
             FROM messages
             JOIN files f ON f.id = CAST(messages.file_id AS INTEGER)
             WHERE messages MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![fts_query, to_i64(limit as u64)], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        let mut hits = Vec::new();
        for row in rows {
            let (agent, path, role, timestamp, byte_offset, snippet) = row?;
            // A row naming an agent this build does not know (for example from a newer version) is skipped.
            let Ok(agent) = agent.parse::<AgentKind>() else { continue };
            hits.push(SearchHit {
                agent,
                path: PathBuf::from(path),
                role,
                timestamp,
                byte_offset: u64::try_from(byte_offset).unwrap_or(0),
                snippet,
            });
        }
        Ok(hits)
    }
}

/// True only when the file's folder exists and the file is definitely not in it. Any other
/// failure (permissions, a missing folder) is not proof of deletion.
fn is_confirmed_deleted(path: &Path) -> bool {
    let folder_exists = path.parent().is_some_and(Path::is_dir);
    folder_exists
        && matches!(std::fs::metadata(path), Err(error) if error.kind() == ErrorKind::NotFound)
}

/// SQLite integers are signed 64-bit; clamp rather than wrap for values beyond `i64::MAX`.
fn to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
