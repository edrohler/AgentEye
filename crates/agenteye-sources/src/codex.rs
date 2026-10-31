//! Codex CLI: `<home>/sessions/YYYY/MM/DD/rollout-*.jsonl`, append-only.

use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use agenteye_core::{
    AgentKind, MessageRecord, Result, Role, SessionFile, SessionSource, SessionSummary,
};
use serde::Deserialize;

use crate::content::Content;
use crate::text::{fill_if_empty, is_user_prompt, title_from};
use crate::{fs, jsonl};

/// `session_meta`, `turn_context` and the first prompt all appear near the top of the file.
const HEAD_LINES: usize = 60;

#[derive(Debug, Deserialize)]
struct Line {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    payload: Option<Payload>,
}

#[derive(Debug, Deserialize)]
struct Payload {
    #[serde(rename = "type")]
    kind: Option<String>,
    id: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
    role: Option<String>,
    content: Option<Content>,
}

impl Payload {
    fn is_message(&self) -> bool {
        self.kind.as_deref() == Some("message")
    }
}

#[derive(Debug)]
pub struct CodexSource {
    sessions_dir: PathBuf,
}

impl CodexSource {
    pub fn new(sessions_dir: impl Into<PathBuf>) -> Self {
        Self { sessions_dir: sessions_dir.into() }
    }

    /// Uses `CODEX_HOME` when set, otherwise `~/.codex`.
    pub fn from_env() -> Option<Self> {
        let codex_home = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))?;
        Some(Self::new(codex_home.join("sessions")))
    }
}

impl SessionSource for CodexSource {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn discover(&self) -> Result<Vec<SessionFile>> {
        // `<year>/<month>/<day>/rollout-*.jsonl`
        Ok(fs::discover_jsonl(AgentKind::Codex, &self.sessions_dir, 1..=5, "rollout-"))
    }

    fn summarize(&self, file: &SessionFile) -> Result<SessionSummary> {
        let mut summary = SessionSummary::from_file(file);
        let mut first_prompt: Option<String> = None;
        let mut lines_seen = 0_usize;

        jsonl::for_each_line(&file.path, 0, |_, raw| {
            lines_seen += 1;
            if let Some(Line { kind, payload: Some(payload), .. }) = jsonl::parse::<Line>(raw) {
                match kind.as_deref() {
                    Some("session_meta") => {
                        fill_if_empty(&mut summary.session_id, payload.id);
                        fill_if_empty(&mut summary.cwd, payload.cwd);
                    }
                    Some("turn_context") => fill_if_empty(&mut summary.model, payload.model),
                    Some("response_item")
                        if first_prompt.is_none()
                            && payload.is_message()
                            && payload.role.as_deref() == Some("user") =>
                    {
                        first_prompt = payload
                            .content
                            .and_then(|c| c.into_parts().into_iter().find(|t| is_user_prompt(t)));
                    }
                    _ => {}
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

        summary.title = first_prompt.as_deref().map(title_from);
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

fn message_record(offset: u64, raw: &[u8]) -> Option<MessageRecord> {
    let line: Line = jsonl::parse(raw)?;
    if line.kind.as_deref() != Some("response_item") {
        return None;
    }
    let payload = line.payload?;
    if !payload.is_message() {
        return None;
    }
    let role = Role::from_label(payload.role.as_deref()?)?;
    let text = payload.content?.into_text();
    if text.trim().is_empty() {
        return None;
    }
    Some(MessageRecord { role, text, timestamp: line.timestamp, offset })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const SESSION: &str = concat!(
        r#"{"timestamp":"2026-01-01T00:00:00Z","type":"session_meta","payload":{"id":"c1","cwd":"/work"}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:01Z","type":"turn_context","payload":{"model":"gpt-x","cwd":"/work"}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:02Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"system rules"}]}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:03Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>x</environment_context>"}]}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:04Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"add a flag"}]}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:05Z","type":"response_item","payload":{"type":"function_call_output","output":"ignored"}}"#,
        "\n",
        r#"{"timestamp":"2026-01-01T00:00:06Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Added."}]}}"#,
        "\n",
    );

    fn source_with_session(contents: &str) -> (tempfile::TempDir, CodexSource) {
        let dir = tempfile::tempdir().unwrap();
        let day = dir.path().join("2026").join("01").join("01");
        fs::create_dir_all(&day).unwrap();
        fs::write(day.join("rollout-2026-01-01T00-00-00-c1.jsonl"), contents).unwrap();
        fs::write(day.join("notes.jsonl"), "{}\n").unwrap();
        let source = CodexSource::new(dir.path());
        (dir, source)
    }

    #[test]
    fn discovers_only_rollout_files() {
        let (_dir, source) = source_with_session(SESSION);
        assert_eq!(source.discover().unwrap().len(), 1);
    }

    #[test]
    fn summarizes_from_head() {
        let (_dir, source) = source_with_session(SESSION);
        let file = source.discover().unwrap().remove(0);
        let summary = source.summarize(&file).unwrap();

        assert_eq!(summary.session_id.as_deref(), Some("c1"));
        assert_eq!(summary.cwd.as_deref(), Some("/work"));
        assert_eq!(summary.model.as_deref(), Some("gpt-x"));
        assert_eq!(summary.title.as_deref(), Some("add a flag"));
    }

    #[test]
    fn scans_user_and_assistant_messages_but_not_developer_or_tools() {
        let (_dir, source) = source_with_session(SESSION);
        let file = source.discover().unwrap().remove(0);

        let mut records = Vec::new();
        source.scan_messages(&file.path, 0, &mut |r| records.push(r)).unwrap();

        let texts: Vec<_> = records.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, ["<environment_context>x</environment_context>", "add a flag", "Added."]);
    }
}
