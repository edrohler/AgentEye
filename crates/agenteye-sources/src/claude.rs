//! Claude Code: `<config>/projects/<project>/<session>.jsonl`, append-only.

use std::ffi::OsStr;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use agenteye_core::{
    AgentKind, Error, MemoryFile, MemorySource, MessageRecord, Result, Role, SessionFile,
    SessionSource, SessionSummary,
};

use serde::Deserialize;

use crate::content::Content;
use crate::text::{fill_if_empty, is_user_prompt, title_from};
use crate::{fs, jsonl};

/// Metadata (cwd, model, first prompt) appears within the first few lines.
const HEAD_LINES: usize = 40;
/// The latest `ai-title` record is written near the end of the file.
const TAIL_BYTES: u64 = 256 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Line {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    cwd: Option<String>,
    session_id: Option<String>,
    ai_title: Option<String>,
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    model: Option<String>,
    content: Option<Content>,
}

#[derive(Debug)]
pub struct ClaudeSource {
    projects_dir: PathBuf,
}

impl ClaudeSource {
    pub fn new(projects_dir: impl Into<PathBuf>) -> Self {
        Self { projects_dir: projects_dir.into() }
    }

    /// Uses `CLAUDE_CONFIG_DIR` when set, otherwise `~/.claude`.
    pub fn from_env() -> Option<Self> {
        Some(Self::new(projects_dir_from_env()?))
    }
}

impl SessionSource for ClaudeSource {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn discover(&self) -> Result<Vec<SessionFile>> {
        // Depth 2 is `<project>/<session>.jsonl`; deeper files are subagent transcripts.
        Ok(fs::discover_jsonl(AgentKind::Claude, &self.projects_dir, 2..=2, ""))
    }

    fn summarize(&self, file: &SessionFile) -> Result<SessionSummary> {
        let mut summary = SessionSummary::from_file(file);
        let mut first_prompt: Option<String> = None;
        let mut lines_seen = 0_usize;

        jsonl::for_each_line(&file.path, 0, |_, raw| {
            lines_seen += 1;
            if let Some(line) = jsonl::parse::<Line>(raw) {
                fill_if_empty(&mut summary.session_id, line.session_id);
                fill_if_empty(&mut summary.cwd, line.cwd);
                if let Some(message) = line.message {
                    fill_if_empty(&mut summary.model, message.model);
                    if first_prompt.is_none() && line.kind.as_deref() == Some("user") {
                        first_prompt = message
                            .content
                            .and_then(|c| c.into_parts().into_iter().find(|t| is_user_prompt(t)));
                    }
                }
            }
            let complete =
                summary.cwd.is_some() && summary.model.is_some() && first_prompt.is_some();
            if complete || lines_seen >= HEAD_LINES {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        })?;

        let tail = jsonl::read_tail(&file.path, TAIL_BYTES)?;
        let ai_title = tail
            .split(|&b| b == b'\n')
            .rev()
            .find_map(|raw| jsonl::parse::<Line>(raw).and_then(|line| line.ai_title));
        summary.title = ai_title.or_else(|| first_prompt.as_deref().map(title_from));
        Ok(summary)
    }

    fn scan_messages(
        &self,
        path: &Path,
        from_offset: u64,
        sink: &mut dyn FnMut(MessageRecord),
    ) -> Result<u64> {
        jsonl::for_each_line(path, from_offset, |offset, raw| {
            if let Some(record) = message_record(offset, raw) {
                sink(record);
            }
            ControlFlow::Continue(())
        })
    }
}

impl MemorySource for ClaudeSource {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn discover(&self) -> Result<Vec<MemoryFile>> {
        Ok(fs::discover_md(AgentKind::Claude, &self.projects_dir, 3..=3, ""))
    }

