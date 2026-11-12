//! Lazy, per-record migration of managed document record types to front matter.

use crate::context::Context;
use crate::error::AppError;
use crate::record::descriptor::RecordDescriptor;
use crate::record::document;
use crate::record::frontmatter;
use crate::record::frontmatter::FrontMatterValue;
use crate::record::patch::{self, PatchOp};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const GENERAL_KEYS: &[&str] = &[
    "id",
    "title",
    "created_date",
    "updated_date",
    "labels",
    "references",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationEntry {
    pub path: String,
    pub action: String,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationConflict {
    DuplicateId {
        id: String,
        paths: Vec<String>,
    },
    FrontmatterFilenameMismatch {
        path: String,
        filename_id: String,
        frontmatter_id: String,
    },
    UnknownId {
        path: String,
        id: String,
    },
}

impl std::fmt::Display for MigrationConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationConflict::DuplicateId { id, paths } => {
                write!(f, "duplicate id {}: {:?}", id, paths)
            }
            MigrationConflict::FrontmatterFilenameMismatch {
                path,
                filename_id,
                frontmatter_id,
            } => {
                write!(
                    f,
                    "front-matter/filename mismatch at {}: filename {} vs front matter {}",
                    path, filename_id, frontmatter_id
                )
            }
            MigrationConflict::UnknownId { path, id } => {
                write!(f, "unknown visible id {} at {}", id, path)
            }
        }
    }
}

pub fn is_managed_path(project_dir: &Path, path: &Path) -> bool {
    managed_descriptor(project_dir, path).is_some()
}

/// Resolve the descriptor of a managed record.
///
/// A path is managed when it sits directly in one of the descriptor's
/// configured read locations (active, archive, legacy alias, or deprecated)
/// and its stem is a native Papyrus stem for that family. Backlog-shaped
/// stems resolve to no descriptor and are therefore invisible.
pub fn managed_descriptor(project_dir: &Path, path: &Path) -> Option<&'static RecordDescriptor> {
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    let cfg = load_cfg_for(project_dir);
    let parent = path.parent()?;
    for desc in crate::record::descriptor::descriptors() {
        if !desc.matches_stem(&cfg, stem) {
            continue;
        }
        for dir in desc.resolve_read_directories(&cfg, project_dir) {
            if dir == parent {
                return Some(desc);
            }
        }
    }
    None
}

/// Canonical numeric key of a native stem for a descriptor family.
///
/// Dotted document IDs keep every numeric component, so `REQ-01.001` and
/// `REQ-01.002` remain distinct.
fn stem_key(desc: &RecordDescriptor, cfg: &crate::config::Config, stem: &str) -> Option<String> {
    desc.configured_prefixes(cfg)
        .iter()
        .find_map(|prefix| crate::record::identity::stem_key(stem, prefix, desc.is_dotted_id()))
}

/// Canonical numeric key of a Papyrus ID value for a descriptor family.
fn id_value_key(desc: &RecordDescriptor, cfg: &crate::config::Config, id: &str) -> Option<String> {
    desc.configured_prefixes(cfg).iter().find_map(|prefix| {
        crate::record::identity::parse_id_parts(id, prefix, desc.is_dotted_id())
            .map(|parts| parts.key(prefix))
    })
}

fn candidate_id_key(
    desc: &RecordDescriptor,
    cfg: &crate::config::Config,
    content: Option<&str>,
    stem: &str,
) -> Option<String> {
    if let Some(content) = content
        && let Some(fm_id) = candidate_frontmatter_id(desc, content)
        && let Some(key) = id_value_key(desc, cfg, &fm_id)
    {
        return Some(key);
    }
    stem_key(desc, cfg, stem)
}

fn matching_prefix_of(
    desc: &RecordDescriptor,
    cfg: &crate::config::Config,
    value: &str,
) -> Option<String> {
    desc.configured_prefixes(cfg).into_iter().find(|prefix| {
        crate::record::identity::parse_id_parts(value, prefix, desc.is_dotted_id()).is_some()
    })
}

pub fn has_front_matter(content: &str) -> bool {
    frontmatter::has_front_matter(content)
}

pub fn ensure_front_matter(ctx: &Context, path: &Path) -> Result<String, AppError> {
    ensure_front_matter_internal(ctx, path).map(|(content, _)| content)
}

/// Like [`ensure_front_matter`], but returns the record path unchanged.
///
/// Lazy adoption never relocates or renames a record, so the returned path is
/// always the path that was given.
pub fn ensure_front_matter_canonical(ctx: &Context, path: &Path) -> Result<PathBuf, AppError> {
    ensure_front_matter_internal(ctx, path).map(|(_, canonical)| canonical)
}

fn ensure_front_matter_internal(ctx: &Context, path: &Path) -> Result<(String, PathBuf), AppError> {
    let safe = crate::record::store::resolve_path(ctx, path).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            path.display()
        ))
    })?;
    if !safe.exists() {
        return Err(AppError::NotFound(format!(
            "record not found: {}",
            path.display()
        )));
    }
    let content = fs::read_to_string(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    let Some(desc) = managed_descriptor(&ctx.project_dir, &safe) else {
        return Ok((content, safe));
    };

    // Conflict detection applies to both shapes. A record that already
    // carries front matter is returned byte-identically: adoption never
    // completes missing keys and never retires sections on read.
    let conflicts = scan_migration_conflicts(ctx, desc);
    if let Some(reason) = blocking_conflict_for_path(ctx, &safe, desc, &content, &conflicts) {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            rel_path(&ctx.project_dir, &safe),
            reason
        )));
    }
    if frontmatter::has_front_matter(&content) {
        return Ok((content, safe));
    }

    let (new_content, changed, action, keys) = migrate_content(ctx, &safe, &content, false)?;
    if !changed {
        return Ok((content, safe));
    }
    crate::record::store::write_atomic(ctx, &safe, &new_content)?;
    let rel = rel_path(&ctx.project_dir, &safe);
    println!("{}\t{}\t{}", rel, action, keys.join(","));
    Ok((new_content, safe))
}

