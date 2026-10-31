use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use agenteye_core::{SessionFile, SessionSource};
use agenteye_index::{Index, HIGHLIGHT_END, HIGHLIGHT_START};
use agenteye_sources::ClaudeSource;

fn user_line(text: &str) -> String {
    format!(r#"{{"type":"user","message":{{"role":"user","content":"{text}"}}}}"#) + "\n"
}

fn append(path: &Path, contents: &str) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

fn current(source: &ClaudeSource) -> SessionFile {
    source.discover().unwrap().remove(0)
}

#[test]
fn indexes_incrementally_and_finds_text() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("proj");
    fs::create_dir_all(&project).unwrap();
    let session = project.join("s1.jsonl");
    fs::write(&session, user_line("rotate the api keys")).unwrap();

    let source = ClaudeSource::new(dir.path());
    let mut index = Index::open_in_memory().unwrap();

    let first = index.sync_file(&source, &current(&source)).unwrap();
    assert_eq!(first.messages_added, 1);
    assert!(!first.rebuilt);

    // Nothing new on disk: nothing is read.
    let idle = index.sync_file(&source, &current(&source)).unwrap();
    assert_eq!(idle.messages_added, 0);
    assert_eq!(idle.bytes_scanned, 0);

    // Appended lines are picked up without re-reading earlier ones.
    append(&session, &user_line("deploy the staging build"));
    let second = index.sync_file(&source, &current(&source)).unwrap();
    assert_eq!(second.messages_added, 1);

    let hits = index.search("staging", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].snippet.contains(&format!("{HIGHLIGHT_START}staging{HIGHLIGHT_END}")));
    assert!(hits[0].byte_offset > 0);

    assert_eq!(index.search("rotate keys", 10).unwrap().len(), 1);
    assert_eq!(index.search("nonexistent", 10).unwrap(), Vec::new());
}

#[test]
fn rebuilds_when_the_file_shrinks() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("proj");
    fs::create_dir_all(&project).unwrap();
    let session = project.join("s1.jsonl");
    fs::write(&session, format!("{}{}", user_line("alpha one"), user_line("alpha two"))).unwrap();

    let source = ClaudeSource::new(dir.path());
    let mut index = Index::open_in_memory().unwrap();
    index.sync_file(&source, &current(&source)).unwrap();
    assert_eq!(index.search("alpha", 10).unwrap().len(), 2);

    fs::write(&session, user_line("beta")).unwrap();
    let outcome = index.sync_file(&source, &current(&source)).unwrap();
    assert!(outcome.rebuilt);
    assert_eq!(index.search("alpha", 10).unwrap(), Vec::new());
    assert_eq!(index.search("beta", 10).unwrap().len(), 1);
}

#[test]
fn read_only_connection_searches_a_database_a_writer_has_open() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("proj");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("s1.jsonl"), user_line("needle in a haystack")).unwrap();

    let source = ClaudeSource::new(dir.path());
    let db_path = dir.path().join("index.db");
    let mut writer = Index::open(&db_path).unwrap();
    writer.sync_file(&source, &current(&source)).unwrap();

    let reader = Index::open_read_only(&db_path).unwrap();
    assert_eq!(reader.search("needle", 10).unwrap().len(), 1);
}

#[test]
fn read_only_open_fails_for_a_missing_database() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Index::open_read_only(dir.path().join("typo.db")).is_err());
}
