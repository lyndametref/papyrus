//! Record indexing, parsing, validation, and atomic persistence for the local
//! browser increment (specification sections 4, 7, and 8).
//!
//! Every filesystem path is resolved through [`Context::ensure_within`] before
//! it is read or written. Record IDs are resolved by exact match against the
//! active directory listing and are never joined to a path.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::requirement::parse_heading;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Maximum accepted record ID length in bytes (specification section 7.1).
pub const MAX_ID_BYTES: usize = 128;
/// Maximum accepted title length in Unicode scalar values (section 7.3).
pub const MAX_TITLE_CHARS: usize = 200;
/// Maximum accepted story or description length (section 7.3).
pub const MAX_TEXT_CHARS: usize = 20000;

/// Browser content category. Covers both document-store and backlog-store records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Req,
    Nfr,
    Task,
    Milestone,
    Release,
}

impl Category {
    pub fn key(self) -> &'static str {
        match self {
            Category::Req => "req",
            Category::Nfr => "nfr",
            Category::Task => "task",
            Category::Milestone => "milestone",
            Category::Release => "release",
        }
    }

    pub fn supports_story(self) -> bool {
        matches!(self, Category::Req)
    }

    pub fn is_requirement(self) -> bool {
        matches!(self, Category::Req | Category::Nfr)
    }
}

/// The two record forms in this increment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Group,
    Requirement,
    Task,
    Milestone,
    Release,
}

impl Form {
    pub fn label(self) -> &'static str {
        match self {
            Form::Group => "Group",
            Form::Requirement => "Requirement",
            Form::Task => "Task",
            Form::Milestone => "Milestone",
            Form::Release => "Release",
        }
    }
}

/// A parsed `## ` section. `body` keeps every line exactly as stored so an
/// unknown section can be written back content-identically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub body: Vec<String>,
}

/// A parsed record document: heading line plus ordered sections.
#[derive(Clone, Debug)]
pub struct Doc {
    pub id: String,
    pub title: String,
    pub heading: String,
    /// Lines between the heading and the first section (usually a blank line).
    pub preamble: Vec<String>,
    pub sections: Vec<Section>,
}

/// A loaded record with its metadata.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub kind: Category,
    pub form: Form,
    pub title: String,
    pub story: Option<String>,
    pub group_id: Option<String>,
    pub group_title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub milestone_id: Option<String>,
    pub milestone_title: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub comments: Vec<(String, String)>,
    pub release_id: Option<String>,
    pub release_title: Option<String>,
    pub doc: Doc,
    pub path: PathBuf,
    pub rel_path: String,
    pub version: String,
}

impl Record {
    pub fn kind_key(&self) -> &'static str {
        self.kind.key()
    }

    pub fn type_label(&self) -> String {
        let base = match self.kind {
            Category::Req => "Business requirement",
            Category::Nfr => "Non-functional requirement",
            Category::Task => "Task",
            Category::Milestone => "Milestone",
            Category::Release => "Release",
        };
        match self.form {
            Form::Group => format!("{} group", base),
            Form::Requirement => base.to_string(),
            Form::Task | Form::Milestone | Form::Release => base.to_string(),
        }
    }
}

/// An indexed document category. Records are sorted by ID ascending.
#[derive(Debug, Default)]
pub struct Index {
    pub records: Vec<Record>,
    pub groups: Vec<Record>,
    pub requirements: Vec<Record>,
    /// Member requirement indexes keyed by resolved group ID.
    pub members: BTreeMap<String, Vec<usize>>,
    /// Requirement indexes whose stored group reference is absent or unresolved.
    pub other: Vec<usize>,
}

