//! Papyrus ID parse, format, validate, and allocation helpers.

use std::fmt;
use std::path::Path;

pub const DEFAULT_TASK_PREFIX: &str = "TASK";
pub const DEFAULT_MILESTONE_PREFIX: &str = "MS";
pub const DEFAULT_RELEASE_PREFIX: &str = "REL";
pub const DEFAULT_ID_PADDING: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum RecordKind {
    Task,
    Milestone,
}

impl RecordKind {
    pub fn default_prefix(self) -> &'static str {
        match self {
            RecordKind::Task => DEFAULT_TASK_PREFIX,
            RecordKind::Milestone => DEFAULT_MILESTONE_PREFIX,
        }
    }
    pub fn default_spec(self) -> IdSpec {
        match self {
            RecordKind::Task => IdSpec::task(self.default_prefix(), DEFAULT_ID_PADDING),
            RecordKind::Milestone => IdSpec::milestone(self.default_prefix(), DEFAULT_ID_PADDING),
        }
    }
    pub fn default_base_number(self) -> i64 {
        1
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdSpec {
    pub kind: RecordKind,
    pub prefix: String,
    pub padding: usize,
}

impl IdSpec {
    pub fn task(prefix: impl Into<String>, padding: usize) -> Self {
        Self {
            kind: RecordKind::Task,
            prefix: prefix.into(),
            padding: padding.max(1),
        }
    }
    pub fn milestone(prefix: impl Into<String>, padding: usize) -> Self {
        Self {
            kind: RecordKind::Milestone,
            prefix: prefix.into(),
            padding: padding.max(1),
        }
    }
    pub fn default_task() -> Self {
        Self::task(DEFAULT_TASK_PREFIX, DEFAULT_ID_PADDING)
    }
    pub fn default_milestone() -> Self {
        Self::milestone(DEFAULT_MILESTONE_PREFIX, DEFAULT_ID_PADDING)
    }

    pub fn parse_id(&self, id: &str) -> Result<i64, IdentityError> {
        let raw = id.trim();
        let (prefix, number) = split_id(raw)?;
        if !prefix.eq_ignore_ascii_case(&self.prefix) {
            return Err(IdentityError::InvalidPrefix {
                expected: self.prefix.clone(),
                found: prefix.to_string(),
            });
        }
        parse_number(&number)
    }
    pub fn parse_display_number(&self, id: &str) -> Result<i64, IdentityError> {
        self.parse_id(id)
    }
    pub fn parse_storage_number(&self, id: &str) -> Result<i64, IdentityError> {
        self.parse_id(id)
    }
    pub fn format_id(&self, number: i64) -> String {
        format!("{}-{}", self.prefix, pad_number(number, self.padding))
    }
    pub fn validate_input(&self, input: &str) -> Result<(), IdentityError> {
        self.parse_id(input).map(|_| ())
    }
    pub fn normalize(&self, id: &str) -> Result<i64, IdentityError> {
        self.parse_id(id)
    }
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
    InvalidPrefix {
        expected: String,
        found: String,
    },
    NonNumeric(String),
    OutOfRange(String),
    StorageInput {
        storage_id: String,
        display_id: String,
    },
    Conflict(String),
    NotFound(String),
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
            IdentityError::StorageInput {
                storage_id,
                display_id,
            } => write!(
                f,
                "storage id '{}' is not accepted as input; use display id '{}'",
                storage_id, display_id
            ),
            IdentityError::Conflict(v) => write!(f, "conflict: {}", v),
            IdentityError::NotFound(v) => write!(f, "not found: {}", v),
        }
    }
}

impl std::error::Error for IdentityError {}

pub fn format_id(prefix: &str, number: i64, padding: usize) -> String {
    format!("{}-{}", prefix, pad_number(number, padding))
}

pub fn format_seq_id(group: &str, number: i64, padding: usize) -> String {
    format!("{}.{}", group, pad_number(number, padding))
}

