//! Line-ending normalization for the browser read/save path (ADR-0013 D2).
//!
//! The shared `record::document`, `record::patch`, and `record::store` contract
//! is byte preservation for CLI edits, import, and migration. The browser has a
//! stricter output contract: a saved record uses LF line endings and exactly one
//! trailing newline. This module is the one implementation of that browser-only
//! normalization; callers must not reimplement it.

/// Convert CRLF and lone CR sequences to LF, leaving trailing newlines alone.
/// Form-field validation uses this before checking each submitted line.
pub fn unify(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Convert CRLF and lone CR sequences to LF and keep exactly one trailing
/// newline. Empty input stays empty.
pub fn normalize(content: &str) -> String {
    let unified = unify(content);
    let trimmed = unified.trim_end_matches('\n');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_crlf_and_lone_cr_to_lf() {
        assert_eq!(normalize("a\r\nb\rc\n"), "a\nb\nc\n");
    }

    #[test]
    fn unify_keeps_trailing_newlines() {
        assert_eq!(unify("a\r\nb\r\n"), "a\nb\n");
        assert_eq!(unify("a\rb"), "a\nb");
    }

    #[test]
    fn keeps_exactly_one_trailing_newline() {
        assert_eq!(normalize("a\n"), "a\n");
        assert_eq!(normalize("a\n\n\n"), "a\n");
        assert_eq!(normalize("a"), "a\n");
    }

    #[test]
    fn normalizing_twice_is_idempotent() {
        let once = normalize("# T\r\n\r\n## Description\r\nBody\r\n");
        assert_eq!(once, "# T\n\n## Description\nBody\n");
        assert_eq!(normalize(&once), once);
    }

    #[test]
    fn empty_content_stays_empty() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("\r\n"), "");
    }
}
