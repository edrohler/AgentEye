//! Offset-based, incremental JSONL reading.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::ops::ControlFlow;
use std::path::Path;

use agenteye_core::{Error, Result};
use serde::Deserialize;

const READ_BUFFER_BYTES: usize = 256 * 1024;

/// Calls `visit(offset, line)` for each complete line at or after `from`.
///
/// `line` excludes its line terminator. A trailing line without a terminator is assumed to be
/// mid-write and is not visited. Returns the offset just past the last visited line, so callers
/// can resume from there.
pub(crate) fn for_each_line(
    path: &Path,
    from: u64,
    mut visit: impl FnMut(u64, &[u8]) -> ControlFlow<()>,
) -> Result<u64> {
    let io_err = |e| Error::io(path, e);
    let mut file = File::open(path).map_err(io_err)?;
    file.seek(SeekFrom::Start(from)).map_err(io_err)?;

    let mut reader = BufReader::with_capacity(READ_BUFFER_BYTES, file);
    let mut line = Vec::new();
    let mut offset = from;
    loop {
        line.clear();
        let read = reader.read_until(b'\n', &mut line).map_err(io_err)?;
        if read == 0 || line.last() != Some(&b'\n') {
            break;
        }
        let flow = visit(offset, trim_terminator(&line));
        offset += read as u64;
        if flow.is_break() {
            break;
        }
    }
    Ok(offset)
}

/// Reads up to the last `max_bytes` of the file, starting at a line boundary.
///
/// If the file is larger than `max_bytes`, the leading partial line is dropped.
pub(crate) fn read_tail(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let io_err = |e| Error::io(path, e);
    let mut file = File::open(path).map_err(io_err)?;
    let len = file.metadata().map_err(io_err)?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start)).map_err(io_err)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(io_err)?;
    if start > 0 {
        let first_line = buf.iter().position(|&b| b == b'\n').map_or(buf.len(), |i| i + 1);
        buf.drain(..first_line);
    }
    Ok(buf)
}

/// Lenient JSON decode: unknown or malformed lines are skipped rather than failing a scan.
pub(crate) fn parse<'a, T: Deserialize<'a>>(line: &'a [u8]) -> Option<T> {
    serde_json::from_slice(line).ok()
}

fn trim_terminator(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(contents).unwrap();
        file
    }

    fn collect(path: &Path, from: u64) -> (Vec<(u64, String)>, u64) {
        let mut lines = Vec::new();
        let end = for_each_line(path, from, |offset, line| {
            lines.push((offset, String::from_utf8_lossy(line).into_owned()));
            ControlFlow::Continue(())
        })
        .unwrap();
        (lines, end)
    }

    #[test]
    fn reports_offsets_and_strips_crlf() {
        let file = temp_file(b"ab\r\ncde\n");
        let (lines, end) = collect(file.path(), 0);
        assert_eq!(lines, vec![(0, "ab".into()), (4, "cde".into())]);
        assert_eq!(end, 8);
    }

    #[test]
    fn leaves_partial_trailing_line_for_next_pass() {
        let file = temp_file(b"one\ntwo");
        let (lines, end) = collect(file.path(), 0);
        assert_eq!(lines, vec![(0, "one".into())]);
        assert_eq!(end, 4);
    }

    #[test]
    fn resumes_from_offset() {
        let file = temp_file(b"one\ntwo\n");
        let (lines, end) = collect(file.path(), 4);
        assert_eq!(lines, vec![(4, "two".into())]);
        assert_eq!(end, 8);
    }

    #[test]
    fn tail_starts_at_a_line_boundary() {
        let file = temp_file(b"first\nsecond\nthird\n");
        assert_eq!(read_tail(file.path(), 10).unwrap(), b"third\n");
        assert_eq!(read_tail(file.path(), 1024).unwrap(), b"first\nsecond\nthird\n");
    }
}
