//! Read-only project-local free-text search (TASK-5 + TASK-10).

use crate::CommandOutput;
use crate::SearchType;
use crate::commands::load_effective_config;
use crate::config::{Config, DocTypeEntry};
use crate::context::Context;
use crate::error::AppError;
use std::fs;
use std::path::PathBuf;

const DOC_TYPE: &str = "doc";
const DOC_KEYS: [&str; 11] = [
    "req",
    "nfr",
    "glossary",
    "ui-spec",
    "ui-mockup",
    "rule",
    "audit",
    "tech-spec",
    "use-case",
    "test-case",
    "test-report",
];

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
        collect_backlog_store(ctx, &cfg, "task", "task", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Milestone) {
        collect_backlog_store(ctx, &cfg, "milestone", "milestone", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Release) {
        collect_backlog_store(ctx, &cfg, "release", "release", &needle, &mut hits)?;
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
        let allowed = match key {
            "ui-mockup" => path.extension().and_then(|s| s.to_str()) == Some("html"),
            _ => path.extension().and_then(|s| s.to_str()) == Some("md"),
        };
        if !allowed {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Only native stems are records; a document family keeps its dotted
        // shape and may carry a legacy kebab slug, which is not part of the ID.
        let dotted = matches!(key, "req" | "nfr");
        let parsed =
            crate::record::identity::parse_id_parts(stem, &entry.prefix, dotted).or_else(|| {
                dotted
                    .then(|| crate::record::identity::parse_id_parts(stem, &entry.prefix, false))
                    .flatten()
            });
        let Some(id) = parsed.map(|parts| parts.display) else {
            continue;
        };
        let id = id.as_str();
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        let Ok(content) = fs::read_to_string(&safe) else {
            continue;
        };
        if !content.to_lowercase().contains(needle) {
            continue;
        }
        let title = if key == "ui-mockup" {
            crate::ui_mockup::extract_title(&content, id)
        } else {
            let heading = crate::record::document::parse(&content)
                .ok()
                .and_then(|d| d.heading)
                .unwrap_or_default();
            let rest = heading.trim_start_matches('#').trim();
            let mut parts = rest.splitn(2, char::is_whitespace);
            let _ = parts.next();
            let t = parts.next().unwrap_or("").trim().to_string();
            if t.is_empty() { id.to_string() } else { t }
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
    descriptor_name: &str,
    kind: &'static str,
    needle: &str,
    hits: &mut Vec<Hit>,
) -> Result<(), AppError> {
    let Some(desc) = crate::record::descriptor::descriptor_for(descriptor_name) else {
        return Ok(());
    };
    let dir = match desc.resolve_directory(cfg, &ctx.project_dir) {
        Ok(dir) if dir.exists() => dir,
        _ => return Ok(()),
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
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(id) = desc.configured_prefixes(cfg).iter().find_map(|prefix| {
            crate::record::identity::parse_id_parts(stem, prefix, desc.is_dotted_id())
                .map(|parts| parts.display)
        }) else {
            continue;
        };
        let id = id.as_str();
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        let Ok(content) = fs::read_to_string(&safe) else {
            continue;
        };
        if !content.to_lowercase().contains(needle) {
            continue;
        }
        let title = crate::record::document::parse(&content)
            .ok()
            .map(|d| crate::record::document::heading_id_title_with_frontmatter(&d).1)
            .unwrap_or_default();
        let title = if title.trim().is_empty() {
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
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Boundary check before any eligibility/read, so symlink escapes
        // always surface as exit 1 regardless of type mapping.
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        // Canonical ID: the slug is cosmetic and never part of the identity.
        let Some(id) = crate::decision::canonical_id(cfg, stem) else {
            continue;
        };
        let id = id.as_str();
        let Ok(content) = fs::read_to_string(&safe) else {
            continue;
        };
        if !content.to_lowercase().contains(needle) {
            continue;
        }
        let title = crate::record::document::parse(&content)
            .ok()
            .map(|d| crate::record::document::heading_id_title_with_frontmatter(&d).1)
            .unwrap_or_default();
        let title = if title.trim().is_empty() {
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
