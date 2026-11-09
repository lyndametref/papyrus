//! Bidirectional translation between Papyrus display IDs and Backlog.md storage IDs.

use std::fmt;
use std::path::Path;

pub const TASK_OFFSET: i64 = 0;
pub const MILESTONE_OFFSET: i64 = 1;

pub const DEFAULT_TASK_DISPLAY_PREFIX: &str = "TASK";
pub const DEFAULT_TASK_STORAGE_PREFIX: &str = "task";
pub const DEFAULT_MILESTONE_DISPLAY_PREFIX: &str = "MS";
pub const DEFAULT_MILESTONE_STORAGE_PREFIX: &str = "m";
pub const DEFAULT_ID_PADDING: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum RecordKind {
    Task,
    Milestone,
}

impl RecordKind {
    pub fn offset(self) -> i64 {
        match self {
            RecordKind::Task => TASK_OFFSET,
            RecordKind::Milestone => MILESTONE_OFFSET,
        }
    }
    pub fn default_display_prefix(self) -> &'static str {
        match self {
            RecordKind::Task => DEFAULT_TASK_DISPLAY_PREFIX,
            RecordKind::Milestone => DEFAULT_MILESTONE_DISPLAY_PREFIX,
        }
    }
    pub fn default_storage_prefix(self) -> &'static str {
        match self {
            RecordKind::Task => DEFAULT_TASK_STORAGE_PREFIX,
            RecordKind::Milestone => DEFAULT_MILESTONE_STORAGE_PREFIX,
        }
    }
    pub fn default_base_storage_number(self) -> i64 {
        match self {
            RecordKind::Task => 1,
            RecordKind::Milestone => 0,
        }
    }
    pub fn default_spec(self) -> IdSpec {
        match self {
            RecordKind::Task => IdSpec::task(
                self.default_display_prefix(),
                self.default_storage_prefix(),
                DEFAULT_ID_PADDING,
            ),
            RecordKind::Milestone => IdSpec::milestone(
                self.default_display_prefix(),
                self.default_storage_prefix(),
                DEFAULT_ID_PADDING,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdSpec {
    pub kind: RecordKind,
    pub display_prefix: String,
    pub storage_prefix: String,
    pub frontmatter_prefix: String,
    pub offset: i64,
    pub padding: usize,
}

impl IdSpec {
    pub fn task(
        display_prefix: impl Into<String>,
        storage_prefix: impl Into<String>,
        padding: usize,
    ) -> Self {
        let storage_prefix = storage_prefix.into();
        let frontmatter_prefix = storage_prefix.to_ascii_uppercase();
        Self {
            kind: RecordKind::Task,
            display_prefix: display_prefix.into(),
            storage_prefix,
            frontmatter_prefix,
            offset: TASK_OFFSET,
            padding: padding.max(1),
        }
    }
    pub fn milestone(
        display_prefix: impl Into<String>,
        storage_prefix: impl Into<String>,
        padding: usize,
    ) -> Self {
        let storage_prefix = storage_prefix.into();
        Self {
            kind: RecordKind::Milestone,
            display_prefix: display_prefix.into(),
            storage_prefix: storage_prefix.clone(),
            frontmatter_prefix: storage_prefix,
            offset: MILESTONE_OFFSET,
            padding: padding.max(1),
        }
    }
    pub fn default_task() -> Self {
        Self::task(
            DEFAULT_TASK_DISPLAY_PREFIX,
            DEFAULT_TASK_STORAGE_PREFIX,
            DEFAULT_ID_PADDING,
        )
    }
    pub fn default_milestone() -> Self {
        Self::milestone(
            DEFAULT_MILESTONE_DISPLAY_PREFIX,
            DEFAULT_MILESTONE_STORAGE_PREFIX,
            DEFAULT_ID_PADDING,
        )
    }

    fn display_prefix_matches(&self, prefix: &str) -> bool {
        self.display_prefix.eq_ignore_ascii_case(prefix)
    }
    fn storage_prefix_matches(&self, prefix: &str) -> bool {
        self.storage_prefix.eq_ignore_ascii_case(prefix)
            || self.frontmatter_prefix.eq_ignore_ascii_case(prefix)
    }

    pub fn parse_display_number(&self, display_id: &str) -> Result<i64, IdentityError> {
        let (prefix, number) = split_id(display_id)?;
        if !self.display_prefix_matches(prefix) {
            return Err(IdentityError::InvalidPrefix {
                expected: self.display_prefix.clone(),
                found: prefix.to_string(),
            });
        }
        let n = parse_number(number)?;
        let storage_number = n - self.offset;
        if storage_number < 0 {
            return Err(IdentityError::OutOfRange(format!(
                "display number {} is below offset {}",
                n, self.offset
            )));
        }
        Ok(storage_number)
    }

    pub fn parse_storage_number(&self, storage_id: &str) -> Result<i64, IdentityError> {
        let (prefix, number) = split_id(storage_id)?;
        if !self.storage_prefix_matches(prefix) {
            return Err(IdentityError::InvalidPrefix {
                expected: self.storage_prefix.clone(),
                found: prefix.to_string(),
            });
        }
        // Storage IDs are canonical and unpadded: reject leading zeros (except "0").
        if number.len() > 1 && number.starts_with('0') {
            return Err(IdentityError::InvalidFormat(format!(
                "storage number must not have leading zeros: {}",
                number
            )));
        }
        let n = parse_number(number)?;
        if n < 0 {
            return Err(IdentityError::OutOfRange(
                "storage number must be non-negative".to_string(),
            ));
        }
        Ok(n)
    }

    pub fn to_storage_id(&self, display_id: &str) -> Result<String, IdentityError> {
        let n = self.parse_display_number(display_id)?;
        Ok(format!("{}-{}", self.storage_prefix, n))
    }

    pub fn to_frontmatter_id(&self, display_id: &str) -> Result<String, IdentityError> {
        let n = self.parse_display_number(display_id)?;
        Ok(format!("{}-{}", self.frontmatter_prefix, n))
    }

    pub fn to_display_id(&self, storage_id: &str) -> Result<String, IdentityError> {
        let n = self.parse_storage_number(storage_id)?;
        let display_number = n + self.offset;
        Ok(format!(
            "{}-{}",
            self.display_prefix,
            pad_number(display_number, self.padding)
        ))
    }

    pub fn validate_input(&self, input: &str) -> Result<(), IdentityError> {
        // Reject any storage-shaped input first, even when its number width
        // equals or exceeds the padding (which makes the case-insensitive
        // string comparison ambiguous).
        if self.parse_storage_number(input).is_ok() {
            let display = self
                .to_display_id(input)
                .unwrap_or_else(|_| input.to_string());
            return Err(IdentityError::StorageInput {
                storage_id: input.to_string(),
                display_id: display,
            });
        }
        if self.parse_display_number(input).is_ok() {
            return Ok(());
        }
        Err(IdentityError::InvalidFormat(input.to_string()))
    }

    pub fn normalize(&self, id: &str) -> Result<i64, IdentityError> {
        if let Ok(n) = self.parse_storage_number(id) {
            return Ok(n);
        }
        self.parse_display_number(id)
    }
}

fn split_id(raw: &str) -> Result<(&str, &str), IdentityError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(IdentityError::InvalidFormat(raw.to_string()));
    }
    let (prefix, number) = raw
        .rsplit_once('-')
        .ok_or_else(|| IdentityError::InvalidFormat(raw.to_string()))?;
    if prefix.is_empty() || number.is_empty() {
        return Err(IdentityError::InvalidFormat(raw.to_string()));
    }
    if !number.chars().all(|c| c.is_ascii_digit()) {
        return Err(IdentityError::NonNumeric(number.to_string()));
    }
    Ok((prefix, number))
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

fn default_specs() -> [IdSpec; 2] {
    [IdSpec::default_task(), IdSpec::default_milestone()]
}

pub fn to_storage_id(display_id: &str) -> Result<String, IdentityError> {
    for spec in default_specs() {
        if spec.parse_display_number(display_id).is_ok() {
            return spec.to_storage_id(display_id);
        }
    }
    Err(IdentityError::InvalidFormat(display_id.to_string()))
}

pub fn to_display_id(storage_id: &str) -> Result<String, IdentityError> {
    for spec in default_specs() {
        if spec.parse_storage_number(storage_id).is_ok() {
            return spec.to_display_id(storage_id);
        }
    }
    Err(IdentityError::InvalidFormat(storage_id.to_string()))
}

pub fn validate_input(input: &str) -> Result<(), IdentityError> {
    let prefix = split_prefix(input);
    for spec in default_specs() {
        if spec.display_prefix_matches(prefix) || spec.storage_prefix_matches(prefix) {
            return spec.validate_input(input);
        }
    }
    for spec in default_specs() {
        if let Err(e) = spec.validate_input(input)
            && matches!(e, IdentityError::StorageInput { .. })
        {
            return Err(e);
        }
    }
    for spec in default_specs() {
        if spec.validate_input(input).is_ok() {
            return Ok(());
        }
    }
    Err(IdentityError::InvalidFormat(input.to_string()))
}

fn split_prefix(raw: &str) -> &str {
    raw.split('-').next().unwrap_or(raw).trim()
}

pub fn normalize(id: &str) -> Result<i64, IdentityError> {
    for spec in default_specs() {
        if (spec.parse_storage_number(id).is_ok() || spec.parse_display_number(id).is_ok())
            && let Ok(n) = spec.normalize(id)
        {
            return Ok(n);
        }
    }
    Err(IdentityError::InvalidFormat(id.to_string()))
}

pub fn to_storage_id_for(kind: RecordKind, display_id: &str) -> Result<String, IdentityError> {
    kind.default_spec().to_storage_id(display_id)
}

pub fn to_display_id_for(kind: RecordKind, storage_id: &str) -> Result<String, IdentityError> {
    kind.default_spec().to_display_id(storage_id)
}

pub fn validate_input_for(kind: RecordKind, input: &str) -> Result<(), IdentityError> {
    kind.default_spec().validate_input(input)
}

pub fn to_storage_id_with(spec: &IdSpec, display_id: &str) -> Result<String, IdentityError> {
    spec.to_storage_id(display_id)
}

pub fn to_display_id_with(spec: &IdSpec, storage_id: &str) -> Result<String, IdentityError> {
    spec.to_display_id(storage_id)
}

pub fn validate_input_with(spec: &IdSpec, input: &str) -> Result<(), IdentityError> {
    spec.validate_input(input)
}

pub fn task_to_storage_id(display_id: &str) -> Result<String, IdentityError> {
    RecordKind::Task.default_spec().to_storage_id(display_id)
}
pub fn task_to_display_id(storage_id: &str) -> Result<String, IdentityError> {
    RecordKind::Task.default_spec().to_display_id(storage_id)
}
pub fn task_validate_input(input: &str) -> Result<(), IdentityError> {
    RecordKind::Task.default_spec().validate_input(input)
}
pub fn milestone_to_storage_id(display_id: &str) -> Result<String, IdentityError> {
    RecordKind::Milestone
        .default_spec()
        .to_storage_id(display_id)
}
pub fn milestone_to_display_id(storage_id: &str) -> Result<String, IdentityError> {
    RecordKind::Milestone
        .default_spec()
        .to_display_id(storage_id)
}
pub fn milestone_validate_input(input: &str) -> Result<(), IdentityError> {
    RecordKind::Milestone.default_spec().validate_input(input)
}

pub fn storage_id(display_id: &str) -> Result<String, IdentityError> {
    to_storage_id(display_id)
}
pub fn display_id(storage_id: &str) -> Result<String, IdentityError> {
    to_display_id(storage_id)
}

pub fn base_storage_number(kind: RecordKind) -> i64 {
    kind.default_base_storage_number()
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

pub fn allocate_next_storage_number_for(kind: RecordKind, observed: &[i64]) -> i64 {
    allocate_next_storage_number(observed, kind.default_base_storage_number())
}

pub fn allocate_next_display_number(
    kind: RecordKind,
    observed_storage_numbers: &[i64],
    padding: usize,
    display_prefix: &str,
) -> String {
    let storage = allocate_next_storage_number_for(kind, observed_storage_numbers);
    let display_number = storage + kind.offset();
    format!("{}-{}", display_prefix, pad_number(display_number, padding))
}

pub fn allocate_from_observed_ids(
    spec: &IdSpec,
    observed_storage_ids: &[&str],
) -> Result<String, IdentityError> {
    let mut observed = Vec::new();
    for id in observed_storage_ids {
        observed.push(spec.parse_storage_number(id)?);
    }
    let storage = allocate_next_storage_number_for(spec.kind, &observed);
    let display_number = storage + spec.offset;
    Ok(format!(
        "{}-{}",
        spec.display_prefix,
        pad_number(display_number, spec.padding)
    ))
}

pub fn allocate_next(observed: &[i64]) -> i64 {
    allocate_next_storage_number(observed, 1)
}
pub fn allocate_next_number(observed: &[i64]) -> i64 {
    allocate_next_storage_number(observed, 1)
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

fn display_id_for_storage(raw: &str) -> String {
    for spec in default_specs() {
        if spec.parse_storage_number(raw).is_ok()
            && let Ok(d) = spec.to_display_id(raw)
        {
            return d;
        }
    }
    raw.to_string()
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conflict::DuplicateStorageId { storage_id, paths } => {
                let display = display_id_for_storage(storage_id);
                write!(
                    f,
                    "duplicate storage id {} (display id {}): {:?}",
                    storage_id, display, paths
                )
            }
            Conflict::FrontmatterFilenameMismatch {
                path,
                filename_id,
                frontmatter_id,
            } => {
                let fdisp = display_id_for_storage(filename_id);
                let fm_disp = display_id_for_storage(frontmatter_id);
                write!(
                    f,
                    "front-matter/filename mismatch at {}: {} (display {}) vs {} (display {})",
                    path, filename_id, fdisp, frontmatter_id, fm_disp
                )
            }
        }
    }
}

pub fn detect_conflicts(candidates: &[RecordCandidate]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    for c in candidates {
        if let (Some(f), Some(fm)) = (&c.filename_id, &c.frontmatter_id) {
            let spec = c.kind.default_spec();
            let nf = spec.parse_storage_number(f).ok();
            let nfm = spec.parse_storage_number(fm).ok();
            match (nf, nfm) {
                (Some(a), Some(b)) if a != b => {
                    conflicts.push(Conflict::FrontmatterFilenameMismatch {
                        path: c.path.clone(),
                        filename_id: f.clone(),
                        frontmatter_id: fm.clone(),
                    });
                }
                (Some(_), Some(_)) => {}
                _ => {
                    if f != fm {
                        conflicts.push(Conflict::FrontmatterFilenameMismatch {
                            path: c.path.clone(),
                            filename_id: f.clone(),
                            frontmatter_id: fm.clone(),
                        });
                    }
                }
            }
        }
    }

    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, (RecordKind, i64, Vec<String>)> = BTreeMap::new();
    for c in candidates {
        let spec = c.kind.default_spec();
        let raw = c.frontmatter_id.as_ref().or(c.filename_id.as_ref());
        if let Some(raw) = raw
            && let Ok(n) = spec.parse_storage_number(raw)
        {
            let key = format!("{:?}:{}", c.kind, n);
            let entry = groups.entry(key).or_insert_with(|| (c.kind, n, Vec::new()));
            entry.2.push(c.path.clone());
        }
    }
    for (_key, (kind, n, paths)) in groups {
        if paths.len() > 1 {
            let spec = kind.default_spec();
            let storage_id = format!("{}-{}", spec.storage_prefix, n);
            conflicts.push(Conflict::DuplicateStorageId { storage_id, paths });
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
        if let Ok(n) = spec.parse_storage_number(raw) {
            groups.entry(n).or_default().push(path.clone());
        }
    }
    let mut out = Vec::new();
    for (n, paths) in groups {
        if paths.len() > 1 {
            let storage_id = format!("{}-{}", spec.storage_prefix, n);
            out.push(Conflict::DuplicateStorageId { storage_id, paths });
        }
    }
    out
}

pub fn detect_duplicates(kind: RecordKind, entries: &[(String, String)]) -> Vec<Conflict> {
    detect_duplicate_ids(kind, entries)
}

pub fn format_id(prefix: &str, number: i64, padding: usize) -> String {
    format!("{}-{}", prefix, pad_number(number, padding))
}

pub fn format_seq_id(group: &str, number: i64, padding: usize) -> String {
    format!("{}.{}", group, pad_number(number, padding))
}

pub fn id_number(stem: &str, prefix: &str) -> Option<u32> {
    let rest = strip_prefix_number(stem, prefix)?;
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u32>().ok()
}

pub fn strip_prefix_number<'a>(stem: &'a str, prefix: &str) -> Option<&'a str> {
    stem.strip_prefix(prefix)?.strip_prefix('-')
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
    observed_numbers_in_dirs(&[active, deprecated], prefix)
        .into_iter()
        .map(|n| n as u32)
        .max()
        .unwrap_or(0)
}

pub fn scan_max_single(dir: &Path, prefix: &str) -> u32 {
    observed_numbers_in_dirs(&[dir], prefix)
        .into_iter()
        .map(|n| n as u32)
        .max()
        .unwrap_or(0)
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
                if let Some(n) = task_number_from_stem(stem)
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
                if let Some(n) = milestone_number_from_stem(stem)
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
    let base = stem.split(" - ").next()?.trim();
    let storage_id = task_to_storage_id(base).unwrap_or_else(|_| base.to_lowercase());
    let num = storage_id.strip_prefix("task-")?;
    num.parse::<u32>().ok()
}

pub fn milestone_number_from_stem(stem: &str) -> Option<u32> {
    let base = stem.split(" - ").next()?.trim();
    let storage_id = milestone_to_storage_id(base).unwrap_or_else(|_| base.to_lowercase());
    let num = storage_id.strip_prefix("m-")?;
    num.parse::<u32>().ok().map(|n| n + 1)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_round_trip() {
        let spec = IdSpec::default_task();
        for n in [1i64, 18, 60, 1000] {
            let display = format!("TASK-{}", pad_number(n, 3));
            let storage = spec.to_storage_id(&display).unwrap();
            assert_eq!(storage, format!("task-{}", n));
            let back = spec.to_display_id(&storage).unwrap();
            assert_eq!(back, display);
        }
    }

    #[test]
    fn milestone_offset() {
        let spec = IdSpec::default_milestone();
        assert_eq!(spec.to_storage_id("MS-001").unwrap(), "m-0");
        assert_eq!(spec.to_display_id("m-0").unwrap(), "MS-001");
        assert_eq!(spec.to_storage_id("MS-013").unwrap(), "m-12");
        assert_eq!(spec.to_display_id("m-12").unwrap(), "MS-013");
        for n in 0i64..=20 {
            let display = format!("MS-{}", pad_number(n + 1, 3));
            let storage = format!("m-{}", n);
            assert_eq!(spec.to_storage_id(&display).unwrap(), storage);
            assert_eq!(spec.to_display_id(&storage).unwrap(), display);
        }
    }

    #[test]
    fn task_example() {
        assert_eq!(to_storage_id("TASK-018").unwrap(), "task-18");
        assert_eq!(to_display_id("task-18").unwrap(), "TASK-018");
        assert_eq!(to_display_id("TASK-18").unwrap(), "TASK-018");
    }

    #[test]
    fn padding_minimum() {
        let spec = IdSpec::task("TASK", "task", 3);
        let display = spec.to_display_id("task-10000").unwrap();
        assert_eq!(display, "TASK-10000");
        let storage = spec.to_storage_id("TASK-10000").unwrap();
        assert_eq!(storage, "task-10000");
    }

    #[test]
    fn frontmatter_id() {
        let spec = IdSpec::default_task();
        assert_eq!(spec.to_frontmatter_id("TASK-018").unwrap(), "TASK-18");
    }

    #[test]
    fn validate_input_rejects_storage() {
        let spec = IdSpec::default_task();
        assert!(spec.validate_input("task-18").is_err());
        assert!(spec.validate_input("TASK-18").is_err());
        let err = spec.validate_input("task-18").unwrap_err();
        assert!(err.to_string().contains("TASK-018"));
        assert!(spec.validate_input("TASK-018").is_ok());
    }

    #[test]
    fn validate_input_milestone() {
        let spec = IdSpec::default_milestone();
        assert!(spec.validate_input("m-12").is_err());
        let err = spec.validate_input("m-12").unwrap_err();
        assert!(err.to_string().contains("MS-013"));
        assert!(spec.validate_input("MS-013").is_ok());
        assert!(spec.validate_input("MS-12").is_ok());
    }

    #[test]
    fn malformed_rejected() {
        let spec = IdSpec::default_task();
        assert!(spec.to_storage_id("nope").is_err());
        assert!(spec.to_storage_id("TASK-").is_err());
        assert!(spec.to_storage_id("TASK-abc").is_err());
        assert!(spec.to_storage_id("OTHER-1").is_err());
        assert!(spec.to_display_id("foo-1").is_err());
        assert!(spec.to_display_id("task-").is_err());
    }

    #[test]
    fn out_of_range_milestone() {
        let spec = IdSpec::default_milestone();
        assert!(spec.to_storage_id("MS-000").is_err());
    }

    #[test]
    fn normalize() {
        let spec = IdSpec::default_task();
        assert_eq!(spec.normalize("task-18").unwrap(), 18);
        assert_eq!(spec.normalize("TASK-18").unwrap(), 18);
        assert_eq!(spec.normalize("TASK-018").unwrap(), 18);
        assert_eq!(spec.normalize("task-018").unwrap(), 18);
    }

    #[test]
    fn default_free_functions() {
        assert_eq!(to_storage_id("TASK-018").unwrap(), "task-18");
        assert_eq!(to_display_id("task-18").unwrap(), "TASK-018");
        assert_eq!(to_display_id("TASK-18").unwrap(), "TASK-018");
        assert!(validate_input("task-18").is_err());
        assert!(validate_input("TASK-018").is_ok());
    }

    #[test]
    fn allocation() {
        let observed = vec![18i64, 19, 20, 26, 48, 60];
        assert_eq!(
            allocate_next_storage_number_for(RecordKind::Task, &observed),
            61
        );
        let display = allocate_next_display_number(RecordKind::Task, &observed, 3, "TASK");
        assert_eq!(display, "TASK-061");
        let mobs = vec![0i64, 1, 2, 12];
        assert_eq!(
            allocate_next_storage_number_for(RecordKind::Milestone, &mobs),
            13
        );
        assert_eq!(
            allocate_next_display_number(RecordKind::Milestone, &mobs, 3, "MS"),
            "MS-014"
        );
        assert_eq!(allocate_next_storage_number_for(RecordKind::Task, &[]), 1);
        assert_eq!(
            allocate_next_storage_number_for(RecordKind::Milestone, &[]),
            0
        );
    }

    #[test]
    fn allocate_from_observed_ids_test() {
        let spec = IdSpec::default_task();
        let ids = ["task-18", "task-19", "TASK-20"];
        let d = allocate_from_observed_ids(&spec, &ids).unwrap();
        assert_eq!(d, "TASK-021");
    }

    #[test]
    fn conflict_duplicate() {
        let cands = vec![
            RecordCandidate {
                path: "a.md".into(),
                kind: RecordKind::Task,
                filename_id: Some("task-18".into()),
                frontmatter_id: None,
            },
            RecordCandidate {
                path: "b.md".into(),
                kind: RecordKind::Task,
                filename_id: Some("task-18".into()),
                frontmatter_id: None,
            },
        ];
        let cs = detect_conflicts(&cands);
        assert_eq!(cs.len(), 1);
        match &cs[0] {
            Conflict::DuplicateStorageId { paths, .. } => assert_eq!(paths.len(), 2),
            _ => panic!("expected duplicate"),
        }
    }

    #[test]
    fn conflict_mismatch() {
        let cands = vec![RecordCandidate {
            path: "x.md".into(),
            kind: RecordKind::Task,
            filename_id: Some("task-18".into()),
            frontmatter_id: Some("TASK-19".into()),
        }];
        let cs = detect_conflicts(&cands);
        assert_eq!(cs.len(), 1);
        assert!(matches!(
            cs[0],
            Conflict::FrontmatterFilenameMismatch { .. }
        ));
    }

    #[test]
    fn conflict_not_slug() {
        let cands = vec![
            RecordCandidate {
                path: "a.md".into(),
                kind: RecordKind::Task,
                filename_id: Some("task-1".into()),
                frontmatter_id: None,
            },
            RecordCandidate {
                path: "b.md".into(),
                kind: RecordKind::Task,
                filename_id: Some("task-2".into()),
                frontmatter_id: None,
            },
        ];
        assert!(detect_conflicts(&cands).is_empty());
    }

    #[test]
    fn round_trip_property_ish() {
        let spec = IdSpec::default_task();
        for n in 0..=500i64 {
            let display = format!("TASK-{}", pad_number(n, 3));
            let storage = spec.to_storage_id(&display).unwrap();
            assert_eq!(storage, format!("task-{}", n));
            assert_eq!(spec.to_display_id(&storage).unwrap(), display);
        }
        let mspec = IdSpec::default_milestone();
        for n in 0..=500i64 {
            let display = format!("MS-{}", pad_number(n + 1, 3));
            let storage = format!("m-{}", n);
            assert_eq!(mspec.to_storage_id(&display).unwrap(), storage);
            assert_eq!(mspec.to_display_id(&storage).unwrap(), display);
        }
    }

    #[test]
    fn monotonic() {
        let spec = IdSpec::default_task();
        let mut prev = -1i64;
        for n in [1i64, 2, 5, 10, 100] {
            let d = format!("TASK-{}", pad_number(n, 3));
            let s = spec.parse_display_number(&d).unwrap();
            assert!(s > prev);
            prev = s;
        }
    }

    #[test]
    fn injectivity() {
        let spec = IdSpec::default_task();
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for n in 0..=100i64 {
            let d = format!("TASK-{}", pad_number(n, 3));
            let s = spec.to_storage_id(&d).unwrap();
            assert!(seen.insert(s));
        }
    }

    #[test]
    fn detect_duplicate_ids_fn() {
        let entries = vec![
            ("a.md".to_string(), "task-1".to_string()),
            ("b.md".to_string(), "task-1".to_string()),
        ];
        let cs = detect_duplicate_ids(RecordKind::Task, &entries);
        assert_eq!(cs.len(), 1);
    }

    #[test]
    fn property_round_trip_custom_prefixes_paddings_offsets() {
        let prefixes = [
            ("FOO", "foo"),
            ("BAR", "bar"),
            ("X", "x"),
            ("TASK", "task"),
            ("MS", "m"),
        ];
        let paddings = [1usize, 2, 3, 5, 8];
        let offsets = [0i64, 1, 2, 5];
        for (dp, sp) in prefixes {
            for &pad in &paddings {
                for &off in &offsets {
                    let spec = IdSpec {
                        kind: RecordKind::Task,
                        display_prefix: dp.to_string(),
                        storage_prefix: sp.to_string(),
                        frontmatter_prefix: sp.to_string(),
                        offset: off,
                        padding: pad.max(1),
                    };
                    for storage_number in 0i64..=200 {
                        let display_number = storage_number + off;
                        if display_number < 0 {
                            continue;
                        }
                        let storage_id = format!("{}-{}", sp, storage_number);
                        let display_id = format!("{}-{}", dp, pad_number(display_number, pad));
                        assert_eq!(
                            spec.to_display_id(&storage_id).unwrap(),
                            display_id,
                            "storage->display failed for {} pad {} off {}",
                            storage_id,
                            pad,
                            off
                        );
                        assert_eq!(
                            spec.to_storage_id(&display_id).unwrap(),
                            storage_id,
                            "display->storage failed for {} pad {} off {}",
                            display_id,
                            pad,
                            off
                        );
                        // round trip
                        assert_eq!(
                            spec.to_storage_id(&spec.to_display_id(&storage_id).unwrap())
                                .unwrap(),
                            storage_id
                        );
                        assert_eq!(
                            spec.to_display_id(&spec.to_storage_id(&display_id).unwrap())
                                .unwrap(),
                            display_id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn property_monotonic_custom() {
        let spec = IdSpec {
            kind: RecordKind::Task,
            display_prefix: "FOO".to_string(),
            storage_prefix: "foo".to_string(),
            frontmatter_prefix: "foo".to_string(),
            offset: 3,
            padding: 4,
        };
        let mut prev_storage = -1i64;
        let mut prev_display = -1i64;
        for n in [0i64, 1, 2, 5, 10, 99, 1000] {
            let storage_id = format!("foo-{}", n);
            let display_id = spec.to_display_id(&storage_id).unwrap();
            let sn = spec.parse_storage_number(&storage_id).unwrap();
            let dn = spec.parse_display_number(&display_id).unwrap();
            assert!(sn > prev_storage);
            assert!(dn > prev_display);
            prev_storage = sn;
            prev_display = dn;
        }
    }

    #[test]
    fn property_injectivity_custom() {
        use std::collections::HashSet;
        let spec = IdSpec {
            kind: RecordKind::Task,
            display_prefix: "BAR".to_string(),
            storage_prefix: "bar".to_string(),
            frontmatter_prefix: "bar".to_string(),
            offset: 1,
            padding: 2,
        };
        let mut seen_storage = HashSet::new();
        let mut seen_display = HashSet::new();
        for n in 0..=300i64 {
            let storage_id = format!("bar-{}", n);
            let display_id = spec.to_display_id(&storage_id).unwrap();
            assert!(seen_storage.insert(storage_id.clone()));
            assert!(seen_display.insert(display_id));
        }
    }

    #[test]
    fn property_malformed_rejected_custom() {
        let spec = IdSpec {
            kind: RecordKind::Task,
            display_prefix: "FOO".to_string(),
            storage_prefix: "foo".to_string(),
            frontmatter_prefix: "foo".to_string(),
            offset: 0,
            padding: 3,
        };
        for bad in [
            "", "nope", "FOO-", "FOO-abc", "BAR-1", "foo", "FOO--1", "FOO-1-2",
            "FOO-01", // leading zero storage-ish? display allows? parse_display ok actually -> storage? let's not assert
        ] {
            // Some may be valid display; only assert no panic and consistent handling.
            let _ = spec.to_storage_id(bad);
            let _ = spec.to_display_id(bad);
            let _ = spec.validate_input(bad);
        }
        // definitely malformed
        for bad in [
            "", "nope", "FOO-", "FOO-abc", "BAR-1", "foo", "FOO--1", "FOO-1-2",
        ] {
            assert!(
                spec.to_storage_id(bad).is_err(),
                "to_storage_id should reject {}",
                bad
            );
            assert!(
                spec.to_display_id(bad).is_err(),
                "to_display_id should reject {}",
                bad
            );
        }
        // storage-shaped inputs rejected by validate_input
        for bad in ["foo-1", "foo-100", "FOO-1"] {
            assert!(
                spec.validate_input(bad).is_err(),
                "validate_input should reject {}",
                bad
            );
        }
    }

    #[test]
    fn property_storage_leading_zeros_rejected() {
        let spec = IdSpec::default_task();
        for bad in ["task-01", "task-001", "TASK-01", "TASK-001"] {
            assert!(
                spec.parse_storage_number(bad).is_err(),
                "leading zero storage should reject {}",
                bad
            );
        }
        // canonical unpadded still works
        assert_eq!(spec.parse_storage_number("task-1").unwrap(), 1);
        assert_eq!(spec.parse_storage_number("TASK-1").unwrap(), 1);
    }
}
