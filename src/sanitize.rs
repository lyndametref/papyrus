use std::path::{Component, Path, PathBuf};

/// Reject any Unicode control character (C0, DEL, and C1) from an already
/// trimmed single-line value.
fn has_control_character(value: &str) -> bool {
    value.chars().any(char::is_control)
}

/// Normalize user-controlled input. Returns `None` if a valid value cannot be
/// produced.
pub fn sanitize_name(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if has_control_character(trimmed) {
        return None;
    }
    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return None;
    }
    // Reject path separators to avoid path traversal in names.
    if trimmed.contains('/') || trimmed.contains('\\') {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn sanitize_path_component(input: &str) -> Option<String> {
    // Evaluate the control-character predicate on the raw component before the
    // trim below: failing input must be rejected, not silently shortened
    // (NFR-02.002). This also subsumes the former explicit NUL check.
    if has_control_character(input) {
        return None;
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Prevent traversal segments.
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        return None;
    }
    Some(trimmed.to_string())
}

/// Normalize a user-controlled path, sanitizing each component and rejecting
/// traversal segments (`..`). Returns `None` if any component cannot be
/// normalized.
pub fn sanitize_path(input: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    if input.is_absolute() {
        out.push("/");
    }
    for comp in input.components() {
        match comp {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => return None,
            Component::Normal(s) => {
                let name = s.to_str()?;
                let sanitized = sanitize_path_component(name)?;
                out.push(sanitized);
            }
        }
    }
    Some(out)
}

/// Sanitize a reference or documentation path, allowing path separators while
/// rejecting traversal segments and control characters.
pub fn sanitize_path_string(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if has_control_character(trimmed) {
        return None;
    }
    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return None;
    }
    // Reject backslashes to keep a single canonical separator.
    if trimmed.contains('\\') {
        return None;
    }
    let path = std::path::Path::new(trimmed);
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => return None,
            std::path::Component::Normal(s) => {
                let name = s.to_str()?;
                if name.is_empty() || name.contains('\0') {
                    return None;
                }
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => return None,
            std::path::Component::CurDir => {}
        }
    }
    Some(trimmed.to_string())
}

/// Alias for callers that think in terms of references.
pub fn sanitize_reference(input: &str) -> Option<String> {
    sanitize_path_string(input)
}

/// Alias for callers that think in terms of documentation paths.
pub fn sanitize_documentation(input: &str) -> Option<String> {
    sanitize_path_string(input)
}

/// Alias matching the path-safe sanitizer wording.
pub fn sanitize_path_value(input: &str) -> Option<String> {
    sanitize_path_string(input)
}

/// Derive a kebab-case file-name slug from a record title.
///
/// Lowercase ASCII alphanumerics are kept, every other run collapses to a
/// single `-`, and leading/trailing `-` are removed. An empty result means the
/// caller should write the bare `<ID>.md` name.
pub fn kebab_slug(input: &str) -> String {
    let mut out = String::new();
    let mut pending_separator = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_separator && !out.is_empty() {
                out.push('-');
            }
            pending_separator = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_separator = true;
        }
    }
    out
}

#[cfg(test)]
mod control_character_tests {
    use super::{sanitize_name, sanitize_path, sanitize_path_component, sanitize_path_string};
    use std::path::{Path, PathBuf};

    /// Every C0 control character and DEL, embedded between ordinary
    /// characters, must be rejected by both sanitizers.
    #[test]
    fn rejects_c0_and_del_for_both_sanitizers() {
        let mut scalars: Vec<char> = (0u32..=0x1F).filter_map(char::from_u32).collect();
        scalars.push('\u{7F}');
        for ch in scalars {
            let input = format!("a{ch}b");
            assert_eq!(sanitize_name(&input), None, "sanitize_name accepted {ch:?}");
            assert_eq!(
                sanitize_path_string(&input),
                None,
                "sanitize_path_string accepted {ch:?}"
            );
        }
    }

