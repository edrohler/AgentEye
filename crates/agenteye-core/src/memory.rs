use std::path::PathBuf;

use crate::AgentKind;

/// A memory file found on disk. Cheap to produce: file metadata only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryFile {
    pub agent: AgentKind,
    pub project: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_ms: i64,
}

/// Fields from a memory file's frontmatter; any may be missing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryHeader {
    pub name: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>, // `type` is a keyword in Rust, so we use `kind` instead.
}

impl MemoryHeader {
    /// Parses the YAML-style frontmatter of a memory file. Never fails.
    ///
    /// # Edge cases
    /// - No frontmatter, or an unterminated `---`: returns the default header.
    /// - `type` is accepted at any indentation, so `metadata:\n  type: x` works.
    /// - Duplicate keys: the first one wins.
    /// - Surrounding quotes on values are stripped; empty values are skipped.
    /// - A leading BOM and CRLF line endings are tolerated.
    /// - Unsupported: multi-line YAML values (`>` and `|`).
    pub fn parse(content: &str) -> Self {
        let content = content.strip_prefix('\u{feff}').unwrap_or(content);

        let mut lines = content.lines();

        let Some(first) = lines.next() else {
            return Self::default();
        };

        if first.trim() != "---" {
            return Self::default();
        }

        let mut header: MemoryHeader = Self::default();
        let mut closed: bool = false;

        for line in lines {
            if line.trim() == "---" {
                closed = true;
                break;
            }

            let Some((key, value)) = line.trim().split_once(':') else {
                continue;
            };

            let key = key.trim();
            let value = unquote(value.trim());

            if value.is_empty() {
                continue;
            }

            match key {
                "name" if header.name.is_none() => {
                    header.name = Some(value.to_owned());
                }
                "description" if header.description.is_none() => {
                    header.description = Some(value.to_owned());
                }
                "type" if header.kind.is_none() => {
                    header.kind = Some(value.to_owned());
                }
                _ => {}
            }
        }

        if closed {
            header
        } else {
            Self::default()
        }
    }
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(quote).and_then(|rest| rest.strip_suffix(quote)) {
            return inner;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_gives_default() {
        assert_eq!(MemoryHeader::parse("just a body"), MemoryHeader::default());
    }

    #[test]
    fn reads_name() {
        let header = MemoryHeader::parse("---\nname: foo\n---\nbody");
        assert_eq!(header.name.as_deref(), Some("foo"));
    }

    #[test]
    fn reads_nested_type() {
        let content = "---\nname: a\nmetadata:\n  type: project\n---\n";
        assert_eq!(MemoryHeader::parse(content).kind.as_deref(), Some("project"));
    }

    #[test]
    fn first_duplicate_key_wins() {
        let content = "---\nname: one\nname: two\n---\n";
        assert_eq!(MemoryHeader::parse(content).name.as_deref(), Some("one"));
    }

    #[test]
    fn unterminated_header_gives_default() {
        assert_eq!(MemoryHeader::parse("---\nname: foo\nbody"), MemoryHeader::default());
    }

    #[test]
    fn strips_double_and_single_quotes() {
        let content = "---\nname: 'a'\ndescription: \"b\"\n---\n";
        let header = MemoryHeader::parse(content);
        assert_eq!(header.name.as_deref(), Some("a"));
        assert_eq!(header.description.as_deref(), Some("b"));
    }

    #[test]
    fn keeps_colon_inside_value() {
        let header = MemoryHeader::parse("---\ndescription: \"a: b\"\n---\n");
        assert_eq!(header.description.as_deref(), Some("a: b"));
    }

    #[test]
    fn lone_quote_is_kept_and_does_not_panic() {
        let header = MemoryHeader::parse("---\nname: \"\n---\n");
        assert_eq!(header.name.as_deref(), Some("\""));
    }

    #[test]
    fn handles_crlf_and_bom() {
        let header = MemoryHeader::parse("\u{feff}---\r\nname: foo\r\n---\r\n");
        assert_eq!(header.name.as_deref(), Some("foo"));
    }
}
