//! Browser record view model built from the shared record modules.
//!
//! Reading, parsing, identity, projection, and writing all live under
//! `crate::record/`. This module only reshapes the shared [`Document`] and
//! [`RecordView`] into the render-only [`Record`] view model and keeps the
//! browser's validation rules. The owning [`TypeDescriptor`] is carried on
//! every record; behaviour is selected by `DocumentModel`/`TypedHandler`, never
//! by a browser-local category enum or a slug literal.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::document::{self, Document};
use crate::record::identity;
use crate::record::view::RecordView;
use crate::registry::{
    ArtifactCategory as CategoryKind, ArtifactRegistry, DocumentModel, IdShape, StatusModel,
    TypeDescriptor, TypedHandler,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum accepted record ID length in bytes (specification section 7.1).
pub const MAX_ID_BYTES: usize = 128;
/// Maximum accepted title length in Unicode scalar values (section 7.3).
pub const MAX_TITLE_CHARS: usize = 200;
/// Maximum accepted story or description length (section 7.3).
pub const MAX_TEXT_CHARS: usize = 20000;

/// A loaded record with its owning descriptor.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub descriptor: TypeDescriptor,
    /// A grouped family's group record (`REQ-001`) rather than a member.
    pub is_group: bool,
    pub title: String,
    pub story: Option<String>,
    pub group_id: Option<String>,
    pub group_title: Option<String>,
    pub description: Option<String>,
    pub rationale: Option<String>,
    pub applicability: Option<String>,
    pub requirements: Vec<String>,
    pub audit_type: Option<String>,
    pub definition: Option<String>,
    pub view: Option<String>,
    pub status: Option<String>,
    pub ordinal: Option<i64>,
    pub milestone_id: Option<String>,
    pub milestone_title: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub comments: Vec<(String, String)>,
    pub release_id: Option<String>,
    pub release_title: Option<String>,
    pub decision: Option<String>,
    pub context: Option<String>,
    pub consequences: Option<String>,
    pub decision_type: Option<String>,
    pub superseded_by: Option<String>,
    pub superseded_title: Option<String>,
    pub specs: Vec<String>,
    pub refs: Vec<String>,
    pub mockup_task: String,
    pub doc: Document,
    /// Record content exactly as stored on disk, before line-ending
    /// normalization. It is the byte comparison baseline for a save.
    pub stored_source: String,
    pub path: PathBuf,
    pub rel_path: String,
    pub version: String,
}

impl Record {
    /// Registry slug of the record's type.
    pub fn slug(&self) -> &str {
        &self.descriptor.slug
    }

    /// Typed handler for built-in types; `None` for generic config types.
    pub fn handler(&self) -> Option<TypedHandler> {
        self.descriptor.handler()
    }

    pub fn is_generic(&self) -> bool {
        self.descriptor.schema.is_none()
    }

    pub fn is_html_mockup(&self) -> bool {
        self.descriptor.document_model == DocumentModel::SingleHtmlMockup
    }

    pub fn is_requirement(&self) -> bool {
        matches!(self.handler(), Some(TypedHandler::Requirement(_)))
    }

    pub fn supports_story(&self) -> bool {
        self.handler()
            == Some(TypedHandler::Requirement(
                crate::registry::RequirementKind::Business,
            ))
    }

    pub fn type_label(&self) -> String {
        crate::browser::routes::label(self.slug())
    }

    /// Stored status rendered read-only, or `(none)` when absent.
    pub fn status_text(&self) -> String {
        self.status
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("(none)")
            .to_string()
    }
}

/// An indexed document type. Records are sorted by ID ascending.
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
}

/// Resolve a descriptor's active directory against the project boundary.
pub fn descriptor_directory(
    ctx: &Context,
    cfg: &Config,
    descriptor: &TypeDescriptor,
) -> Option<PathBuf> {
    if !descriptor.enabled {
        return None;
    }
    let base = match descriptor.directory.config_key {
        "backlog_directory" => ctx.project_dir.join(&cfg.backlog_directory),
        "decision_directory" => ctx.project_dir.join(&cfg.decision_directory),
        "doc_directory" => ctx.project_dir.join(&cfg.doc_directory),
        _ => return None,
    };
    let raw = if descriptor.directory.relative.is_empty() {
        base
    } else {
        base.join(&descriptor.directory.relative)
    };
    Context::ensure_within(&ctx.project_dir, &raw).ok()
}

