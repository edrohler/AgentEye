//! Small text helpers for building list-view titles.

const MAX_TITLE_CHARS: usize = 120;

/// Stores `value` in `slot` unless `slot` already holds a value (first value wins).
pub(crate) fn fill_if_empty(slot: &mut Option<String>, value: Option<String>) {
    if slot.is_none() {
        *slot = value;
    }
}

/// Whether text looks like something the user typed, as opposed to context an agent or IDE injected
/// (those start with an XML-ish tag such as `<environment_context>`).
pub(crate) fn is_user_prompt(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && !text.starts_with('<')
}

/// First line of `text`, trimmed and capped to a readable length.
pub(crate) fn title_from(text: &str) -> String {
    let first_line = text.trim().lines().next().unwrap_or_default();
    first_line.chars().take(MAX_TITLE_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_context_is_not_a_prompt() {
        assert!(!is_user_prompt("<environment_context>..."));
        assert!(!is_user_prompt("   "));
        assert!(is_user_prompt("fix the build"));
    }

    #[test]
    fn title_is_first_line_and_capped() {
        assert_eq!(title_from("  hello\nworld"), "hello");
        assert_eq!(title_from(&"x".repeat(500)).chars().count(), MAX_TITLE_CHARS);
    }
}
