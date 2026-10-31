use serde::Serialize;

use crate::SyncProgress;

/// What the background indexer is doing, as shown in the UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum IndexStatus {
    /// Nothing running; the index is as current as the last sync left it.
    #[default]
    Idle,
    #[serde(rename_all = "camelCase")]
    Running {
        files_done: usize,
        files_total: usize,
        messages_added: usize,
    },
    Failed {
        message: String,
    },
}

impl From<SyncProgress> for IndexStatus {
    fn from(progress: SyncProgress) -> Self {
        Self::Running {
            files_done: progress.files_done,
            files_total: progress.files_total,
            messages_added: progress.messages_added,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_a_state_tag_and_camel_case_fields() {
        let running =
            IndexStatus::from(SyncProgress { files_done: 3, files_total: 10, messages_added: 42 });
        assert_eq!(
            serde_json::to_string(&running).unwrap(),
            r#"{"state":"running","filesDone":3,"filesTotal":10,"messagesAdded":42}"#
        );
        assert_eq!(serde_json::to_string(&IndexStatus::Idle).unwrap(), r#"{"state":"idle"}"#);
    }
}