fn relative_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn section_or_none(doc: &Document, name: &str) -> Option<String> {
    document::section_text(doc, name)
}

fn list_from_section(doc: &Document, name: &str) -> Vec<String> {
    let Some(text) = document::section_text(doc, name) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix('-'))
            .unwrap_or(trimmed)
            .trim();
        if !item.is_empty() && !out.contains(&item.to_string()) {
            out.push(item.to_string());
        }
    }
    out
}

fn heading_title(doc: &Document) -> String {
    document::heading_id_title(doc).1
}

fn frontmatter_title(doc: &Document) -> String {
    document::heading_id_title_with_frontmatter(doc).1
}

fn non_empty_or(value: String, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn is_group_id(descriptor: &TypeDescriptor, stem: &str) -> bool {
    descriptor.id_shape == IdShape::Grouped
        && identity::parse_id_parts(stem, &descriptor.prefix, true).is_none()
        && identity::parse_id_parts(stem, &descriptor.prefix, false).is_some()
}

fn canonical_id(descriptor: &TypeDescriptor, value: &str) -> Option<String> {
    match descriptor.id_shape {
        IdShape::Grouped => identity::parse_id_parts(value, &descriptor.prefix, true)
            .or_else(|| identity::parse_id_parts(value, &descriptor.prefix, false))
            .map(|parts| parts.display),
        IdShape::Dotted => {
            identity::parse_id_parts(value, &descriptor.prefix, true).map(|parts| parts.display)
        }
        IdShape::Plain => {
            identity::parse_id_parts(value, &descriptor.prefix, false).map(|parts| parts.display)
        }
    }
}

/// Whether a file stem denotes a record of the descriptor's type.
pub fn matches_stem(descriptor: &TypeDescriptor, stem: &str) -> bool {
    identity::is_valid_id_component(stem) && canonical_id(descriptor, stem).is_some()
}

fn content_extension(descriptor: &TypeDescriptor) -> &'static str {
    match descriptor.document_model {
        DocumentModel::SingleHtmlMockup => "html",
        _ => "md",
    }
}

fn default_record(
    id: String,
    descriptor: TypeDescriptor,
    is_group: bool,
    doc: Document,
    path: PathBuf,
    rel_path: String,
    version: String,
) -> Record {
    Record {
        id,
        descriptor,
        is_group,
        title: String::new(),
        story: None,
        group_id: None,
        group_title: None,
        description: None,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        audit_type: None,
        definition: None,
        view: None,
        status: None,
        ordinal: None,
        milestone_id: None,
        milestone_title: None,
        assignee: None,
        labels: Vec::new(),
        dependencies: Vec::new(),
        acceptance_criteria: Vec::new(),
        comments: Vec::new(),
        release_id: None,
        release_title: None,
        decision: None,
        context: None,
        consequences: None,
        decision_type: None,
        superseded_by: None,
        superseded_title: None,
        specs: Vec::new(),
        refs: Vec::new(),
        mockup_task: String::new(),
        doc,
        stored_source: String::new(),
        path,
        rel_path,
        version,
    }
}