pub fn rel_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Adopt a managed record by adding its documented front-matter keys.
///
/// Adoption is front-matter-only for tasks and milestones: the body,
/// including any legacy heading or metadata section, is left byte-identical.
/// Retirement is a separate, write-only operation ([`retire_legacy_body_ops`]).
pub fn migrate_content(
    ctx: &Context,
    path: &Path,
    content: &str,
    _dry_run: bool,
) -> Result<(String, bool, String, Vec<String>), AppError> {
    let desc = match managed_descriptor(&ctx.project_dir, path) {
        Some(d) => d,
        None => return Ok((content.to_string(), false, String::new(), Vec::new())),
    };

    if frontmatter::has_front_matter(content) {
        return Ok((
            content.to_string(),
            false,
            "already-migrated".to_string(),
            existing_keys(content),
        ));
    }

    if desc.name == "task" || desc.name == "milestone" {
        let doc = document::parse(content)?;
        let ops = adoption_ops(ctx, path, desc, content, &doc)?;
        let new_content = patch::apply_patch(&doc, &ops)?;
        return Ok((
            new_content,
            true,
            "added-front-matter".to_string(),
            required_keys(desc)
                .iter()
                .map(|k| (*k).to_string())
                .collect(),
        ));
    }

    // Document records (TASK-68): general keys plus audit type retirement.
    let (id, title) = extract_id_title(content, path);
    let created = created_date(ctx, path, desc, content);
    let updated = frontmatter::utc_now();

    let mut keys: Vec<String> = GENERAL_KEYS.iter().map(|s| s.to_string()).collect();
    let mut ops: Vec<PatchOp> = vec![
        PatchOp::SetFrontMatter("id".into(), id),
        PatchOp::SetFrontMatter("title".into(), title),
        PatchOp::SetFrontMatter("created_date".into(), created),
        PatchOp::SetFrontMatter("updated_date".into(), updated),
        PatchOp::SetFrontMatter("labels".into(), "[]".into()),
        PatchOp::SetFrontMatter("references".into(), "[]".into()),
    ];

    let doc = document::parse(content)?;
    let mut action = "added-front-matter".to_string();
    let had_audit_type = doc.sections.iter().any(|s| s.heading == "## Audit Type");

    if desc.name == "audit record" {
        keys.push("type".to_string());
        let audit_type = doc
            .sections
            .iter()
            .find(|s| s.heading == "## Audit Type")
            .map(|s| s.body.join("\n").trim().to_string())
            .unwrap_or_default();
        ops.push(PatchOp::SetFrontMatter("type".into(), audit_type));
        ops.push(PatchOp::ClearSection("## Audit Type".into()));
        if had_audit_type {
            action = "added-front-matter;cleared-retired-section".to_string();
        }
    }

    let new_content = patch::apply_patch(&doc, &ops)?;
    Ok((new_content, true, action, keys))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SectionValue {
    Missing,
    Valid(String),
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SectionList {
    Missing,
    Valid(Vec<String>),
    Invalid,
}

fn find_section<'a>(doc: &'a document::Document, name: &str) -> Option<&'a document::Section> {
    let heading = format!("## {}", name);
    doc.sections.iter().find(|s| s.heading == heading)
}

/// Parse a scalar metadata section. More than one non-empty line cannot be
/// represented as a single front-matter value and is reported as invalid so
/// the caller can keep the section in place.
fn section_scalar(doc: &document::Document, name: &str) -> SectionValue {
    let Some(section) = find_section(doc, name) else {
        return SectionValue::Missing;
    };
    let lines: Vec<&str> = section
        .body
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    match lines.len() {
        0 => SectionValue::Valid(String::new()),
        1 => SectionValue::Valid(lines[0].to_string()),
        _ => SectionValue::Invalid,
    }
}

/// Parse a metadata section that stores one item per line. Empty list items
/// are invalid so the section is preserved rather than silently losing data.
fn section_list(doc: &document::Document, name: &str) -> SectionList {
    let Some(section) = find_section(doc, name) else {
        return SectionList::Missing;
    };
    let mut items = Vec::new();
    for raw in &section.body {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "-" {
            return SectionList::Invalid;
        }
        let item = if let Some(rest) = line.strip_prefix("- ") {
            rest.trim()
        } else {
            line
        };
        if item.is_empty() {
            return SectionList::Invalid;
        }
        items.push(item.to_string());
    }
    SectionList::Valid(items)
}

fn scalar_value(section: SectionValue) -> Option<String> {
    match section {
        SectionValue::Valid(value) => Some(value),
        _ => None,
    }
}

fn list_value(section: SectionList) -> Vec<String> {
    match section {
        SectionList::Valid(items) => items,
        _ => Vec::new(),
    }
}

fn list_ops_value(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        items.join("\n")
    }
}

fn null_or_string(value: Option<String>) -> FrontMatterValue {
    match value.filter(|v| !v.trim().is_empty()) {
        Some(v) => FrontMatterValue::String(v),
        None => FrontMatterValue::Null,
    }
}

