//! Papyrus ID parse, format, validate, and allocation helpers.
//!
//! A Papyrus ID is `<PREFIX>-<padded number>`; there is one ID space per
//! record family and no storage/display translation.

use std::fmt;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdSpec {
    pub prefix: String,
    pub padding: usize,
}

impl IdSpec {
    pub fn new(prefix: impl Into<String>, padding: usize) -> Self {
        Self {
            prefix: prefix.into(),
            padding: padding.max(1),
        }
    }

    /// Parse a Papyrus ID and return its number. The prefix is matched
    /// case-insensitively and leading zeros are optional.
    pub fn parse_id(&self, id: &str) -> Result<i64, IdentityError> {
        let raw = id.trim();
        let (prefix, number) = split_id(raw)?;
        if !prefix.eq_ignore_ascii_case(&self.prefix) {
            return Err(IdentityError::InvalidPrefix {
                expected: self.prefix.clone(),
                found: prefix.to_string(),
            });
        }
        if is_backlog_shaped(&prefix, &number, &self.prefix) {
            return Err(IdentityError::BacklogShape(raw.to_string()));
        }
        parse_number(&number)
    }

    pub fn format_id(&self, number: i64) -> String {
        format_id(&self.prefix, number, self.padding)
    }

    pub fn validate_input(&self, input: &str) -> Result<(), IdentityError> {
        self.parse_id(input).map(|_| ())
    }
}

/// A file stem is Backlog-shaped, and therefore not a Papyrus record, when
/// its prefix is the all-lowercase form of a configured prefix that contains
/// at least one uppercase letter and its number is unpadded.
pub fn is_backlog_shaped(stem_prefix: &str, number: &str, configured_prefix: &str) -> bool {
    !stem_prefix.is_empty()
        && stem_prefix.chars().all(|c| c.is_ascii_lowercase())
        && configured_prefix.chars().any(|c| c.is_ascii_uppercase())
        && !(number.len() > 1 && number.starts_with('0'))
}

fn split_id(raw: &str) -> Result<(String, String), IdentityError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(IdentityError::InvalidFormat(raw.to_string()));
    }
    let base = raw.split_whitespace().next().unwrap_or(raw);
    let (prefix, rest) = base
        .split_once('-')
        .ok_or_else(|| IdentityError::InvalidFormat(raw.to_string()))?;
    if prefix.is_empty() {
        return Err(IdentityError::InvalidFormat(raw.to_string()));
    }
    let number: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if number.is_empty() {
        return Err(IdentityError::NonNumeric(rest.to_string()));
    }
    Ok((prefix.to_string(), number))
}

fn parse_number(number: &str) -> Result<i64, IdentityError> {
    if number.is_empty() {
        return Err(IdentityError::NonNumeric(number.to_string()));
    }
    if !number.chars().all(|c| c.is_ascii_digit()) {
        return Err(IdentityError::NonNumeric(number.to_string()));
    }
    number
        .parse::<i64>()
        .map_err(|_| IdentityError::OutOfRange(number.to_string()))
}

pub fn pad_number(n: i64, padding: usize) -> String {
    let s = n.to_string();
    if s.len() >= padding {
        s
    } else {
        format!("{:0>width$}", s, width = padding)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError {
    InvalidFormat(String),
    InvalidPrefix { expected: String, found: String },
    NonNumeric(String),
    OutOfRange(String),
    BacklogShape(String),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentityError::InvalidFormat(v) => write!(f, "invalid id format: {}", v),
            IdentityError::InvalidPrefix { expected, found } => {
                write!(f, "invalid prefix: expected {}, found {}", expected, found)
            }
            IdentityError::NonNumeric(v) => write!(f, "non-numeric id component: {}", v),
            IdentityError::OutOfRange(v) => write!(f, "id out of range: {}", v),
            IdentityError::BacklogShape(v) => {
                write!(f, "Backlog.md storage id is not a Papyrus id: {}", v)
            }
        }
    }
}

impl std::error::Error for IdentityError {}

/// The numeric parts of a Papyrus ID and the ID as written in its source.
///
/// The `display` value keeps the digit widths of the source so an existing ID
/// is never reformatted when configuration padding changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedId {
    pub prefix: String,
    pub numbers: Vec<i64>,
    pub display: String,
}

impl ParsedId {
    /// Stable key for duplicate detection: configured prefix plus every
    /// numeric component, so padding and case never create a false duplicate
    /// and dotted IDs keep their components distinct.
    pub fn key(&self, configured_prefix: &str) -> String {
        let mut key = format!(
            "{}-{}",
            configured_prefix.to_ascii_uppercase(),
            self.numbers.first().copied().unwrap_or_default()
        );
        for number in self.numbers.iter().skip(1) {
            key.push('.');
            key.push_str(&number.to_string());
        }
        key
    }
}

