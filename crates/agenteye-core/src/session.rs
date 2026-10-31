use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::AgentKind;

/// A session log found on disk. Cheap to produce: it only needs file metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFile {
    pub agent: AgentKind,
    pub path: PathBuf,
    pub size_bytes: u64,
    /// Last modification time, milliseconds since the Unix epoch.
    pub modified_ms: i64,
}

/// Lightweight description of a session for list views.
///
/// Built from file metadata plus the head and tail of the file, never a full parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub agent: AgentKind,
    pub path: PathBuf,
    pub session_id: Option<String>,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub size_bytes: u64,
    pub last_activity_ms: i64,
}

impl SessionSummary {
    /// A summary carrying only what file metadata provides.
    pub fn from_file(file: &SessionFile) -> Self {
        Self {
            agent: file.agent,
            path: file.path.clone(),
            session_id: None,
            title: None,
            cwd: None,
            model: None,
            size_bytes: file.size_bytes,
            last_activity_ms: file.modified_ms,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }

    /// Maps an agent's role label to a [`Role`]; labels we do not index (system, tool, ...) yield `None`.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            _ => None,
        }
    }
}

/// Human-readable text of one message, plus where it lives in the source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub role: Role,
    pub text: String,
    pub timestamp: Option<String>,
    /// Byte offset of the line this message came from.
    pub offset: u64,
}