    /// U+0080..U+009F (C1) is rejected for the same reason as C0: the shared
    /// check uses `char::is_control()`.
    #[test]
    fn rejects_c1_sample_for_both_sanitizers() {
        let input = "a\u{9B}b";
        assert_eq!(sanitize_name(input), None);
        assert_eq!(sanitize_path_string(input), None);
    }

    #[test]
    fn accepts_legitimate_input() {
        for sample in ["Release notes", "Résumé – v2", "a.b", "a-b", "x (y) 100%"] {
            assert_eq!(sanitize_name(sample), Some(sample.to_string()));
            assert_eq!(sanitize_path_string(sample), Some(sample.to_string()));
        }
        assert_eq!(
            sanitize_path_string("docs/design.md"),
            Some("docs/design.md".to_string())
        );
    }

    #[test]
    fn trims_before_rejecting_controls() {
        assert_eq!(sanitize_name("  Ti\ttle  "), None);
        assert_eq!(sanitize_path_string("  Ti\ttle  "), None);
        assert_eq!(sanitize_name("\t"), None);
        assert_eq!(sanitize_path_string("\t"), None);
        assert_eq!(sanitize_name("   "), None);
        assert_eq!(sanitize_path_string("   "), None);
        assert_eq!(sanitize_name("Title\t"), Some("Title".to_string()));
        assert_eq!(sanitize_path_string("Title\t"), Some("Title".to_string()));
    }

    /// AC #1 and AC #4: the boundary scalars of C0, DEL, and C1 are rejected
    /// by `sanitize_path_component` and by `sanitize_path`.
    #[test]
    fn rejects_control_boundaries_for_path_functions() {
        for input in [
            "a\u{0000}b",
            "a\u{001F}b",
            "a\u{007F}b",
            "a\u{0080}b",
            "a\u{009F}b",
        ] {
            assert_eq!(sanitize_path_component(input), None, "{input:?}");
            assert_eq!(sanitize_path(Path::new(input)), None, "{input:?}");
        }
    }

    /// AC #1 and AC #4: every C0 scalar, DEL, and every C1 scalar is rejected
    /// when embedded between ordinary characters.
    #[test]
    fn rejects_every_c0_del_and_c1_scalar_for_path_functions() {
        let mut scalars: Vec<char> = (0u32..=0x1F).filter_map(char::from_u32).collect();
        scalars.push('\u{7F}');
        scalars.extend((0x80u32..=0x9F).filter_map(char::from_u32));
        for ch in scalars {
            let input = format!("a{ch}b");
            assert_eq!(sanitize_path_component(&input), None, "{ch:?}");
            assert_eq!(sanitize_path(Path::new(&input)), None, "{ch:?}");
        }
    }

    /// AC #1 and AC #4: the predicate runs before the trim, so a leading or
    /// trailing control (including U+0085, which `trim` would strip) is
    /// rejected instead of being silently cleaned.
    #[test]
    fn rejects_leading_and_trailing_controls_before_trim() {
        for (leading, trailing) in [
            ("\ta", "a\t"),
            ("\na", "a\n"),
            ("\u{85}a", "a\u{85}"),
            ("\u{1F}a", "a\u{1F}"),
            ("\u{9F}a", "a\u{9F}"),
        ] {
            assert_eq!(sanitize_path_component(leading), None, "{leading:?}");
            assert_eq!(sanitize_path_component(trailing), None, "{trailing:?}");
            assert_eq!(sanitize_path(Path::new(leading)), None, "{leading:?}");
            assert_eq!(sanitize_path(Path::new(trailing)), None, "{trailing:?}");
        }
    }

    /// AC #2 and AC #4: exactly one contaminated component poisons the whole
    /// path and no partial prefix is returned.
    #[test]
    fn rejects_nested_path_with_one_contaminated_component() {
        assert_eq!(sanitize_path(Path::new("docs/design\nnotes/api")), None);
        assert_eq!(
            sanitize_path(Path::new("docs/design/notes\u{7F}/api")),
            None
        );
        assert_eq!(sanitize_path(Path::new("do\u{1}cs/design")), None);
        assert_eq!(sanitize_path(Path::new("docs/\u{9F}/design")), None);
    }