/// Parse an ID or record stem for a family. `dotted` selects the document
/// family shape `<PREFIX>-<group>.<sequence>`; other families use
/// `<PREFIX>-<number>`.
///
/// The prefix is matched case-insensitively and leading zeros are optional.
/// A trailing `-<slug>` or ` - <slug>` is tolerated. Backlog-shaped stems
/// resolve to `None`.
pub fn parse_id_parts(value: &str, prefix: &str, dotted: bool) -> Option<ParsedId> {
    let raw = value.trim();
    let base = raw.split(" - ").next().unwrap_or(raw).trim();
    let (stem_prefix, rest) = base.split_once('-')?;
    if stem_prefix.is_empty() || !stem_prefix.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let first_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    let first = &rest[..first_end];
    if first.is_empty() || is_backlog_shaped(stem_prefix, first, prefix) {
        return None;
    }
    let mut numbers = vec![first.parse::<i64>().ok()?];
    let mut display = format!("{}-{}", prefix, first);
    let remainder = &rest[first_end..];
    if dotted {
        let after_dot = remainder.strip_prefix('.')?;
        let second_end = after_dot
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_dot.len());
        let second = &after_dot[..second_end];
        if second.is_empty() {
            return None;
        }
        numbers.push(second.parse::<i64>().ok()?);
        display.push('.');
        display.push_str(second);
        let tail = &after_dot[second_end..];
        if !tail.is_empty() && !tail.starts_with('-') {
            return None;
        }
    } else if !remainder.is_empty() && !remainder.starts_with('-') {
        return None;
    }
    Some(ParsedId {
        prefix: prefix.to_string(),
        numbers,
        display,
    })
}

/// Canonical ID portion of a record stem, including the prefix case and the
/// digit widths of the source. The slug is never part of the result.
pub fn canonical_stem_id(stem: &str, prefix: &str, dotted: bool) -> Option<String> {
    parse_id_parts(stem, prefix, dotted).map(|parts| parts.display)
}

/// Canonical numeric key of a record stem, used for duplicate detection.
pub fn stem_key(stem: &str, prefix: &str, dotted: bool) -> Option<String> {
    parse_id_parts(stem, prefix, dotted).map(|parts| parts.key(prefix))
}

pub fn format_id(prefix: &str, number: i64, padding: usize) -> String {
    format!("{}-{}", prefix, pad_number(number, padding))
}

pub fn format_seq_id(group: &str, number: i64, padding: usize) -> String {
    format!("{}.{}", group, pad_number(number, padding))
}

/// Return the number of a record stem for the configured prefix.
///
/// Accepted stems are `<PREFIX>-<digits>`, optionally followed by `-<slug>`
/// or ` - <slug>`. Prefix comparison is case-insensitive, leading zeros are
/// optional, and Backlog.md storage stems resolve to `None`.
pub fn id_number(stem: &str, prefix: &str) -> Option<u32> {
    let base = stem.split(" - ").next().unwrap_or(stem).trim();
    let (stem_prefix, rest) = base.split_once('-')?;
    if stem_prefix.is_empty() || !stem_prefix.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || is_backlog_shaped(stem_prefix, &digits, prefix) {
        return None;
    }
    digits.parse::<u32>().ok()
}

pub fn is_record_stem(stem: &str, prefix: &str) -> bool {
    id_number(stem, prefix).is_some()
}

pub fn observed_numbers_in_dirs(dirs: &[&Path], prefix: &str) -> Vec<i64> {
    let mut observed = Vec::new();
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Some(n) = id_number(stem, prefix)
            {
                observed.push(n as i64);
            }
        }
    }
    observed
}

pub fn next_number_in_dirs(dirs: &[&Path], prefix: &str) -> u32 {
    allocate_next(&observed_numbers_in_dirs(dirs, prefix)) as u32
}

pub fn next_number(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    next_number_in_dirs(&[active, deprecated], prefix)
}

pub fn scan_max_ordinal_in_dirs(dirs: &[&Path]) -> i64 {
    let mut max: Option<i64> = None;
    for dir in dirs {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(Some(fm)) = crate::record::frontmatter::parse(&content)
                    && let Some(v) =
                        crate::record::frontmatter::current_value_string(&fm.raw, "ordinal")
                    && let Ok(n) = v.parse::<i64>()
                {
                    max = Some(max.map_or(n, |m: i64| m.max(n)));
                }
            }
        }
    }
    max.map_or(0, |m| m + 1)
}

pub fn is_group_id(id: &str, prefix: &str) -> bool {
    if let Some(rest) = id.strip_prefix(&format!("{prefix}-")) {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

pub fn observed_numbers_in_dirs_ext(dirs: &[&Path], prefix: &str, extension: &str) -> Vec<i64> {
    let mut observed = Vec::new();
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some(extension) {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Some(n) = id_number(stem, prefix)
            {
                observed.push(n as i64);
            }
        }
    }
    observed
}

pub fn next_seq_number(active: &Path, deprecated: &Path, group_id: &str) -> u32 {
    let prefix = format!("{group_id}.");
    let mut observed: Vec<i64> = Vec::new();
    for dir in [active, deprecated] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Some(rest) = stem.strip_prefix(&prefix)
                    && !rest.is_empty()
                    && rest.chars().all(|c| c.is_ascii_digit())
                    && let Ok(n) = rest.parse::<u32>()
                {
                    observed.push(n as i64);
                }
            }
        }
    }
    allocate_next(&observed) as u32
}