/// Load one managed record from the shared document parser and projection.
pub fn load_record(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    if descriptor.document_model == DocumentModel::SingleHtmlMockup {
        return load_ui_mockup(ctx, cfg, descriptor, path);
    }
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        Err(_) => return Ok(None),
    };
    let Some(stem) = safe
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
    else {
        return Ok(None);
    };
    if !matches_stem(descriptor, &stem) {
        return Ok(None);
    }
    if descriptor.category == CategoryKind::Decision
        && crate::decision::type_key_for_id(cfg, &stem).is_none()
    {
        return Ok(None);
    }
    let Ok(read) = crate::record::store::read_document_normalized(ctx, &safe) else {
        return Ok(None);
    };
    let id = canonical_id(descriptor, &stem).unwrap_or_else(|| stem.clone());
    let view = RecordView::from_content(&read.document.source, &id, &read.version).ok();
    let rel_path = relative_path(&ctx.project_dir, &safe);
    let version = read.version;
    let is_group = is_group_id(descriptor, &stem);
    let mut record = default_record(
        id,
        descriptor.clone(),
        is_group,
        read.document,
        safe,
        rel_path,
        version,
    );
    record.stored_source = read.stored_source;

    match record.handler() {
        Some(TypedHandler::Task) => {
            let td = crate::task::task_from_document(&record.doc);
            record.title = if td.title.is_empty() {
                stem.to_string()
            } else {
                td.title
            };
            record.description = td.description;
            record.status = (!td.status.is_empty()).then_some(td.status);
            record.ordinal = td.ordinal;
            record.milestone_id = td.milestone;
            record.assignee = td.assignee;
            record.labels = td.labels;
            record.dependencies = td.deps;
            record.acceptance_criteria = td.acs;
            record.comments = td.comments;
        }
        Some(TypedHandler::Milestone) => {
            record.title = non_empty_or(frontmatter_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.status = record
                .doc
                .front_matter
                .as_ref()
                .and_then(|fm| {
                    if crate::record::frontmatter::contains_key(&fm.raw, "status") {
                        crate::record::frontmatter::current_value_string(&fm.raw, "status")
                    } else {
                        None
                    }
                })
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| Some("open".to_string()));
            record.release_id = match record.doc.front_matter.as_ref() {
                Some(fm) if crate::record::frontmatter::contains_key(&fm.raw, "release") => {
                    crate::record::frontmatter::current_value_string(&fm.raw, "release")
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                }
                _ => section_or_none(&record.doc, "Release").map(|s| s.trim().to_string()),
            };
        }
        Some(TypedHandler::Release) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.status = record
                .doc
                .front_matter
                .as_ref()
                .and_then(|fm| {
                    if crate::record::frontmatter::contains_key(&fm.raw, "status") {
                        crate::record::frontmatter::current_value_string(&fm.raw, "status")
                    } else {
                        None
                    }
                })
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| Some("planned".to_string()));
        }
        Some(TypedHandler::Decision) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            let status = section_or_none(&record.doc, "Status").unwrap_or_default();
            record.status = Some(if status.trim().is_empty() {
                "proposed".to_string()
            } else {
                status.trim().to_string()
            });
            record.superseded_by = record
                .status
                .as_deref()
                .and_then(|s| s.strip_prefix("Superseded by "))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            record.decision_type = crate::decision::type_key_for_id(cfg, &stem);
            record.decision = section_or_none(&record.doc, "Decision");
            record.context = section_or_none(&record.doc, "Context");
            record.consequences = section_or_none(&record.doc, "Consequences");
        }
        Some(TypedHandler::Requirement(_)) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.story = section_or_none(&record.doc, "User Story");
            record.description = section_or_none(&record.doc, "Description");
            if let Some(group) = section_or_none(&record.doc, "Group") {
                let mut parts = group
                    .lines()
                    .next()
                    .unwrap_or("")
                    .splitn(2, char::is_whitespace);
                record.group_id = parts
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                record.group_title = parts
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
            }
        }
        Some(TypedHandler::Glossary) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.definition = section_or_none(&record.doc, "Definition");
        }
        Some(TypedHandler::UiSpec) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.view = section_or_none(&record.doc, "View");
            record.description = section_or_none(&record.doc, "Description");
        }
        Some(TypedHandler::Rule) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.rationale = section_or_none(&record.doc, "Rationale");
            record.applicability = section_or_none(&record.doc, "Applicability");
            record.requirements = list_from_section(&record.doc, "Requirements");
        }
        Some(TypedHandler::Audit) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            let fm_type =
                record.doc.front_matter.as_ref().and_then(|fm| {
                    crate::record::frontmatter::current_value_string(&fm.raw, "type")
                });
            record.audit_type = Some(
                fm_type
                    .filter(|t| !t.trim().is_empty())
                    .or_else(|| section_or_none(&record.doc, "Audit Type"))
                    .map(|t| t.trim().to_string())
                    .unwrap_or_else(|| "security".to_string()),
            );
            record.description = section_or_none(&record.doc, "Content");
        }
        Some(TypedHandler::TechSpec) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
        }
        Some(TypedHandler::UseCase)
        | Some(TypedHandler::TestCase)
        | Some(TypedHandler::TestReport) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.applicability = section_or_none(&record.doc, "Applicability");
        }
        Some(TypedHandler::UiMockup) => {}
        None => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
        }
    }

    // Every document type, including generic config-defined types, projects
    // the front-matter `labels` and `references` lists. A handler that already
    // owns a field (task labels, UI mockup references) keeps its value.
    if record.labels.is_empty() {
        record.labels = view
            .as_ref()
            .map(|v| v.list_value("labels"))
            .unwrap_or_default();
    }
    if record.refs.is_empty() {
        record.refs = view
            .as_ref()
            .map(|v| v.list_value("references"))
            .unwrap_or_default();
    }
    if record.status.is_none() {
        record.status = view
            .as_ref()
            .and_then(|v| v.str_value("status"))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    if let Some(mid) = record.milestone_id.clone() {
        record.milestone_title = resolve_title(ctx, cfg, registry, &mid);
    }
    if let Some(rid) = record.release_id.clone() {
        record.release_title = resolve_title(ctx, cfg, registry, &rid);
    }
    if let Some(sid) = record.superseded_by.clone() {
        record.superseded_title = resolve_title(ctx, cfg, registry, &sid);
    }
    Ok(Some(record))
}

