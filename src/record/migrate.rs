//! Lazy, per-record migration of managed document record types to front matter.

use crate::context::Context;
use crate::error::AppError;
use crate::record::descriptor::RecordDescriptor;
use crate::record::document;
use crate::record::frontmatter;
use crate::record::patch::{self, PatchOp};
use std::fs;
use std::path::Path;

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

pub fn is_managed_path(project_dir: &Path, path: &Path) -> bool {
    managed_descriptor(project_dir, path).is_some()
}

pub fn managed_descriptor(project_dir: &Path, path: &Path) -> Option<&'static RecordDescriptor> {
    let rel = path.strip_prefix(project_dir).ok()?;
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    // ADR decision records (frozen bodies).
    if rel_str.starts_with("docs/architecture/adr/") {
        let name = path.file_name()?.to_string_lossy();
        if (name.starts_with("ADR-") || name.starts_with("BDR-"))
            && path.extension().and_then(|e| e.to_str()) == Some("md")
        {
            return crate::record::descriptor::descriptor_for("decision");
        }
    }
    for desc in crate::record::descriptor::descriptors() {
        // Only document record types in scope for TASK-68.
        if desc.name == "task" || desc.name == "milestone" {
            continue;
        }
        let dir = desc.directory;
        if !rel_str.starts_with(dir) {
            continue;
        }
        // ensure it's directly under dir or subdirectory
        let stem = path.file_stem()?.to_string_lossy().to_string();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if matches_pattern(&stem, desc.display_id_pattern)
            || (desc.name == "decision" && (stem.starts_with("ADR-") || stem.starts_with("BDR-")))
        {
            // For decision descriptor also allow ADR/BDR stems via special case above.
            return Some(desc);
        }
    }
    None
}

fn matches_pattern(stem: &str, pattern: &str) -> bool {
    // A run of '#' in the pattern matches one or more ASCII digits, so
    // identifiers with variable digit counts (e.g. REQ-01.001, NFR-04.001)
    // are recognized. Literal characters must match exactly.
    let stem_chars: Vec<char> = stem.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    matches_from(&stem_chars, &pattern_chars, 0, 0)
}

fn matches_from(stem: &[char], pattern: &[char], si: usize, pi: usize) -> bool {
    if pi == pattern.len() {
        return si == stem.len();
    }
    if pattern[pi] == '#' {
        // Consume the entire run of '#' as a single digit placeholder.
        let mut run_end = pi;
        while run_end < pattern.len() && pattern[run_end] == '#' {
            run_end += 1;
        }
        // Try every non-empty run of digits.
        let mut k = 1usize;
        while si + k <= stem.len() && stem[si + k - 1].is_ascii_digit() {
            if matches_from(stem, pattern, si + k, run_end) {
                return true;
            }
            k += 1;
            if si + k - 1 >= stem.len() {
                break;
            }
        }
        false
    } else if si < stem.len() && stem[si] == pattern[pi] {
        matches_from(stem, pattern, si + 1, pi + 1)
    } else {
        false
    }
}

pub fn has_front_matter(content: &str) -> bool {
    frontmatter::has_front_matter(content)
}

pub fn ensure_front_matter(ctx: &Context, path: &Path) -> Result<String, AppError> {
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
        return Ok(content);
    }
    if managed_descriptor(&ctx.project_dir, &safe).is_none() {
        return Ok(content);
    }
    let (new_content, changed, action, keys) = migrate_content(ctx, &safe, &content, false)?;
    if changed {
        crate::record::store::write_atomic(ctx, &safe, &new_content)?;
        let rel = safe
            .strip_prefix(&ctx.project_dir)
            .unwrap_or(&safe)
            .to_string_lossy()
            .replace('\\', "/");
        println!("{}\t{}\t{}", rel, action, keys.join(","));
    }
    Ok(new_content)
}

