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
    pub diff: Option<String>,
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

/// Parse the number of a native stem for a descriptor family.
fn stem_number(desc: &RecordDescriptor, cfg: &crate::config::Config, stem: &str) -> Option<i64> {
    desc.configured_prefixes(cfg)
        .iter()
        .find_map(|prefix| crate::record::identity::id_number(stem, prefix).map(i64::from))
}

/// Parse the number of a Papyrus ID value for a descriptor family.
fn id_value_number(desc: &RecordDescriptor, cfg: &crate::config::Config, id: &str) -> Option<i64> {
    desc.configured_prefixes(cfg).iter().find_map(|prefix| {
        crate::record::identity::IdSpec::new(prefix.clone(), 1)
            .parse_id(id)
            .ok()
    })
}

/// Canonical key for duplicate detection: the configured prefix plus the
/// unpadded number, so `TASK-1`, `TASK-001`, and `task-001` collide.
fn id_key(prefix: &str, number: i64) -> String {
    format!("{}-{}", prefix.to_ascii_uppercase(), number)
}

fn candidate_id_key(
    desc: &RecordDescriptor,
    cfg: &crate::config::Config,
    content: Option<&str>,
    stem: &str,
) -> Option<String> {
    if let Some(content) = content
        && let Some(fm_id) = candidate_frontmatter_id(desc, content)
    {
        for prefix in desc.configured_prefixes(cfg) {
            if let Ok(number) =
                crate::record::identity::IdSpec::new(prefix.clone(), 1).parse_id(&fm_id)
            {
                return Some(id_key(&prefix, number));
            }
        }
    }
    for prefix in desc.configured_prefixes(cfg) {
        if let Some(number) = crate::record::identity::id_number(stem, &prefix) {
            return Some(id_key(&prefix, i64::from(number)));
        }
    }
    None
}

fn matching_prefix_of(
    desc: &RecordDescriptor,
    cfg: &crate::config::Config,
    value: &str,
) -> Option<String> {
    desc.configured_prefixes(cfg)
        .into_iter()
        .find(|prefix| crate::record::identity::id_number(value, prefix).is_some())
}

pub fn has_front_matter(content: &str) -> bool {
    frontmatter::has_front_matter(content)
}

pub fn ensure_front_matter(ctx: &Context, path: &Path) -> Result<String, AppError> {
    ensure_front_matter_internal(ctx, path).map(|(content, _)| content)
}

/// Like [`ensure_front_matter`], but returns the canonical path of the record
/// after any migration or relocation. Callers that need to open the file by
/// path after a first-read migration should use this helper.
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
    if frontmatter::has_front_matter(&content) {
        if let Some(desc) = managed_descriptor(&ctx.project_dir, &safe) {
            // Conflict detection for already-front-mattered records (covers
            // list/view/write paths that read through this helper).
            let conflicts = scan_migration_conflicts(ctx, desc);
            if let Some(reason) = blocking_conflict_for_path(ctx, &safe, desc, &content, &conflicts)
            {
                return Err(AppError::Validation(format!(
                    "migration conflict at {}: {}",
                    rel_path(&ctx.project_dir, &safe),
                    reason
                )));
            }

            let required = required_keys(desc);
            let keys = existing_keys(&content);
            let missing: Vec<&str> = required
                .iter()
                .copied()
                .filter(|k| !keys.iter().any(|kk| kk == *k))
                .collect();
            let (content, safe): (String, PathBuf) = if !missing.is_empty() {
                let (new_content, changed, action, keys) =
                    migrate_content(ctx, &safe, &content, false)?;
                if changed {
                    crate::record::store::write_atomic(ctx, &safe, &new_content)?;
                    let rel = rel_path(&ctx.project_dir, &safe);
                    println!("{}\t{}\t{}", rel, action, keys.join(","));
                    (new_content, safe)
                } else {
                    (content, safe)
                }
            } else {
                (content, safe)
            };

            return Ok((content, safe));
        }
        return Ok((content, safe));
    }
    if managed_descriptor(&ctx.project_dir, &safe).is_none() {
        return Ok((content, safe));
    }
    let desc = managed_descriptor(&ctx.project_dir, &safe).unwrap();

    // Conflict detection before migration.
    let conflicts = scan_migration_conflicts(ctx, desc);
    if let Some(reason) = blocking_conflict_for_path(ctx, &safe, desc, &content, &conflicts) {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            rel_path(&ctx.project_dir, &safe),
            reason
        )));
    }

    let (new_content, changed, action, keys) = migrate_content(ctx, &safe, &content, false)?;
    if !changed {
        if action == "conflict" {
            let rel = rel_path(&ctx.project_dir, &safe);
            println!("{}\tconflict\t{}", rel, keys.join(","));
        }
        return Ok((content, safe));
    }
    crate::record::store::write_atomic(ctx, &safe, &new_content)?;
    let rel = rel_path(&ctx.project_dir, &safe);
    println!("{}\t{}\t{}", rel, action, keys.join(","));
    if action == "converted-legacy-to-backlog-md" {
        let diff = unified_diff(&content, &new_content, &rel, &rel);
        if !diff.is_empty() {
            for line in diff.lines() {
                println!("{}", line);
            }
        }
    }
    Ok((new_content, safe))
}