    /// AC #2 and AC #4: an absolute path with a contaminated component is
    /// rejected, whichever component carries the control character.
    #[test]
    fn rejects_absolute_path_with_contaminated_component() {
        assert_eq!(sanitize_path(Path::new("/abs/pa\u{1F}th")), None);
        assert_eq!(sanitize_path(Path::new("/\u{9F}abs/path")), None);
        assert_eq!(sanitize_path(Path::new("/abs/path\u{7F}")), None);
    }

    /// AC #3: legitimate components and paths sanitize exactly as before.
    #[test]
    fn leaves_legitimate_paths_unchanged() {
        for sample in [
            "Résumé – v2",
            "space name",
            "a.b",
            "a-b",
            "a_b",
            "x (y) 100%",
        ] {
            assert_eq!(
                sanitize_path_component(sample),
                Some(sample.to_string()),
                "{sample:?}"
            );
            assert_eq!(
                sanitize_path(Path::new(sample)),
                Some(PathBuf::from(sample)),
                "{sample:?}"
            );
        }
        assert_eq!(
            sanitize_path_component("  padded  "),
            Some("padded".to_string())
        );
        assert_eq!(
            sanitize_path(Path::new("/abs/path")),
            Some(PathBuf::from("/abs/path"))
        );
        assert_eq!(
            sanitize_path(Path::new("docs/./design.md")),
            Some(PathBuf::from("docs/design.md"))
        );
        assert_eq!(
            sanitize_path(Path::new("docs/design/notes")),
            Some(PathBuf::from("docs/design/notes"))
        );
    }

    /// AC #3: traversal guards are unchanged.
    #[test]
    fn keeps_traversal_guards_for_path_functions() {
        for input in [".", "..", " ./ ", " ../ "] {
            assert_eq!(sanitize_path_component(input), None, "{input:?}");
        }
        for input in ["a/b", "a\\b"] {
            assert_eq!(sanitize_path_component(input), None, "{input:?}");
        }
        assert_eq!(sanitize_path(Path::new("..")), None);
        assert_eq!(sanitize_path(Path::new("a/../b")), None);
        assert_eq!(sanitize_path(Path::new(".")), Some(PathBuf::new()));
        assert_eq!(sanitize_path(Path::new("./a")), Some(PathBuf::from("a")));
    }

    /// AC #5: `sanitize_name` and `sanitize_path_string` keep trimming a
    /// trailing control character, which now deliberately differs from the
    /// path functions.
    #[test]
    fn name_sanitizers_keep_trim_before_check_behavior() {
        assert_eq!(sanitize_name("Title\t"), Some("Title".to_string()));
        assert_eq!(sanitize_path_string("Title\t"), Some("Title".to_string()));
        assert_eq!(sanitize_path_component("Title\t"), None);
        assert_eq!(sanitize_path(Path::new("Title\t")), None);
    }

    #[test]
    fn keeps_existing_rejections() {
        for input in ["a\0b", "a\nb", "a\rb"] {
            assert_eq!(sanitize_name(input), None, "{input:?}");
            assert_eq!(sanitize_path_string(input), None, "{input:?}");
        }
        for input in ["a/b", "a\\b"] {
            assert_eq!(sanitize_name(input), None, "{input:?}");
        }
        for input in ["a\\b", "a/../b", "/abs/path", "../../etc/passwd"] {
            assert_eq!(sanitize_path_string(input), None, "{input:?}");
        }
    }
}

#[cfg(test)]
mod slug_tests {
    use super::kebab_slug;

    #[test]
    fn derives_kebab_slug_from_title() {
        assert_eq!(
            kebab_slug("Papyrus-native on-disk record layout"),
            "papyrus-native-on-disk-record-layout"
        );
        assert_eq!(kebab_slug("  A/B: C!  "), "a-b-c");
        assert_eq!(kebab_slug("!!!"), "");
    }
}