pub fn migrate_content(
    ctx: &Context,
    path: &Path,
    content: &str,
    _dry_run: bool,
) -> Result<(String, bool, String, Vec<String>), AppError> {
    if frontmatter::has_front_matter(content) {
        // already migrated; return as-is
        let keys = existing_keys(content);
        return Ok((
            content.to_string(),
            false,
            "already-migrated".to_string(),
            keys,
        ));
    }
    let desc = match managed_descriptor(&ctx.project_dir, path) {
        Some(d) => d,
        None => return Ok((content.to_string(), false, String::new(), Vec::new())),
    };

    let (id, title) = extract_id_title(content, path);
    let created = created_date(ctx, path, desc, content);
    let updated = frontmatter::utc_now();

    let mut keys: Vec<String> = GENERAL_KEYS.iter().map(|s| s.to_string()).collect();
    let mut ops: Vec<PatchOp> = Vec::new();
    ops.push(PatchOp::SetFrontMatter("id".into(), id));
    ops.push(PatchOp::SetFrontMatter("title".into(), title));
    ops.push(PatchOp::SetFrontMatter("created_date".into(), created));
    ops.push(PatchOp::SetFrontMatter("updated_date".into(), updated));
    ops.push(PatchOp::SetFrontMatter("labels".into(), "[]".into()));
    ops.push(PatchOp::SetFrontMatter("references".into(), "[]".into()));

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
    // find heading
    let mut heading: Option<String> = None;
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            heading = Some(t.to_string());
        }
        break;
    }
    if let Some(h) = heading {
        let rest = h.trim_start_matches('#').trim();
        let mut parts = rest.splitn(2, char::is_whitespace);
        let first = parts.next().unwrap_or("").trim();
        let title = parts.next().unwrap_or("").trim().to_string();
        if first.contains('-') && first.chars().any(|c| c.is_ascii_digit()) {
            return (first.to_string(), title);
        }
        // no id in heading; derive id from filename
        let id = id_from_filename(path);
        return (id, rest.to_string());
    }
    let id = id_from_filename(path);
    (id, String::new())
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
    if desc.name == "decision" {
        if let Ok(doc) = document::parse(content) {
            if let Some(sec) = doc.sections.iter().find(|s| s.heading == "## Date") {
                let body = sec.body.join("\n");
                if let Some(ts) = parse_date_section(&body) {
                    return ts;
                }
            }
        }
    }
    // file mtime
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(m) = meta.modified() {
            let secs = m
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            return frontmatter::utc_from_unix(secs);
        }
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

// Dry-run traversal and migration helpers.
pub fn migrate_corpus(ctx: &Context, dry_run: bool) -> Result<Vec<MigrationEntry>, AppError> {
    let mut entries = Vec::new();
    // descriptor dirs
    for desc in crate::record::descriptor::descriptors() {
        if desc.name == "task" || desc.name == "milestone" {
            continue;
        }
        let dir = ctx.project_dir.join(desc.directory);
        if dir.exists() {
            collect_dir(ctx, &dir, dry_run, &mut entries)?;
        }
    }
    // ADR decision records
    let adr_dir = ctx.project_dir.join("docs/architecture/adr");
    if adr_dir.exists() {
        collect_adr_dir(ctx, &adr_dir, dry_run, &mut entries)?;
    }
    // print lines for dry-run and live? Live corpus migration not a command, but print changed.
    for e in &entries {
        if dry_run {
            println!("{}\t{}\t{}", e.path, e.action, e.keys.join(","));
        } else if e.action != "already-migrated" {
            // live entries are changed files only; print
            println!("{}\t{}\t{}", e.path, e.action, e.keys.join(","));
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

fn collect_adr_dir(
    ctx: &Context,
    dir: &Path,
    dry_run: bool,
    entries: &mut Vec<MigrationEntry>,
) -> Result<(), AppError> {
    collect_dir(ctx, dir, dry_run, entries)
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
            });
        } else {
            let (_, _, _, keys) = migrate_content(ctx, path, &content, true)?;
            entries.push(MigrationEntry {
                path: rel,
                action: "would-migrate".to_string(),
                keys,
            });
        }
    } else {
        if frontmatter::has_front_matter(&content) {
            return Ok(());
        }
        let (new_content, changed, action, keys) = migrate_content(ctx, path, &content, false)?;
        if changed {
            crate::record::store::write_atomic(ctx, path, &new_content)?;
            entries.push(MigrationEntry {
                path: rel,
                action,
                keys,
            });
        }
    }
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
    let rel = safe
        .strip_prefix(&ctx.project_dir)
        .unwrap_or(&safe)
        .to_string_lossy()
        .replace('\\', "/");
    let entry = MigrationEntry {
        path: rel.clone(),
        action: action.clone(),
        keys: keys.clone(),
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