impl Index {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// A record resolved by exact ID match.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub record: Record,
    pub kind: Category,
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Parse a record document. The first line is the heading; every `## ` line
/// starts a section and every following line belongs to that section.
pub fn parse_doc(content: &str) -> Doc {
    let normalized = normalize_newlines(content);
    let mut lines: Vec<&str> = normalized.split('\n').collect();
    // A trailing newline produces one empty final element; drop it so section
    // bodies do not gain a synthetic blank line at the end of the file.
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let heading = lines.first().copied().unwrap_or("").to_string();
    let (id, title) = parse_heading(&heading);
    let mut sections: Vec<Section> = Vec::new();
    let mut preamble: Vec<String> = Vec::new();
    let mut i = 1usize;
    while i < lines.len() {
        if let Some(rest) = lines[i].strip_prefix("## ") {
            let heading_line = format!("## {}", rest);
            let mut body: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("## ") {
                body.push(lines[i].to_string());
                i += 1;
            }
            sections.push(Section {
                heading: heading_line,
                body,
            });
        } else if sections.is_empty() {
            preamble.push(lines[i].to_string());
            i += 1;
        } else {
            // Content between sections is part of the previous section body.
            if let Some(last) = sections.last_mut() {
                last.body.push(lines[i].to_string());
            }
            i += 1;
        }
    }
    Doc {
        id,
        title,
        heading,
        preamble,
        sections,
    }
}

fn section_value(doc: &Doc, heading: &str) -> Option<String> {
    let section = doc.sections.iter().find(|s| s.heading == heading)?;
    let text = section.body.join("\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn section_first_line(doc: &Doc, heading: &str) -> Option<String> {
    let section = doc.sections.iter().find(|s| s.heading == heading)?;
    let text = section.body.join("\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.lines().next().unwrap_or("").to_string())
    }
}

fn parse_group(doc: &Doc) -> (Option<String>, Option<String>) {
    let Some(line) = section_first_line(doc, "## Group") else {
        return (None, None);
    };
    let mut parts = line.splitn(2, char::is_whitespace);
    let id = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let title = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    (id, title)
}

/// Validate a record ID (specification section 7.1).
pub fn valid_id(id: &str) -> bool {
    if id.is_empty() || id.len() > MAX_ID_BYTES {
        return false;
    }
    if id.starts_with('.') || id.starts_with('-') {
        return false;
    }
    if id.contains("..") || id == "." || id == ".." {
        return false;
    }
    id.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn form_for_stem(prefix: &str, stem: &str) -> Option<Form> {
    let rest = stem.strip_prefix(prefix)?.strip_prefix('-')?;
    if rest.is_empty() {
        return None;
    }
    if rest.bytes().all(|b| b.is_ascii_digit()) {
        return Some(Form::Group);
    }
    let (group, sequence) = rest.split_once('.')?;
    if !group.is_empty()
        && !sequence.is_empty()
        && group.bytes().all(|b| b.is_ascii_digit())
        && sequence.bytes().all(|b| b.is_ascii_digit())
    {
        return Some(Form::Requirement);
    }
    None
}

fn active_entry(cfg: &Config, kind: Category) -> Option<crate::config::DocTypeEntry> {
    match kind {
        Category::Req | Category::Nfr => cfg
            .document_types
            .get(kind.key())
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::Task => Some(crate::config::DocTypeEntry {
            prefix: cfg.task_prefix.clone(),
            directory: "tasks".to_string(),
            enabled: true,
        }),
        Category::Milestone => Some(crate::config::DocTypeEntry {
            prefix: cfg.milestone_prefix.clone(),
            directory: "milestones".to_string(),
            enabled: true,
        }),
        Category::Release => Some(crate::config::DocTypeEntry {
            prefix: cfg.release_prefix.clone(),
            directory: "releases".to_string(),
            enabled: true,
        }),
    }
}

fn active_dir(ctx: &Context, cfg: &Config, kind: Category) -> Option<PathBuf> {
    let entry = active_entry(cfg, kind)?;
    let base = match kind {
        Category::Req | Category::Nfr => ctx.project_dir.join(&cfg.doc_directory),
        Category::Task | Category::Milestone | Category::Release => {
            ctx.project_dir.join(&cfg.backlog_directory)
        }
    };
    let raw = base.join(&entry.directory);
    Context::ensure_within(&ctx.project_dir, &raw).ok()
}

/// Format the optimistic-concurrency token from file metadata.
pub fn version_token(meta: &fs::Metadata) -> String {
    let modified = meta.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!(
        "{}.{}.{}",
        duration.as_secs(),
        duration.subsec_nanos(),
        meta.len()
    )
}

/// Whether a token has the documented `seconds.nanos.length` shape.
pub fn valid_version_token(token: &str) -> bool {
    let mut parts = token.split('.');
    let (Some(seconds), Some(nanos), Some(length), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    !seconds.is_empty()
        && !nanos.is_empty()
        && !length.is_empty()
        && seconds.bytes().all(|b| b.is_ascii_digit())
        && nanos.bytes().all(|b| b.is_ascii_digit())
        && length.bytes().all(|b| b.is_ascii_digit())
}

fn relative_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn load_backlog_record(
    ctx: &Context,
    kind: Category,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        Err(_) => return Ok(None),
    };
    let content = match fs::read_to_string(&safe) {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };
    let meta = match fs::metadata(&safe) {
        Ok(meta) => meta,
        Err(_) => return Ok(None),
    };
    let Some(stem) = safe.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, kind) {
        Some(e) => e,
        None => return Ok(None),
    };
    // validate stem against prefix
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }

    let (
        title,
        status,
        milestone_id,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
        description,
    ) = match kind {
        Category::Task => {
            let td = crate::task::parse_task(&content);
            let title = if td.title.is_empty() {
                stem.to_string()
            } else {
                td.title.clone()
            };
            (
                title,
                if td.status.is_empty() {
                    None
                } else {
                    Some(td.status.clone())
                },
                td.milestone.clone(),
                td.assignee.clone(),
                td.labels.clone(),
                td.deps.clone(),
                td.acs.clone(),
                td.comments.clone(),
                None,
                td.description.clone(),
            )
        }
        Category::Milestone => {
            let (_id, title, desc, release) = crate::milestone::parse_milestone(&content);
            let title = if title.is_empty() {
                stem.to_string()
            } else {
                title
            };
            (
                title,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                release,
                desc,
            )
        }
        Category::Release => {
            let (_id, title, desc) = crate::release::parse_release(&content);
            let title = if title.is_empty() {
                stem.to_string()
            } else {
                title
            };
            (
                title,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
                desc,
            )
        }
        _ => return Ok(None),
    };

    // milestone title resolution
    let milestone_title = if kind == Category::Task {
        milestone_id.as_ref().and_then(|mid| {
            // read milestone file
            let dir = active_dir(ctx, &cfg, Category::Milestone)?;
            let path = dir.join(format!("{}.md", mid));
            let safe = Context::ensure_within(&ctx.project_dir, &path).ok()?;
            if !safe.exists() {
                return None;
            }
            let c = fs::read_to_string(&safe).ok()?;
            let (_, t, _, _) = crate::milestone::parse_milestone(&c);
            if t.is_empty() { None } else { Some(t) }
        })
    } else {
        None
    };
    let release_title = if kind == Category::Milestone {
        release_id.as_ref().and_then(|rid| {
            let dir = active_dir(ctx, &cfg, Category::Release)?;
            let path = dir.join(format!("{}.md", rid));
            let safe = Context::ensure_within(&ctx.project_dir, &path).ok()?;
            if !safe.exists() {
                return None;
            }
            let c = fs::read_to_string(&safe).ok()?;
            let (_, t, _) = crate::release::parse_release(&c);
            if t.is_empty() { None } else { Some(t) }
        })
    } else {
        None
    };

    let form = match kind {
        Category::Task => Form::Task,
        Category::Milestone => Form::Milestone,
        Category::Release => Form::Release,
        _ => return Ok(None),
    };

    Ok(Some(Record {
        id: stem.to_string(),
        kind,
        form,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        status,
        milestone_id,
        milestone_title,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
        release_title,
        doc: parse_doc(&content),
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_record_from_path(
    ctx: &Context,
    kind: Category,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    if matches!(
        kind,
        Category::Task | Category::Milestone | Category::Release
    ) {
        return load_backlog_record(ctx, kind, path);
    }
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        // A symlink escaping the project is not listed and is not readable.
        Err(_) => return Ok(None),
    };
    let content = match fs::read_to_string(&safe) {
        Ok(content) => content,
        // A non-UTF-8 or unreadable record is skipped by the index.
        Err(_) => return Ok(None),
    };
    let meta = match fs::metadata(&safe) {
        Ok(meta) => meta,
        Err(_) => return Ok(None),
    };
    let doc = parse_doc(&content);
    let Some(stem) = safe.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    if !valid_id(stem) {
        return Ok(None);
    }
    let Some(entry) = active_entry_owned(ctx, kind) else {
        return Ok(None);
    };
    let Some(form) = form_for_stem(&entry.prefix, stem) else {
        return Ok(None);
    };
    let (group_id, group_title) = if form == Form::Requirement {
        parse_group(&doc)
    } else {
        (None, None)
    };
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let (
        status,
        milestone_id,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
    ) = match kind {
        Category::Req | Category::Nfr => (
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        ),
        _ => (
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        ),
    };
    Ok(Some(Record {
        id: stem.to_string(),
        kind,
        form,
        title,
        story: section_value(&doc, "## User Story"),
        group_id,
        group_title,
        description: section_value(&doc, "## Description"),
        status,
        milestone_id,
        milestone_title: None,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
        release_title: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn active_entry_owned(ctx: &Context, kind: Category) -> Option<crate::config::DocTypeEntry> {
    let cfg = crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir).ok()?;
    active_entry(&cfg, kind)
}

/// Build the index for one category. Missing or unreadable directories render
/// as an empty category; they are not server errors.
pub fn build_index(ctx: &Context, cfg: &Config, kind: Category) -> Index {
    let mut index = Index::default();
    if matches!(
        kind,
        Category::Task | Category::Milestone | Category::Release
    ) {
        let Some(entry) = active_entry(cfg, kind) else {
            return index;
        };
        let Some(dir) = active_dir(ctx, cfg, kind) else {
            return index;
        };
        let Ok(read_dir) = fs::read_dir(&dir) else {
            return index;
        };
        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !valid_id(stem) {
                continue;
            }
            let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
                Some(r) => r,
                None => continue,
            };
            if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            if let Ok(Some(record)) = load_record_from_path(ctx, kind, &path) {
                index.records.push(record);
            }
        }
        index.records.sort_by(|a, b| a.id.cmp(&b.id));
        return index;
    }
    let Some(entry) = active_entry(cfg, kind) else {
        return index;
    };
    let Some(dir) = active_dir(ctx, cfg, kind) else {
        return index;
    };
    let Ok(read_dir) = fs::read_dir(&dir) else {
        return index;
    };
    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !valid_id(stem) || form_for_stem(&entry.prefix, stem).is_none() {
            continue;
        }
        if let Ok(Some(record)) = load_record_from_path(ctx, kind, &path) {
            index.records.push(record);
        }
    }
    index.records.sort_by(|a, b| a.id.cmp(&b.id));
    index.groups = index
        .records
        .iter()
        .filter(|r| r.form == Form::Group)
        .cloned()
        .collect();
    index.requirements = index
        .records
        .iter()
        .filter(|r| r.form == Form::Requirement)
        .cloned()
        .collect();

    let group_ids: BTreeMap<String, usize> = index
        .groups
        .iter()
        .enumerate()
        .map(|(position, group)| (group.id.clone(), position))
        .collect();
    for (position, requirement) in index.requirements.iter().enumerate() {
        match requirement
            .group_id
            .as_ref()
            .and_then(|id| group_ids.get(id))
        {
            Some(_) => {
                index
                    .members
                    .entry(requirement.group_id.clone().unwrap_or_default())
                    .or_default()
                    .push(position);
            }
            None => index.other.push(position),
        }
    }
    index
}

/// Resolve an ID by exact, case-sensitive match against the active directory
/// listings. Returns `Ok(None)` for an unknown, invalid, or deprecated ID.
pub fn resolve_id(ctx: &Context, cfg: &Config, id: &str) -> Result<Option<Resolved>, AppError> {
    if !valid_id(id) {
        return Ok(None);
    }
    let mut matches: Vec<(Category, PathBuf)> = Vec::new();
    for kind in [
        Category::Req,
        Category::Nfr,
        Category::Task,
        Category::Milestone,
        Category::Release,
    ] {
        let Some(entry) = active_entry(cfg, kind) else {
            continue;
        };
        let Some(dir) = active_dir(ctx, cfg, kind) else {
            continue;
        };
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if path.file_stem().and_then(|s| s.to_str()) != Some(id) {
                continue;
            }
            let valid = if kind.is_requirement() {
                form_for_stem(&entry.prefix, id).is_some()
            } else {
                id.strip_prefix(&format!("{}-", entry.prefix))
                    .map(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
                    .unwrap_or(false)
            };
            if !valid {
                continue;
            }
            // Reject an entry that escapes the project directory.
            if Context::ensure_within(&ctx.project_dir, &path).is_err() {
                continue;
            }
            matches.push((kind, path));
        }
    }
    match matches.len() {
        0 => Ok(None),
        1 => {
            let (kind, path) = matches.remove(0);
            let Some(record) = load_record_from_path(ctx, kind, &path)? else {
                // A symlink that escaped is treated as unknown; a non-UTF-8
                // record is an internal failure.
                let safe = Context::ensure_within(&ctx.project_dir, &path)
                    .map_err(|_| AppError::Runtime("record is not readable".to_string()))?;
                if fs::read_to_string(&safe).is_err() {
                    return Err(AppError::Runtime("record is not valid UTF-8".to_string()));
                }
                return Ok(None);
            };
            Ok(Some(Resolved { record, kind }))
        }
        _ => Err(AppError::Runtime(
            "record ID is ambiguous across document types".to_string(),
        )),
    }
}

/// Validate a title and return the stored value (specification section 7.3).
pub fn validate_title(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    let chars = value.chars().count();
    if chars == 0 {
        return Err("Title is required.".to_string());
    }
    if chars > MAX_TITLE_CHARS {
        return Err(format!(
            "Title must be at most {} characters.",
            MAX_TITLE_CHARS
        ));
    }
    if value.chars().any(|c| c.is_control()) {
        return Err("Title cannot contain control characters.".to_string());
    }
    Ok(value.to_string())
}

/// Validate a multi-line field and return the stored value, or `None` when the
/// value is empty (specification sections 7.3 and 7.4).
pub fn validate_multiline(label: &str, raw: &str) -> Result<Option<String>, String> {
    let normalized = normalize_newlines(raw);
    let value = normalized.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{} must be at most 20000 characters.", label));
    }
    if value
        .chars()
        .any(|c| c != '\n' && c != '\t' && c.is_control())
    {
        return Err(format!("{} cannot contain control characters.", label));
    }
    if value.lines().any(|line| line.starts_with("## ")) {
        return Err(format!("{} contains a line starting with ## .", label));
    }
    Ok(Some(value.to_string()))
}

/// A change to an editable multi-line field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldChange {
    Keep,
    Clear,
    Set(String),
}

fn trailing_empty_lines(body: &[String]) -> usize {
    body.iter().rev().take_while(|line| line.is_empty()).count()
}

fn remove_section(sections: &mut Vec<Section>, index: usize) {
    // Remove only the section itself. Neighbouring sections keep their exact
    // bodies, which preserves editor-added sections content-identically.
    sections.remove(index);
}

fn known_index(heading: &str, known: &[&str]) -> Option<usize> {
    known.iter().position(|h| *h == heading)
}

fn set_section(sections: &mut Vec<Section>, heading: &str, value: Option<String>, known: &[&str]) {
    let existing = sections.iter().position(|s| s.heading == heading);
    match (existing, value) {
        (Some(index), Some(value)) => {
            let trailing = trailing_empty_lines(&sections[index].body);
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            for _ in 0..trailing {
                body.push(String::new());
            }
            sections[index].body = body;
        }
        (Some(index), None) => remove_section(sections, index),
        (None, Some(value)) => {
            let order = known_index(heading, known).unwrap_or(usize::MAX);
            let mut position = None;
            for (index, section) in sections.iter().enumerate() {
                if let Some(section_order) = known_index(&section.heading, known)
                    && section_order < order
                {
                    position = Some(index + 1);
                }
            }
            if position.is_none() {
                for (index, section) in sections.iter().enumerate() {
                    if let Some(section_order) = known_index(&section.heading, known)
                        && section_order > order
                    {
                        position = Some(index);
                        break;
                    }
                }
            }
            let position = position.unwrap_or(sections.len());
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            if position < sections.len() {
                body.push(String::new());
            }
            sections.insert(
                position,
                Section {
                    heading: heading.to_string(),
                    body,
                },
            );
        }
        (None, None) => {}
    }
}

const REQ_KNOWN: &[&str] = &["## User Story", "## Group", "## Description"];
const TASK_KNOWN: &[&str] = &[
    "## Description",
    "## Status",
    "## Milestone",
    "## Assignee",
    "## Labels",
    "## Dependencies",
    "## Acceptance Criteria",
    "## Comments",
];
const MILESTONE_KNOWN: &[&str] = &["## Description", "## Release"];
const RELEASE_KNOWN: &[&str] = &["## Description"];

/// Serialize a document with a new title and optional story/description
/// changes, preserving every unknown `## ` section content-identically.
pub fn apply_edit(
    doc: &Doc,
    id: &str,
    title: &str,
    story: FieldChange,
    description: FieldChange,
    supports_story: bool,
) -> String {
    let mut sections = doc.sections.clone();
    if supports_story {
        match story {
            FieldChange::Keep => {}
            FieldChange::Clear => set_section(&mut sections, "## User Story", None, REQ_KNOWN),
            FieldChange::Set(value) => {
                set_section(&mut sections, "## User Story", Some(value), REQ_KNOWN);
            }
        }
    }
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, REQ_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), REQ_KNOWN)
        }
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# {} {}", id, title));
    lines.extend(doc.preamble.iter().cloned());
    for section in sections {
        lines.push(section.heading);
        lines.extend(section.body);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

/// Validate a list field (one value per line) and return the parsed items.
pub fn validate_list_items(label: &str, raw: &str) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    for line in normalize_newlines(raw).lines() {
        let value = line.trim();
        if value.is_empty() {
            continue;
        }
        let chars = value.chars().count();
        if chars == 0 || chars > MAX_TITLE_CHARS {
            return Err(format!("{} must be at most 200 characters.", label));
        }
        if value.chars().any(|c| c.is_control()) {
            return Err(format!("{} cannot contain control characters.", label));
        }
        if value.starts_with("## ") {
            return Err(format!("{} contains a line starting with ## .", label));
        }
        if !items.contains(&value.to_string()) {
            items.push(value.to_string());
        }
    }
    Ok(items)
}

