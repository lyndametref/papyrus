use std::path::{Component, Path, PathBuf};

/// Normalize user-controlled input. Returns `None` if a valid value cannot be
/// produced.
pub fn sanitize_name(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
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
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
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
