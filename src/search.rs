//! Read-only project-local free-text search (TASK-5 + TASK-10).
//!
//! The searched document set is the enabled `Document` descriptors of the
//! startup artifact registry (ADR-0017, TASK-113). No module-local document
//! slug list selects or shapes a document type.

use crate::CommandOutput;
use crate::SearchType;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::registry::{ArtifactCategory, ArtifactRegistry, DocumentModel, IdShape, TypeDescriptor};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

const DOC_TYPE: &str = "doc";

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
    cfg: &Config,
    registry: &ArtifactRegistry,
    query_raw: &str,
    types: &[SearchType],
    limit: Option<usize>,
) -> Result<CommandOutput, AppError> {
    let query = sanitize_query(query_raw)?;
    let needle = query.to_lowercase();

    let mut hits = Vec::new();
    if types.is_empty() || types.contains(&SearchType::Doc) {
        for descriptor in document_type_descriptors(registry) {
            collect_document_type(ctx, cfg, registry, descriptor, &needle, &mut hits)?;
        }
    }
    if types.is_empty() || types.contains(&SearchType::Task) {
        collect_backlog_store(ctx, cfg, "task", "task", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Milestone) {
        collect_backlog_store(ctx, cfg, "milestone", "milestone", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Release) {
        collect_backlog_store(ctx, cfg, "release", "release", &needle, &mut hits)?;
    }
    if types.is_empty() || types.contains(&SearchType::Decision) {
        collect_decision_store(ctx, cfg, &needle, &mut hits)?;
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

    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

/// The collected document-type set: exactly the enabled `Document` descriptors,
/// in registry order (Behavior 1.1).
fn document_type_descriptors(registry: &ArtifactRegistry) -> Vec<&TypeDescriptor> {
    registry.enabled(ArtifactCategory::Document).collect()
}

/// Index one descriptor's active directory with descriptor-driven extension,
/// identity shape, and title projection (Behavior 2).
fn collect_document_type(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    needle: &str,
    hits: &mut Vec<Hit>,
) -> Result<(), AppError> {
    let Some(active) = document_active_dir(ctx, cfg, descriptor)? else {
        return Ok(());
    };
    let extension = document_extension(descriptor);
    let entries = fs::read_dir(&active).map_err(|e| {
        AppError::Runtime(format!("cannot read directory {}: {}", active.display(), e))
    })?;
    for dir_entry in entries {
        let dir_entry = dir_entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = dir_entry.path();
        // Only files directly inside the active directory; subdirectories such
        // as the archived folder are never scanned (Behavior 2.2).
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(OsStr::to_str) != Some(extension) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
            continue;
        };
        let Some(stem_id) = record_id(stem, descriptor) else {
            continue;
        };
        // Boundary check before any eligibility decision or read, so symlink
        // escapes always surface as exit 1.
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        // Read-path auto-relocation required by REQ-005.007. Failure is a
        // validation outcome naming the source record (Behavior 4.2).
        let safe = crate::record::store::relocate_if_archived(ctx, cfg, registry, &safe)
            .map_err(|error| AppError::Validation(error.to_string()))?;
        // A relocated record left the active directory and is not returned.
        if safe.parent() != Some(active.as_path()) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&safe) else {
            continue;
        };
        // An archived UI mockup is excluded by its HTML status without
        // relocation (Assumption A2).
        if descriptor.document_model == DocumentModel::SingleHtmlMockup
            && crate::ui_mockup::parse_record(&content, &stem_id).status == "archived"
        {
            continue;
        }
        if !content.to_lowercase().contains(needle) {
            continue;
        }
        let (id, title) = document_identity_and_title(descriptor, &content, &stem_id);
        hits.push(Hit {
            id,
            kind: DOC_TYPE,
            title,
            path: safe,
        });
    }
    Ok(())
}

/// A document descriptor's active directory: `doc_directory` plus the
/// descriptor's relative directory, contained by the project boundary.
fn document_active_dir(
    ctx: &Context,
    cfg: &Config,
    descriptor: &TypeDescriptor,
) -> Result<Option<PathBuf>, AppError> {
    let base = match descriptor.directory.config_key {
        "doc_directory" => cfg.doc_directory.as_str(),
        other => {
            return Err(AppError::Validation(format!(
                "unsupported document directory key: {}",
                other
            )));
        }
    };
    let raw = ctx
        .project_dir
        .join(base)
        .join(&descriptor.directory.relative);
    if !raw.exists() {
        return Ok(None);
    }
    Context::ensure_within(&ctx.project_dir, &raw).map(Some)
}

fn document_extension(descriptor: &TypeDescriptor) -> &'static str {
    match descriptor.document_model {
        DocumentModel::SingleHtmlMockup => "html",
        DocumentModel::SingleMarkdown | DocumentModel::GroupedMarkdown => "md",
    }
}

/// Parse a record stem with the descriptor's prefix and ID shape (Behavior 2.4).
fn record_id(stem: &str, descriptor: &TypeDescriptor) -> Option<String> {
    let prefix = &descriptor.prefix;
    let parsed = match descriptor.id_shape {
        IdShape::Grouped => crate::record::identity::parse_id_parts(stem, prefix, true)
            .or_else(|| crate::record::identity::parse_id_parts(stem, prefix, false)),
        IdShape::Dotted => crate::record::identity::parse_id_parts(stem, prefix, true),
        IdShape::Plain => crate::record::identity::parse_id_parts(stem, prefix, false),
    };
    parsed.map(|parts| parts.display)
}

/// Identity and title from the shared list/view projection (Behavior 2.6).
fn document_identity_and_title(
    descriptor: &TypeDescriptor,
    content: &str,
    stem_id: &str,
) -> (String, String) {
    if descriptor.document_model == DocumentModel::SingleHtmlMockup {
        return (
            stem_id.to_string(),
            crate::ui_mockup::extract_title(content, stem_id),
        );
    }
    let projected = crate::record::document::parse(content)
        .ok()
        .map(|doc| crate::record::document::heading_id_title_with_frontmatter(&doc));
    let (id, title) = match projected {
        Some((id, title)) => (id.trim().to_string(), title.trim().to_string()),
        None => (String::new(), String::new()),
    };
    let id = if id.is_empty() {
        stem_id.to_string()
    } else {
        id
    };
    let title = if title.is_empty() { id.clone() } else { title };
    (id, title)
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
        if path.extension().and_then(OsStr::to_str) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
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
        if path.extension().and_then(OsStr::to_str) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
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

    fn note_entry(prefix: &str, directory: &str, enabled: bool) -> crate::config::DocTypeEntry {
        crate::config::DocTypeEntry {
            prefix: prefix.to_string(),
            directory: directory.to_string(),
            enabled,
            statuses: None,
            default_status: None,
            ..Default::default()
        }
    }

    // T-113-13: the collected type set equals the enabled Document descriptors.
    #[test]
    fn collected_document_types_equal_enabled_registry_descriptors() {
        let mut cfg = Config::default();
        cfg.document_types
            .insert("note".to_string(), note_entry("NOTE", "notes/", true));
        cfg.document_types
            .insert("shelved".to_string(), note_entry("SHLV", "shelved/", false));
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");

        let collected: Vec<&str> = document_type_descriptors(&registry)
            .iter()
            .map(|descriptor| descriptor.slug.as_str())
            .collect();
        let expected: Vec<&str> = registry
            .enabled(ArtifactCategory::Document)
            .map(|descriptor| descriptor.slug.as_str())
            .collect();

        assert_eq!(collected, expected);
        assert!(collected.contains(&"note"));
        assert!(!collected.contains(&"shelved"));
    }

    // T-113-13: no static document key list or slug arm remains in this module.
    #[test]
    fn search_module_has_no_static_document_key_list_or_slug_arms() {
        let source = include_str!("search.rs");
        let forbidden_key_list = ["DOC", "_KEYS"].concat();
        let forbidden_collector = ["collect", "_type"].concat();
        assert!(!source.contains(&forbidden_key_list));
        assert!(!source.contains(&forbidden_collector));
    }
}