fn resolve_title(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    id: &str,
) -> Option<String> {
    resolve_id(ctx, cfg, registry, id)
        .ok()
        .flatten()
        .map(|r| r.record.title)
}

fn load_ui_mockup(
    ctx: &Context,
    cfg: &Config,
    descriptor: &TypeDescriptor,
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
    if !identity::is_valid_id_component(stem) || !matches_stem(descriptor, stem) {
        return Ok(None);
    }
    let meta_record = crate::ui_mockup::parse_browser_metadata(&content, stem);
    let version = crate::record::store::version_token(&meta);
    let id = canonical_id(descriptor, stem).unwrap_or_else(|| stem.to_string());
    let doc = Document {
        source: content,
        front_matter: None,
        frontmatter: None,
        heading: None,
        preamble: Vec::new(),
        sections: Vec::new(),
    };
    let rel_path = relative_path(&ctx.project_dir, &safe);
    let mut record = default_record(id, descriptor.clone(), false, doc, safe, rel_path, version);
    record.title = meta_record.title.clone();
    record.status = (!meta_record.status.is_empty()).then_some(meta_record.status.clone());
    record.superseded_by = meta_record.superseded_by.clone();
    record.specs = meta_record.specs.clone();
    record.refs = meta_record.refs.clone();
    record.mockup_task = meta_record.task.clone();
    record.stored_source = record.doc.source.clone();
    let _ = cfg;
    Ok(Some(record))
}

/// The filterable status values offered by a descriptor, in order.
pub fn filter_statuses(descriptor: &TypeDescriptor) -> Vec<String> {
    match &descriptor.status_model {
        StatusModel::DocumentMain { custom, .. } => {
            let mut values = vec!["active".to_string()];
            values.extend(custom.iter().cloned());
            values
        }
        StatusModel::HtmlMetadata { states } => states
            .iter()
            .filter(|value| **value != "archived")
            .map(|value| (*value).to_string())
            .collect(),
        StatusModel::TaskConfigurable { statuses, .. } => statuses.clone(),
        StatusModel::Fixed { states } => states.iter().map(|value| (*value).to_string()).collect(),
    }
}

