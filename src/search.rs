//! Read-only project-local free-text search (TASK-5).
//!
//! Implements the first increment of `papyrus search`: business and
//! non-functional requirement records plus their requirement group documents.
//! The behavior contract is `docs/specifications/search-requirements.md`.

use crate::CommandOutput;
use crate::SearchType;
use crate::commands::load_effective_config;
use crate::config::{Config, DocTypeEntry};
use crate::context::Context;
use crate::error::AppError;
use crate::requirement::parse_heading;
use std::fs;
use std::path::PathBuf;

/// Document type value reported for every hit in this increment.
const DOC_TYPE: &str = "doc";

/// Document type keys searched by the `doc` filter (spec section 2.1).
const DOC_KEYS: [&str; 2] = ["req", "nfr"];

/// Normalize and validate a user-supplied query (spec section 5.3).
///
/// Leading and trailing whitespace is trimmed; empty or whitespace-only
/// queries and queries containing NUL, CR, or LF are rejected with exit 4.
pub fn sanitize_query(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("query must not be empty".to_string()));
    }
    if trimmed.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        return Err(AppError::Validation(
            "query must not contain control characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

struct Hit {
    id: String,
    kind: &'static str,
    title: String,
    path: PathBuf,
}

/// Execute a search and render the requested output (spec section 4).
pub fn run(
    ctx: &Context,
    query_raw: &str,
    types: &[SearchType],
    limit: Option<usize>,
) -> Result<CommandOutput, AppError> {
    let query = sanitize_query(query_raw)?;
    let needle = query.to_lowercase();
    let cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;

    let mut hits = Vec::new();
    if types.is_empty() || types.contains(&SearchType::Doc) {
        for key in DOC_KEYS {
            collect_type(ctx, &cfg, key, &needle, &mut hits)?;
        }
    }

    hits.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.kind.cmp(b.kind))
            .then_with(|| a.path.cmp(&b.path))
    });
    if let Some(n) = limit {
        hits.truncate(n);
    }

    let text = hits
        .iter()
        .map(|hit| format!("{} {} {}", hit.id, hit.kind, hit.title))
        .collect::<Vec<_>>()
        .join("\n");

    let json = if ctx.json {
        let array: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "id": hit.id,
                    "type": hit.kind,
                    "title": hit.title,
                    "path": hit.path.display().to_string(),
                })
            })
            .collect();
        Some(serde_json::Value::Array(array))
    } else {
        None
    };

    Ok(CommandOutput { text, json })
}

fn collect_type(
    ctx: &Context,
    cfg: &Config,
    key: &str,
    needle: &str,
    hits: &mut Vec<Hit>,
) -> Result<(), AppError> {
    let Some(entry) = cfg.document_types.get(key) else {
        return Ok(());
    };
    if !entry.enabled {
        return Ok(());
    }
    let Some(dir) = active_dir(ctx, cfg, entry)? else {
        return Ok(());
    };
    let entries = fs::read_dir(&dir).map_err(|e| {
        AppError::Runtime(format!("cannot read directory {}: {}", dir.display(), e))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Boundary-check every candidate before reading it (spec section 5.2).
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        // Unreadable or non-UTF-8 records are skipped (spec section 2.12).
        let Ok(content) = fs::read_to_string(&safe) else {
            continue;
        };
        if !content.to_lowercase().contains(needle) {
            continue;
        }
        let (_, title) = parse_heading(&content);
        let title = if title.is_empty() {
            id.to_string()
        } else {
            title
        };
        hits.push(Hit {
            id: id.to_string(),
            kind: DOC_TYPE,
            title,
            path: safe,
        });
    }
    Ok(())
}

/// Resolve a configured type directory, returning `None` when it does not
/// exist (spec section 2.11). A symlink that resolves outside the project is a
/// boundary violation and propagates as exit code 1 (spec section 5.2).
fn active_dir(
    ctx: &Context,
    cfg: &Config,
    entry: &DocTypeEntry,
) -> Result<Option<PathBuf>, AppError> {
    let raw = ctx
        .project_dir
        .join(&cfg.doc_directory)
        .join(&entry.directory);
    if !raw.exists() {
        return Ok(None);
    }
    Context::ensure_within(&ctx.project_dir, &raw).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_trimmed() {
        assert_eq!(sanitize_query("  abc  ").unwrap(), "abc");
        assert_eq!(sanitize_query("a b").unwrap(), "a b");
    }

    #[test]
    fn empty_and_whitespace_only_queries_are_rejected() {
        for raw in ["", " ", "   ", "\t", "\u{00a0}"] {
            assert!(sanitize_query(raw).is_err(), "raw={raw:?}");
        }
    }

    #[test]
    fn control_characters_are_rejected() {
        for raw in ["a\0b", "a\rb", "a\nb"] {
            assert!(sanitize_query(raw).is_err(), "raw={raw:?}");
        }
    }

    #[test]
    fn path_separators_are_literal_search_characters() {
        assert_eq!(sanitize_query("a/b").unwrap(), "a/b");
        assert_eq!(sanitize_query("a\\b").unwrap(), "a\\b");
    }
}
