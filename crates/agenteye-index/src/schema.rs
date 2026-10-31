/// Connection settings: WAL lets searches read while the single writer indexes, and the busy
/// timeout makes a connection wait briefly for a lock instead of failing at once.
pub(crate) const PRAGMAS: &str = "
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA busy_timeout = 5000;
";

/// `files` tracks indexing progress per session file. `messages` is the FTS5 table; only `text`
/// is searchable, the other columns are stored pointers and display fields.
pub(crate) const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS files (
        id             INTEGER PRIMARY KEY,
        path           TEXT NOT NULL UNIQUE,
        agent          TEXT NOT NULL,
        size_bytes     INTEGER NOT NULL,
        mtime_ms       INTEGER NOT NULL,
        indexed_offset INTEGER NOT NULL
    );

    CREATE VIRTUAL TABLE IF NOT EXISTS messages USING fts5(
        text,
        file_id     UNINDEXED,
        byte_offset UNINDEXED,
        role        UNINDEXED,
        timestamp   UNINDEXED,
        tokenize = 'unicode61'
    );
";
