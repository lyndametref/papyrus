//! Canonical-ID resolution for managed record families.
//!
//! Resolution is tolerant of prefix case, optional leading zeros, and a
//! cosmetic `-<slug>` (or ` - <slug>`) suffix. The reported identity is always
//! the canonical ID portion of the stem or front-matter `id`, never the slug.

use crate::context::Context;
use crate::error::AppError;
use crate::record::identity::{ParsedId, parse_id_parts};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ResolvedRecord {
    pub path: PathBuf,
    /// Canonical ID of the record: the front-matter `id` when present,
    /// otherwise the canonical ID portion of the stem.
    pub id: String,
    pub stem_id: String,
    pub frontmatter_id: Option<String>,
    /// The stem and front-matter IDs disagree.
    pub mismatch: bool,
}

impl ResolvedRecord {
    /// A write is refused while the front-matter ID disagrees with the stem.
    pub fn ensure_writable(&self) -> Result<(), AppError> {
        if self.mismatch {
            return Err(AppError::Validation(format!(
                "front-matter/filename mismatch at {}: filename {} vs front matter {}",
                self.path.display(),
                self.stem_id,
                self.frontmatter_id.as_deref().unwrap_or_default()
            )));
        }
        Ok(())
    }
}

fn md_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(AppError::Runtime(e.to_string())),
    };
    let mut files = Vec::new();
    for entry in rd {
        let entry = entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(files)
}

pub fn frontmatter_id(content: &str) -> Option<String> {
    let fm = crate::record::frontmatter::parse(content).ok().flatten()?;
    let id = crate::record::frontmatter::current_value_string(&fm.raw, "id")?;
    let id = id.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn parse_any(value: &str, prefixes: &[String], dotted: bool) -> Option<(ParsedId, String)> {
    prefixes.iter().find_map(|prefix| {
        parse_id_parts(value, prefix, dotted).map(|parts| (parts, prefix.clone()))
    })
}

fn wanted_keys(input: &str, prefixes: &[String], dotted: bool) -> Vec<String> {
    let mut keys = Vec::new();
    for prefix in prefixes {
        if let Some(parts) = parse_id_parts(input, prefix, dotted) {
            let key = parts.key(prefix);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

fn describe_record(
    ctx: &Context,
    path: &Path,
    prefixes: &[String],
    dotted: bool,
) -> Result<Option<ResolvedRecord>, AppError> {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    let Some((stem_parts, stem_prefix)) = parse_any(stem, prefixes, dotted) else {
        return Ok(None);
    };
    let safe = Context::ensure_within(&ctx.project_dir, path)?;
    let content = fs::read_to_string(&safe).unwrap_or_default();
    let fm = frontmatter_id(&content)
        .and_then(|id| parse_any(&id, prefixes, dotted))
        .map(|(parts, prefix)| {
            let key = parts.key(&prefix);
            (parts.display, key)
        });
    let mismatch = fm
        .as_ref()
        .is_some_and(|(_, key)| *key != stem_parts.key(&stem_prefix));
    let (id, frontmatter_id) = match fm {
        Some((display, _)) => (display.clone(), Some(display)),
        None => (stem_parts.display.clone(), None),
    };
    Ok(Some(ResolvedRecord {
        path: safe,
        id,
        stem_id: stem_parts.display,
        frontmatter_id,
        mismatch,
    }))
}

/// Resolve an ID across the given family directories.
///
/// If more than one file carries the same canonical ID across the locations,
/// the record is a duplicate: resolution fails and writes are refused.
pub fn resolve_in_dirs(
    ctx: &Context,
    dirs: &[PathBuf],
    input: &str,
    prefixes: &[String],
    dotted: bool,
) -> Result<Option<ResolvedRecord>, AppError> {
    let wanted = wanted_keys(input, prefixes, dotted);
    if wanted.is_empty() {
        return Ok(None);
    }
    let mut found: Vec<ResolvedRecord> = Vec::new();
    for dir in dirs {
        for path in md_files(dir)? {
            let Some(record) = describe_record(ctx, &path, prefixes, dotted)? else {
                continue;
            };
            let stem_key = parse_any(
                record
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default(),
                prefixes,
                dotted,
            )
            .map(|(parts, prefix)| parts.key(&prefix));
            let fm_key = record.frontmatter_id.as_deref().and_then(|id| {
                parse_any(id, prefixes, dotted).map(|(parts, prefix)| parts.key(&prefix))
            });
            let matches = stem_key.as_ref().is_some_and(|k| wanted.contains(k))
                || fm_key.as_ref().is_some_and(|k| wanted.contains(k));
            if matches && !found.iter().any(|r| r.path == record.path) {
                found.push(record);
            }
        }
    }
    if found.len() > 1 {
        let id = found[0].id.clone();
        let paths = found
            .iter()
            .map(|r| r.path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AppError::Validation(format!(
            "duplicate id {}: [{}]",
            id, paths
        )));
    }
    Ok(found.pop())
}

/// Canonical entries of one directory, used by list commands.
pub fn list_in_dir(
    ctx: &Context,
    dir: &Path,
    prefixes: &[String],
    dotted: bool,
) -> Result<Vec<ResolvedRecord>, AppError> {
    let mut records = Vec::new();
    for path in md_files(dir)? {
        if let Some(record) = describe_record(ctx, &path, prefixes, dotted)? {
            records.push(record);
        }
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(records)
}

/// Report whether a family has duplicate canonical IDs across its locations.
pub fn duplicate_ids(
    ctx: &Context,
    dirs: &[PathBuf],
    prefixes: &[String],
    dotted: bool,
) -> Result<Vec<(String, Vec<PathBuf>)>, AppError> {
    let mut seen: Vec<(String, Vec<PathBuf>)> = Vec::new();
    for dir in dirs {
        for path in md_files(dir)? {
            let Some(record) = describe_record(ctx, &path, prefixes, dotted)? else {
                continue;
            };
            if let Some((_, paths)) = seen.iter_mut().find(|(id, _)| *id == record.id) {
                if !paths.contains(&record.path) {
                    paths.push(record.path);
                }
            } else {
                seen.push((record.id, vec![record.path]));
            }
        }
    }
    Ok(seen
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect())
}
