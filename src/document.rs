//! Generic, descriptor-driven document handler (ADR-0018, ADR-0017).

use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record;
use crate::record::descriptor::RecordDescriptor;
use crate::record::document::{self, Document, Section};
use crate::record::patch::{self, PatchOp};
use crate::registry::{
    ArtifactCategory, ArtifactRegistry, ContentModel, StatusModel, TypeDescriptor, TypedHandler,
};
use crate::sanitize;

use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// The document verbs a descriptor may expose (REQ-07.003).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentVerb {
    Create,
    CreateGroup,
    List,
    View,
    Edit,
    Archive,
    Delete,
    Supersede,
    Status,
}

impl DocumentVerb {
    /// Map a command name to its verb. This is the fixed verb vocabulary, not
    /// a per-type mapping.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "create" => DocumentVerb::Create,
            "create-group" => DocumentVerb::CreateGroup,
            "list" => DocumentVerb::List,
            "view" => DocumentVerb::View,
            "edit" => DocumentVerb::Edit,
            "archive" => DocumentVerb::Archive,
            "delete" => DocumentVerb::Delete,
            "supersede" => DocumentVerb::Supersede,
            "status" => DocumentVerb::Status,
            _ => return None,
        })
    }
}

impl From<DocumentVerb> for crate::registry::Verb {
    fn from(v: DocumentVerb) -> Self {
        match v {
            DocumentVerb::Create => crate::registry::Verb::Create,
            DocumentVerb::CreateGroup => crate::registry::Verb::CreateGroup,
            DocumentVerb::List => crate::registry::Verb::List,
            DocumentVerb::View => crate::registry::Verb::View,
            DocumentVerb::Edit => crate::registry::Verb::Edit,
            DocumentVerb::Archive => crate::registry::Verb::Archive,
            DocumentVerb::Delete => crate::registry::Verb::Delete,
            DocumentVerb::Supersede => crate::registry::Verb::Supersede,
            DocumentVerb::Status => crate::registry::Verb::Status,
        }
    }
}

/// Every non-global argument explicitly present on a document verb.
///
/// A missing key means "absent"; an empty value list means "present with no
/// value" (a switch or an optional-value flag used without a value).
#[derive(Clone, Debug, Default)]
pub struct DocumentArgs {
    values: BTreeMap<String, Vec<String>>,
}

impl DocumentArgs {
    pub fn insert(&mut self, id: impl Into<String>, values: Vec<String>) {
        self.values.insert(id.into(), values);
    }

    pub fn get(&self, id: &str) -> Option<&str> {
        self.values
            .get(id)
            .and_then(|values| values.first())
            .map(String::as_str)
    }

    pub fn get_all(&self, id: &str) -> &[String] {
        self.values.get(id).map(Vec::as_slice).unwrap_or_default()
    }

    pub fn flag(&self, id: &str) -> bool {
        self.values.contains_key(id)
    }

    /// `None` when absent, `Some(None)` when present without a value.
    pub fn optional(&self, id: &str) -> Option<Option<&str>> {
        self.values
            .get(id)
            .map(|values| values.first().map(String::as_str))
    }