pub fn id_number(stem: &str, prefix: &str) -> Option<u32> {
    let base = stem.split(" - ").next().unwrap_or(stem).trim();
    let prefix_with_dash = format!("{}-", prefix);
    if !base
        .to_ascii_lowercase()
        .starts_with(&prefix_with_dash.to_ascii_lowercase())
    {
        return None;
    }
    let rest = &base[prefix.len() + 1..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

pub fn strip_prefix_number<'a>(stem: &'a str, prefix: &str) -> Option<&'a str> {
    let base = stem.split(" - ").next().unwrap_or(stem).trim();
    if base
        .to_ascii_lowercase()
        .starts_with(&format!("{}-", prefix).to_ascii_lowercase())
    {
        let rest = &base[prefix.len() + 1..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() { None } else { Some(base) }
    } else {
        None
    }
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
    let observed = observed_numbers_in_dirs(dirs, prefix);
    allocate_next_storage_number(&observed, 1) as u32
}

pub fn scan_max_in_dirs(dirs: &[&Path], prefix: &str) -> u32 {
    observed_numbers_in_dirs(dirs, prefix)
        .into_iter()
        .map(|n| n as u32)
        .max()
        .unwrap_or(0)
}

pub fn scan_max(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    scan_max_in_dirs(&[active, deprecated], prefix)
}

pub fn scan_max_single(dir: &Path, prefix: &str) -> u32 {
    scan_max_in_dirs(&[dir], prefix)
}

pub fn scan_max_task(active: &Path, archived: &Path, archive: &Path) -> u32 {
    let mut max = 0u32;
    for dir in [active, archived, archive] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = id_number(stem, DEFAULT_TASK_PREFIX)
                    && n > max
                {
                    max = n;
                }
            }
        }
    }
    max
}

pub fn scan_max_milestone(active: &Path, archived: &Path, archive: &Path) -> u32 {
    let mut max = 0u32;
    for dir in [active, archived, archive] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = id_number(stem, DEFAULT_MILESTONE_PREFIX)
                    && n > max
                {
                    max = n;
                }
            }
        }
    }
    max
}

pub fn task_number_from_stem(stem: &str) -> Option<u32> {
    id_number(stem, DEFAULT_TASK_PREFIX)
}

pub fn milestone_number_from_stem(stem: &str) -> Option<u32> {
    id_number(stem, DEFAULT_MILESTONE_PREFIX)
}

pub fn scan_max_ordinal(active: &Path, archived: &Path) -> i64 {
    let mut max: Option<i64> = None;
    for dir in [active, archived] {
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

pub fn scan_max_group(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    scan_max(active, deprecated, prefix)
}

pub fn scan_max_seq(active: &Path, deprecated: &Path, group_id: &str) -> u32 {
    let mut max = 0u32;
    let prefix = format!("{group_id}.");
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
                    && n > max
                {
                    max = n;
                }
            }
        }
    }
    max
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

pub fn scan_max_in_dirs_ext(dirs: &[&Path], prefix: &str, extension: &str) -> u32 {
    observed_numbers_in_dirs_ext(dirs, prefix, extension)
        .into_iter()
        .map(|n| n as u32)
        .max()
        .unwrap_or(0)
}

pub fn next_number(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    next_number_in_dirs(&[active, deprecated], prefix)
}

pub fn next_task_number(active: &Path, archived: &Path, archive: &Path) -> u32 {
    let mut observed: Vec<i64> = Vec::new();
    for dir in [active, archived, archive] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = task_number_from_stem(stem) {
                    observed.push(n as i64);
                }
            }
        }
    }
    allocate_next_storage_number(&observed, 1) as u32
}

pub fn next_milestone_number(active: &Path, archived: &Path, archive: &Path) -> u32 {
    let mut observed: Vec<i64> = Vec::new();
    for dir in [active, archived, archive] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = milestone_number_from_stem(stem) {
                    observed.push(n as i64);
                }
            }
        }
    }
    allocate_next_storage_number(&observed, 1) as u32
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
    allocate_next_storage_number(&observed, 1) as u32
}

pub fn next_html_number(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    let observed = observed_numbers_in_dirs_ext(&[active, deprecated], prefix, "html");
    allocate_next_storage_number(&observed, 1) as u32
}

pub fn allocate_next_storage_number(observed: &[i64], base: i64) -> i64 {
    let max = observed.iter().copied().max();
    match max {
        Some(m) if m >= base => m + 1,
        Some(m) => {
            if m + 1 > base {
                m + 1
            } else {
                base
            }
        }
        None => base,
    }
}

pub fn allocate_next_number(observed: &[i64]) -> i64 {
    allocate_next_storage_number(observed, 1)
}
pub fn allocate_next(observed: &[i64]) -> i64 {
    allocate_next_number(observed)
}

pub fn allocate_next_storage_number_for(kind: RecordKind, observed: &[i64]) -> i64 {
    allocate_next_storage_number(observed, kind.default_base_number())
}

pub fn allocate_next_display_number(
    kind: RecordKind,
    observed_storage_numbers: &[i64],
    padding: usize,
    display_prefix: &str,
) -> String {
    let storage = allocate_next_storage_number_for(kind, observed_storage_numbers);
    format!("{}-{}", display_prefix, pad_number(storage, padding))
}

