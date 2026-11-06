//! Read-only project-local free-text search (TASK-5 + TASK-10).

use crate::CommandOutput;
use crate::SearchType;
use crate::commands::load_effective_config;
use crate::config::{Config, DocTypeEntry};
use crate::context::Context;
use crate::error::AppError;
use crate::requirement::parse_heading;
use std::fs;
use std::path::PathBuf;

const DOC_TYPE: &str = "doc";
const DOC_KEYS: [&str; 5] = ["req", "nfr", "glossary", "ui-spec", "ui-mockup"];

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
    if types.is_empty() || types.contains(&SearchType::Task) {
        collect_backlog_store(ctx, &cfg, "tasks", "task", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Milestone) {
        collect_backlog_store(ctx, &cfg, "milestones", "milestone", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Release) {
        collect_backlog_store(ctx, &cfg, "releases", "release", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Decision) {
        collect_decision_store(ctx, &cfg, &needle, &mut hits)?;
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
    for dir_entry in entries {
        let dir_entry = dir_entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // UI specifications and UI mockups share their directories with other
        // documents (UI descriptions, HTML review packages); only `<prefix>-<digits>`
        // files are records (BR-012, BR-016).
        if key == "ui-spec" || key == "ui-mockup" {
            let prefix = format!("{}-", entry.prefix);
            let Some(rest) = id.strip_prefix(&prefix) else {
                continue;
            };
            if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
        }
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
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

fn collect_backlog_store(
    ctx: &Context,
    cfg: &Config,
    folder: &str,
    kind: &'static str,
    needle: &str,
    hits: &mut Vec<Hit>,
) -> Result<(), AppError> {
    let backlog_raw = ctx.project_dir.join(&cfg.backlog_directory);
    let backlog_safe = Context::ensure_within(&ctx.project_dir, &backlog_raw)?;
    let dir_raw = backlog_safe.join(folder);
    if !dir_raw.exists() {
        return Ok(());
    }
    let dir = Context::ensure_within(&ctx.project_dir, &dir_raw)?;
    let entries = fs::read_dir(&dir).map_err(|e| {
        AppError::Runtime(format!("cannot read directory {}: {}", dir.display(), e))
    })?;
    for dir_entry in entries {
        let dir_entry = dir_entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
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
            kind,
            title,
            path: safe,
        });
    }
    Ok(())
}

fn collect_decision_store(
    ctx: &Context,
    cfg: &Config,
    needle: &str,
    hits: &mut Vec<Hit>,
) -> Result<(), AppError> {
    let dir_raw = ctx.project_dir.join(&cfg.decision_directory);
    if !dir_raw.exists() {
        return Ok(());
    }
    let dir = Context::ensure_within(&ctx.project_dir, &dir_raw)?;
    let entries = fs::read_dir(&dir).map_err(|e| {
        AppError::Runtime(format!("cannot read directory {}: {}", dir.display(), e))
    })?;
    for dir_entry in entries {
        let dir_entry = dir_entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = dir_entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Boundary check before any eligibility/read, so symlink escapes
        // always surface as exit 1 regardless of type mapping.
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        // Only enabled decision types (delegate to decision module for
        // consistent prefix matching, including hyphenated prefixes).
        let Some(_type_key) = crate::decision::type_key_for_id(cfg, id) else {
            continue;
        };
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
            kind: "decision",
            title,
            path: safe,
        });
    }
    Ok(())
}

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
