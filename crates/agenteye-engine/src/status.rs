use serde::Serialize;

use crate::{SyncProgress, SyncSummary};

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

/// A finished sync is `Idle` only if every session was indexed; otherwise the user is told how
/// many were not, instead of seeing "ready" for an index that has gaps.
impl From<SyncSummary> for IndexStatus {
    fn from(summary: SyncSummary) -> Self {
        if summary.files_failed == 0 {
            Self::Idle
        } else {
            Self::Failed {
                message: format!(
                    "{} of {} sessions could not be indexed",
                    summary.files_failed, summary.files_total
                ),
            }
        }
    }
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

    #[test]
    fn a_sync_with_failed_files_is_reported_as_failed() {
        let clean = SyncSummary { files_total: 10, ..SyncSummary::default() };
        assert_eq!(IndexStatus::from(clean), IndexStatus::Idle);

        let partial = SyncSummary { files_total: 10, files_failed: 3, ..SyncSummary::default() };
        assert_eq!(
            IndexStatus::from(partial),
            IndexStatus::Failed { message: "3 of 10 sessions could not be indexed".into() }
        );
    }
}