pub fn rel_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

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

    // Already has front matter: complete -> no-op; incomplete -> extend.
    if frontmatter::has_front_matter(content) {
        let keys = existing_keys(content);
        let required = required_keys(desc);
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|k| !keys.iter().any(|kk| kk == *k))
            .collect();
        if missing.is_empty() {
            return Ok((
                content.to_string(),
                false,
                "already-migrated".to_string(),
                keys,
            ));
        }
        // build ops for missing keys
        let (id, title) = extract_id_title(content, path);
        let created = created_date(ctx, path, desc, content);
        let now = frontmatter::utc_now();
        let mut ops: Vec<PatchOp> = Vec::new();
        let mut new_keys = keys.clone();
        for key in &missing {
            match *key {
                "id" => {
                    let val = id.clone();
                    ops.push(PatchOp::SetFrontMatter((*key).to_string(), val));
                }
                "title" => {
                    ops.push(PatchOp::SetFrontMatter((*key).to_string(), title.clone()));
                }
                "created_date" => {
                    ops.push(PatchOp::SetFrontMatter((*key).to_string(), created.clone()));
                }
                "updated_date" => {
                    ops.push(PatchOp::SetFrontMatter((*key).to_string(), now.clone()));
                }
                "labels" | "references" | "documentation" | "assignee" | "dependencies" => {
                    ops.push(PatchOp::SetFrontMatter(
                        (*key).to_string(),
                        "[]".to_string(),
                    ));
                }
                "status" => {
                    ops.push(PatchOp::SetFrontMatter(
                        (*key).to_string(),
                        "To Do".to_string(),
                    ));
                }
                "milestone" | "release" | "type" | "superseded_by" => {
                    ops.push(PatchOp::SetFrontMatterValue(
                        (*key).to_string(),
                        FrontMatterValue::Null,
                    ));
                }
                "ordinal" => {
                    ops.push(PatchOp::SetFrontMatterValue(
                        (*key).to_string(),
                        FrontMatterValue::Integer(0),
                    ));
                }
                _ => {}
            }
            new_keys.push((*key).to_string());
        }
        // ensure id/title if missing but already present? handled by missing.
        let doc = document::parse(content)?;
        let new_content = patch::apply_patch(&doc, &ops)?;
        return Ok((
            new_content,
            true,
            "added-front-matter".to_string(),
            new_keys,
        ));
    }

    let desc_name = desc.name;
    if desc_name == "task" || desc_name == "milestone" {
        // Legacy-shaped bodies are converted in place; no read or write path
        // relocates or renames a record.
        if is_legacy_shape(desc, content) {
            if desc_name == "task" {
                let (new_content, keys) = convert_task(ctx, path, content)?;
                return Ok((
                    new_content,
                    true,
                    "converted-legacy-to-backlog-md".to_string(),
                    keys,
                ));
            } else {
                let (new_content, keys) = convert_milestone(ctx, path, content)?;
                return Ok((
                    new_content,
                    true,
                    "converted-legacy-to-backlog-md".to_string(),
                    keys,
                ));
            }
        }
        // storage-shaped lacking front matter: added-front-matter, body unchanged
        let (id, title) = extract_id_title(content, path);
        let created = created_date(ctx, path, desc, content);
        let updated = frontmatter::utc_now();
        let mut keys: Vec<String> = GENERAL_KEYS.iter().map(|s| s.to_string()).collect();
        let mut ops: Vec<PatchOp> = Vec::new();
        let fm_id = id.clone();
        ops.push(PatchOp::SetFrontMatter("id".into(), fm_id));
        ops.push(PatchOp::SetFrontMatter("title".into(), title));
        ops.push(PatchOp::SetFrontMatter("created_date".into(), created));
        ops.push(PatchOp::SetFrontMatter("updated_date".into(), updated));
        ops.push(PatchOp::SetFrontMatter("labels".into(), "[]".into()));
        ops.push(PatchOp::SetFrontMatter("references".into(), "[]".into()));
        if desc_name == "task" {
            keys.extend(
                [
                    "status",
                    "assignee",
                    "milestone",
                    "dependencies",
                    "ordinal",
                    "documentation",
                    "type",
                ]
                .iter()
                .map(|s| s.to_string()),
            );
            ops.push(PatchOp::SetFrontMatter("status".into(), "To Do".into()));
            ops.push(PatchOp::SetFrontMatter("assignee".into(), "[]".into()));
            ops.push(PatchOp::SetFrontMatterValue(
                "milestone".into(),
                FrontMatterValue::Null,
            ));
            ops.push(PatchOp::SetFrontMatter("dependencies".into(), "[]".into()));
            ops.push(PatchOp::SetFrontMatterValue(
                "ordinal".into(),
                FrontMatterValue::Integer(0),
            ));
            ops.push(PatchOp::SetFrontMatter("documentation".into(), "[]".into()));
            ops.push(PatchOp::SetFrontMatterValue(
                "type".into(),
                FrontMatterValue::Null,
            ));
        } else {
            keys.push("release".to_string());
            ops.push(PatchOp::SetFrontMatterValue(
                "release".into(),
                FrontMatterValue::Null,
            ));
        }
        let doc = document::parse(content)?;
        let new_content = patch::apply_patch(&doc, &ops)?;
        return Ok((new_content, true, "added-front-matter".to_string(), keys));
    }

    // document records path (TASK-68)
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
    frontmatter_number: Option<i64>,
    stem_number: Option<i64>,
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
        let fm_num = fm_id
            .as_deref()
            .and_then(|value| id_value_number(desc, cfg, value));
        let number = stem_number(desc, cfg, stem);
        let prefix = matching_prefix_of(desc, cfg, stem);
        out.push(Candidate {
            path,
            rel,
            frontmatter_id: fm_id,
            frontmatter_number: fm_num,
            stem_number: number,
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
        if let (Some(fm), Some(stem_number), Some(stem_prefix)) =
            (&c.frontmatter_id, c.stem_number, &c.stem_prefix)
        {
            let same = c.frontmatter_number == Some(stem_number)
                && fm
                    .split_once('-')
                    .is_some_and(|(prefix, _)| prefix.eq_ignore_ascii_case(stem_prefix));
            if !same {
                conflicts.push(MigrationConflict::FrontmatterFilenameMismatch {
                    path: c.rel.clone(),
                    filename_id: format!("{}-{}", stem_prefix, stem_number),
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
        if c.frontmatter_id.is_none() && c.stem_number.is_none() {
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

    if let (Some(fm_id), Some(stem_number), Some(stem_prefix)) = (
        candidate_frontmatter_id(desc, content),
        stem_number(desc, &cfg, stem),
        matching_prefix_of(desc, &cfg, stem),
    ) {
        let fm_number = id_value_number(desc, &cfg, &fm_id);
        let same = fm_number == Some(stem_number)
            && fm_id
                .split_once('-')
                .is_some_and(|(prefix, _)| prefix.eq_ignore_ascii_case(&stem_prefix));
        if !same {
            return Some(format!(
                "front-matter/filename mismatch: filename {}-{} vs front matter {}",
                stem_prefix, stem_number, fm_id
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

// Diff helpers.

#[derive(Clone, Copy, Debug)]
enum DiffOp<'a> {
    Equal(&'a str),
    Insert(&'a str),
    Delete(&'a str),
}

fn diff_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<DiffOp<'a>> {
    let lcs = longest_common_subsequence(old, new);
    let mut ops: Vec<DiffOp<'a>> = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    let mut k = 0usize;
    while k < lcs.len() {
        while i < old.len() && old[i] != lcs[k] {
            ops.push(DiffOp::Delete(old[i]));
            i += 1;
        }
        while j < new.len() && new[j] != lcs[k] {
            ops.push(DiffOp::Insert(new[j]));
            j += 1;
        }
        ops.push(DiffOp::Equal(old[i]));
        i += 1;
        j += 1;
        k += 1;
    }
    while i < old.len() {
        ops.push(DiffOp::Delete(old[i]));
        i += 1;
    }
    while j < new.len() {
        ops.push(DiffOp::Insert(new[j]));
        j += 1;
    }
    ops
}

fn unified_diff(old: &str, new: &str, old_label: &str, new_label: &str) -> String {
    const CONTEXT: usize = 3;
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let ops = diff_lines(&old_lines, &new_lines);
    if ops.iter().all(|op| matches!(op, DiffOp::Equal(_))) {
        return String::new();
    }

    // Cumulative line counts in the old/new files up to each op index.
    let mut old_before = vec![0usize; ops.len() + 1];
    let mut new_before = vec![0usize; ops.len() + 1];
    for (idx, op) in ops.iter().enumerate() {
        old_before[idx + 1] = old_before[idx]
            + match op {
                DiffOp::Equal(_) | DiffOp::Delete(_) => 1,
                DiffOp::Insert(_) => 0,
            };
        new_before[idx + 1] = new_before[idx]
            + match op {
                DiffOp::Equal(_) | DiffOp::Insert(_) => 1,
                DiffOp::Delete(_) => 0,
            };
    }

    let mut hunks: Vec<String> = Vec::new();
    hunks.push(format!("--- {}", old_label));
    hunks.push(format!("+++ {}", new_label));

    let n = ops.len();
    let mut i = 0usize;
    while i < n {
        while i < n && matches!(ops[i], DiffOp::Equal(_)) {
            i += 1;
        }
        if i >= n {
            break;
        }

        let hunk_start = i.saturating_sub(CONTEXT);
        let mut j = i;
        let mut last_change = i;
        while j < n {
            if !matches!(ops[j], DiffOp::Equal(_)) {
                last_change = j;
            }
            if j > last_change && j - last_change > CONTEXT {
                break;
            }
            j += 1;
        }
        let hunk_end = j.min(n);

        let old_start = old_before[hunk_start] + 1;
        let new_start = new_before[hunk_start] + 1;
        let old_len = old_before[hunk_end] - old_before[hunk_start];
        let new_len = new_before[hunk_end] - new_before[hunk_start];
        hunks.push(format!(
            "@@ -{},{} +{},{} @@",
            old_start, old_len, new_start, new_len
        ));

        for op in ops.iter().take(hunk_end).skip(hunk_start) {
            match *op {
                DiffOp::Equal(line) => hunks.push(format!(" {}", line)),
                DiffOp::Insert(line) => hunks.push(format!("+{}", line)),
                DiffOp::Delete(line) => hunks.push(format!("-{}", line)),
            }
        }

        i = hunk_end;
    }

    hunks.join("\n")
}

fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            if a[i] == b[j] {
                dp[i][j] = 1 + dp[i + 1][j + 1];
            } else {
                dp[i][j] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }
    let mut lcs = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < m && j < n {
        if a[i] == b[j] {
            lcs.push(a[i]);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    lcs
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
        if let Some(d) = &e.diff {
            for line in d.lines() {
                println!("{}", line);
            }
        }
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
    let rel = path
        .strip_prefix(&ctx.project_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    if dry_run {
        if frontmatter::has_front_matter(&content) {
            entries.push(MigrationEntry {
                path: rel,
                action: "already-migrated".to_string(),
                keys: existing_keys(&content),
                diff: None,
            });
        } else {
            let desc = managed_descriptor(&ctx.project_dir, path);
            let legacy = desc.map(|d| is_legacy_shape(d, &content)).unwrap_or(false);
            let (new_content, _, _, keys) = migrate_content(ctx, path, &content, true)?;
            let diff = if legacy {
                Some(unified_diff(&content, &new_content, &rel, &rel))
            } else {
                None
            };
            entries.push(MigrationEntry {
                path: rel,
                action: if legacy {
                    "would-convert".to_string()
                } else {
                    "would-migrate".to_string()
                },
                keys,
                diff,
            });
        }
        return Ok(());
    }
    if frontmatter::has_front_matter(&content) {
        return Ok(());
    }
    if managed_descriptor(&ctx.project_dir, path).is_none() {
        return Ok(());
    }
    let (new_content, changed, action, keys) = migrate_content(ctx, path, &content, false)?;
    if !changed {
        if action == "conflict" {
            entries.push(MigrationEntry {
                path: rel.clone(),
                action: "conflict".to_string(),
                keys: keys.clone(),
                diff: None,
            });
            println!("{}\tconflict\t{}", rel, keys.join(","));
        }
        return Ok(());
    }
    crate::record::store::write_atomic(ctx, path, &new_content)?;
    let diff = if action == "converted-legacy-to-backlog-md" {
        Some(unified_diff(&content, &new_content, &rel, &rel))
    } else {
        None
    };
    entries.push(MigrationEntry {
        path: rel,
        action,
        keys,
        diff,
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
        if action == "conflict" {
            let rel = safe
                .strip_prefix(&ctx.project_dir)
                .unwrap_or(&safe)
                .to_string_lossy()
                .replace('\\', "/");
            println!("{}\tconflict\t{}", rel, keys.join(","));
        }
        return Ok(None);
    }
    crate::record::store::write_atomic(ctx, &safe, &new_content)?;
    let rel = safe
        .strip_prefix(&ctx.project_dir)
        .unwrap_or(&safe)
        .to_string_lossy()
        .replace('\\', "/");
    let diff = if action == "converted-legacy-to-backlog-md" {
        Some(unified_diff(&content, &new_content, &rel, &rel))
    } else {
        None
    };
    let entry = MigrationEntry {
        path: rel.clone(),
        action: action.clone(),
        keys: keys.clone(),
        diff,
    };
    println!("{}\t{}\t{}", rel, action, keys.join(","));
    Ok(Some(entry))
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

fn file_created_date(path: &Path) -> String {
    if let Ok(meta) = fs::metadata(path)
        && let Ok(m) = meta.modified()
    {
        let secs = m
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        return frontmatter::utc_from_unix(secs);
    }
    frontmatter::utc_now()
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

fn is_legacy_shape(desc: &RecordDescriptor, content: &str) -> bool {
    if let Ok(doc) = document::parse(content) {
        let retired: &[&str] = match desc.name {
            "task" => &["Status", "Milestone", "Assignee", "Labels", "Dependencies"],
            "milestone" => &["Release"],
            _ => &[],
        };
        for s in &doc.sections {
            let h = s.heading.trim_start_matches("## ").trim();
            if retired.contains(&h) {
                return true;
            }
        }
        if let Some(h) = &doc.heading {
            let rest = h.trim_start_matches('#').trim();
            let first = rest.split_whitespace().next().unwrap_or("");
            let up = first.to_uppercase();
            if desc.name == "task" && up.starts_with("TASK-") {
                return true;
            }
            if desc.name == "milestone" && up.starts_with("MS-") {
                return true;
            }
        }
    }
    if content.contains("<!-- SECTION:DESCRIPTION:BEGIN -->")
        || content.contains("<!-- AC:BEGIN -->")
    {
        return false;
    }
    false
}

fn section_text_doc(doc: &document::Document, name: &str) -> Option<String> {
    doc.sections
        .iter()
        .find(|s| s.heading == format!("## {}", name))
        .map(|s| s.body.join("\n").trim().to_string())
        .filter(|s| !s.is_empty())
}

fn section_lines_doc(doc: &document::Document, name: &str) -> Vec<String> {
    doc.sections
        .iter()
        .find(|s| s.heading == format!("## {}", name))
        .map(|s| {
            s.body
                .iter()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn convert_task(
    ctx: &Context,
    path: &Path,
    content: &str,
) -> Result<(String, Vec<String>), AppError> {
    let cfg = load_cfg(ctx);
    let doc = document::parse(content)?;
    let (display_id, title) = extract_id_title(content, path);
    let storage_id = display_id.clone();
    let fm_id = display_id.clone();

    let status = section_text_doc(&doc, "Status").unwrap_or_else(|| cfg.default_status.clone());
    let milestone_raw = section_text_doc(&doc, "Milestone");
    let milestone_storage = milestone_raw
        .as_ref()
        .map(|m| m.trim().to_string())
        .filter(|s| !s.is_empty());
    let assignee_raw = section_text_doc(&doc, "Assignee");
    let assignee_list: Vec<String> = assignee_raw
        .map(|a| {
            if a.trim().is_empty() {
                Vec::new()
            } else {
                vec![a.trim().to_string()]
            }
        })
        .unwrap_or_default();
    let labels_lines = section_lines_doc(&doc, "Labels");
    let labels: Vec<String> = labels_lines
        .iter()
        .map(|l| l.trim().trim_start_matches("- ").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let deps_lines = section_lines_doc(&doc, "Dependencies");
    let deps_display: Vec<String> = deps_lines
        .iter()
        .map(|l| l.trim().trim_start_matches("- ").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let deps_storage: Vec<String> = deps_display.clone();
    let desc = section_text_doc(&doc, "Description");
    let ac_lines = section_lines_doc(&doc, "Acceptance Criteria");
    let mut acs: Vec<String> = Vec::new();
    for l in &ac_lines {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let mut item = t.strip_prefix("- ").unwrap_or(t).trim().to_string();
        for pref in ["[ ] ", "[x] ", "[X] "] {
            if let Some(rest) = item.strip_prefix(pref) {
                item = rest.trim().to_string();
                break;
            }
        }
        // strip leading "#n " if present
        if let Some(rest) = item.strip_prefix('#') {
            // remove number up to space
            if let Some(sp) = rest.find(' ') {
                let num_part = &rest[..sp];
                if num_part.chars().all(|c| c.is_ascii_digit()) {
                    item = rest[sp + 1..].trim().to_string();
                }
            }
        }
        if !item.is_empty() {
            acs.push(item);
        }
    }
    // comments
    let comment_lines = section_lines_doc(&doc, "Comments");
    let mut comments: Vec<(String, String)> = Vec::new();
    for l in &comment_lines {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let item = t.strip_prefix("- ").unwrap_or(t).trim().to_string();
        if let Some(inner) = item.strip_prefix("**")
            && let Some(end) = inner.find("**")
        {
            let author = inner[..end].trim().to_string();
            let after = inner[end + 2..].trim();
            let text = after.trim_start_matches(':').trim().to_string();
            comments.push((author, text));
            continue;
        }
        comments.push(("unknown".to_string(), item));
    }

    // unknown sections
    let known: &[&str] = &[
        "Description",
        "Status",
        "Milestone",
        "Assignee",
        "Labels",
        "Dependencies",
        "Acceptance Criteria",
        "Comments",
    ];
    let unknown: Vec<&document::Section> = doc
        .sections
        .iter()
        .filter(|s| {
            let h = s.heading.trim_start_matches("## ").trim();
            !known.contains(&h)
        })
        .collect();

    // ordinal default
    let task_desc = crate::record::descriptor::descriptor_for("task")
        .ok_or_else(|| AppError::Runtime("task descriptor is unavailable".to_string()))?;
    let dirs = task_desc.resolve_read_directories(&cfg, &ctx.project_dir);
    let dir_refs: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
    let ordinal_final = crate::record::identity::scan_max_ordinal_in_dirs(&dir_refs);

    let created = file_created_date(path);
    let updated = frontmatter::utc_now();

    // build front matter
    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", fm_id));
    fm.push_str(&format!(
        "title: {}\n",
        frontmatter::format_scalar_pub(&title)
    ));
    fm.push_str(&format!("created_date: '{}'\n", created));
    fm.push_str(&format!("updated_date: '{}'\n", updated));
    if labels.is_empty() {
        fm.push_str("labels: []\n");
    } else {
        fm.push_str("labels:\n");
        for l in &labels {
            fm.push_str(&format!("  - {}\n", frontmatter::format_scalar_pub(l)));
        }
    }
    fm.push_str("references: []\n");
    fm.push_str(&format!(
        "status: {}\n",
        frontmatter::format_scalar_pub(&status)
    ));
    if assignee_list.is_empty() {
        fm.push_str("assignee: []\n");
    } else {
        fm.push_str("assignee:\n");
        for a in &assignee_list {
            fm.push_str(&format!("  - {}\n", frontmatter::format_scalar_pub(a)));
        }
    }
    if let Some(m) = &milestone_storage {
        fm.push_str(&format!(
            "milestone: {}\n",
            frontmatter::format_scalar_pub(m)
        ));
    } else {
        fm.push_str("milestone: null\n");
    }
    if deps_storage.is_empty() {
        fm.push_str("dependencies: []\n");
    } else {
        fm.push_str("dependencies:\n");
        for d in &deps_storage {
            fm.push_str(&format!("  - {}\n", frontmatter::format_scalar_pub(d)));
        }
    }
    fm.push_str(&format!("ordinal: {}\n", ordinal_final));
    fm.push_str("documentation: []\n");
    fm.push_str("type: null\n");

    // build body
    let mut parts: Vec<String> = Vec::new();
    let desc_text = desc.unwrap_or_default();
    let mut desc_block = String::from("## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\n");
    if !desc_text.trim().is_empty() {
        desc_block.push_str(&desc_text);
        if !desc_text.ends_with('\n') {
            desc_block.push('\n');
        }
    }
    desc_block.push_str("<!-- SECTION:DESCRIPTION:END -->");
    parts.push(desc_block);

    if !acs.is_empty() {
        let mut ac_block = String::from("## Acceptance Criteria\n<!-- AC:BEGIN -->");
        for (i, c) in acs.iter().enumerate() {
            ac_block.push_str(&format!("\n- [ ] #{} {}", i + 1, c));
        }
        ac_block.push_str("\n<!-- AC:END -->");
        parts.push(ac_block);
    }

    if !comments.is_empty() {
        let mut c_block = String::from("## Comments\n<!-- COMMENTS:BEGIN -->");
        for (a, t) in &comments {
            c_block.push_str(&format!("\n- **{}**: {}", a, t));
        }
        c_block.push_str("\n<!-- COMMENTS:END -->");
        parts.push(c_block);
    }

    for sec in &unknown {
        let mut block = sec.heading.clone();
        if !sec.body.is_empty() {
            block.push('\n');
            block.push_str(&sec.body.join("\n"));
        }
        parts.push(block);
    }

    let body = parts.join("\n\n") + "\n";
    let new_content = frontmatter::wrap(&fm, &body);

    let keys = vec![
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
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let _ = storage_id;
    Ok((new_content, keys))
}

fn convert_milestone(
    ctx: &Context,
    path: &Path,
    content: &str,
) -> Result<(String, Vec<String>), AppError> {
    let cfg = load_cfg(ctx);
    let doc = document::parse(content)?;
    let (display_id, title) = extract_id_title(content, path);
    let fm_id = display_id.clone();

    let release_raw = section_text_doc(&doc, "Release");
    let release = release_raw
        .map(|r| r.trim().to_string())
        .filter(|s| !s.is_empty());
    let desc = section_text_doc(&doc, "Description");

    let known: &[&str] = &["Description", "Release"];
    let unknown: Vec<&document::Section> = doc
        .sections
        .iter()
        .filter(|s| {
            let h = s.heading.trim_start_matches("## ").trim();
            !known.contains(&h)
        })
        .collect();

    let created = file_created_date(path);
    let updated = frontmatter::utc_now();

    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", fm_id));
    fm.push_str(&format!(
        "title: {}\n",
        frontmatter::format_scalar_pub(&title)
    ));
    fm.push_str(&format!("created_date: '{}'\n", created));
    fm.push_str(&format!("updated_date: '{}'\n", updated));
    fm.push_str("labels: []\n");
    fm.push_str("references: []\n");
    if let Some(r) = &release {
        fm.push_str(&format!("release: {}\n", frontmatter::format_scalar_pub(r)));
    } else {
        fm.push_str("release: null\n");
    }

    let mut parts: Vec<String> = Vec::new();
    let desc_text = desc.unwrap_or_default();
    parts.push(format!("## Description\n\n{}", desc_text.trim()));
    for sec in &unknown {
        let mut block = sec.heading.clone();
        if !sec.body.is_empty() {
            block.push('\n');
            block.push_str(&sec.body.join("\n"));
        }
        parts.push(block);
    }
    let body = parts.join("\n\n") + "\n";
    let new_content = frontmatter::wrap(&fm, &body);

    let keys = vec![
        "id",
        "title",
        "created_date",
        "updated_date",
        "labels",
        "references",
        "release",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let _ = cfg;
    Ok((new_content, keys))
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

    #[test]
    fn native_task_is_left_byte_identical() {
        let dir = tmpdir("native-task");
        let c = ctx(&dir);
        let body = "## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody line\n<!-- SECTION:DESCRIPTION:END -->\n";
        let content = format!(
            "---\nid: TASK-5\ntitle: Title\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 5\ndocumentation: []\ntype: null\n---\n\n{}",
            body
        );
        let path = dir.join("backlog/tasks/TASK-005.md");
        write(&path, &content);
        let result = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(result, content);
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn native_milestone_gets_additive_keys() {
        let dir = tmpdir("native-milestone");
        let c = ctx(&dir);
        let body = "## Description\n\nMilestone body\n";
        let content = format!("---\nid: MS-002\ntitle: Title\n---\n\n{}", body);
        let path = dir.join("backlog/milestones/MS-002.md");
        write(&path, &content);
        let result = ensure_front_matter(&c, &path).unwrap();
        assert!(result.starts_with("---\n"));
        assert!(result.contains("created_date:"));
        assert!(result.contains("updated_date:"));
        assert!(result.contains("labels: []"));
        assert!(result.contains("references: []"));
        assert!(result.contains("release:"));
        assert!(result.contains("## Description"));
        assert!(result.contains("Milestone body"));
    }

    #[test]
    fn legacy_task_is_converted_in_place() {
        let dir = tmpdir("legacy-task");
        let c = ctx(&dir);
        let legacy = "# TASK-001 Old Title\n\n## Status\nDone\n\n## Description\nLegacy body.\n\n## Acceptance Criteria\n- [ ] #1 first thing\n- [x] #2 second thing\n\n## Labels\n- bug\n\n## Unknown Section\nkeep me\n";
        let path = dir.join("backlog/tasks/TASK-001.md");
        write(&path, legacy);
        let _ = ensure_front_matter(&c, &path).unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("id: TASK-001"));
        assert!(content.contains("status: Done"));
        assert!(content.contains("<!-- SECTION:DESCRIPTION:BEGIN -->"));
        assert!(content.contains("Legacy body."));
        assert!(content.contains("<!-- AC:BEGIN -->"));
        assert!(content.contains("- [ ] #1 first thing"));
        assert!(content.contains("- [ ] #2 second thing"));
        assert!(content.contains("## Unknown Section"));
        assert!(content.contains("keep me"));
    }

    #[test]
    fn legacy_milestone_is_converted() {
        let dir = tmpdir("legacy-milestone");
        let c = ctx(&dir);
        let legacy = "# MS-001 Mile Title\n\n## Release\nREL-001\n\n## Description\nDesc.\n";
        let path = dir.join("backlog/milestones/MS-001.md");
        write(&path, legacy);
        let _ = ensure_front_matter(&c, &path).unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("id: MS-001"));
        assert!(content.contains("release: REL-001"));
        assert!(!content.contains("## Release"));
    }

    #[test]
    fn converted_legacy_entry_reports_diff() {
        let dir = tmpdir("legacy-diff");
        let c = ctx(&dir);
        let legacy = "# TASK-010 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-010.md");
        write(&path, legacy);
        let entry = migrate_file(&c, &path).unwrap().expect("entry expected");
        assert_eq!(entry.action, "converted-legacy-to-backlog-md");
        let diff = entry.diff.expect("diff expected");
        assert!(diff.contains("--- "));
        assert!(diff.contains("+++ "));
        assert!(diff.contains("-# TASK-010 Title"));
        assert!(diff.contains("+id: TASK-010"));
    }

    #[test]
    fn duplicate_ids_are_detected() {
        let dir = tmpdir("duplicate");
        let c = ctx(&dir);
        let a = dir.join("backlog/tasks/TASK-007.md");
        let b = dir.join("backlog/tasks/TASK-0007.md");
        write(
            &a,
            "---\nid: TASK-7\ntitle: A\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 7\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nA\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        write(
            &b,
            "---\nid: TASK-7\ntitle: B\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 7\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nB\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert_eq!(conflicts.len(), 1);
        match &conflicts[0] {
            MigrationConflict::DuplicateId { id, paths } => {
                assert_eq!(id, "TASK-7");
                assert_eq!(paths.len(), 2);
            }
            _ => panic!("expected duplicate id conflict"),
        }
    }

    #[test]
    fn same_title_distinct_numbers_are_not_conflicts() {
        let dir = tmpdir("same-title-distinct");
        let c = ctx(&dir);
        let a = dir.join("backlog/tasks/TASK-008.md");
        let b = dir.join("backlog/tasks/TASK-009.md");
        let tpl = "---\nid: TASK-{n}\ntitle: SameTitle\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: {n}\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody\n<!-- SECTION:DESCRIPTION:END -->\n";
        write(&a, &tpl.replace("{n}", "8"));
        write(&b, &tpl.replace("{n}", "9"));
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert!(
            conflicts.is_empty(),
            "same title distinct numbers must not conflict: {:?}",
            conflicts
        );
        // Views for both distinct records succeed.
        assert!(ensure_front_matter(&c, &a).is_ok());
        assert!(ensure_front_matter(&c, &b).is_ok());
    }

    #[test]
    fn dropped_task_keys_are_readded_on_next_read() {
        // Simulate Backlog.md editing a migrated file and dropping additive keys.
        let dir = tmpdir("dropped-keys-task");
        let c = ctx(&dir);
        let body = "## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody\n<!-- SECTION:DESCRIPTION:END -->\n";
        let minimal = format!(
            "---\nid: TASK-6\ntitle: Interop\nstatus: To Do\n---\n\n{}",
            body
        );
        let path = dir.join("backlog/tasks/TASK-006.md");
        write(&path, &minimal);
        let result = ensure_front_matter(&c, &path).unwrap();
        for key in [
            "created_date:",
            "updated_date:",
            "labels:",
            "references:",
            "assignee:",
            "milestone:",
            "dependencies:",
            "ordinal:",
            "documentation:",
            "type:",
        ] {
            assert!(result.contains(key), "missing {} in: {}", key, result);
        }
        assert!(result.contains("## Description"));
        assert!(result.contains("Body"));
        let on_disk = fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, result);
        // Body byte-identical apart from front matter.
        let (_, out_body) = frontmatter::split(&result).unwrap();
        assert_eq!(out_body, format!("\n{}", body));
    }

    #[test]
    fn frontmatter_filename_mismatch_detected() {
        let dir = tmpdir("mismatch");
        let c = ctx(&dir);
        let path = dir.join("backlog/tasks/TASK-005.md");
        write(
            &path,
            "---\nid: TASK-99\ntitle: Title\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 5\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        let conflicts = scan_migration_conflicts(&c, desc);
        assert!(conflicts.iter().any(|c| matches!(
            c,
            MigrationConflict::FrontmatterFilenameMismatch { path: p, .. }
            if p.contains("TASK-005")
        )));
    }

    #[test]
    fn idempotency_second_run_is_no_op() {
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
    fn read_path_write_failure_keeps_original() {
        let dir = tmpdir("readonly");
        let c = ctx(&dir);
        let legacy = "# TASK-003 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-003.md");
        write(&path, legacy);
        // Make the target directory read-only so atomic write fails.
        let tasks_dir = dir.join("backlog/tasks");
        let original_mode = fs::metadata(&tasks_dir).unwrap().permissions().mode();
        let mut perms = fs::metadata(&tasks_dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&tasks_dir, perms).unwrap();
        let result = ensure_front_matter(&c, &path);
        fs::set_permissions(&tasks_dir, fs::Permissions::from_mode(original_mode)).unwrap();
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
        // No temp files left in the project.
        for entry in fs::read_dir(&tasks_dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().to_string();
            assert!(!name.contains(".tmp"), "leftover temp: {}", name);
        }
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
        assert!(
            !dir.join("backlog/archive/tasks/TASK-004 - Title.md")
                .exists()
        );
    }

    #[test]
    fn dry_run_does_not_write() {
        let dir = tmpdir("dryrun");
        let c = ctx(&dir);
        let legacy = "# TASK-005 Title\n\n## Description\nBody\n";
        let path = dir.join("backlog/tasks/TASK-005.md");
        write(&path, legacy);
        let entries = migrate_corpus(&c, true).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "would-convert");
        assert!(entries[0].diff.is_some());
        // Original untouched.
        assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
    }

    #[test]
    fn unified_diff_includes_before_and_after() {
        let old = "line one\nline two\n";
        let new = "line one\nline two changed\n";
        let diff = unified_diff(old, new, "a/old.md", "b/new.md");
        assert!(diff.contains("--- a/old.md"));
        assert!(diff.contains("+++ b/new.md"));
        assert!(diff.contains("-line two"));
        assert!(diff.contains("+line two changed"));
    }

    #[test]
    fn backlog_shaped_stem_is_not_managed() {
        let dir = tmpdir("backlog-shaped");
        let c = ctx(&dir);
        let content = "## Description\n\nBody line\n";
        let path = dir.join("backlog/tasks/task-5 - Parsed-Title.md");
        write(&path, content);
        assert!(
            managed_descriptor(&dir, &path).is_none(),
            "Backlog-shaped stems must resolve to no record"
        );
        let result = ensure_front_matter(&c, &path).unwrap();
        assert_eq!(result, content, "invisible record must not be migrated");
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        let desc = crate::record::descriptor::descriptor_for("task").unwrap();
        assert!(scan_migration_conflicts(&c, desc).is_empty());
    }

    #[test]
    fn incomplete_front_matter_uses_yaml_null_and_unquoted_integer() {
        let dir = tmpdir("incomplete-null");
        let c = ctx(&dir);
        let body = "## Description\n\nMilestone body\n";
        let content = format!("---\nid: MS-002\ntitle: Title\n---\n\n{}", body);
        let path = dir.join("backlog/milestones/MS-002.md");
        write(&path, &content);
        let result = ensure_front_matter(&c, &path).unwrap();
        assert!(result.contains("release: null"), "got: {}", result);
        assert!(!result.contains("release: \"null\""), "got: {}", result);
    }

    #[test]
    fn conflict_check_fails_for_front_mattered_duplicate() {
        let dir = tmpdir("frontmatter-conflict");
        let c = ctx(&dir);
        let a = dir.join("backlog/tasks/TASK-007.md");
        let b = dir.join("backlog/tasks/TASK-0007.md");
        write(
            &a,
            "---\nid: TASK-7\ntitle: A\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 7\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nA\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        write(
            &b,
            "---\nid: TASK-7\ntitle: B\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 7\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nB\n<!-- SECTION:DESCRIPTION:END -->\n",
        );
        let err = ensure_front_matter(&c, &a).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate id"), "{}", msg);
        assert!(msg.contains("TASK-7"), "{}", msg);
    }

    #[test]
    fn unified_diff_ordered_non_overlapping() {
        let old = "a\nb\nc\nd\ne\nf\ng\n";
        let new = "a\nb\nc\nX\nd\ne\nf\ng\nY\n";
        let diff = unified_diff(old, new, "old.md", "new.md");
        let hunks: Vec<&str> = diff.lines().filter(|l| l.starts_with("@@")).collect();
        assert_eq!(hunks.len(), 2, "expected two hunks, got: {}", diff);
        // Hunk headers should be in file-order.
        let first = hunks[0];
        let second = hunks[1];
        let first_old_start: usize = first[4..].split(',').next().unwrap().parse().unwrap();
        let second_old_start: usize = second[4..].split(',').next().unwrap().parse().unwrap();
        assert!(
            first_old_start < second_old_start,
            "hunks out of order: {:?}",
            hunks
        );
    }
}
