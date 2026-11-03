//! Message content as written by both Claude and Codex: either a plain string or a list of blocks.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Content {
    Text(String),
    Blocks(Vec<Block>),
}

/// Only the `text` of a block matters for indexing; thinking, tool calls, etc. have no `text`.
#[derive(Debug, Deserialize)]
pub(crate) struct Block {
    text: Option<String>,
}

impl Content {
    /// The text of each block, in order. A plain string is a single part.
    pub(crate) fn into_parts(self) -> Vec<String> {
        match self {
            Content::Text(text) => vec![text],
            Content::Blocks(blocks) => blocks.into_iter().filter_map(|b| b.text).collect(),
        }
    }

    pub(crate) fn into_text(self) -> String {
        self.into_parts().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_string() {
        let content: Content = serde_json::from_str(r#""hello""#).unwrap();
        assert_eq!(content.into_text(), "hello");
    }

    #[test]
    fn blocks_keep_only_text() {
        let json = r#"[{"type":"text","text":"a"},{"type":"tool_use","name":"x"},{"type":"text","text":"b"}]"#;
        let content: Content = serde_json::from_str(json).unwrap();
        assert_eq!(content.into_text(), "a\nb");
    }
}
