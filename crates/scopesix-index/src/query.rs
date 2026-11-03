/// Turns free-form user input into a safe FTS5 query.
///
/// Each whitespace-separated word becomes a quoted phrase (so FTS5 operators and punctuation in
/// user input cannot cause syntax errors), and the phrases are AND-ed. Returns `None` for input
/// with no words.
pub(crate) fn to_fts_query(input: &str) -> Option<String> {
    let phrases: Vec<String> =
        input.split_whitespace().map(|word| format!("\"{}\"", word.replace('"', "\"\""))).collect();
    if phrases.is_empty() {
        None
    } else {
        Some(phrases.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_each_word() {
        assert_eq!(to_fts_query("fix  build").as_deref(), Some("\"fix\" \"build\""));
    }

    #[test]
    fn neutralizes_operators_and_quotes() {
        assert_eq!(to_fts_query("a\"b OR").as_deref(), Some("\"a\"\"b\" \"OR\""));
    }

    #[test]
    fn blank_input_is_no_query() {
        assert_eq!(to_fts_query("   "), None);
    }
}