    pub fn require(&self, id: &str) -> Result<&str, AppError> {
        self.get(id)
            .ok_or_else(|| AppError::Validation(format!("missing required argument: {}", id)))
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

/// One resolved document scope, derived from the descriptor.
///
/// The registry stays I/O-free; directory containment and configuration
/// loading happen here.
#[derive(Clone, Debug)]
pub struct DocumentScope {
    pub config: Config,
    pub active: PathBuf,
    pub archived: PathBuf,
    pub prefix: String,
    pub id_padding: usize,
}

impl DocumentScope {
    pub fn resolve(ctx: &Context, descriptor: &TypeDescriptor) -> Result<Self, AppError> {
        let config = load_config(ctx)?;
        let base = match descriptor.directory.config_key {
            "doc_directory" => config.doc_directory.clone(),
            other => {
                return Err(AppError::Runtime(format!(
                    "unsupported document directory key: {}",
                    other
                )));
            }
        };
        let raw = ctx
            .project_dir
            .join(base)
            .join(&descriptor.directory.relative);
        let active = Context::ensure_within(&ctx.project_dir, &raw)?;
        let archived = Context::ensure_within(&ctx.project_dir, &active.join("archived"))?;
        Ok(DocumentScope {
            id_padding: config.id_padding as usize,
            config,
            active,
            archived,
            prefix: descriptor.prefix.clone(),
        })
    }
}

pub struct DocumentInvocation {
    pub slug: String,
    pub verb: DocumentVerb,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub references: Vec<String>,
    pub editor: bool,
    pub status_value: Option<String>,
    pub force: bool,
    /// Every explicitly present non-global argument, for the typed handlers.
    pub args: DocumentArgs,
}

impl DocumentInvocation {
    pub fn new(slug: impl Into<String>, verb: DocumentVerb) -> Self {
        DocumentInvocation {
            slug: slug.into(),
            verb,
            id: None,
            title: None,
            description: None,
            labels: Vec::new(),
            references: Vec::new(),
            editor: false,
            status_value: None,
            force: false,
            args: DocumentArgs::default(),
        }
    }
}

pub fn run(
    ctx: &Context,
    registry: &ArtifactRegistry,
    invocation: DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let descriptor = registry.descriptor(&invocation.slug).ok_or_else(|| {
        AppError::Validation(format!(
            "document_types.{}: not configured",
            invocation.slug
        ))
    })?;
    if descriptor.category != ArtifactCategory::Document {
        return Err(AppError::Validation(format!(
            "document_types.{}: not a document type",
            invocation.slug
        )));
    }
    if !descriptor.is_enabled() {
        return Err(AppError::Validation(format!(
            "document_types.{}: disabled",
            invocation.slug
        )));
    }
    let verb: crate::registry::Verb = invocation.verb.into();
    if !descriptor.has_verb(verb) {
        return Err(AppError::Validation(format!(
            "document_types.{}: verb not supported",
            invocation.slug
        )));
    }

    let scope = DocumentScope::resolve(ctx, descriptor)?;
    let record = registry
        .record_descriptor(&descriptor.slug)
        .ok_or_else(|| {
            AppError::Runtime(format!(
                "document_types.{}: record descriptor is unavailable",
                descriptor.slug
            ))
        })?;
    let slug = invocation.slug.clone();
    match invocation.verb {
        DocumentVerb::Create
        | DocumentVerb::CreateGroup
        | DocumentVerb::List
        | DocumentVerb::View
        | DocumentVerb::Edit
        | DocumentVerb::Supersede => match descriptor.content_model {
            ContentModel::Generic => match invocation.verb {
                DocumentVerb::Create => create_generic(ctx, descriptor, record, invocation),
                DocumentVerb::List => list_records(ctx, record),
                DocumentVerb::View => view_record(ctx, record, invocation.id),
                DocumentVerb::Edit => edit_generic(ctx, descriptor, record, invocation),
                other => Err(AppError::Validation(format!(
                    "document_types.{}: verb {} is not supported by the generic model",
                    slug,
                    crate::registry::Verb::from(other).as_str()
                ))),
            },
            ContentModel::Typed(_) | ContentModel::HtmlMockup(_) => dispatch_typed(
                ctx,
                descriptor,
                registry,
                record,
                &scope,
                invocation.verb,
                &invocation.args,
            ),
        },
        DocumentVerb::Archive | DocumentVerb::Delete | DocumentVerb::Status => {
            match descriptor.content_model {
                // UI mockups keep their HTML metadata semantics.
                ContentModel::HtmlMockup(_) => dispatch_typed(
                    ctx,
                    descriptor,
                    registry,
                    record,
                    &scope,
                    invocation.verb,
                    &invocation.args,
                ),
                _ => match invocation.verb {
                    DocumentVerb::Archive => archive_record(ctx, record, invocation.id),
                    DocumentVerb::Delete => {
                        delete_record(ctx, record, invocation.id, invocation.force)
                    }
                    DocumentVerb::Status => {
                        status_record(ctx, record, invocation.id, invocation.status_value)
                    }
                    _ => unreachable!("outer match limits record verbs"),
                },
            }
        }
    }
}

/// The create response shared by every typed module.
pub(crate) fn typed_created(ctx: &Context, id: String) -> crate::CommandOutput {
    let json = if ctx.json {
        Some(json!({"id": id}))
    } else {
        None
    };
    crate::CommandOutput {
        text: id,
        json,
        ..Default::default()
    }
}

/// The error a module returns for a verb it does not own.
pub(crate) fn unsupported_verb(verb: DocumentVerb) -> AppError {
    AppError::Validation(format!(
        "unsupported document verb: {}",
        crate::registry::Verb::from(verb).as_str()
    ))
}

/// Delegate a typed content verb to its owning module (never by slug).
fn dispatch_typed(
    ctx: &Context,
    descriptor: &TypeDescriptor,
    registry: &ArtifactRegistry,
    record: &RecordDescriptor,
    scope: &DocumentScope,
    verb: DocumentVerb,
    args: &DocumentArgs,
) -> Result<crate::CommandOutput, AppError> {
    let handler = descriptor.content_model.handler().ok_or_else(|| {
        AppError::Validation(format!(
            "document_types.{}: content model is not typed",
            descriptor.slug
        ))
    })?;
    match handler {
        TypedHandler::Requirement(kind) => {
            let kind = match kind {
                crate::registry::RequirementKind::Business => crate::requirement::Kind::Req,
                crate::registry::RequirementKind::NonFunctional => crate::requirement::Kind::Nfr,
            };
            crate::requirement::handle(ctx, scope, registry, record, kind, verb, args)
        }
        TypedHandler::Glossary => crate::glossary::handle(ctx, scope, record, verb, args),
        TypedHandler::Rule => crate::rule::handle(ctx, scope, record, verb, args),
        TypedHandler::UiSpec => crate::ui_spec::handle(ctx, scope, record, verb, args),
        TypedHandler::Audit => crate::audit::handle(ctx, scope, record, verb, args),
        TypedHandler::TechSpec => crate::tech_spec::handle(ctx, scope, record, verb, args),
        TypedHandler::UseCase => crate::use_case::handle(ctx, scope, record, verb, args),
        TypedHandler::TestCase => crate::test_case::handle(ctx, scope, record, verb, args),
        TypedHandler::TestReport => crate::test_report::handle(ctx, scope, record, verb, args),
        TypedHandler::UiMockup => crate::ui_mockup::handle(ctx, scope, verb, args),
        other => Err(AppError::Validation(format!(
            "document_types.{}: unsupported typed handler {:?}",
            descriptor.slug, other
        ))),
    }
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn sanitize_field(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn current_list(doc: &Document, key: &str) -> Vec<String> {
    if let Some(fm) = &doc.front_matter
        && let yaml_serde::Value::Mapping(m) = &fm.data
        && let Some(yaml_serde::Value::Sequence(seq)) =
            m.get(yaml_serde::Value::String(key.to_string()))
    {
        return seq
            .iter()
            .filter_map(|v| match v {
                yaml_serde::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
    }
    Vec::new()
}

fn create_generic(
    ctx: &Context,
    descriptor: &TypeDescriptor,
    record: &RecordDescriptor,
    invocation: DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let title_raw = invocation
        .title
        .clone()
        .ok_or_else(|| AppError::Validation("title is required".to_string()))?;
    let title = sanitize_field(&title_raw)?;
    let description = match invocation.description {
        Some(d) => Some(sanitize_field(&d)?),
        None => None,
    };

    let mut labels: Vec<String> = Vec::new();
    for l in &invocation.labels {
        let v = sanitize_field(l)?;
        if !labels.contains(&v) {
            labels.push(v);
        }
    }
    let mut references: Vec<String> = Vec::new();
    for r in &invocation.references {
        let v = sanitize::sanitize_path_string(r)
            .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
        if !references.contains(&v) {
            references.push(v);
        }
    }

    let cfg = load_config(ctx)?;
    let active = record.resolve_directory(&cfg, &ctx.project_dir)?;
    fs::create_dir_all(&active).map_err(|e| AppError::Runtime(e.to_string()))?;

    let archived_dir = active.join("archived");
    let next = record::identity::next_number(&active, &archived_dir, &descriptor.prefix);
    let id = record::identity::format_id(&descriptor.prefix, next as i64, cfg.id_padding as usize);

    let status = match &descriptor.status_model {
        StatusModel::DocumentMain { default, .. } => default.clone(),
        _ => "active".to_string(),
    };

    let now = record::frontmatter::utc_now();

    let mut sections: Vec<Section> = Vec::new();
    if let Some(desc_text) = &description {
        sections.push(Section {
            heading: "## Description".to_string(),
            body: vec![desc_text.clone()],
        });
    }
    let body_doc = Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {} {}", id, title)),
        preamble: vec![String::new()],
        sections,
    };
    let body = document::render(&body_doc);
    let parsed = document::parse(&body)?;

    let labels_value = if labels.is_empty() {
        "[]".to_string()
    } else {
        labels.join("\n")
    };
    let refs_value = if references.is_empty() {
        "[]".to_string()
    } else {
        references.join("\n")
    };

    let ops = vec![
        PatchOp::SetFrontMatterValue(
            "id".to_string(),
            record::frontmatter::FrontMatterValue::String(id.clone()),
        ),
        PatchOp::SetFrontMatterValue(
            "title".to_string(),
            record::frontmatter::FrontMatterValue::String(title.clone()),
        ),
        PatchOp::SetFrontMatterValue(
            "created_date".to_string(),
            record::frontmatter::FrontMatterValue::String(now.clone()),
        ),
        PatchOp::SetFrontMatterValue(
            "updated_date".to_string(),
            record::frontmatter::FrontMatterValue::String(now.clone()),
        ),
        PatchOp::SetFrontMatterValue(
            "labels".to_string(),
            record::frontmatter::FrontMatterValue::String(labels_value),
        ),
        PatchOp::SetFrontMatterValue(
            "references".to_string(),
            record::frontmatter::FrontMatterValue::String(refs_value),
        ),
        PatchOp::SetFrontMatterValue(
            "status".to_string(),
            record::frontmatter::FrontMatterValue::String(status),
        ),
    ];
    let file_content = patch::apply_patch(&parsed, &ops)?;
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    record::store::write_atomic(ctx, &safe_path, &file_content)?;

    let json = if ctx.json {
        Some(json!({"id": id}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: id,
        json,
        ..Default::default()
    })
}

fn list_records(
    ctx: &Context,
    record: &RecordDescriptor,
) -> Result<crate::CommandOutput, AppError> {
    // Read-path auto-relocation: records whose status is `archived` but which
    // still live in the active folder are relocated by `record::resolve::list_in_dir`
    // (via `record::store::relocate_if_archived`). On destination-write failure
    // this surfaces as `Err(Validation)`; on source-unlink failure as
    // `Err(Runtime)` naming both paths. The operation fails loudly rather than
    // returning a read-only handle.
    let cfg = load_config(ctx)?;
    let active = record.resolve_directory(&cfg, &ctx.project_dir)?;
    let _ = fs::create_dir_all(&active);
    let prefixes = record.configured_prefixes(&cfg);
    let dotted = record.is_dotted_id();
    let records = record::resolve::list_in_dir(ctx, &active, &prefixes, dotted)?;
    let mut items: Vec<(String, String, String)> = Vec::new();
    for record in records {
        if record.path.parent() != Some(active.as_path()) {
            continue;
        }
        let (doc_read, _token) = record::store::read_document(ctx, &record.path)?;
        let content = doc_read.source.clone();
        let (_, t) = document::heading_id_title_with_frontmatter(&doc_read);
        let title = if t.is_empty() { record.id.clone() } else { t };
        items.push((record.id, title, content));
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, title, content)| record::view::RecordView::lenient(content, id).list_line(title))
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, title, content)| {
                json!({
                    "id": id,
                    "title": title,
                    "metadata": record::view::RecordView::lenient(content, id).metadata_json(),
                })
            })
            .collect();
        Some(serde_json::Value::Array(arr))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn view_record(
    ctx: &Context,
    record: &RecordDescriptor,
    id: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let cfg = load_config(ctx)?;
    let active = record.resolve_directory(&cfg, &ctx.project_dir)?;
    let _ = fs::create_dir_all(&active);
    let prefixes = record.configured_prefixes(&cfg);
    let dotted = record.is_dotted_id();
    let resolved = record::resolve::resolve_in_dirs(ctx, &[active], &requested, &prefixes, dotted)?;
    let record =
        resolved.ok_or_else(|| AppError::NotFound(format!("record not found: {}", requested)))?;
    let id = record.id.clone();
    let path = record.path.clone();
    let (doc, _token) = record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();

    let json = if ctx.json {
        Some(json!({
            "id": id,
            "path": path.display().to_string(),
            "content": content,
            "metadata": record::view::RecordView::lenient(&content, &id).metadata_json(),
        }))
    } else {
        None
    };
    let text = content.trim_end_matches('\n').to_string();
    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn edit_generic(
    ctx: &Context,
    _descriptor: &TypeDescriptor,
    record: &RecordDescriptor,
    invocation: DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = invocation
        .id
        .clone()
        .ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;

    let cfg = load_config(ctx)?;
    let active = record.resolve_directory(&cfg, &ctx.project_dir)?;
    let _ = fs::create_dir_all(&active);
    let prefixes = record.configured_prefixes(&cfg);
    let dotted = record.is_dotted_id();
    let resolved = record::resolve::resolve_in_dirs(
        ctx,
        std::slice::from_ref(&active),
        &requested,
        &prefixes,
        dotted,
    )?;
    let record =
        resolved.ok_or_else(|| AppError::NotFound(format!("record not found: {}", requested)))?;
    record.ensure_writable()?;
    let id = record.id.clone();
    let path = record.path.clone();
    if path.parent() != Some(active.as_path()) {
        return Err(AppError::Validation(format!(
            "record {} is archived; edits are not allowed",
            id
        )));
    }

    let (doc, _token) = record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();

    let mut ops: Vec<PatchOp> = Vec::new();
    let mut changed = false;

    if let Some(title_raw) = invocation.title {
        let new_title = sanitize_field(&title_raw)?;
        let current_title = record::frontmatter::current_value_string(
            doc.front_matter
                .as_ref()
                .map(|f| f.raw.as_str())
                .unwrap_or(""),
            "title",
        )
        .unwrap_or_else(|| {
            let (_, t) = document::heading_id_title(&doc);
            t
        });
        if new_title != current_title {
            ops.push(PatchOp::SetFrontMatter(
                "title".to_string(),
                new_title.clone(),
            ));
            ops.push(PatchOp::SetHeadingTitle(new_title));
            changed = true;
        }
    }

    if let Some(desc_raw) = invocation.description {
        let new_desc = sanitize_field(&desc_raw)?;
        let current_desc = document::section_text(&doc, "Description").unwrap_or_default();
        if new_desc != current_desc.trim() {
            ops.push(PatchOp::SetSection("## Description".to_string(), new_desc));
            changed = true;
        }
    }

    if !invocation.labels.is_empty() {
        let mut fm_labels = current_list(&doc, "labels");
        let mut appended = false;
        for l in &invocation.labels {
            let lv = sanitize_field(l)?;
            if !fm_labels.contains(&lv) {
                fm_labels.push(lv);
                appended = true;
            }
        }
        if appended {
            ops.push(PatchOp::SetFrontMatter(
                "labels".to_string(),
                fm_labels.join("\n"),
            ));
            changed = true;
        }
    }

    if !invocation.references.is_empty() {
        let mut fm_refs = current_list(&doc, "references");
        let mut appended = false;
        for r in &invocation.references {
            let rv = sanitize::sanitize_path_string(r)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
            if !fm_refs.contains(&rv) {
                fm_refs.push(rv);
                appended = true;
            }
        }
        if appended {
            ops.push(PatchOp::SetFrontMatter(
                "references".to_string(),
                fm_refs.join("\n"),
            ));
            changed = true;
        }
    }

    if changed {
        let now = record::frontmatter::utc_now();
        ops.push(PatchOp::SetFrontMatter("updated_date".to_string(), now));
    }

    let new_content = patch::apply_patch(&doc, &ops)?;
    if new_content != content {
        let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
        record::store::write_atomic(ctx, &safe_path, &new_content)?;
    }

    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
    if invocation.editor {
        let command = cfg
            .effective_editor()
            .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?;
        run_editor(&command, &safe_path)?;
    }

    let json = if ctx.json {
        Some(json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

pub fn archive_record(
    ctx: &Context,
    record: &RecordDescriptor,
    id: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let _ = record::store::archive(ctx, record, &requested)?;
    let json = if ctx.json {
        Some(json!({"status": "ok", "id": requested}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

pub fn delete_record(
    ctx: &Context,
    record: &RecordDescriptor,
    id: Option<String>,
    force: bool,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    if !force {
        use std::io::{self, Write};
        print!("Delete {}? [y/N] ", requested);
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let ans = input.trim().to_lowercase();
        if ans != "y" && ans != "yes" {
            let json = if ctx.json {
                Some(serde_json::json!({"status": "aborted", "id": requested}))
            } else {
                None
            };
            return Ok(crate::CommandOutput {
                text: String::new(),
                json,
                ..Default::default()
            });
        }
    }
    record::store::delete(ctx, record, &requested)?;
    let json = if ctx.json {
        Some(json!({"status": "ok", "id": requested}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

pub fn status_record(
    ctx: &Context,
    record: &RecordDescriptor,
    id: Option<String>,
    value: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    // Read-path auto-relocation: see `list_records`; a record relocated during
    // the read step fails loudly (`Err(Validation)` on destination-write failure,
    // `Err(Runtime)` naming both paths on source-unlink failure) instead of
    // returning a read-only handle.
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let value =
        value.ok_or_else(|| AppError::Validation("status value is required".to_string()))?;
    // Auto-relocate archived-in-active records and surface undeclared statuses.
    let cfg = load_config(ctx)?;
    let read_dirs = record.resolve_read_directories(&cfg, &ctx.project_dir);
    let prefixes = record.configured_prefixes(&cfg);
    let dotted = record.is_dotted_id();
    if let Some(rec) =
        record::resolve::resolve_in_dirs(ctx, &read_dirs, &requested, &prefixes, dotted)?
    {
        let _ = record::store::read_document(ctx, &rec.path)?;
    }
    record::store::set_status(ctx, record, &requested, &value)?;
    let json = if ctx.json {
        Some(json!({"status": "ok", "id": requested}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

fn run_editor(command: &str, path: &std::path::Path) -> Result<(), AppError> {
    let mut parts = command.split_whitespace();
    let prog = parts.next().unwrap_or("");
    if prog.is_empty() {
        return Err(AppError::Runtime("no editor configured".to_string()));
    }
    let status = std::process::Command::new(prog)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    if !status.success() {
        return Err(AppError::Runtime(format!("editor exited with {}", status)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::registry::ArtifactRegistry;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("document-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn ctx(dir: &std::path::Path) -> Context {
        Context {
            project_dir: dir.to_path_buf(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        }
    }

    fn test_config() -> Config {
        // The generic-handler suite exercises `deploy`, which is no longer a
        // default document type; declare it explicitly as a user-defined type.
        let mut cfg = Config {
            project_name: Some("Test".to_string()),
            ..Config::default()
        };
        cfg.document_types.insert(
            "deploy".to_string(),
            crate::config::DocTypeEntry {
                prefix: "DEP".to_string(),
                directory: "architecture/deployment/".to_string(),
                enabled: true,
                statuses: Some(vec!["active".to_string(), "archived".to_string()]),
                default_status: None,
            },
        );
        cfg
    }

    fn default_config(dir: &std::path::Path) {
        fs::write(
            dir.join("papyrus.config.yml"),
            yaml_serde::to_string(&test_config()).unwrap(),
        )
        .unwrap();
    }

    fn custom_config(dir: &std::path::Path) {
        fs::write(
            dir.join("papyrus.config.yml"),
            "project_name: Test\ndocument_types:\n  note:\n    prefix: NOTE\n    directory: notes/\n    enabled: true\n    statuses:\n      - active\n      - archived\n      - review\n    default_status: review\n",
        )
        .unwrap();
    }

    fn registry(ctx: &Context) -> ArtifactRegistry {
        let cfg = load_config(ctx).unwrap();
        ArtifactRegistry::build(&cfg).unwrap()
    }

    fn scope_for(ctx: &Context, slug: &str) -> DocumentScope {
        let cfg = load_config(ctx).unwrap();
        let reg = ArtifactRegistry::build(&cfg).unwrap();
        let descriptor = reg.descriptor(slug).unwrap_or_else(|| panic!("{slug}"));
        DocumentScope::resolve(ctx, descriptor).unwrap()
    }

    fn typed_args(pairs: &[(&str, &[&str])]) -> DocumentArgs {
        let mut args = DocumentArgs::default();
        for (id, values) in pairs {
            args.insert(
                *id,
                values.iter().map(|value| (*value).to_string()).collect(),
            );
        }
        args
    }

    #[allow(clippy::too_many_arguments)]
    fn generic_invocation(
        slug: &str,
        verb: DocumentVerb,
        id: Option<String>,
        title: Option<String>,
        description: Option<String>,
        labels: Vec<String>,
        references: Vec<String>,
        status_value: Option<String>,
    ) -> DocumentInvocation {
        let mut args = DocumentArgs::default();
        if let Some(value) = &id {
            args.insert("id", vec![value.clone()]);
        }
        if let Some(value) = &title {
            args.insert("title", vec![value.clone()]);
        }
        if let Some(value) = &description {
            args.insert("description", vec![value.clone()]);
        }
        if !labels.is_empty() {
            args.insert("label", labels.clone());
        }
        if !references.is_empty() {
            args.insert("reference", references.clone());
        }
        if let Some(value) = &status_value {
            args.insert("value", vec![value.clone()]);
        }
        args.insert("force", Vec::new());
        DocumentInvocation {
            slug: slug.to_string(),
            verb,
            id,
            title,
            description,
            labels,
            references,
            editor: false,
            status_value,
            force: true,
            args,
        }
    }

    #[test]
    fn generic_create_writes_seven_keys_and_optional_description() {
        let dir = tmpdir("create");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("My Deploy".to_string()),
                Some("desc".to_string()),
                vec!["l1".to_string()],
                vec!["r1".to_string()],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "DEP-001");
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = fs::read_to_string(&path).unwrap();
        for key in [
            "id:",
            "title:",
            "created_date:",
            "updated_date:",
            "labels:",
            "references:",
            "status:",
        ] {
            assert!(content.contains(key), "missing {} in {}", key, content);
        }
        assert!(content.contains("## Description"));
        assert!(content.contains("desc"));
    }

    #[test]
    fn explicitly_declared_deploy_is_valid_and_dispatched() {
        let dir = tmpdir("declareddeploy");
        default_config(&dir);
        let c = ctx(&dir);
        let cfg = load_config(&c).unwrap();
        assert!(cfg.validate(Some(&dir)).is_ok());
        let reg = registry(&c);
        let deploy = reg
            .descriptor("deploy")
            .expect("declared deploy descriptor");
        assert_eq!(deploy.prefix, "DEP");
        assert_eq!(deploy.directory.relative, "architecture/deployment/");
    }

    #[test]
    fn generic_create_requires_title() {
        let dir = tmpdir("createtitle");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let res = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        );
        assert!(res.is_err());
    }

    #[test]
    fn generic_create_starts_at_default_or_active() {
        let dir = tmpdir("default");
        custom_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "note",
                DocumentVerb::Create,
                None,
                Some("N".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "NOTE-001");
        let content = fs::read_to_string(dir.join("docs/notes/NOTE-001.md")).unwrap();
        assert!(content.contains("status: review"));
    }

    #[test]
    fn generic_create_allocates_across_active_and_archived() {
        let dir = tmpdir("alloc");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "DEP-001");
        // archive it
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Archive,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("B".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "DEP-002");
    }

    #[test]
    fn generic_list_hides_archived() {
        let dir = tmpdir("listhide");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Archive,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!out.text.contains("DEP-001"));
    }

    #[test]
    fn generic_view_returns_metadata() {
        let dir = tmpdir("view");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("desc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::View,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("# DEP-001 A"));
    }

    #[test]
    fn generic_edit_patches_title_heading_description() {
        let dir = tmpdir("edit");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("olddesc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                Some("B".to_string()),
                Some("newdesc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::View,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("# DEP-001 B"));
        assert!(out.text.contains("newdesc"));
    }

    #[test]
    fn generic_edit_appends_labels_and_references() {
        let dir = tmpdir("editlabels");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec!["l1".to_string()],
                vec!["r1".to_string()],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                None,
                None,
                vec!["l2".to_string()],
                vec!["r2".to_string()],
                None,
            ),
        )
        .unwrap();
        let content =
            fs::read_to_string(dir.join("docs/architecture/deployment/DEP-001.md")).unwrap();
        assert!(content.contains("l2"));
        assert!(content.contains("r2"));
        assert!(content.contains("l1"));
        assert!(content.contains("r1"));
    }

    #[test]
    fn generic_edit_preserves_unknown_section_and_key() {
        let dir = tmpdir("preserve");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("desc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let mut content = fs::read_to_string(&path).unwrap();
        content = content.replace("status: active", "status: active\nfoo: bar");
        content.push_str("\n## Notes\nkeep\n");
        fs::write(&path, content).unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                Some("B".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("foo: bar"));
        assert!(content.contains("## Notes"));
        assert!(content.contains("keep"));
    }

    #[test]
    fn generic_edit_no_write_when_unchanged() {
        let dir = tmpdir("nowrite");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("desc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let before = fs::read_to_string(&path).unwrap();
        let meta_before = fs::metadata(&path).unwrap().modified().unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                Some("A".to_string()),
                Some("desc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        let meta_after = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(before, after);
        assert_eq!(meta_before, meta_after);
    }

    #[test]
    fn generic_archive_round_trip_and_idempotent() {
        let dir = tmpdir("archive");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Archive,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        // idempotent
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Archive,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!dir.join("docs/architecture/deployment/DEP-001.md").exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_status_accepts_custom_and_rejects_archived() {
        let dir = tmpdir("status");
        custom_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "note",
                DocumentVerb::Create,
                None,
                Some("N".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "note",
                DocumentVerb::Status,
                Some("NOTE-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        )
        .unwrap();
        // archive then status rejected
        run(
            &c,
            &reg,
            generic_invocation(
                "note",
                DocumentVerb::Archive,
                Some("NOTE-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let res = run(
            &c,
            &reg,
            generic_invocation(
                "note",
                DocumentVerb::Status,
                Some("NOTE-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        );
        assert!(res.is_err());
    }

    #[test]
    fn generic_delete_archives_then_removes() {
        let dir = tmpdir("delete");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Delete,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!dir.join("docs/architecture/deployment/DEP-001.md").exists());
        assert!(
            !dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_archived_in_active_auto_relocates() {
        let dir = tmpdir("relocate");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        fs::write(&path, content).unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!out.text.contains("DEP-001"));
        assert!(!path.exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_undeclared_status_preserved_and_write_refused() {
        let dir = tmpdir("undeclared");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: weird");
        fs::write(&path, content).unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::View,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("weird"));
        let res = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Status,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        );
        assert!(res.is_err());
    }

    #[test]
    fn generic_hand_added_section_preserved() {
        let dir = tmpdir("handsection");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("desc".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let mut content = fs::read_to_string(&path).unwrap();
        content.push_str("\n## Notes\nkeep\n");
        fs::write(&path, content).unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                Some("B".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("## Notes"));
        assert!(content.contains("keep"));
    }

    #[test]
    fn typed_req_nfr_baseline_via_document() {
        let dir = tmpdir("typed");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let req_scope = scope_for(&c, "req");
        let nfr_scope = scope_for(&c, "nfr");
        let _ = crate::requirement::create_group(
            &c,
            &req_scope,
            crate::requirement::Kind::Req,
            "G",
            None,
            None,
        )
        .unwrap();
        let rid = crate::requirement::create(
            &c,
            &req_scope,
            &reg,
            crate::requirement::Kind::Req,
            Some("R"),
            None,
            None,
            Some("desc"),
            "REQ-001",
            &[],
            &[],
        )
        .unwrap();
        let _ = crate::requirement::create_group(
            &c,
            &nfr_scope,
            crate::requirement::Kind::Nfr,
            "NG",
            None,
            None,
        )
        .unwrap();
        let nid = crate::requirement::create(
            &c,
            &nfr_scope,
            &reg,
            crate::requirement::Kind::Nfr,
            Some("NR"),
            None,
            None,
            Some("desc"),
            "NFR-001",
            &[],
            &[],
        )
        .unwrap();

        let out = run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains(&rid));
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::View,
                Some(rid.clone()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains(&rid));
        run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::Status,
                Some(rid.clone()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::Archive,
                Some(rid.clone()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!out.text.contains(&rid));

        run(
            &c,
            &reg,
            generic_invocation(
                "nfr",
                DocumentVerb::Delete,
                Some(nid.clone()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "nfr",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!out.text.contains(&nid));
    }

    #[test]
    fn typed_ui_mockup_basic() {
        let dir = tmpdir("uimockup");
        default_config(&dir);
        let c = ctx(&dir);
        let scope = scope_for(&c, "ui-mockup");
        let id = crate::ui_mockup::handle(
            &c,
            &scope,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        let out =
            crate::ui_mockup::handle(&c, &scope, DocumentVerb::List, &DocumentArgs::default())
                .unwrap();
        assert!(out.text.contains(&id));
        let out = crate::ui_mockup::handle(
            &c,
            &scope,
            DocumentVerb::View,
            &typed_args(&[("id", &[id.as_str()])]),
        )
        .unwrap();
        assert!(out.text.contains(&id));
    }

    #[test]
    fn generic_edit_editor_touches_description_only() {
        let dir = tmpdir("editor");
        let script = dir.join("editor.sh");
        std::fs::write(&script, "#!/bin/sh\nsed -i 's/olddesc/newdesc/' \"$1\"\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut cfg = test_config();
        cfg.editor = Some(script.to_str().unwrap().to_string());
        std::fs::write(
            dir.join("papyrus.config.yml"),
            yaml_serde::to_string(&cfg).unwrap(),
        )
        .unwrap();
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                Some("olddesc".to_string()),
                vec!["l1".to_string()],
                vec!["r1".to_string()],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let before = std::fs::read_to_string(&path).unwrap();
        let updated_before = before
            .lines()
            .find(|l| l.starts_with("updated_date:"))
            .unwrap()
            .to_string();
        let inv = DocumentInvocation {
            slug: "deploy".to_string(),
            verb: DocumentVerb::Edit,
            id: Some("DEP-001".to_string()),
            title: None,
            description: None,
            labels: vec![],
            references: vec![],
            editor: true,
            status_value: None,
            force: true,
            args: DocumentArgs::default(),
        };
        run(&c, &reg, inv).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        let updated_after = after
            .lines()
            .find(|l| l.starts_with("updated_date:"))
            .unwrap()
            .to_string();
        assert!(after.contains("newdesc"));
        assert!(!after.contains("olddesc"));
        assert!(after.contains("l1"));
        assert!(after.contains("r1"));
        assert_eq!(updated_before, updated_after);
    }

    #[test]
    fn generic_archived_in_active_relocates_on_view() {
        let dir = tmpdir("relocateview");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        std::fs::write(&path, content).unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::View,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("DEP-001"));
        assert!(!path.exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_archived_in_active_relocates_on_edit() {
        let dir = tmpdir("relocateedit");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        std::fs::write(&path, content).unwrap();
        let res = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Edit,
                Some("DEP-001".to_string()),
                Some("B".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        );
        assert!(res.is_err());
        assert!(!path.exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_archived_in_active_relocates_on_archive() {
        let dir = tmpdir("relocatearchive");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        std::fs::write(&path, content).unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Archive,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!path.exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_archived_in_active_relocates_on_delete() {
        let dir = tmpdir("relocatedelete");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        std::fs::write(&path, content).unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Delete,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(!path.exists());
        assert!(
            !dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_archived_in_active_relocates_on_status() {
        let dir = tmpdir("relocatestatus");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let path = dir.join("docs/architecture/deployment/DEP-001.md");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("status: active", "status: archived");
        std::fs::write(&path, content).unwrap();
        let res = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Status,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        );
        assert!(res.is_err());
        assert!(!path.exists());
        assert!(
            dir.join("docs/architecture/deployment/archived/DEP-001.md")
                .exists()
        );
    }

    #[test]
    fn generic_minimal_config_defined_type_all_verbs() {
        let dir = tmpdir("minimal");
        std::fs::write(
            dir.join("papyrus.config.yml"),
            "project_name: Test\ndocument_types:\n  thing:\n    prefix: THING\n    directory: things/\n    enabled: true\n",
        )
        .unwrap();
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::Create,
                None,
                Some("T".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "THING-001");
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("THING-001"));
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::View,
                Some("THING-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("THING-001"));
        run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::Edit,
                Some("THING-001".to_string()),
                Some("T2".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::Status,
                Some("THING-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                Some("active".to_string()),
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::Archive,
                Some("THING-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        run(
            &c,
            &reg,
            generic_invocation(
                "thing",
                DocumentVerb::Delete,
                Some("THING-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
    }

    #[test]
    fn generic_no_extra_files() {
        let dir = tmpdir("noextra");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("A".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        let active = dir.join("docs/architecture/deployment");
        let count = std::fs::read_dir(&active)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    == Some("md")
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn generic_vs_typed_conformance() {
        let dir = tmpdir("conformance");
        default_config(&dir);
        let c = ctx(&dir);
        // generic
        let reg = registry(&c);
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::Create,
                None,
                Some("G".to_string()),
                Some("d".to_string()),
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "DEP-001");
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "deploy",
                DocumentVerb::View,
                Some("DEP-001".to_string()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains("# DEP-001 G"));
        // typed req
        let req_scope = scope_for(&c, "req");
        let _ = crate::requirement::create_group(
            &c,
            &req_scope,
            crate::requirement::Kind::Req,
            "G",
            None,
            None,
        )
        .unwrap();
        let rid = crate::requirement::create(
            &c,
            &req_scope,
            &reg,
            crate::requirement::Kind::Req,
            Some("R"),
            None,
            None,
            Some("desc"),
            "REQ-001",
            &[],
            &[],
        )
        .unwrap();
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::View,
                Some(rid.clone()),
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains(&rid));
        let out = run(
            &c,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::List,
                None,
                None,
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert!(out.text.contains(&rid));
    }

    #[test]
    fn typed_extensions_create_group_and_supersede_still_work() {
        let dir = tmpdir("typedext");
        default_config(&dir);
        let c = ctx(&dir);
        let req_scope = scope_for(&c, "req");
        let gid = crate::requirement::create_group(
            &c,
            &req_scope,
            crate::requirement::Kind::Req,
            "G",
            None,
            None,
        )
        .unwrap();
        assert!(!gid.is_empty());
        let mock_scope = scope_for(&c, "ui-mockup");
        let id = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M1"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        let id2 = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M2"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        crate::ui_mockup::handle(
            &c,
            &mock_scope,
            DocumentVerb::Edit,
            &typed_args(&[("id", &[id.as_str()]), ("superseded_by", &[id2.as_str()])]),
        )
        .unwrap();
        let out = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            DocumentVerb::View,
            &typed_args(&[("id", &[id.as_str()])]),
        )
        .unwrap();
        assert!(out.text.contains(&id2));
    }
}