/// Build the index for one descriptor. Only the active directory is read.
pub fn build_index(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    status: Option<&str>,
) -> Index {
    let mut index = Index::default();
    if !descriptor.enabled {
        return index;
    }
    let Some(dir) = descriptor_directory(ctx, cfg, descriptor) else {
        return index;
    };
    let extension = content_extension(descriptor);
    let Ok(read_dir) = fs::read_dir(&dir) else {
        return index;
    };
    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some(extension) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !matches_stem(descriptor, stem) {
            continue;
        }
        if let Ok(Some(record)) = load_record(ctx, cfg, registry, descriptor, &path) {
            // A record auto-relocated out of the active directory by the read
            // path is no longer listed.
            if !record.path.exists() {
                continue;
            }
            if let Some(wanted) = status
                && record.status.as_deref() != Some(wanted)
            {
                continue;
            }
            index.records.push(record);
        }
    }
    index.records.sort_by(|a, b| a.id.cmp(&b.id));
    if descriptor.id_shape != IdShape::Grouped {
        return index;
    }
    index.groups = index
        .records
        .iter()
        .filter(|r| r.is_group)
        .cloned()
        .collect();
    index.requirements = index
        .records
        .iter()
        .filter(|r| !r.is_group)
        .cloned()
        .collect();

    let group_ids: BTreeMap<String, usize> = index
        .groups
        .iter()
        .enumerate()
        .map(|(position, group)| (group.id.clone(), position))
        .collect();
    for (position, requirement) in index.requirements.iter().enumerate() {
        let group_id = requirement.group_id.clone().unwrap_or_default();
        if group_id.is_empty() || !group_ids.contains_key(&group_id) {
            index.other.push(position);
        } else {
            index.members.entry(group_id).or_default().push(position);
        }
    }
    index
}

/// Resolve an ID by exact canonical match against the enabled descriptors.
pub fn resolve_id(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    id: &str,
) -> Result<Option<Resolved>, AppError> {
    if !identity::is_valid_id_component(id) {
        return Ok(None);
    }
    for descriptor in registry.descriptors() {
        if !descriptor.enabled {
            continue;
        }
        let Some(dir) = descriptor_directory(ctx, cfg, descriptor) else {
            continue;
        };
        if descriptor.document_model == DocumentModel::SingleHtmlMockup {
            let Some(wanted) = canonical_id(descriptor, id) else {
                continue;
            };
            for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("html") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if canonical_id(descriptor, stem).as_ref() == Some(&wanted)
                    && Context::ensure_within(&ctx.project_dir, &path).is_ok()
                    && let Some(record) = load_record(ctx, cfg, registry, descriptor, &path)?
                {
                    if !record.path.exists() {
                        continue;
                    }
                    return Ok(Some(Resolved { record }));
                }
            }
            continue;
        }
        if canonical_id(descriptor, id).is_none() {
            continue;
        }
        let dirs = vec![dir];
        let prefixes = vec![descriptor.prefix.clone()];
        let dotted = descriptor.id_shape != IdShape::Plain;
        let mut resolved =
            crate::record::resolve::resolve_in_dirs(ctx, &dirs, id, &prefixes, dotted)
                .ok()
                .flatten();
        if resolved.is_none() && dotted {
            resolved = crate::record::resolve::resolve_in_dirs(ctx, &dirs, id, &prefixes, false)
                .ok()
                .flatten();
        }
        if let Some(found) = resolved {
            if let Some(record) = load_record(ctx, cfg, registry, descriptor, &found.path)? {
                if !record.path.exists() {
                    continue;
                }
                return Ok(Some(Resolved { record }));
            }
            let safe = Context::ensure_within(&ctx.project_dir, &found.path)
                .map_err(|_| AppError::Runtime("record is not readable".to_string()))?;
            if fs::read_to_string(&safe).is_err() {
                return Err(AppError::Runtime("record is not valid UTF-8".to_string()));
            }
        }
    }
    Ok(None)
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
    let normalized = crate::record::line_endings::unify(raw);
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

/// Validate a list field (one value per line) and return the parsed items.
pub fn validate_list_items(label: &str, raw: &str) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    let normalized = crate::record::line_endings::unify(raw);
    for line in normalized.lines() {
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

fn html_escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn title_inner_span(content: &str) -> Option<(usize, usize, bool)> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<title") {
        let open_start = search + rel;
        let after_open = open_start + "<title".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<title".len();
            continue;
        }
        let gt = content[open_start..].find('>')?;
        let inner_start = open_start + gt + 1;
        let close_rel = content[inner_start..].find("</title>")?;
        let inner_end = inner_start + close_rel;
        let raw = &content[inner_start..inner_end];
        let dash = raw.find('—');
        let entity = raw.find("&mdash;");
        let uses_entity = match (dash, entity) {
            (Some(d), Some(e)) => e < d,
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => false,
        };
        return Some((inner_start, inner_end, uses_entity));
    }
    None
}