    fn read(&self, file: &MemoryFile) -> Result<String> {
        if !file.path.starts_with(&self.projects_dir)
            || file.path.extension() != Some(OsStr::new("md"))
        {
            return Err(Error::PathOutsideRoot { path: file.path.clone() });
        }

        std::fs::read_to_string(&file.path).map_err(|e| Error::io(&file.path, e))
    }
}

fn message_record(offset: u64, raw: &[u8]) -> Option<MessageRecord> {
    let line: Line = jsonl::parse(raw)?;
    let role = Role::from_label(line.kind.as_deref()?)?;
    let text = line.message?.content?.into_text();
    if text.trim().is_empty() {
        return None;
    }
    Some(MessageRecord { role, text, timestamp: line.timestamp, offset })
}

fn projects_dir_from_env() -> Option<PathBuf> {
    let config_root = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".claude")));

    Some(config_root?.join("projects"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const SESSION: &str = concat!(
        r#"{"type":"user","sessionId":"s1","cwd":"/work","timestamp":"2026-01-01T00:00:00Z","message":{"role":"user","content":[{"type":"text","text":"<ide_opened_file>x</ide_opened_file>"},{"type":"text","text":"fix the build"}]}}"#,
        "\n",
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:05Z","message":{"model":"claude-x","role":"assistant","content":[{"type":"thinking","thinking":""},{"type":"text","text":"Done."}]}}"#,
        "\n",
        r#"{"type":"attachment","attachment":{"kind":"ignored"}}"#,
        "\n",
        r#"{"type":"ai-title","aiTitle":"Fix the build","sessionId":"s1"}"#,
        "\n",
    );

    fn source_with_session(contents: &str) -> (tempfile::TempDir, ClaudeSource) {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("abc.jsonl"), contents).unwrap();
        let source = ClaudeSource::new(dir.path());
        (dir, source)
    }

    #[test]
    fn discovers_top_level_sessions_only() {
        let (dir, source) = source_with_session(SESSION);
        let nested = dir.path().join("proj").join("abc").join("subagents");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("agent-1.jsonl"), SESSION).unwrap();

        let files = SessionSource::discover(&source).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("abc.jsonl"));
    }

    #[test]
    fn missing_root_is_empty() {
        let source = ClaudeSource::new("does/not/exist");
        assert_eq!(SessionSource::discover(&source).unwrap(), Vec::new());
    }

    #[test]
    fn summarizes_from_head_and_tail() {
        let (_dir, source) = source_with_session(SESSION);
        let file = SessionSource::discover(&source).unwrap().remove(0);
        let summary = source.summarize(&file).unwrap();

        assert_eq!(summary.session_id.as_deref(), Some("s1"));
        assert_eq!(summary.cwd.as_deref(), Some("/work"));
        assert_eq!(summary.model.as_deref(), Some("claude-x"));
        assert_eq!(summary.title.as_deref(), Some("Fix the build"));
    }

    #[test]
    fn scans_only_user_and_assistant_text() {
        let (_dir, source) = source_with_session(SESSION);
        let file = SessionSource::discover(&source).unwrap().remove(0);

        let mut records = Vec::new();
        let end = source.scan_messages(&file.path, 0, &mut |r| records.push(r)).unwrap();

        assert_eq!(end, file.size_bytes);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].role, Role::User);
        assert!(records[0].text.ends_with("fix the build"));
        assert_eq!(records[0].offset, 0);
        assert_eq!(records[1].role, Role::Assistant);
        assert_eq!(records[1].text, "Done.");
    }

    const MEMORY: &str = "---\nname: note\ntype: project\n---\nbody\n";

    /// `proj/memory/{MEMORY.md,a.md,notes.txt,sub/b.md}` plus `proj/other/c.md`.
    fn source_with_memory() -> (tempfile::TempDir, ClaudeSource) {
        let dir = tempfile::tempdir().unwrap();
        let memory = dir.path().join("proj").join("memory");
        fs::create_dir_all(memory.join("sub")).unwrap();
        fs::create_dir_all(dir.path().join("proj").join("other")).unwrap();
        fs::write(memory.join("MEMORY.md"), "- [a](a.md)\n").unwrap();
        fs::write(memory.join("a.md"), MEMORY).unwrap();
        fs::write(memory.join("notes.txt"), "ignored").unwrap();
        fs::write(memory.join("sub").join("b.md"), MEMORY).unwrap();
        fs::write(dir.path().join("proj").join("other").join("c.md"), MEMORY).unwrap();
        let source = ClaudeSource::new(dir.path());
        (dir, source)
    }

    #[test]
    fn discovers_memory_files_directly_under_memory_dir() {
        let (_dir, source) = source_with_memory();

        let mut files = MemorySource::discover(&source).unwrap();
        files.sort_by(|a, b| a.name.cmp(&b.name));

        let names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["MEMORY.md", "a.md"]);
        assert!(files.iter().all(|f| f.project == "proj"));
        assert!(files.iter().all(|f| f.agent == AgentKind::Claude));
        assert_eq!(files[1].size_bytes, MEMORY.len() as u64);
    }

    #[test]
    fn missing_memory_root_is_empty() {
        let source = ClaudeSource::new("does/not/exist");
        assert_eq!(MemorySource::discover(&source).unwrap(), Vec::new());
    }

    #[test]
    fn project_without_memory_dir_is_empty() {
        let (dir, source) = source_with_session(SESSION);
        assert!(dir.path().join("proj").is_dir());
        assert_eq!(MemorySource::discover(&source).unwrap(), Vec::new());
    }

    #[test]
    fn reads_discovered_memory_file() {
        let (_dir, source) = source_with_memory();
        let file = MemorySource::discover(&source)
            .unwrap()
            .into_iter()
            .find(|f| f.name == "a.md")
            .unwrap();

        assert_eq!(source.read(&file).unwrap(), MEMORY);
    }

    #[test]
    fn read_rejects_path_outside_root() {
        let (_dir, source) = source_with_memory();
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("x.md"), MEMORY).unwrap();
        let mut file = MemorySource::discover(&source).unwrap().remove(0);
        file.path = other.path().join("x.md");

        let err = source.read(&file).unwrap_err();
        assert!(matches!(err, Error::PathOutsideRoot { .. }));
    }

    #[test]
    fn read_rejects_non_markdown_file() {
        let (dir, source) = source_with_memory();
        let mut file = MemorySource::discover(&source).unwrap().remove(0);
        file.path = dir.path().join("proj").join("memory").join("notes.txt");

        let err = source.read(&file).unwrap_err();
        assert!(matches!(err, Error::PathOutsideRoot { .. }));
    }

    #[test]
    fn read_of_vanished_file_is_io_error() {
        let (_dir, source) = source_with_memory();
        let file = MemorySource::discover(&source).unwrap().remove(0);
        fs::remove_file(&file.path).unwrap();

        let err = source.read(&file).unwrap_err();
        assert!(matches!(err, Error::Io { .. }));
    }
}