/// Render a list field body.
fn render_list_items(items: &[String], prefix: &str) -> String {
    items
        .iter()
        .map(|v| format!("{} {}", prefix, v))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(clippy::too_many_arguments)]
pub fn apply_task_edit(
    doc: &Doc,
    id: &str,
    title: &str,
    description: FieldChange,
    status: &str,
    milestone: FieldChange,
    assignee: FieldChange,
    labels: Vec<String>,
    dependencies: Vec<String>,
    acceptance_criteria: Vec<String>,
) -> String {
    let mut sections = doc.sections.clone();
    // Status always set (required)
    set_section(
        &mut sections,
        "## Status",
        Some(status.to_string()),
        TASK_KNOWN,
    );
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), TASK_KNOWN)
        }
    }
    match milestone {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Milestone", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Milestone", Some(value), TASK_KNOWN)
        }
    }
    match assignee {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Assignee", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Assignee", Some(value), TASK_KNOWN)
        }
    }
    if labels.is_empty() {
        set_section(&mut sections, "## Labels", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Labels",
            Some(render_list_items(&labels, "-")),
            TASK_KNOWN,
        );
    }
    if dependencies.is_empty() {
        set_section(&mut sections, "## Dependencies", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Dependencies",
            Some(render_list_items(&dependencies, "-")),
            TASK_KNOWN,
        );
    }
    if acceptance_criteria.is_empty() {
        set_section(&mut sections, "## Acceptance Criteria", None, TASK_KNOWN);
    } else {
        let body = acceptance_criteria
            .iter()
            .map(|v| format!("- [ ] {}", v))
            .collect::<Vec<_>>()
            .join("\n");
        set_section(
            &mut sections,
            "## Acceptance Criteria",
            Some(body),
            TASK_KNOWN,
        );
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# {} {}", id, title));
    lines.extend(doc.preamble.iter().cloned());
    for section in sections {
        lines.push(section.heading);
        lines.extend(section.body);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

pub fn apply_milestone_edit(
    doc: &Doc,
    id: &str,
    title: &str,
    description: FieldChange,
    release: FieldChange,
) -> String {
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => set_section(
            &mut sections,
            "## Description",
            Some(value),
            MILESTONE_KNOWN,
        ),
    }
    match release {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Release", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Release", Some(value), MILESTONE_KNOWN)
        }
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# {} {}", id, title));
    lines.extend(doc.preamble.iter().cloned());
    for section in sections {
        lines.push(section.heading);
        lines.extend(section.body);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

pub fn apply_release_edit(doc: &Doc, id: &str, title: &str, description: FieldChange) -> String {
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, RELEASE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), RELEASE_KNOWN)
        }
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# {} {}", id, title));
    lines.extend(doc.preamble.iter().cloned());
    for section in sections {
        lines.push(section.heading);
        lines.extend(section.body);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

/// Write a record atomically in its own directory, preserving permissions and
/// removing the temporary file on every failure path.
pub fn write_atomic(ctx: &Context, target: &Path, content: &str) -> Result<(), AppError> {
    let safe_target = Context::ensure_within(&ctx.project_dir, target)
        .map_err(|_| AppError::NotFound("record not found".to_string()))?;
    let directory = safe_target
        .parent()
        .ok_or_else(|| AppError::Runtime("record directory is unavailable".to_string()))?
        .to_path_buf();
    let file_name = safe_target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::Runtime("record name is unavailable".to_string()))?
        .to_string();
    let original_permissions = fs::metadata(&safe_target)
        .map_err(|_| AppError::NotFound("record not found".to_string()))?
        .permissions();

    let mut temporary: Option<PathBuf> = None;
    let mut handle = None;
    for suffix in 0..1000u32 {
        let name = if suffix == 0 {
            format!("{}.tmp", file_name)
        } else {
            format!("{}.tmp.{}", file_name, suffix)
        };
        let candidate = directory.join(name);
        let safe_candidate = Context::ensure_within(&ctx.project_dir, &candidate)
            .map_err(|_| AppError::Runtime("temporary path is unavailable".to_string()))?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&safe_candidate)
        {
            Ok(file) => {
                temporary = Some(safe_candidate);
                handle = Some(file);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(AppError::Runtime(error.to_string())),
        }
    }
    let (Some(temporary), Some(mut handle)) = (temporary, handle) else {
        return Err(AppError::Runtime(
            "cannot create a temporary record file".to_string(),
        ));
    };

    let result = (|| -> Result<(), AppError> {
        handle
            .write_all(content.as_bytes())
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        handle
            .sync_all()
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        drop(handle);
        fs::set_permissions(&temporary, original_permissions)
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        fs::rename(&temporary, &safe_target)
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        if let Ok(directory_handle) = fs::File::open(&directory) {
            let _ = directory_handle.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_validation() {
        assert!(valid_id("REQ-001.001"));
        assert!(valid_id("NFR-01_2"));
        assert!(!valid_id(""));
        assert!(!valid_id(".hidden"));
        assert!(!valid_id("-x"));
        assert!(!valid_id("a/b"));
        assert!(!valid_id("a..b"));
        assert!(!valid_id("a b"));
        assert!(!valid_id(&"a".repeat(129)));
    }

    #[test]
    fn stem_forms() {
        assert_eq!(form_for_stem("REQ", "REQ-001"), Some(Form::Group));
        assert_eq!(form_for_stem("REQ", "REQ-001.001"), Some(Form::Requirement));
        assert_eq!(form_for_stem("REQ", "REQ-001.x"), None);
        assert_eq!(form_for_stem("REQ", "NFR-001"), None);
        assert_eq!(form_for_stem("REQ", "REQ-001.001.002"), None);
    }

    #[test]
    fn parse_and_preserve_unknown_sections() {
        let input = "# REQ-001.001 Title\n\n## User Story\nAs a user\n\n## Group\nREQ-001 Group\n\n## Criticality\nMust have\n\n## Description\nBody\n";
        let doc = parse_doc(input);
        assert_eq!(doc.id, "REQ-001.001");
        assert_eq!(doc.title, "Title");
        assert_eq!(doc.sections.len(), 4);
        let output = apply_edit(
            &doc,
            "REQ-001.001",
            "New title",
            FieldChange::Keep,
            FieldChange::Set("New body".to_string()),
            true,
        );
        assert!(output.starts_with("# REQ-001.001 New title\n"));
        assert!(output.contains("## Criticality\nMust have\n"));
        assert!(output.contains("## Description\nNew body\n"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn remove_and_insert_sections() {
        let doc = parse_doc("# REQ-001 Title\n\n## Description\nOld\n");
        let output = apply_edit(
            &doc,
            "REQ-001",
            "Title",
            FieldChange::Set("Story".to_string()),
            FieldChange::Clear,
            true,
        );
        assert!(output.contains("## User Story\nStory\n"));
        assert!(!output.contains("## Description"));
    }

    #[test]
    fn version_token_shape() {
        assert!(valid_version_token("1762000000.123456789.412"));
        assert!(!valid_version_token("1.2"));
        assert!(!valid_version_token("a.b.c"));
        assert!(!valid_version_token("1.2.3.4"));
    }

    #[test]
    fn title_and_multiline_validation() {
        assert!(validate_title("  Title  ").is_ok());
        assert!(validate_title("").is_err());
        assert!(validate_title(&"x".repeat(201)).is_err());
        assert!(validate_title("a\tb").is_err());
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert!(validate_multiline("Description", "a\n## Section\nb").is_err());
        assert!(validate_multiline("Description", "a\u{0007}b").is_err());
        assert!(validate_multiline("Description", "a\tb").is_ok());
    }
}