fn heading_id_title(doc: &document::Document, path: &Path) -> (String, String) {
    if doc.heading.as_deref().is_some_and(|h| h.starts_with("# ")) {
        let (id, title) = document::heading_id_title(doc);
        if !id.is_empty() {
            return (id, title);
        }
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let id = stem.split_once(" - ").map_or(stem, |(base, _)| base).trim();
    (id.to_string(), String::new())
}

/// Build the full canonical front-matter key set for an adopted task or
/// milestone, deriving every value from the legacy body where present.
fn adoption_ops(
    ctx: &Context,
    path: &Path,
    desc: &RecordDescriptor,
    content: &str,
    doc: &document::Document,
) -> Result<Vec<PatchOp>, AppError> {
    let cfg = load_cfg(ctx);
    let (id, title) = heading_id_title(doc, path);
    let created = created_date(ctx, path, desc, content);
    let updated = frontmatter::utc_now();

    let mut ops = vec![
        PatchOp::SetFrontMatterValue("id".into(), FrontMatterValue::String(id)),
        PatchOp::SetFrontMatterValue("title".into(), FrontMatterValue::String(title)),
        PatchOp::SetFrontMatterValue("created_date".into(), FrontMatterValue::String(created)),
        PatchOp::SetFrontMatterValue("updated_date".into(), FrontMatterValue::String(updated)),
        PatchOp::SetFrontMatterValue(
            "labels".into(),
            FrontMatterValue::String(list_ops_value(&list_value(section_list(doc, "Labels")))),
        ),
        PatchOp::SetFrontMatterValue("references".into(), FrontMatterValue::String("[]".into())),
    ];

    if desc.name == "task" {
        let status = match section_scalar(doc, "Status") {
            SectionValue::Valid(value) if !value.is_empty() => value,
            _ => cfg.default_status.clone(),
        };
        let assignee = list_value(section_list(doc, "Assignee"));
        let milestone = null_or_string(scalar_value(section_scalar(doc, "Milestone")));
        let dependencies = list_value(section_list(doc, "Dependencies"));

        let task_desc = crate::record::descriptor::descriptor_for("task")
            .ok_or_else(|| AppError::Runtime("task descriptor is unavailable".to_string()))?;
        let dirs = task_desc.resolve_read_directories(&cfg, &ctx.project_dir);
        let dir_refs: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
        let ordinal = crate::record::identity::scan_max_ordinal_in_dirs(&dir_refs);

        ops.push(PatchOp::SetFrontMatterValue(
            "status".into(),
            FrontMatterValue::String(status),
        ));
        ops.push(PatchOp::SetFrontMatterValue(
            "assignee".into(),
            FrontMatterValue::String(list_ops_value(&assignee)),
        ));
        ops.push(PatchOp::SetFrontMatterValue("milestone".into(), milestone));
        ops.push(PatchOp::SetFrontMatterValue(
            "dependencies".into(),
            FrontMatterValue::String(list_ops_value(&dependencies)),
        ));
        ops.push(PatchOp::SetFrontMatterValue(
            "ordinal".into(),
            FrontMatterValue::Integer(ordinal),
        ));
        ops.push(PatchOp::SetFrontMatterValue(
            "documentation".into(),
            FrontMatterValue::String("[]".into()),
        ));
        ops.push(PatchOp::SetFrontMatterValue(
            "type".into(),
            FrontMatterValue::Null,
        ));
    } else {
        let release = null_or_string(scalar_value(section_scalar(doc, "Release")));
        ops.push(PatchOp::SetFrontMatterValue("release".into(), release));
    }

    Ok(ops)
}

/// Patch operations that retire legacy metadata sections and the title heading.
///
/// A section whose value cannot be parsed is left in place and a warning names
/// the record and section. When the key already exists in front matter, the
/// front-matter value wins and only the duplicate section is removed.
pub(crate) fn retire_legacy_body_ops(
    doc: &document::Document,
    desc: &RecordDescriptor,
    path: &Path,
) -> Vec<PatchOp> {
    let retired: &[(&str, &str, bool)] = match desc.name {
        "task" => &[
            ("Status", "status", false),
            ("Assignee", "assignee", true),
            ("Milestone", "milestone", false),
            ("Labels", "labels", true),
            ("Dependencies", "dependencies", true),
        ],
        "milestone" => &[("Release", "release", false)],
        _ => &[],
    };

    let mut ops = Vec::new();
    for (section_name, key, is_list) in retired {
        let heading = format!("## {}", section_name);
        let occurrences = doc.sections.iter().filter(|s| s.heading == heading).count();
        if occurrences == 0 {
            continue;
        }
        let key_present = doc
            .front_matter
            .as_ref()
            .is_some_and(|fm| frontmatter::contains_key(&fm.raw, key));

        let parsed = if *is_list {
            match section_list(doc, section_name) {
                SectionList::Missing => continue,
                SectionList::Invalid => {
                    eprintln!(
                        "warning: cannot retire section '{}' at {}: a list item is empty",
                        heading,
                        path.display()
                    );
                    continue;
                }
                SectionList::Valid(items) => Some(FrontMatterValue::String(list_ops_value(&items))),
            }
        } else {
            match section_scalar(doc, section_name) {
                SectionValue::Missing => continue,
                SectionValue::Invalid => {
                    eprintln!(
                        "warning: cannot retire section '{}' at {}: more than one non-empty line",
                        heading,
                        path.display()
                    );
                    continue;
                }
                SectionValue::Valid(value) => {
                    if *key == "milestone" || *key == "release" {
                        Some(null_or_string(Some(value)))
                    } else {
                        Some(FrontMatterValue::String(value))
                    }
                }
            }
        };

        if !key_present && let Some(value) = parsed {
            ops.push(PatchOp::SetFrontMatterValue((*key).to_string(), value));
        }
        for _ in 0..occurrences {
            ops.push(PatchOp::ClearSection(heading.clone()));
        }
    }

    if doc.heading.as_deref().is_some_and(|h| h.starts_with("# ")) {
        ops.push(PatchOp::ClearHeading);
    }
    ops
}

/// Whether a record that already carries front matter still has a retired
/// metadata section or a title heading.
pub(crate) fn has_retired_sections(doc: &document::Document, desc: &RecordDescriptor) -> bool {
    let retired: &[&str] = match desc.name {
        "task" => &["Status", "Assignee", "Milestone", "Labels", "Dependencies"],
        "milestone" => &["Release"],
        _ => &[],
    };
    if doc.heading.as_deref().is_some_and(|h| h.starts_with("# ")) {
        return true;
    }
    doc.sections.iter().any(|s| {
        retired
            .iter()
            .any(|name| s.heading == format!("## {}", name))
    })
}

fn existing_keys(content: &str) -> Vec<String> {
    if let Ok(Some(fm)) = frontmatter::parse(content) {
        // preserve order by scanning raw
        let mut keys = Vec::new();
        for line in fm.raw.lines() {
            if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
                continue;
            }
            if let Some((k, _)) = line.split_once(':') {
                let k = k.trim().to_string();
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        if keys.is_empty() {
            // fallback yaml mapping order not guaranteed; use raw scan already
        }
        return keys;
    }
    Vec::new()
}

fn extract_id_title(content: &str, path: &Path) -> (String, String) {
    // A legacy Papyrus file starts with a level-1 heading "# <id> <title>".
    // Storage-shaped and body-only files derive id/title from the filename
    // stem in the form "<prefix>-<n> - <slug>.md".
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("# ") {
            let rest = t.trim_start_matches('#').trim();
            let mut parts = rest.splitn(2, char::is_whitespace);
            let first = parts.next().unwrap_or("").trim();
            let title = parts.next().unwrap_or("").trim().to_string();
            if first.contains('-') && first.chars().any(|c| c.is_ascii_digit()) {
                return (first.to_string(), title);
            }
            return (id_from_filename(path), rest.to_string());
        }
        break;
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let (id, title) = if let Some((base, slug)) = stem.split_once(" - ") {
        (base.trim().to_string(), slug.trim().to_string())
    } else {
        (stem.to_string(), String::new())
    };
    if id.is_empty() {
        (id_from_filename(path), title)
    } else {
        (id, title)
    }
}

fn id_from_filename(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if stem.starts_with("ADR-") || stem.starts_with("BDR-") {
        // take first two segments
        let segs: Vec<&str> = stem.split('-').collect();
        if segs.len() >= 2 {
            return format!("{}-{}", segs[0], segs[1]);
        }
    }
    // if stem matches a managed id pattern exactly, use it
    stem
}

fn created_date(ctx: &Context, path: &Path, desc: &RecordDescriptor, content: &str) -> String {
    if desc.name == "decision"
        && let Ok(doc) = document::parse(content)
        && let Some(sec) = doc.sections.iter().find(|s| s.heading == "## Date")
    {
        let body = sec.body.join("\n");
        if let Some(ts) = parse_date_section(&body) {
            return ts;
        }
    }
    // file mtime
    if let Ok(meta) = fs::metadata(path)
        && let Ok(m) = meta.modified()
    {
        let secs = m
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        return frontmatter::utc_from_unix(secs);
    }
    let _ = ctx;
    frontmatter::utc_now()
}

fn parse_date_section(body: &str) -> Option<String> {
    let t = body.trim();
    if t.is_empty() {
        return None;
    }
    for token in t.split_whitespace() {
        let token = token.trim_matches(|c| c == '`' || c == '\'' || c == '"');
        if token.len() >= 10 {
            let datepart = &token[..10];
            if is_date(datepart) {
                if token.len() >= 20 && frontmatter::is_timestamp_pub(token) {
                    return Some(token.to_string());
                }
                return Some(format!("{}T00:00:00Z", datepart));
            }
        }
    }
    None
}

fn is_date(s: &str) -> bool {
    s.len() == 10
        && s.as_bytes()[4] == b'-'
        && s.as_bytes()[7] == b'-'
        && s[..4].chars().all(|c| c.is_ascii_digit())
        && s[5..7].chars().all(|c| c.is_ascii_digit())
        && s[8..10].chars().all(|c| c.is_ascii_digit())
}

// Conflict scanning helpers.

#[derive(Clone, Debug)]
struct Candidate {
    path: PathBuf,
    rel: String,
    frontmatter_id: Option<String>,
    frontmatter_key: Option<String>,
    stem_key: Option<String>,
    stem_prefix: Option<String>,
}

fn candidate_frontmatter_id(_desc: &RecordDescriptor, content: &str) -> Option<String> {
    if !frontmatter::has_front_matter(content) {
        return None;
    }
    let fm = frontmatter::parse(content).ok().flatten()?;
    let id = frontmatter::current_value_string(&fm.raw, "id")?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(id.to_string())
}

fn collect_candidates(ctx: &Context, desc: &RecordDescriptor) -> Vec<Candidate> {
    let mut out = Vec::new();
    let cfg = load_cfg(ctx);
    for dir in desc.resolve_read_directories(&cfg, &ctx.project_dir) {
        collect_candidates_dir(ctx, desc, &cfg, &dir, &mut out);
    }
    out
}

fn collect_candidates_dir(
    ctx: &Context,
    desc: &RecordDescriptor,
    cfg: &crate::config::Config,
    dir: &Path,
    out: &mut Vec<Candidate>,
) {
    if !dir.exists() {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let path = e.path();
        if path.is_dir() {
            collect_candidates_dir(ctx, desc, cfg, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if !is_managed_path(&ctx.project_dir, &path) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let rel = path
            .strip_prefix(&ctx.project_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&path).unwrap_or_default();
        let fm_id = candidate_frontmatter_id(desc, &content);
        let fm_key = fm_id
            .as_deref()
            .and_then(|value| id_value_key(desc, cfg, value));
        let number = stem_key(desc, cfg, stem);
        let prefix = matching_prefix_of(desc, cfg, stem);
        out.push(Candidate {
            path,
            rel,
            frontmatter_id: fm_id,
            frontmatter_key: fm_key,
            stem_key: number,
            stem_prefix: prefix,
        });
    }
}

pub fn scan_migration_conflicts(ctx: &Context, desc: &RecordDescriptor) -> Vec<MigrationConflict> {
    let cfg = load_cfg(ctx);
    let candidates = collect_candidates(ctx, desc);
    // Deduplicate by canonical path; read directories never overlap, but a
    // symlink could still surface the same file twice.
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let candidates: Vec<Candidate> = candidates
        .into_iter()
        .filter(|c| {
            let canon = c.path.canonicalize().unwrap_or_else(|_| c.path.clone());
            seen.insert(canon)
        })
        .collect();
    let mut conflicts = Vec::new();

    // Front-matter / stem mismatch. Numbers are compared after normalization,
    // so optional leading zeros and case variants are not mismatches.
    for c in &candidates {
        if let (Some(fm), Some(stem_key), Some(stem_prefix)) =
            (&c.frontmatter_id, c.stem_key.as_ref(), &c.stem_prefix)
        {
            let same = c.frontmatter_key.as_ref() == Some(stem_key);
            if !same {
                let stem = c
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let filename_id = crate::record::identity::canonical_stem_id(
                    stem,
                    stem_prefix,
                    desc.is_dotted_id(),
                )
                .unwrap_or_else(|| stem.to_string());
                conflicts.push(MigrationConflict::FrontmatterFilenameMismatch {
                    path: c.rel.clone(),
                    filename_id,
                    frontmatter_id: fm.clone(),
                });
            }
        }
    }

    // Duplicate IDs across every read location of the family.
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in &candidates {
        let content = fs::read_to_string(&c.path).unwrap_or_default();
        let stem = c
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if let Some(key) = candidate_id_key(desc, &cfg, Some(&content), stem) {
            groups.entry(key).or_default().push(c.rel.clone());
        }
    }
    for (id, paths) in groups {
        if paths.len() > 1 {
            conflicts.push(MigrationConflict::DuplicateId { id, paths });
        }
    }

    // Unknown IDs: files whose stem or front-matter id cannot be parsed.
    for c in &candidates {
        if c.frontmatter_id.is_none() && c.stem_key.is_none() {
            let raw = c
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            conflicts.push(MigrationConflict::UnknownId {
                path: c.rel.clone(),
                id: raw,
            });
        }
    }

    conflicts
}

pub(crate) fn check_conflicts_for_path(
    ctx: &Context,
    path: &Path,
    desc: &RecordDescriptor,
    content: &str,
    conflicts: &[MigrationConflict],
) -> Option<String> {
    let cfg = load_cfg(ctx);
    let rel = rel_path(&ctx.project_dir, path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if let (Some(fm_id), Some(stem_key), Some(stem_prefix)) = (
        candidate_frontmatter_id(desc, content),
        stem_key(desc, &cfg, stem),
        matching_prefix_of(desc, &cfg, stem),
    ) {
        let fm_key = id_value_key(desc, &cfg, &fm_id);
        let same = fm_key.as_ref() == Some(&stem_key);
        if !same {
            let filename_id =
                crate::record::identity::canonical_stem_id(stem, &stem_prefix, desc.is_dotted_id())
                    .unwrap_or_else(|| stem.to_string());
            return Some(format!(
                "front-matter/filename mismatch: filename {} vs front matter {}",
                filename_id, fm_id
            ));
        }
    }

    let sid = candidate_id_key(desc, &cfg, Some(content), stem);
    for c in conflicts {
        match c {
            MigrationConflict::DuplicateId { id, paths } => {
                if Some(id.clone()) == sid && paths.iter().any(|p| p == &rel) {
                    return Some(c.to_string());
                }
            }
            MigrationConflict::FrontmatterFilenameMismatch {
                path: p,
                filename_id,
                frontmatter_id,
            } => {
                if p == &rel {
                    return Some(format!(
                        "front-matter/filename mismatch: filename {} vs front matter {}",
                        filename_id, frontmatter_id
                    ));
                }
            }
            MigrationConflict::UnknownId { path: p, id } => {
                if p == &rel {
                    return Some(format!("unknown id: {}", id));
                }
            }
        }
    }
    None
}

/// Conflicts that must block a read or write of a single record. A
/// front-matter/filename mismatch is intentionally not blocking on the read
/// path: the front-matter ID is canonical.
pub(crate) fn blocking_conflict_for_path(
    ctx: &Context,
    path: &Path,
    _desc: &RecordDescriptor,
    _content: &str,
    conflicts: &[MigrationConflict],
) -> Option<String> {
    let rel = rel_path(&ctx.project_dir, path);
    for c in conflicts {
        match c {
            MigrationConflict::DuplicateId { id: _, paths } => {
                if paths.iter().any(|p| p == &rel) {
                    return Some(c.to_string());
                }
            }
            MigrationConflict::FrontmatterFilenameMismatch { .. } => {
                // Non-blocking on read; writes check separately.
            }
            MigrationConflict::UnknownId { path: p, id } => {
                if p == &rel {
                    return Some(format!("unknown id: {}", id));
                }
            }
        }
    }
    None
}

/// Return the migration conflict that involves `path`, if any. Used by list
/// commands to flag invalid records without silently listing duplicates.
pub fn path_migration_conflict(
    ctx: &Context,
    path: &Path,
    _desc: &RecordDescriptor,
    _content: &str,
    conflicts: &[MigrationConflict],
) -> Option<MigrationConflict> {
    let rel = rel_path(&ctx.project_dir, path);
    conflicts
        .iter()
        .find(|c| match c {
            MigrationConflict::DuplicateId { paths, .. } => paths.iter().any(|p| p == &rel),
            MigrationConflict::FrontmatterFilenameMismatch { path: p, .. } => p == &rel,
            MigrationConflict::UnknownId { path: p, .. } => p == &rel,
        })
        .cloned()
}

// Dry-run traversal and migration helpers.
pub fn migrate_corpus(ctx: &Context, dry_run: bool) -> Result<Vec<MigrationEntry>, AppError> {
    let mut entries = Vec::new();
    let cfg = load_cfg(ctx);
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for desc in crate::record::descriptor::descriptors() {
        for dir in desc.resolve_read_directories(&cfg, &ctx.project_dir) {
            let canon = dir.canonicalize().unwrap_or_else(|_| dir.clone());
            if !seen.insert(canon) || !dir.exists() {
                continue;
            }
            collect_dir(ctx, &dir, dry_run, &mut entries)?;
        }
    }
    let verbose = dry_run;
    for e in &entries {
        if !verbose && e.action == "already-migrated" {
            continue;
        }
        println!("{}\t{}\t{}", e.path, e.action, e.keys.join(","));
    }
    Ok(entries)
}

fn collect_dir(
    ctx: &Context,
    dir: &Path,
    dry_run: bool,
    entries: &mut Vec<MigrationEntry>,
) -> Result<(), AppError> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| AppError::Runtime(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir(ctx, &path, dry_run, entries)?;
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if !is_managed_path(&ctx.project_dir, &path) {
            continue;
        }
        process_file(ctx, &path, dry_run, entries)?;
    }
    Ok(())
}

fn process_file(
    ctx: &Context,
    path: &Path,
    dry_run: bool,
    entries: &mut Vec<MigrationEntry>,
) -> Result<(), AppError> {
    let content = fs::read_to_string(path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let rel = rel_path(&ctx.project_dir, path);
    let desc = managed_descriptor(&ctx.project_dir, path);
    if dry_run {
        if frontmatter::has_front_matter(&content) {
            let (action, keys) = match desc {
                Some(desc) => {
                    let retired = document::parse(&content)
                        .map(|doc| has_retired_sections(&doc, desc))
                        .unwrap_or(false);
                    if retired {
                        ("would-retire-legacy-sections", existing_keys(&content))
                    } else {
                        ("already-migrated", existing_keys(&content))
                    }
                }
                None => ("already-migrated", existing_keys(&content)),
            };
            entries.push(MigrationEntry {
                path: rel,
                action: action.to_string(),
                keys,
            });
        } else {
            let keys = match desc {
                Some(desc) => required_keys(desc)
                    .iter()
                    .map(|k| (*k).to_string())
                    .collect(),
                None => Vec::new(),
            };
            entries.push(MigrationEntry {
                path: rel,
                action: "would-migrate".to_string(),
                keys,
            });
        }
        return Ok(());
    }
    if frontmatter::has_front_matter(&content) {
        return Ok(());
    }
    if desc.is_none() {
        return Ok(());
    }
    let (new_content, changed, action, keys) = migrate_content(ctx, path, &content, false)?;
    if !changed {
        return Ok(());
    }
    crate::record::store::write_atomic(ctx, path, &new_content)?;
    entries.push(MigrationEntry {
        path: rel,
        action,
        keys,
    });
    Ok(())
}

pub fn migrate_file(ctx: &Context, path: &Path) -> Result<Option<MigrationEntry>, AppError> {
    let safe = crate::record::store::resolve_path(ctx, path)?;
    if !safe.exists() {
        return Err(AppError::NotFound(format!(
            "record not found: {}",
            path.display()
        )));
    }
    let content = fs::read_to_string(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    if frontmatter::has_front_matter(&content) {
        return Ok(None);
    }
    if managed_descriptor(&ctx.project_dir, &safe).is_none() {
        return Ok(None);
    }
    let (new_content, changed, action, keys) = migrate_content(ctx, &safe, &content, false)?;
    if !changed {
        return Ok(None);
    }
    crate::record::store::write_atomic(ctx, &safe, &new_content)?;
    let rel = rel_path(&ctx.project_dir, &safe);
    println!("{}\t{}\t{}", rel, action, keys.join(","));
    Ok(Some(MigrationEntry {
        path: rel,
        action,
        keys,
    }))
}

// Expose is_timestamp for date parsing checks.
pub fn is_timestamp_pub(v: &str) -> bool {
    // reuse frontmatter detection via length check
    v.len() == 20
        && v.as_bytes()[4] == b'-'
        && v.as_bytes()[7] == b'-'
        && v.as_bytes()[10] == b'T'
        && v.as_bytes()[13] == b':'
        && v.as_bytes()[16] == b':'
        && v.as_bytes()[19] == b'Z'
}

fn load_cfg(ctx: &Context) -> crate::config::Config {
    crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir).unwrap_or_default()
}

fn load_cfg_for(project_dir: &Path) -> crate::config::Config {
    let config_path = project_dir.join("papyrus.config.yml");
    crate::commands::load_effective_config(&config_path, project_dir).unwrap_or_default()
}

fn required_keys(desc: &RecordDescriptor) -> &'static [&'static str] {
    match desc.name {
        "task" => &[
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
            "status",
            "assignee",
            "milestone",
            "dependencies",
            "ordinal",
            "documentation",
            "type",
        ],
        "milestone" => &[
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
            "release",
        ],
        "audit record" => &[
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
            "type",
        ],
        _ => &[
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("migrate-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn ctx(dir: &Path) -> Context {
        Context {
            project_dir: dir.to_path_buf(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        }
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn task_record(body: &str) -> String {
        format!(
            "---\nid: TASK-001\ntitle: Title\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: Done\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n{}",
            body
        )
    }

    #[test]
    fn read_adoption_adds_front_matter_and_keeps_body_byte_identical() {
        let dir = tmpdir("read-adoption");
        let c = ctx(&dir);
        let legacy = "# TASK-001 Old Title\n\n## Status\nDone\n\n## Description\nLegacy body.\n\n## Acceptance Criteria\n- [ ] first thing\n";
        let path = dir.join("backlog/tasks/TASK-001.md");
        write(&path, legacy);
        let result = ensure_front_matter(&c, &path).unwrap();
        let (fm, body) = frontmatter::split(&result).unwrap();
        let fm = fm.expect("front matter expected");
        assert_eq!(body, legacy, "body must stay byte-identical on read");
        assert!(fm.raw.contains("id: TASK-001"));
        assert!(fm.raw.contains("title: \"Old Title\""));
        assert!(fm.raw.contains("status: Done"));
        assert!(fm.raw.contains("assignee: []"));
        assert!(fm.raw.contains("milestone: null"));
        assert!(fm.raw.contains("documentation: []"));
        assert!(fm.raw.contains("type: null"));
        assert!(
            result.contains("# TASK-001 Old Title"),
            "heading kept on read"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), result);
    }

    #[test]
    fn read_adoption_derives_sections_into_front_matter() {
        let dir = tmpdir("derived");
        let c = ctx(&dir);
        write(&dir.join("backlog/tasks/TASK-002.md"), &task_record(""));
        let legacy = "# TASK-003 Derived\n\n## Status\nIn Progress\n\n## Assignee\n- alice\n- bob\n\n## Milestone\nMS-001\n\n## Labels\n- bug\n\n## Dependencies\n- TASK-002\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-003.md");
        write(&path, legacy);
        let result = ensure_front_matter(&c, &path).unwrap();
        let (fm, _) = frontmatter::split(&result).unwrap();
        let fm = fm.unwrap();
        assert!(fm.raw.contains("id: TASK-003"), "{}", fm.raw);
        assert!(fm.raw.contains("title: Derived"), "{}", fm.raw);
        assert!(fm.raw.contains("status: \"In Progress\""), "{}", fm.raw);
        assert!(
            fm.raw.contains("assignee:\n  - alice\n  - bob"),
            "{}",
            fm.raw
        );
        assert!(fm.raw.contains("milestone: MS-001"), "{}", fm.raw);
        assert!(fm.raw.contains("labels:\n  - bug"), "{}", fm.raw);
        assert!(fm.raw.contains("dependencies:\n  - TASK-002"), "{}", fm.raw);
        assert!(fm.raw.contains("ordinal:"), "{}", fm.raw);
        assert!(fm.raw.contains("created_date:"), "{}", fm.raw);
        assert!(fm.raw.contains("updated_date:"), "{}", fm.raw);
    }

    #[test]
    fn read_adoption_uses_default_status_when_absent() {
        let dir = tmpdir("default-status");
        let c = ctx(&dir);
        let path = dir.join("backlog/tasks/TASK-004.md");
        write(
            &path,
            "# TASK-004 Body Only\n\n## Description\nNo status.\n",
        );
        let result = ensure_front_matter(&c, &path).unwrap();
        let (fm, _) = frontmatter::split(&result).unwrap();
        assert!(
            fm.unwrap().raw.contains("status: ToDo"),
            "default status expected"
        );
    }

    #[test]
    fn read_adoption_derives_milestone_release() {
        let dir = tmpdir("adopt-milestone");
        let c = ctx(&dir);
        let legacy = "# MS-001 Mile\n\n## Release\nREL-001\n\n## Description\nDesc.\n";
        let path = dir.join("backlog/milestones/MS-001.md");
        write(&path, legacy);
        let result = ensure_front_matter(&c, &path).unwrap();
        let (fm, body) = frontmatter::split(&result).unwrap();
        assert!(fm.unwrap().raw.contains("release: REL-001"));
        assert_eq!(body, legacy);
    }

    #[test]
    fn partial_front_matter_is_not_completed_on_read() {
        let dir = tmpdir("partial");
        let c = ctx(&dir);
        let body = "## Description\n\nBody\n";
        let minimal = format!("---\nid: TASK-006\ntitle: Interop\n---\n{}", body);
        let path = dir.join("backlog/tasks/TASK-006.md");
        write(&path, &minimal);
        let result = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(result, minimal);
        assert_eq!(fs::read_to_string(&path).unwrap(), minimal);
    }

    #[test]
    fn adoption_is_idempotent() {
        let dir = tmpdir("idempotent");
        let c = ctx(&dir);
        let legacy = "# TASK-002 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-002.md");
        write(&path, legacy);
        let first = ensure_front_matter(&c, &path).unwrap();
        let first_bytes = fs::read_to_string(&path).unwrap();
        let second = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(first, second);
        assert_eq!(fs::read_to_string(&path).unwrap(), first_bytes);
    }

    #[test]
    fn retirement_removes_heading_and_metadata_sections() {
        let dir = tmpdir("retire");
        let content = "---\nid: TASK-001\ntitle: Title\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\n---\n\n# TASK-001 Title\n\n## Status\nDone\n\n## Assignee\n- alice\n\n## Milestone\nMS-001\n\n## Labels\n- bug\n\n## Dependencies\n- TASK-002\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nKeep me\n<!-- SECTION:DESCRIPTION:END -->\n";
        let path = dir.join("backlog/tasks/TASK-001.md");
        let doc = document::parse(content).unwrap();
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let ops = retire_legacy_body_ops(&doc, desc, &path);
        let out = patch::apply_patch(&doc, &ops).unwrap();
        assert!(!out.contains("# TASK-001 Title"), "{}", out);
        assert!(!out.contains("## Status"), "{}", out);
        assert!(!out.contains("## Assignee"), "{}", out);
        assert!(!out.contains("## Milestone"), "{}", out);
        assert!(!out.contains("## Labels"), "{}", out);
        assert!(!out.contains("## Dependencies"), "{}", out);
        assert!(out.contains("status: Done"), "{}", out);
        assert!(out.contains("assignee:\n  - alice"), "{}", out);
        assert!(out.contains("milestone: MS-001"), "{}", out);
        assert!(out.contains("labels:\n  - bug"), "{}", out);
        assert!(out.contains("dependencies:\n  - TASK-002"), "{}", out);
        assert!(out.contains("Keep me"), "{}", out);
        assert!(
            out.contains("<!-- SECTION:DESCRIPTION:BEGIN -->"),
            "{}",
            out
        );
    }

    #[test]
    fn retirement_removes_milestone_release_section() {
        let dir = tmpdir("retire-milestone");
        let content = "---\nid: MS-001\ntitle: Mile\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\n---\n\n# MS-001 Mile\n\n## Release\nREL-001\n\n## Description\nBody\n";
        let path = dir.join("backlog/milestones/MS-001.md");
        let doc = document::parse(content).unwrap();
        let desc = crate::record::descriptor::descriptor_for("milestone").unwrap();
        let ops = retire_legacy_body_ops(&doc, desc, &path);
        let out = patch::apply_patch(&doc, &ops).unwrap();
        assert!(!out.contains("# MS-001 Mile"));
        assert!(!out.contains("## Release"));
        assert!(out.contains("release: REL-001"));
    }

    #[test]
    fn front_matter_value_wins_over_duplicate_section() {
        let dir = tmpdir("fm-wins");
        let content =
            task_record("\n# TASK-001 Title\n\n## Status\nIn Progress\n\n## Description\nBody\n");
        let path = dir.join("backlog/tasks/TASK-001.md");
        let doc = document::parse(&content).unwrap();
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let ops = retire_legacy_body_ops(&doc, desc, &path);
        let out = patch::apply_patch(&doc, &ops).unwrap();
        assert!(!out.contains("## Status"), "{}", out);
        assert!(out.contains("status: Done"), "{}", out);
        assert!(!out.contains("In Progress"), "{}", out);
    }

    #[test]
    fn unparseable_section_is_kept_and_others_retired() {
        let dir = tmpdir("keep-section");
        let content = task_record(
            "\n# TASK-001 Title\n\n## Status\nLine one\nLine two\n\n## Description\nBody\n",
        );
        let path = dir.join("backlog/tasks/TASK-001.md");
        let doc = document::parse(&content).unwrap();
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let ops = retire_legacy_body_ops(&doc, desc, &path);
        let out = patch::apply_patch(&doc, &ops).unwrap();
        assert!(!out.contains("# TASK-001 Title"), "{}", out);
        assert!(out.contains("## Status\nLine one\nLine two"), "{}", out);
    }

    #[test]
    fn unparseable_list_item_is_kept() {
        let dir = tmpdir("keep-list");
        let content = task_record("\n# TASK-001 Title\n\n## Labels\n- \n\n## Description\nBody\n");
        let path = dir.join("backlog/tasks/TASK-001.md");
        let doc = document::parse(&content).unwrap();
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let ops = retire_legacy_body_ops(&doc, desc, &path);
        let out = patch::apply_patch(&doc, &ops).unwrap();
        assert!(out.contains("## Labels"), "{}", out);
    }

    #[test]
    fn duplicate_ids_are_detected() {
        let dir = tmpdir("duplicate");
        let c = ctx(&dir);
        let a = dir.join("backlog/tasks/TASK-007.md");
        let b = dir.join("backlog/tasks/TASK-0007.md");
        write(
            &a,
            &task_record("\n## Description\nA\n").replace("TASK-001", "TASK-007"),
        );
        write(
            &b,
            &task_record("\n## Description\nB\n").replace("TASK-001", "TASK-0007"),
        );
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert!(conflicts.iter().any(|c| matches!(
            c,
            MigrationConflict::DuplicateId { id, paths }
            if id == "TASK-7" && paths.len() == 2
        )));
    }

    #[test]
    fn same_title_distinct_numbers_are_not_conflicts() {
        let dir = tmpdir("same-title-distinct");
        let c = ctx(&dir);
        let tpl = "---\nid: TASK-{n}\ntitle: SameTitle\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: {n}\ndocumentation: []\ntype: null\n---\n\n## Description\n\nBody\n";
        write(
            &dir.join("backlog/tasks/TASK-008.md"),
            &tpl.replace("{n}", "8"),
        );
        write(
            &dir.join("backlog/tasks/TASK-009.md"),
            &tpl.replace("{n}", "9"),
        );
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert!(conflicts.is_empty(), "{:?}", conflicts);
        assert!(ensure_front_matter(&c, &dir.join("backlog/tasks/TASK-008.md")).is_ok());
        assert!(ensure_front_matter(&c, &dir.join("backlog/tasks/TASK-009.md")).is_ok());
    }

    #[test]
    fn frontmatter_filename_mismatch_detected() {
        let dir = tmpdir("mismatch");
        let c = ctx(&dir);
        let content = task_record("\n## Description\nBody\n").replace("TASK-001", "TASK-99");
        let path = dir.join("backlog/tasks/TASK-005.md");
        write(&path, &content);
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert!(conflicts.iter().any(|c| matches!(
            c,
            MigrationConflict::FrontmatterFilenameMismatch { path: p, .. }
            if p.contains("TASK-005")
        )));
    }

    #[test]
    fn read_path_write_failure_keeps_original() {
        let dir = tmpdir("readonly");
        let c = ctx(&dir);
        let legacy = "# TASK-003 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-003.md");
        write(&path, legacy);
        let tasks_dir = dir.join("backlog/tasks");
        let original_mode = fs::metadata(&tasks_dir).unwrap().permissions().mode();
        let mut perms = fs::metadata(&tasks_dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&tasks_dir, perms).unwrap();
        let result = ensure_front_matter(&c, &path);
        fs::set_permissions(&tasks_dir, fs::Permissions::from_mode(original_mode)).unwrap();
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
    }

    #[test]
    fn legacy_archive_is_read_in_place_not_relocated() {
        let dir = tmpdir("legacy-archive");
        let c = ctx(&dir);
        let legacy = "# TASK-004 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/archived/TASK-004.md");
        write(&path, legacy);
        let canonical = ensure_front_matter_canonical(&c, &path).unwrap();
        assert_eq!(canonical, path, "read must not relocate the record");
        assert!(path.exists());
    }

    #[test]
    fn backlog_shaped_stem_is_not_managed() {
        let dir = tmpdir("backlog-shaped");
        let c = ctx(&dir);
        let content = "## Description\n\nBody line\n";
        let path = dir.join("backlog/tasks/task-5 - Parsed-Title.md");
        write(&path, content);
        assert!(managed_descriptor(&dir, &path).is_none());
        let result = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(result, content);
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn native_task_is_left_byte_identical() {
        let dir = tmpdir("native-task");
        let c = ctx(&dir);
        let content = task_record(
            "\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody line\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        let path = dir.join("backlog/tasks/TASK-005.md");
        write(&path, &content);
        let result = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(result, content);
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn dry_run_reports_would_migrate_and_would_retire_without_writing() {
        let dir = tmpdir("dryrun");
        let c = ctx(&dir);
        let legacy = "# TASK-005 Title\n\n## Description\nBody\n";
        let retired =
            task_record("\n# TASK-006 Title\n\n## Status\nDone\n\n## Description\nBody\n");
        let clean = task_record("\n## Description\nBody\n");
        write(&dir.join("backlog/tasks/TASK-005.md"), legacy);
        write(&dir.join("backlog/tasks/TASK-006.md"), &retired);
        write(&dir.join("backlog/tasks/TASK-007.md"), &clean);

        let entries = migrate_corpus(&c, true).unwrap();
        let find = |suffix: &str| {
            entries
                .iter()
                .find(|e| e.path.ends_with(suffix))
                .map(|e| e.action.as_str())
                .unwrap_or("")
        };
        assert_eq!(find("TASK-005.md"), "would-migrate");
        assert_eq!(find("TASK-006.md"), "would-retire-legacy-sections");
        assert_eq!(find("TASK-007.md"), "already-migrated");

        assert_eq!(
            fs::read_to_string(dir.join("backlog/tasks/TASK-005.md")).unwrap(),
            legacy
        );
        assert_eq!(
            fs::read_to_string(dir.join("backlog/tasks/TASK-006.md")).unwrap(),
            retired
        );
        assert_eq!(
            fs::read_to_string(dir.join("backlog/tasks/TASK-007.md")).unwrap(),
            clean
        );
    }

    #[test]
    fn migrate_file_reports_added_front_matter() {
        let dir = tmpdir("migrate-file");
        let c = ctx(&dir);
        let legacy = "# TASK-010 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-010.md");
        write(&path, legacy);
        let entry = migrate_file(&c, &path).unwrap().expect("entry expected");
        assert_eq!(entry.action, "added-front-matter");
        assert!(entry.keys.iter().any(|k| k == "status"));
        assert!(fs::read_to_string(&path).unwrap().starts_with("---\n"));
    }

    #[test]
    fn conflict_check_fails_for_front_mattered_duplicate() {
        let dir = tmpdir("frontmatter-conflict");
        let c = ctx(&dir);
        let a = dir.join("backlog/tasks/TASK-007.md");
        let b = dir.join("backlog/tasks/TASK-0007.md");
        write(
            &a,
            &task_record("\n## Description\nA\n").replace("TASK-001", "TASK-007"),
        );
        write(
            &b,
            &task_record("\n## Description\nB\n").replace("TASK-001", "TASK-0007"),
        );
        let err = ensure_front_matter(&c, &a).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate id"), "{}", msg);
        assert!(msg.contains("TASK-7"), "{}", msg);
    }
}