fn stub_h1_inner_span(content: &str, id: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<h1") {
        let open_start = search + rel;
        let after_open = open_start + "<h1".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<h1".len();
            continue;
        }
        let gt_rel = content[open_start..].find('>')?;
        let inner_start = open_start + gt_rel + 1;
        let close_rel = content[inner_start..].find("</h1>")?;
        let inner_end = inner_start + close_rel;
        let inner = &content[inner_start..inner_end];
        let follows = inner
            .strip_prefix(id)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| c.is_whitespace());
        if follows {
            return Some((inner_start, inner_end));
        }
        search = inner_end + "</h1>".len();
    }
    None
}

fn comment_span(content: &str, key: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    let prefix = format!("{}:", key);
    while let Some(rel) = content[search..].find("<!--") {
        let start = search + rel;
        let end_rel = content[start..].find("-->")?;
        let inner_start = start + "<!--".len();
        let inner_end = start + end_rel;
        let inner = content[inner_start..inner_end].trim();
        if inner.starts_with(&prefix) {
            return Some((start, inner_end + "-->".len()));
        }
        search = inner_end + "-->".len();
    }
    None
}

fn visible_para_inner_span(content: &str, label: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    let prefix = format!("{}:", label);
    while let Some(rel) = content[search..].find("<p") {
        let open_start = search + rel;
        let after_open = open_start + "<p".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<p".len();
            continue;
        }
        let gt_rel = content[open_start..].find('>')?;
        let inner_start = open_start + gt_rel + 1;
        let close_rel = content[inner_start..].find("</p>")?;
        let inner_end = inner_start + close_rel;
        let inner = content[inner_start..inner_end].trim();
        if inner.starts_with(&prefix) {
            return Some((inner_start, inner_end));
        }
        search = inner_end + "</p>".len();
    }
    None
}

fn body_open_end(content: &str) -> Option<usize> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<body") {
        let open_start = search + rel;
        let after_open = open_start + "<body".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<body".len();
            continue;
        }
        let gt = content[open_start..].find('>')?;
        return Some(open_start + gt + 1);
    }
    None
}

/// Field-preserving serialization for UI mockups (specification section 8).
pub fn edit_ui_mockup(
    original: &str,
    id: &str,
    title: &str,
    specs: &[String],
    refs: &[String],
) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    if let Some((start, end, uses_entity)) = title_inner_span(original) {
        let separator = if uses_entity { "&mdash;" } else { "—" };
        edits.push((
            start,
            end,
            format!("{} {} {}", id, separator, html_escape_text(title)),
        ));
    }
    if let Some((start, end)) = stub_h1_inner_span(original, id) {
        edits.push((start, end, format!("{} {}", id, html_escape_text(title))));
    }
    let specs_comment = comment_span(original, "specs");
    let refs_comment = comment_span(original, "refs");
    if let Some((start, end)) = specs_comment {
        edits.push((start, end, format!("<!-- specs: {} -->", specs.join(" "))));
    }
    if let Some((start, end)) = refs_comment {
        edits.push((start, end, format!("<!-- refs: {} -->", refs.join(" "))));
    }
    if let Some((start, end)) = visible_para_inner_span(original, "UI specifications") {
        edits.push((
            start,
            end,
            format!("UI specifications: {}", html_escape_text(&specs.join(" "))),
        ));
    }
    if let Some((start, end)) = visible_para_inner_span(original, "References") {
        edits.push((
            start,
            end,
            format!("References: {}", html_escape_text(&refs.join(" "))),
        ));
    }
    let missing_specs = specs_comment.is_none();
    let missing_refs = refs_comment.is_none();
    if missing_specs || missing_refs {
        let mut block = String::new();
        if missing_specs {
            block.push_str(&format!("<!-- specs: {} -->\n", specs.join(" ")));
        }
        if missing_refs {
            block.push_str(&format!("<!-- refs: {} -->\n", refs.join(" ")));
        }
        match body_open_end(original) {
            Some(pos) => edits.push((pos, pos, format!("\n{}", block))),
            None => {
                let separator = if original.is_empty() || original.ends_with('\n') {
                    String::new()
                } else {
                    "\n".to_string()
                };
                edits.push((
                    original.len(),
                    original.len(),
                    format!("{}{}", separator, block),
                ));
            }
        }
    }
    edits.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut output = original.to_string();
    for (start, end, replacement) in edits {
        output.replace_range(start..end, &replacement);
    }
    output
}