pub fn allocate_from_observed_ids(
    spec: &IdSpec,
    observed_ids: &[&str],
) -> Result<String, IdentityError> {
    let mut observed = Vec::new();
    for id in observed_ids {
        observed.push(spec.parse_id(id)?);
    }
    let storage = allocate_next_storage_number(&observed, 1);
    Ok(spec.format_id(storage))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordCandidate {
    pub path: String,
    pub kind: RecordKind,
    pub filename_id: Option<String>,
    pub frontmatter_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conflict {
    DuplicateStorageId {
        storage_id: String,
        paths: Vec<String>,
    },
    FrontmatterFilenameMismatch {
        path: String,
        filename_id: String,
        frontmatter_id: String,
    },
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conflict::DuplicateStorageId { storage_id, paths } => {
                write!(f, "duplicate id {}: {:?}", storage_id, paths)
            }
            Conflict::FrontmatterFilenameMismatch {
                path,
                filename_id,
                frontmatter_id,
            } => {
                write!(
                    f,
                    "front-matter/filename mismatch at {}: {} vs {}",
                    path, filename_id, frontmatter_id
                )
            }
        }
    }
}

pub fn detect_conflicts(candidates: &[RecordCandidate]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    for c in candidates {
        if let (Some(f), Some(fm)) = (&c.filename_id, &c.frontmatter_id)
            && f != fm
        {
            let spec = c.kind.default_spec();
            let nf = spec.parse_id(f).ok();
            let nfm = spec.parse_id(fm).ok();
            match (nf, nfm) {
                (Some(a), Some(b)) if a != b => {
                    conflicts.push(Conflict::FrontmatterFilenameMismatch {
                        path: c.path.clone(),
                        filename_id: f.clone(),
                        frontmatter_id: fm.clone(),
                    });
                }
                (None, None) if f != fm => {
                    conflicts.push(Conflict::FrontmatterFilenameMismatch {
                        path: c.path.clone(),
                        filename_id: f.clone(),
                        frontmatter_id: fm.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in candidates {
        let spec = c.kind.default_spec();
        let raw = c.frontmatter_id.as_ref().or(c.filename_id.as_ref());
        if let Some(raw) = raw
            && let Ok(n) = spec.parse_id(raw)
        {
            let key = format!("{:?}:{}", c.kind, n);
            groups.entry(key).or_default().push(c.path.clone());
        }
    }
    for (key, paths) in groups {
        if paths.len() > 1 {
            conflicts.push(Conflict::DuplicateStorageId {
                storage_id: key,
                paths,
            });
        }
    }

    conflicts
}

pub fn find_conflicts(candidates: &[RecordCandidate]) -> Vec<Conflict> {
    detect_conflicts(candidates)
}
pub fn conflicts(candidates: &[RecordCandidate]) -> Vec<Conflict> {
    detect_conflicts(candidates)
}
pub fn check_conflicts(candidates: &[RecordCandidate]) -> Vec<Conflict> {
    detect_conflicts(candidates)
}

pub fn detect_duplicate_ids(kind: RecordKind, entries: &[(String, String)]) -> Vec<Conflict> {
    let spec = kind.default_spec();
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for (path, raw) in entries {
        if let Ok(n) = spec.parse_id(raw) {
            groups.entry(n).or_default().push(path.clone());
        }
    }
    let mut out = Vec::new();
    for (n, paths) in groups {
        if paths.len() > 1 {
            out.push(Conflict::DuplicateStorageId {
                storage_id: format!("{}-{}", spec.prefix, n),
                paths,
            });
        }
    }
    out
}
pub fn detect_duplicates(kind: RecordKind, entries: &[(String, String)]) -> Vec<Conflict> {
    detect_duplicate_ids(kind, entries)
}

pub fn validate_input(input: &str) -> Result<(), IdentityError> {
    for spec in [IdSpec::default_task(), IdSpec::default_milestone()] {
        if spec.validate_input(input).is_ok() {
            return Ok(());
        }
    }
    Err(IdentityError::InvalidFormat(input.to_string()))
}
pub fn validate_input_for(kind: RecordKind, input: &str) -> Result<(), IdentityError> {
    kind.default_spec().validate_input(input)
}
pub fn validate_input_with(spec: &IdSpec, input: &str) -> Result<(), IdentityError> {
    spec.validate_input(input)
}
pub fn task_validate_input(input: &str) -> Result<(), IdentityError> {
    IdSpec::default_task().validate_input(input)
}
pub fn milestone_validate_input(input: &str) -> Result<(), IdentityError> {
    IdSpec::default_milestone().validate_input(input)
}
pub fn normalize(id: &str) -> Result<i64, IdentityError> {
    for spec in [IdSpec::default_task(), IdSpec::default_milestone()] {
        if let Ok(n) = spec.parse_id(id) {
            return Ok(n);
        }
    }
    Err(IdentityError::InvalidFormat(id.to_string()))
}