pub fn next_html_number(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    let observed = observed_numbers_in_dirs_ext(&[active, deprecated], prefix, "html");
    allocate_next(&observed) as u32
}

/// Allocate `max(observed) + 1`, with a minimum of 1.
pub fn allocate_next(observed: &[i64]) -> i64 {
    match observed.iter().copied().max() {
        Some(m) if m >= 1 => m + 1,
        _ => 1,
    }
}

pub fn allocate_from_observed_ids(
    spec: &IdSpec,
    observed_ids: &[&str],
) -> Result<String, IdentityError> {
    let mut observed = Vec::new();
    for id in observed_ids {
        observed.push(spec.parse_id(id)?);
    }
    Ok(spec.format_id(allocate_next(&observed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(prefix: &str) -> IdSpec {
        IdSpec::new(prefix, 3)
    }

    #[test]
    fn parse_and_format_round_trip() {
        let s = spec("TASK");
        assert_eq!(s.parse_id("TASK-001").unwrap(), 1);
        assert_eq!(s.parse_id("Task-1").unwrap(), 1);
        assert_eq!(s.parse_id("task-0001").unwrap(), 1);
        assert_eq!(s.format_id(7), "TASK-007");
    }

    #[test]
    fn parse_rejects_foreign_prefix_and_backlog_shape() {
        let s = spec("TASK");
        assert!(s.parse_id("MS-001").is_err());
        assert!(matches!(
            s.parse_id("task-18"),
            Err(IdentityError::BacklogShape(_))
        ));
        assert!(matches!(
            s.parse_id("task-0"),
            Err(IdentityError::BacklogShape(_))
        ));
        // Padded lowercase is a native case variant.
        assert_eq!(s.parse_id("task-018").unwrap(), 18);
    }

    #[test]
    fn backlog_shaped_stems_resolve_to_no_record() {
        assert_eq!(id_number("task-18", "TASK"), None);
        assert_eq!(id_number("m-0", "MS"), None);
        assert_eq!(id_number("task-001", "TASK"), Some(1));
        assert_eq!(id_number("Task-18", "TASK"), Some(18));
        assert_eq!(id_number("TASK-001", "TASK"), Some(1));
        assert_eq!(id_number("TASK-001 - Some title", "TASK"), Some(1));
        assert_eq!(id_number("TASK-001-some-title", "TASK"), Some(1));
    }

    #[test]
    fn all_lowercase_configured_prefix_treats_unpadded_as_native() {
        assert_eq!(id_number("task-18", "task"), Some(18));
        assert_eq!(id_number("TASK-18", "task"), Some(18));
    }

    #[test]
    fn custom_prefix_is_used_for_lookup_and_allocation() {
        let s = spec("PAP");
        assert_eq!(s.parse_id("PAP-009").unwrap(), 9);
        assert_eq!(s.parse_id("TASK-009").unwrap_err(), {
            IdentityError::InvalidPrefix {
                expected: "PAP".to_string(),
                found: "TASK".to_string(),
            }
        });
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("identity-custom-prefix");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("PAP-004.md"), "x").unwrap();
        std::fs::write(dir.join("TASK-099.md"), "x").unwrap();
        std::fs::write(dir.join("task-18.md"), "x").unwrap();
        assert_eq!(next_number_in_dirs(&[&dir], "PAP"), 5);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn allocation_scans_all_dirs_without_offset() {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("identity-allocation");
        let _ = std::fs::remove_dir_all(&base);
        for dir in ["active", "legacy", "archive"] {
            std::fs::create_dir_all(base.join(dir)).unwrap();
        }
        std::fs::write(base.join("active/MS-001.md"), "x").unwrap();
        std::fs::write(base.join("legacy/MS-007.md"), "x").unwrap();
        std::fs::write(base.join("archive/MS-003.md"), "x").unwrap();
        let next = next_number_in_dirs(
            &[
                &base.join("active"),
                &base.join("legacy"),
                &base.join("archive"),
            ],
            "MS",
        );
        assert_eq!(next, 8);
        assert_eq!(
            spec("MS").format_id(next as i64),
            "MS-008",
            "milestones start at 1 with no Backlog offset"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn empty_family_starts_at_one() {
        assert_eq!(allocate_next(&[]), 1);
        assert_eq!(IdSpec::new("MS", 3).format_id(1), "MS-001");
    }

    #[test]
    fn allocate_from_observed_ids_uses_spec() {
        let s = spec("PAP");
        let id = allocate_from_observed_ids(&s, &["PAP-002", "PAP-7"]).unwrap();
        assert_eq!(id, "PAP-008");
    }
}