/// Trimmed, non-empty submitted tag values in order (for 422 preservation).
pub fn submitted_tag_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Validate `specs` values (specification section 7.2).
pub fn validate_specs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    if trimmed.is_empty() {
        return Err("At least one UI specification id is required.".to_string());
    }
    for value in &trimmed {
        if !identity::is_valid_id_component(value) {
            return Err("UI specification ids must be valid record IDs.".to_string());
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for value in &trimmed {
        if seen.contains(value) {
            return Err(format!("Duplicate UI specification id: {}.", value));
        }
        seen.push(value.clone());
    }
    Ok(trimmed)
}

/// Validate `refs` values (specification section 7.3).
pub fn validate_refs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    for value in &trimmed {
        if value.chars().count() > 200 {
            return Err("References must be at most 200 characters.".to_string());
        }
        if value.chars().any(|c| c.is_control()) {
            return Err("References cannot contain control characters.".to_string());
        }
        if value.chars().any(|c| c.is_whitespace()) {
            return Err("References cannot contain whitespace.".to_string());
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for value in &trimmed {
        if seen.contains(value) {
            return Err(format!("Duplicate reference: {}.", value));
        }
        seen.push(value.clone());
    }
    Ok(trimmed)
}

/// Validate labels (front-matter list): collapse duplicates, 1-200 chars, no control.
pub fn validate_labels(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    let mut out: Vec<String> = Vec::new();
    for value in trimmed {
        if value.chars().count() > MAX_TITLE_CHARS {
            return Err("Labels must be at most 200 characters.".to_string());
        }
        if value.chars().any(|c| c.is_control()) {
            return Err("Labels cannot contain control characters.".to_string());
        }
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub fn validate_requirement_refs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    for value in &trimmed {
        if !identity::is_valid_id_component(value) {
            return Err("Requirement references must be valid record IDs.".to_string());
        }
    }
    let mut out: Vec<String> = Vec::new();
    for value in trimmed {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

/// Whether a decision ID exists in the active or archived store.
pub fn decision_exists(ctx: &Context, cfg: &Config, registry: &ArtifactRegistry, id: &str) -> bool {
    registry.descriptors().any(|descriptor| {
        descriptor.category == CategoryKind::Decision
            && descriptor.enabled
            && descriptor_directory(ctx, cfg, descriptor).is_some_and(|active| {
                let archived = active.join("archived");
                let dirs = if Context::ensure_within(&ctx.project_dir, &archived).is_ok() {
                    vec![active, archived]
                } else {
                    vec![active]
                };
                crate::record::resolve::resolve_in_dirs(
                    ctx,
                    &dirs,
                    id,
                    std::slice::from_ref(&descriptor.prefix),
                    descriptor.id_shape != IdShape::Plain,
                )
                .ok()
                .flatten()
                .is_some()
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_and_multiline_validation() {
        assert!(validate_title("  Title  ").is_ok());
        assert_eq!(validate_title("").unwrap_err(), "Title is required.");
        assert!(validate_title(&"x".repeat(201)).is_err());
        assert_eq!(
            validate_title(&"x".repeat(201)).unwrap_err(),
            "Title must be at most 200 characters."
        );
        assert_eq!(
            validate_title("a\tb").unwrap_err(),
            "Title cannot contain control characters."
        );
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert_eq!(
            validate_multiline("Description", "a\n## Section\nb").unwrap_err(),
            "Description contains a line starting with ## ."
        );
        assert_eq!(
            validate_multiline("Content", "a\u{0007}b").unwrap_err(),
            "Content cannot contain control characters."
        );
        assert!(validate_multiline("Description", "a\tb").is_ok());
    }

    #[test]
    fn list_validation() {
        assert_eq!(
            validate_list_items("Labels", "one\n two \none\n").unwrap(),
            vec!["one".to_string(), "two".to_string()]
        );
        assert_eq!(
            validate_list_items("Labels", "## section").unwrap_err(),
            "Labels contains a line starting with ## ."
        );
    }

    #[test]
    fn ui_mockup_specs_validation() {
        assert_eq!(
            validate_specs(&["UI-001".to_string(), " UI-002 ".to_string()]).unwrap(),
            vec!["UI-001".to_string(), "UI-002".to_string()]
        );
        assert_eq!(
            validate_specs(&["UI-001".to_string(), "".to_string(), "  ".to_string()]).unwrap(),
            vec!["UI-001".to_string()]
        );
        assert_eq!(
            validate_specs(&[]).unwrap_err(),
            "At least one UI specification id is required."
        );
        assert_eq!(
            validate_specs(&["bad id".to_string()]).unwrap_err(),
            "UI specification ids must be valid record IDs."
        );
        assert_eq!(
            validate_specs(&["UI-001".to_string(), "UI-001".to_string()]).unwrap_err(),
            "Duplicate UI specification id: UI-001."
        );
    }

    #[test]
    fn ui_mockup_refs_validation() {
        assert_eq!(
            validate_refs(&["docs/a.md".to_string()]).unwrap(),
            vec!["docs/a.md".to_string()]
        );
        assert_eq!(
            validate_refs(&["x".repeat(201)]).unwrap_err(),
            "References must be at most 200 characters."
        );
        assert_eq!(
            validate_refs(&["a b".to_string()]).unwrap_err(),
            "References cannot contain whitespace."
        );
        assert_eq!(
            validate_refs(&["docs/a.md".to_string(), "docs/a.md".to_string()]).unwrap_err(),
            "Duplicate reference: docs/a.md."
        );
    }

    #[test]
    fn ui_mockup_stub_round_trip() {
        let original = "<!doctype html>\n<html><head><title>MOCK-001 — Old</title></head><body>\n<!-- mockup-id: MOCK-001 -->\n<!-- status: pending -->\n<!-- task: T -->\n<!-- specs: UI-001 -->\n<!-- refs: docs/old.md -->\n<h1>MOCK-001 Old</h1>\n<p>Status: pending</p>\n<p>Task: T</p>\n<p>UI specifications: UI-001</p>\n<p>References: docs/old.md</p>\n<p>Superseded by: </p>\n</body></html>\n";
        let output = edit_ui_mockup(
            original,
            "MOCK-001",
            "New",
            &["UI-002".to_string()],
            &["docs/new.md".to_string()],
        );
        assert!(output.contains("<title>MOCK-001 — New</title>"));
        assert!(output.contains("<h1>MOCK-001 New</h1>"));
        assert!(output.contains("<!-- specs: UI-002 -->"));
        assert!(output.contains("<!-- refs: docs/new.md -->"));
        assert!(output.contains("<p>UI specifications: UI-002</p>"));
        assert!(output.contains("<p>References: docs/new.md</p>"));
        assert!(output.contains("<!-- mockup-id: MOCK-001 -->"));
        assert!(output.contains("<p>Superseded by: </p>"));
    }

    #[test]
    fn ui_mockup_review_package_preserved() {
        let original = "<!doctype html>\n<html><head><title>MOCK-003 &mdash; Old review</title><style>body{color:red}</style><script>var s=1;</script></head><body class=\"x\">\n<h1>TASK-99 Landing</h1>\n<section>Slides &amp; more</section>\n</body></html>\n";
        let output = edit_ui_mockup(
            original,
            "MOCK-003",
            "New review",
            &["UI-010".to_string()],
            &["docs/x.md".to_string()],
        );
        assert!(output.contains("<title>MOCK-003 &mdash; New review</title>"));
        assert!(output.contains("<style>body{color:red}</style>"));
        assert!(output.contains("<script>var s=1;</script>"));
        assert!(output.contains("<h1>TASK-99 Landing</h1>"));
        assert!(output.contains("<!-- specs: UI-010 -->"));
        assert!(output.contains("<!-- refs: docs/x.md -->"));
    }
}
