//! Generic, descriptor-driven document handler (ADR-0018, ADR-0017).

use crate::config::Config;
use crate::config::FieldSpec;
use crate::context::Context;
use crate::error::AppError;
use crate::record;
use crate::record::descriptor::RecordDescriptor;
use crate::record::document::{self, Document};
use crate::record::patch::{self, PatchOp};
use crate::registry::{
    ArtifactCategory, ArtifactRegistry, DocumentModel, StatusModel, TypeDescriptor, TypedHandler,
};
use crate::sanitize;

use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
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
    multiline: BTreeSet<String>,
}

impl DocumentArgs {
    pub fn insert(&mut self, id: impl Into<String>, values: Vec<String>) {
        self.values.insert(id.into(), values);
    }

    /// Mark an argument as resolved from standard input.
    pub fn mark_multiline(&mut self, id: &str) {
        self.multiline.insert(id.to_string());
    }

    pub fn is_multiline(&self, id: &str) -> bool {
        self.multiline.contains(id)
    }

    /// A text argument that remembers whether it came from standard input.
    pub fn text(&self, id: &str) -> Option<sanitize::Text> {
        self.get(id).map(|value| {
            if self.is_multiline(id) {
                sanitize::Text::multiline(value)
            } else {
                sanitize::Text::literal(value)
            }
        })
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

    pub fn require_text(&self, id: &str) -> Result<sanitize::Text, AppError> {
        self.text(id)
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
    pub fn resolve(
        ctx: &Context,
        config: &Config,
        descriptor: &TypeDescriptor,
    ) -> Result<Self, AppError> {
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
            config: config.clone(),
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
    /// True when `description` was resolved from standard input.
    pub description_multiline: bool,
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
            description_multiline: false,
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
    cfg: &Config,
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

    let scope = DocumentScope::resolve(ctx, cfg, descriptor)?;
    let record = registry
        .record_descriptor(&descriptor.slug)
        .ok_or_else(|| {
            AppError::Runtime(format!(
                "document_types.{}: record descriptor is unavailable",
                descriptor.slug
            ))
        })?;

    // The HTML mockup keeps its fixed, metadata-driven module surface: the
    // engine patch path cannot reproduce the HTML metadata block byte-for-byte,
    // so MS-035 owns the consolidation with the browser.
    if descriptor.document_model == DocumentModel::SingleHtmlMockup {
        // The HTML model keeps its own writer, but it reads the same title
        // contract as every Markdown model.
        let args = merge_invocation_args(&invocation, &[]);
        return crate::ui_mockup::handle(ctx, &scope, registry, invocation.verb, &args);
    }

    match invocation.verb {
        DocumentVerb::Create => {
            create_document(ctx, cfg, registry, descriptor, &scope, &invocation)
        }
        DocumentVerb::CreateGroup => {
            create_group_document(ctx, cfg, registry, descriptor, &scope, &invocation)
        }
        DocumentVerb::List => list_documents(ctx, cfg, registry, descriptor, &scope),
        DocumentVerb::View => view_document(ctx, cfg, registry, descriptor, &scope, invocation.id),
        DocumentVerb::Edit => edit_document(ctx, cfg, registry, descriptor, &scope, &invocation),
        DocumentVerb::Archive => archive_record(ctx, cfg, registry, record, invocation.id),
        DocumentVerb::Delete => {
            delete_record(ctx, cfg, registry, record, invocation.id, invocation.force)
        }
        DocumentVerb::Status => status_record(
            ctx,
            cfg,
            registry,
            record,
            invocation.id,
            invocation.status_value,
        ),
        DocumentVerb::Supersede => Err(unsupported_verb(DocumentVerb::Supersede)),
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

fn sanitize_field(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn current_list(doc: &Document, key: &str) -> Vec<String> {
    if let Some(fm) = &doc.front_matter
        && let yaml_serde::Value::Mapping(mapping) = &fm.data
        && let Some(yaml_serde::Value::Sequence(sequence)) =
            mapping.get(yaml_serde::Value::String(key.to_string()))
    {
        return sequence
            .iter()
            .filter_map(|value| match value {
                yaml_serde::Value::String(text) => Some(text.clone()),
                _ => None,
            })
            .collect();
    }
    Vec::new()
}

/// The resolved role field sequence for a verb.
fn document_fields_for_verb(descriptor: &TypeDescriptor, verb: DocumentVerb) -> Vec<FieldSpec> {
    match descriptor.document_model {
        DocumentModel::SingleMarkdown => descriptor.fields.clone(),
        DocumentModel::GroupedMarkdown => match verb {
            DocumentVerb::CreateGroup => descriptor
                .group
                .as_ref()
                .and_then(|role| role.fields.clone())
                .unwrap_or_default(),
            DocumentVerb::Create => descriptor
                .member
                .as_ref()
                .and_then(|role| role.fields.clone())
                .or_else(|| {
                    descriptor
                        .group
                        .as_ref()
                        .and_then(|role| role.fields.clone())
                })
                .unwrap_or_default(),
            DocumentVerb::Edit => {
                let mut fields = Vec::new();
                for role in [descriptor.group.as_ref(), descriptor.member.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    for field in role.fields.iter().flatten() {
                        if !fields
                            .iter()
                            .any(|existing: &FieldSpec| existing.key == field.key)
                        {
                            fields.push(field.clone());
                        }
                    }
                }
                fields
            }
            _ => Vec::new(),
        },
        DocumentModel::SingleHtmlMockup => Vec::new(),
    }
}

fn field_long(field: &FieldSpec) -> String {
    field
        .option
        .clone()
        .unwrap_or_else(|| sanitize::kebab_slug(&field.key))
}

fn field_offered(field: &FieldSpec, verb: DocumentVerb) -> bool {
    field.verbs.iter().any(|field_verb| match field_verb {
        crate::config::FieldVerb::Create => {
            matches!(verb, DocumentVerb::Create | DocumentVerb::CreateGroup)
        }
        crate::config::FieldVerb::Edit => verb == DocumentVerb::Edit,
    })
}

/// Resolve the one `-` sentinel among a verb's multiline fields, before any ID
/// is allocated or file is written.
fn resolve_field_stdin(fields: &[FieldSpec], args: &mut DocumentArgs) -> Result<(), AppError> {
    let sentinels: Vec<&FieldSpec> = fields
        .iter()
        .filter(|field| {
            field.kind == crate::config::FieldKind::Multiline
                && args.get(&field.key) == Some(sanitize::STDIN_SENTINEL)
        })
        .collect();
    if sentinels.len() > 1 {
        let names = sentinels
            .iter()
            .map(|field| format!("--{}", field_long(field)))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AppError::Validation(format!(
            "multiple options request multiline text from standard input: {names}"
        )));
    }
    if let Some(field) = sentinels.first() {
        let text = sanitize::read_stdin_text()?;
        args.insert(field.key.clone(), vec![text]);
        args.mark_multiline(&field.key);
    }
    Ok(())
}

fn merge_invocation_args(invocation: &DocumentInvocation, fields: &[FieldSpec]) -> DocumentArgs {
    let mut args = invocation.args.clone();
    if args.get("description").is_none()
        && fields.iter().any(|field| field.key == "description")
        && let Some(description) = &invocation.description
    {
        args.insert("description", vec![description.clone()]);
        if invocation.description_multiline {
            args.mark_multiline("description");
        }
    }
    if args.get("title").is_none()
        && args.get("title_flag").is_none()
        && let Some(title) = &invocation.title
    {
        args.insert("title", vec![title.clone()]);
    }
    if args.get_all("label").is_empty() && !invocation.labels.is_empty() {
        args.insert("label", invocation.labels.clone());
    }
    if args.get_all("reference").is_empty() && !invocation.references.is_empty() {
        args.insert("reference", invocation.references.clone());
    }
    args
}

fn missing_required_fields(
    fields: &[FieldSpec],
    args: &DocumentArgs,
    defaults: &BTreeMap<String, String>,
) -> Vec<String> {
    fields
        .iter()
        .filter(|field| field.required)
        .filter(|field| args.get(&field.key).is_none() && !defaults.contains_key(&field.key))
        .map(|field| field.key.clone())
        .collect()
}

fn ensure_required_fields(
    fields: &[FieldSpec],
    args: &DocumentArgs,
    defaults: &BTreeMap<String, String>,
) -> Result<(), AppError> {
    let missing = missing_required_fields(fields, args, defaults);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "missing required field(s): {}",
            missing.join(", ")
        )))
    }
}

fn sanitize_labels(raw: &[String]) -> Result<Vec<String>, AppError> {
    let mut labels = Vec::new();
    for value in raw {
        let normalized = sanitize_field(value)?;
        if !labels.contains(&normalized) {
            labels.push(normalized);
        }
    }
    Ok(labels)
}

fn sanitize_references(raw: &[String]) -> Result<Vec<String>, AppError> {
    let mut references = Vec::new();
    for value in raw {
        let normalized = sanitize::sanitize_path_string(value)
            .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
        if !references.contains(&normalized) {
            references.push(normalized);
        }
    }
    Ok(references)
}

/// The one title value a verb accepts: the positional `TITLE` or `--title`.
///
/// Supplying both forms on one invocation is rejected rather than compared, so
/// a create that names the title twice never writes a file (ADR-036 decision
/// 24, TC-125).
fn title_source(args: &DocumentArgs) -> Result<Option<&str>, AppError> {
    match (args.get("title"), args.get("title_flag")) {
        (Some(_), Some(_)) => Err(AppError::Validation(
            "conflicting title sources".to_string(),
        )),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

/// Validate one title: non-empty, single-line, and free of control characters.
///
/// The title is not a field and never a path component; it therefore reuses
/// the shared single-line sanitizer.
fn sanitize_title(raw: &str) -> Result<String, AppError> {
    if raw.trim().is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }
    sanitize::sanitize_name(raw)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

/// The required title of a create or create-group (TC-126).
pub(crate) fn required_title(args: &DocumentArgs) -> Result<String, AppError> {
    let raw =
        title_source(args)?.ok_or_else(|| AppError::Validation("title is required".to_string()))?;
    sanitize_title(raw)
}

/// The optional title of an edit. `None` leaves the stored title unchanged; a
/// supplied title that is empty, whitespace-only, multi-line, or conflicting is
/// rejected, so an edit can never clear the title.
pub(crate) fn optional_title(args: &DocumentArgs) -> Result<Option<String>, AppError> {
    title_source(args)?.map(sanitize_title).transpose()
}

fn scalar_value(args: &DocumentArgs, field: &FieldSpec) -> Result<Option<String>, AppError> {
    let Some(text) = args.text(&field.key) else {
        return Ok(None);
    };
    let value = text.resolve()?;
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "field '{}' must not be empty",
            field.key
        )));
    }
    Ok(Some(value))
}

fn list_values(args: &DocumentArgs, field: &FieldSpec) -> Result<Vec<String>, AppError> {
    let mut values = Vec::new();
    for raw in args.get_all(&field.key) {
        let value = sanitize_field(raw)?;
        if !values.contains(&value) {
            values.push(value);
        }
    }
    Ok(values)
}

fn scalar_section_body(value: &str) -> String {
    format!("{value}\n")
}

fn list_section_body(items: &[String]) -> String {
    let mut body = items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n");
    body.push('\n');
    body
}

fn frontmatter_sequence(doc: &Document, key: &str) -> Vec<String> {
    if let Some(fm) = &doc.front_matter
        && let yaml_serde::Value::Mapping(mapping) = &fm.data
        && let Some(yaml_serde::Value::Sequence(sequence)) =
            mapping.get(yaml_serde::Value::String(key.to_string()))
    {
        return sequence
            .iter()
            .filter_map(|value| match value {
                yaml_serde::Value::String(text) => Some(text.clone()),
                _ => None,
            })
            .collect();
    }
    Vec::new()
}

fn section_sequence(doc: &Document, heading: &str) -> Vec<String> {
    let name = document::section_name(heading);
    let Some(text) = document::section_text(doc, name) else {
        return Vec::new();
    };
    let mut items = Vec::new();
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
        if !item.is_empty() {
            items.push(item.to_string());
        }
    }
    items
}

fn record_list_values(doc: &Document, field: &FieldSpec) -> Vec<String> {
    if let Some(key) = &field.frontmatter_key {
        let values = frontmatter_sequence(doc, key);
        if !values.is_empty() {
            return values;
        }
    }
    if let Some(heading) = &field.heading {
        return section_sequence(doc, heading);
    }
    Vec::new()
}

fn initial_document(id: &str, title: &str) -> Document {
    let heading = format!("# {id} {title}");
    Document {
        source: format!("{heading}\n"),
        front_matter: None,
        frontmatter: None,
        heading: Some(heading),
        preamble: vec![String::new()],
        sections: Vec::new(),
    }
}

fn append_field_create_ops(
    descriptor: &TypeDescriptor,
    fields: &[FieldSpec],
    args: &DocumentArgs,
    body_ops: &mut Vec<PatchOp>,
) -> Result<(), AppError> {
    for field in fields {
        if !field_offered(field, DocumentVerb::Create) {
            continue;
        }
        let default = descriptor.field_defaults.get(&field.key);
        let present = args.get(&field.key).is_some() || default.is_some();
        if !present {
            continue;
        }
        match field.kind {
            crate::config::FieldKind::List => {
                let mut values = if args.get(&field.key).is_some() {
                    list_values(args, field)?
                } else {
                    default.into_iter().cloned().collect()
                };
                values.dedup();
                if let Some(heading) = &field.heading
                    && !values.is_empty()
                {
                    body_ops.push(PatchOp::SetSection(
                        heading.clone(),
                        list_section_body(&values),
                    ));
                }
                if let Some(key) = &field.frontmatter_key {
                    body_ops.push(PatchOp::SetFrontMatterValue(
                        key.clone(),
                        record::frontmatter::FrontMatterValue::Sequence(values),
                    ));
                }
            }
            crate::config::FieldKind::Text | crate::config::FieldKind::Multiline => {
                let value = if args.get(&field.key).is_some() {
                    scalar_value(args, field)?.unwrap_or_default()
                } else {
                    default.cloned().unwrap_or_default()
                };
                if let Some(heading) = &field.heading {
                    body_ops.push(PatchOp::SetSection(
                        heading.clone(),
                        scalar_section_body(&value),
                    ));
                }
                if let Some(key) = &field.frontmatter_key {
                    body_ops.push(PatchOp::SetFrontMatterValue(
                        key.clone(),
                        record::frontmatter::FrontMatterValue::String(value),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Build the create patch for a Markdown model. The engine emits only
/// `PatchOp` values; `record::patch` renders the single final write.
#[allow(clippy::too_many_arguments)]
fn create_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    scope: &DocumentScope,
    invocation: &DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let fields = document_fields_for_verb(descriptor, DocumentVerb::Create);
    let mut args = merge_invocation_args(invocation, &fields);
    resolve_field_stdin(&fields, &mut args)?;
    let title = required_title(&args)?;
    ensure_required_fields(&fields, &args, &descriptor.field_defaults)?;

    let active = scope.active.clone();
    let archived = scope.archived.clone();
    fs::create_dir_all(&active).map_err(|error| AppError::Runtime(error.to_string()))?;
    if descriptor.document_model == DocumentModel::GroupedMarkdown {
        fs::create_dir_all(&archived).map_err(|error| AppError::Runtime(error.to_string()))?;
    }

    let (id, group_link) = if descriptor.document_model == DocumentModel::GroupedMarkdown {
        let group_raw = args
            .get("group")
            .ok_or_else(|| AppError::Validation("missing required argument: group".to_string()))?;
        let group = resolve_group(ctx, cfg, registry, scope, group_raw)?;
        let next = record::identity::next_seq_number(&active, &archived, &group.id);
        let id = record::identity::format_seq_id(&group.id, next as i64, scope.id_padding);
        (id, Some(group))
    } else {
        let next = record::identity::next_number(&active, &archived, &scope.prefix);
        (
            record::identity::format_id(&scope.prefix, next as i64, scope.id_padding),
            None,
        )
    };

    let now = record::frontmatter::utc_now();
    let labels = sanitize_labels(args.get_all("label"))?;
    let references = sanitize_references(args.get_all("reference"))?;
    let labels_value = if labels.is_empty() {
        "[]".to_string()
    } else {
        labels.join("\n")
    };
    let references_value = if references.is_empty() {
        "[]".to_string()
    } else {
        references.join("\n")
    };

    let mut ops = vec![
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
            record::frontmatter::FrontMatterValue::String(now),
        ),
        PatchOp::SetFrontMatterValue(
            "labels".to_string(),
            record::frontmatter::FrontMatterValue::String(labels_value),
        ),
        PatchOp::SetFrontMatterValue(
            "references".to_string(),
            record::frontmatter::FrontMatterValue::String(references_value),
        ),
    ];
    // Every front-matter-backed Markdown model seeds its status from the
    // type's configured `default_status`, or `active` when none is declared
    // (REQ-05.011). The HTML mockup keeps `pending` in its HTML metadata and
    // never reaches this path.
    if let StatusModel::DocumentMain { default, .. } = &descriptor.status_model {
        ops.push(PatchOp::SetFrontMatterValue(
            "status".to_string(),
            record::frontmatter::FrontMatterValue::String(default.clone()),
        ));
    }
    ops.push(PatchOp::SetHeadingTitle(title.clone()));

    let mut body_ops = Vec::new();
    append_field_create_ops(descriptor, &fields, &args, &mut body_ops)?;
    if let Some(group) = &group_link {
        // The group link is a reserved, engine-owned front-matter key. It is
        // never a field, has no option, and cannot be changed after creation.
        ops.push(PatchOp::SetFrontMatterValue(
            "group".to_string(),
            record::frontmatter::FrontMatterValue::String(group.id.clone()),
        ));
    }
    ops.extend(body_ops);

    let doc = initial_document(&id, &title);
    let content = patch::apply_patch(&doc, &ops)?;
    let path = active.join(format!("{id}.md"));
    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
    record::store::write_atomic(ctx, cfg, registry, &safe_path, &content)?;
    Ok(typed_created(ctx, id))
}

/// The engine-owned grouped create-group surface, transitional to TASK-256.
#[allow(clippy::too_many_arguments)]
fn create_group_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    scope: &DocumentScope,
    invocation: &DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let fields = document_fields_for_verb(descriptor, DocumentVerb::CreateGroup);
    let args = merge_invocation_args(invocation, &fields);
    let title = required_title(&args)?;

    let active = scope.active.clone();
    let archived = scope.archived.clone();
    fs::create_dir_all(&active).map_err(|error| AppError::Runtime(error.to_string()))?;
    let next = record::identity::next_number(&active, &archived, &scope.prefix);
    let id = record::identity::format_id(&scope.prefix, next as i64, scope.id_padding);

    let mut ops = Vec::new();
    // Every front-matter-backed Markdown model seeds its status from the
    // type's configured `default_status`, or `active` when none is declared
    // (REQ-05.011). A group is a record of the same type.
    if let StatusModel::DocumentMain { default, .. } = &descriptor.status_model {
        ops.push(PatchOp::SetFrontMatterValue(
            "status".to_string(),
            record::frontmatter::FrontMatterValue::String(default.clone()),
        ));
    }
    ops.push(PatchOp::SetHeadingTitle(title.clone()));
    for field in &fields {
        if !field_offered(field, DocumentVerb::CreateGroup) || args.get(&field.key).is_none() {
            continue;
        }
        let Some(heading) = &field.heading else {
            continue;
        };
        match field.kind {
            crate::config::FieldKind::List => {
                let values = list_values(&args, field)?;
                if !values.is_empty() {
                    ops.push(PatchOp::SetSection(
                        heading.clone(),
                        list_section_body(&values),
                    ));
                }
            }
            crate::config::FieldKind::Text | crate::config::FieldKind::Multiline => {
                if let Some(value) = scalar_value(&args, field)? {
                    ops.push(PatchOp::SetSection(
                        heading.clone(),
                        scalar_section_body(&value),
                    ));
                }
            }
        }
    }

    let doc = initial_document(&id, &title);
    let content = patch::apply_patch(&doc, &ops)?;
    let path = active.join(format!("{id}.md"));
    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
    record::store::write_atomic(ctx, cfg, registry, &safe_path, &content)?;
    Ok(typed_created(ctx, id))
}

struct GroupLink {
    id: String,
}

/// Resolve the owning group of a grouped member create.
///
/// The group must already exist as an active record of the same document type.
/// An unknown, archived, or cross-type group is one validation outcome and
/// writes nothing (ADR-036 decision 9).
fn resolve_group(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    scope: &DocumentScope,
    group_raw: &str,
) -> Result<GroupLink, AppError> {
    let group_id = sanitize_field(group_raw)?;
    if !record::identity::is_group_id(&group_id, &scope.prefix) {
        let other = registry.descriptors().find(|candidate| {
            candidate.category == ArtifactCategory::Document
                && candidate.document_model == DocumentModel::GroupedMarkdown
                && candidate.prefix != scope.prefix
                && record::identity::is_group_id(&group_id, &candidate.prefix)
        });
        if let Some(other) = other
            && let Ok(other_scope) = DocumentScope::resolve(ctx, cfg, other)
            && (other_scope.active.join(format!("{group_id}.md")).exists()
                || other_scope.archived.join(format!("{group_id}.md")).exists())
        {
            return Err(AppError::Validation(format!(
                "group type mismatch: {group_id}"
            )));
        }
        return Err(AppError::Validation(format!("group not found: {group_id}")));
    }

    let active_path = scope.active.join(format!("{group_id}.md"));
    let archived_path = scope.archived.join(format!("{group_id}.md"));
    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let archived_safe = Context::ensure_within(&ctx.project_dir, &archived_path)?;
    if active_safe.exists() {
        // Read the group so a boundary escape or unreadable record is rejected
        // before a member is allocated; the title itself is not stored again.
        let (doc, _token) = record::store::read_document(ctx, cfg, registry, &active_safe)?;
        let (_, title) = document::heading_id_title(&doc);
        if title.is_empty() {
            return Err(AppError::Runtime(format!("group has no title: {group_id}")));
        }
        return Ok(GroupLink { id: group_id });
    }
    if archived_safe.exists() {
        return Err(AppError::Validation(format!(
            "group is archived: {group_id}"
        )));
    }
    Err(AppError::Validation(format!("group not found: {group_id}")))
}

fn list_documents(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    scope: &DocumentScope,
) -> Result<crate::CommandOutput, AppError> {
    let active = scope.active.clone();
    let _ = fs::create_dir_all(&active);
    let dotted = descriptor.id_shape == crate::registry::IdShape::Grouped;
    let prefixes = vec![scope.prefix.clone()];
    let mut records = record::resolve::list_in_dir(ctx, cfg, registry, &active, &prefixes, dotted)?;
    if descriptor.document_model == DocumentModel::GroupedMarkdown && dotted {
        for group in record::resolve::list_in_dir(ctx, cfg, registry, &active, &prefixes, false)? {
            if !group.id.contains('.') {
                records.push(group);
            }
        }
    }
    let mut items: Vec<(String, String, String)> = Vec::new();
    for entry in records {
        if entry.path.parent() != Some(active.as_path()) {
            continue;
        }
        let (doc, _token) = record::store::read_document(ctx, cfg, registry, &entry.path)?;
        let content = doc.source.clone();
        let title = if descriptor.is_config_defined_document() {
            document::heading_id_title_with_frontmatter(&doc).1
        } else {
            document::heading_id_title(&doc).1
        };
        let title = if title.is_empty() {
            entry.id.clone()
        } else {
            title
        };
        items.push((entry.id, title, content));
    }
    items.sort_by(|left, right| left.0.cmp(&right.0));

    let text = items
        .iter()
        .map(|(id, title, content)| record::view::RecordView::lenient(content, id).list_line(title))
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let array: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, title, content)| {
                let metadata = record::view::RecordView::lenient(content, id).metadata_json();
                if matches!(descriptor.handler(), Some(TypedHandler::Glossary)) {
                    json!({
                        "id": id,
                        "term": title,
                        "title": title,
                        "metadata": metadata,
                    })
                } else {
                    json!({
                        "id": id,
                        "title": title,
                        "metadata": metadata,
                    })
                }
            })
            .collect();
        Some(serde_json::Value::Array(array))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn resolve_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    scope: &DocumentScope,
    descriptor: &TypeDescriptor,
    requested: &str,
) -> Result<record::resolve::ResolvedRecord, AppError> {
    // A config-defined type keeps its historical active-only edit/view
    // resolution; seed types read the archived directory too (TECH-066).
    let mut dirs = vec![scope.active.clone()];
    if !descriptor.is_config_defined_document() {
        dirs.push(scope.archived.clone());
    }
    let prefixes = vec![scope.prefix.clone()];
    let dotted = descriptor.id_shape == crate::registry::IdShape::Grouped;
    if let Some(found) =
        record::resolve::resolve_in_dirs(ctx, cfg, registry, &dirs, requested, &prefixes, dotted)?
    {
        return Ok(found);
    }
    if dotted
        && let Some(found) = record::resolve::resolve_in_dirs(
            ctx, cfg, registry, &dirs, requested, &prefixes, false,
        )?
    {
        return Ok(found);
    }
    if descriptor.is_config_defined_document() {
        Err(AppError::NotFound(format!("record not found: {requested}")))
    } else {
        Err(AppError::NotFound(format!("not found: {requested}")))
    }
}

fn preserves_trailing_newline(descriptor: &TypeDescriptor) -> bool {
    // Seed-inexpressible quirk retained deliberately (TASK-255, TASK-259): the
    // glossary and requirement modules historically printed the record with its
    // trailing newline. The handler mapping keeps it until MS-035.
    matches!(
        descriptor.handler(),
        Some(TypedHandler::Glossary) | Some(TypedHandler::Requirement(_))
    )
}

fn view_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    scope: &DocumentScope,
    id: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    let requested = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&requested)?;
    let resolved = resolve_document(ctx, cfg, registry, scope, descriptor, &requested)?;
    let path = resolved.path.clone();
    let id = resolved.id.clone();
    let content = if descriptor.is_config_defined_document() {
        record::store::read_document(ctx, cfg, registry, &path)?
            .0
            .source
    } else {
        record::migrate::ensure_front_matter(ctx, cfg, registry, &path)?
    };
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
    let text = if preserves_trailing_newline(descriptor) {
        content
    } else {
        content.trim_end_matches('\n').to_string()
    };
    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn clear_field_ops(ops: &mut Vec<PatchOp>, field: &FieldSpec) {
    if let Some(heading) = &field.heading {
        ops.push(PatchOp::ClearSection(heading.clone()));
    }
    if let Some(key) = &field.frontmatter_key {
        ops.push(PatchOp::ClearFrontMatter(key.clone()));
    }
}

fn set_scalar_ops(ops: &mut Vec<PatchOp>, field: &FieldSpec, value: &str) {
    if let Some(heading) = &field.heading {
        ops.push(PatchOp::SetSection(
            heading.clone(),
            scalar_section_body(value),
        ));
    }
    if let Some(key) = &field.frontmatter_key {
        ops.push(PatchOp::SetFrontMatterValue(
            key.clone(),
            record::frontmatter::FrontMatterValue::String(value.to_string()),
        ));
    }
}

fn list_field_ops(ops: &mut Vec<PatchOp>, field: &FieldSpec, values: &[String]) {
    if let Some(heading) = &field.heading {
        if values.is_empty() {
            ops.push(PatchOp::ClearSection(heading.clone()));
        } else {
            ops.push(PatchOp::SetSection(
                heading.clone(),
                list_section_body(values),
            ));
        }
    }
    if let Some(key) = &field.frontmatter_key {
        ops.push(PatchOp::SetFrontMatterValue(
            key.clone(),
            record::frontmatter::FrontMatterValue::Sequence(values.to_vec()),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn edit_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    scope: &DocumentScope,
    invocation: &DocumentInvocation,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = invocation
        .id
        .clone()
        .ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let resolved = resolve_document(ctx, cfg, registry, scope, descriptor, &requested)?;
    resolved.ensure_writable()?;
    let id = resolved.id.clone();
    let path = resolved.path.clone();
    if descriptor.is_config_defined_document() && path.parent() != Some(scope.active.as_path()) {
        return Err(AppError::Validation(format!(
            "record {} is archived; edits are not allowed",
            id
        )));
    }

    let fields = if descriptor.document_model == DocumentModel::GroupedMarkdown && !id.contains('.')
    {
        descriptor
            .group
            .as_ref()
            .and_then(|role| role.fields.clone())
            .unwrap_or_default()
    } else {
        document_fields_for_verb(descriptor, DocumentVerb::Edit)
    };
    let mut args = merge_invocation_args(invocation, &fields);
    resolve_field_stdin(&fields, &mut args)?;

    let editor_command = if invocation.editor {
        Some(
            cfg.effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    let (doc, _token) = record::store::read_document(ctx, cfg, registry, &path)?;
    let content = doc.source.clone();
    let mut ops: Vec<PatchOp> = Vec::new();

    if let Some(title) = optional_title(&args)? {
        ops.push(PatchOp::SetFrontMatter("title".into(), title.clone()));
        ops.push(PatchOp::SetHeadingTitle(title.clone()));
    }

    for field in &fields {
        if !field_offered(field, DocumentVerb::Edit) {
            continue;
        }
        match field.kind {
            crate::config::FieldKind::List => {
                let add_key = &field.key;
                let remove_key = format!("no_{}", field.key);
                let has_add = args.get(add_key).is_some();
                let has_remove = args.optional(&remove_key).is_some();
                if !has_add && !has_remove {
                    continue;
                }
                let mut values = record_list_values(&doc, field);
                if has_add {
                    for value in list_values(&args, field)? {
                        if !values.contains(&value) {
                            values.push(value);
                        }
                    }
                }
                if let Some(removal) = args.optional(&remove_key) {
                    match removal {
                        Some(raw) => {
                            let remove = sanitize_field(raw)?;
                            values.retain(|value| value != &remove);
                        }
                        None => values.clear(),
                    }
                }
                if field.required && values.is_empty() {
                    return Err(AppError::Validation(format!(
                        "field '{}' is required and cannot be cleared",
                        field.key
                    )));
                }
                list_field_ops(&mut ops, field, &values);
            }
            crate::config::FieldKind::Text | crate::config::FieldKind::Multiline => {
                if args.get(&field.key).is_none() {
                    continue;
                }
                let text = args.text(&field.key).expect("option is present");
                if text.as_str().trim().is_empty() {
                    if field.required {
                        return Err(AppError::Validation(format!(
                            "field '{}' is required and cannot be cleared",
                            field.key
                        )));
                    }
                    clear_field_ops(&mut ops, field);
                } else {
                    let value = text.resolve()?;
                    set_scalar_ops(&mut ops, field, &value);
                }
            }
        }
    }

    if !args.get_all("label").is_empty() {
        let mut labels = current_list(&doc, "labels");
        for label in sanitize_labels(args.get_all("label"))? {
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
        ops.push(PatchOp::SetFrontMatter("labels".into(), labels.join("\n")));
    }
    if !args.get_all("reference").is_empty() {
        let mut references = current_list(&doc, "references");
        for reference in sanitize_references(args.get_all("reference"))? {
            if !references.contains(&reference) {
                references.push(reference);
            }
        }
        ops.push(PatchOp::SetFrontMatter(
            "references".into(),
            references.join("\n"),
        ));
    }

    let candidate = patch::apply_patch(&doc, &ops)?;
    if candidate != content {
        let mut final_ops = ops;
        final_ops.push(PatchOp::SetFrontMatter(
            "updated_date".into(),
            record::frontmatter::utc_now(),
        ));
        let new_content = patch::apply_patch(&doc, &final_ops)?;
        if new_content != content {
            let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
            record::store::write_atomic(ctx, cfg, registry, &safe_path, &new_content)?;
        }
    }

    if let Some(command) = editor_command {
        let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
        run_editor(&command, &safe_path)?;
        // Seed-inexpressible quirk retained deliberately (TASK-255, TASK-259):
        // a ui-spec body rewritten by the editor re-adopts front matter.
        if matches!(descriptor.handler(), Some(TypedHandler::UiSpec)) {
            record::migrate::ensure_front_matter(ctx, cfg, registry, &safe_path)?;
        }
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
    cfg: &Config,
    registry: &ArtifactRegistry,
    record: &RecordDescriptor,
    id: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let _ = record::store::archive(ctx, cfg, registry, record, &requested)?;
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
    cfg: &Config,
    registry: &ArtifactRegistry,
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
    record::store::delete(ctx, cfg, registry, record, &requested)?;
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
    cfg: &Config,
    registry: &ArtifactRegistry,
    record: &RecordDescriptor,
    id: Option<String>,
    value: Option<String>,
) -> Result<crate::CommandOutput, AppError> {
    // Read-path auto-relocation: see `list_documents`; a record relocated during
    // the read step fails loudly (`Err(Validation)` on destination-write failure,
    // `Err(Runtime)` naming both paths on source-unlink failure) instead of
    // returning a read-only handle.
    let id_raw = id.ok_or_else(|| AppError::Validation("id is required".to_string()))?;
    let requested = sanitize_field(&id_raw)?;
    let value =
        value.ok_or_else(|| AppError::Validation("status value is required".to_string()))?;
    // Auto-relocate archived-in-active records and surface undeclared statuses.
    let read_dirs = record.resolve_read_directories(cfg, &ctx.project_dir);
    let prefixes = record.configured_prefixes(cfg);
    let dotted = record.is_dotted_id();
    if let Some(rec) = record::resolve::resolve_in_dirs(
        ctx, cfg, registry, &read_dirs, &requested, &prefixes, dotted,
    )? {
        let _ = record::store::read_document(ctx, cfg, registry, &rec.path)?;
    }
    record::store::set_status(ctx, cfg, registry, record, &requested, &value)?;
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
    fn access(c: &Context) -> (crate::config::Config, crate::registry::ArtifactRegistry) {
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir)
            .unwrap_or_default();
        let registry = crate::registry::ArtifactRegistry::build(&cfg).expect("registry");
        (cfg, registry)
    }

    fn load_config(ctx: &Context) -> Result<Config, AppError> {
        crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir)
    }
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
                ..Default::default()
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
        DocumentScope::resolve(ctx, &load_config(ctx).unwrap(), descriptor).unwrap()
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
            description_multiline: false,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
    fn generic_create_writes_status_for_a_seed_markdown_record() {
        let dir = tmpdir("seedstatus");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let mut invocation = generic_invocation(
            "glossary",
            DocumentVerb::Create,
            None,
            Some("Cache".to_string()),
            None,
            vec![],
            vec![],
            None,
        );
        invocation.args.insert(
            "definition",
            vec!["A store of reusable results".to_string()],
        );
        let out = run(&c, &access(&c).0, &reg, invocation).unwrap();
        assert_eq!(out.text, "GLO-001");
        let content = fs::read_to_string(dir.join("docs/glossary/GLO-001.md")).unwrap();
        assert!(content.contains("status: active"), "{content}");
    }

    #[test]
    fn generic_create_group_writes_status_for_a_grouped_type() {
        let dir = tmpdir("groupstatus");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &access(&c).0,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::CreateGroup,
                None,
                Some("Group one".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap();
        assert_eq!(out.text, "REQ-001");
        let content =
            fs::read_to_string(dir.join("docs/requirements/business/REQ-001.md")).unwrap();
        assert!(content.contains("status: active"), "{content}");
    }

    #[test]
    fn generic_create_allocates_across_active_and_archived() {
        let dir = tmpdir("alloc");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        let out = run(
            &c,
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
        let req_group = run(
            &c,
            &access(&c).0,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::CreateGroup,
                None,
                Some("G".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap()
        .text;
        let mut req_create = generic_invocation(
            "req",
            DocumentVerb::Create,
            None,
            Some("R".to_string()),
            Some("desc".to_string()),
            vec![],
            vec![],
            None,
        );
        req_create.args.insert("group", vec![req_group]);
        let rid = run(&c, &access(&c).0, &reg, req_create).unwrap().text;
        let nfr_group = run(
            &c,
            &access(&c).0,
            &reg,
            generic_invocation(
                "nfr",
                DocumentVerb::CreateGroup,
                None,
                Some("NG".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap()
        .text;
        let mut nfr_create = generic_invocation(
            "nfr",
            DocumentVerb::Create,
            None,
            Some("NR".to_string()),
            Some("desc".to_string()),
            vec![],
            vec![],
            None,
        );
        nfr_create.args.insert("group", vec![nfr_group]);
        let nid = run(&c, &access(&c).0, &reg, nfr_create).unwrap().text;

        let out = run(
            &c,
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).1,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        let out = crate::ui_mockup::handle(
            &c,
            &scope,
            &access(&c).1,
            DocumentVerb::List,
            &DocumentArgs::default(),
        )
        .unwrap();
        assert!(out.text.contains(&id));
        let out = crate::ui_mockup::handle(
            &c,
            &scope,
            &access(&c).1,
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
            &access(&c).0,
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
            description_multiline: false,
            labels: vec![],
            references: vec![],
            editor: true,
            status_value: None,
            force: true,
            args: DocumentArgs::default(),
        };
        run(&c, &access(&c).0, &reg, inv).unwrap();
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
            &access(&c).0,
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
        let req_group = run(
            &c,
            &access(&c).0,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::CreateGroup,
                None,
                Some("G".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap()
        .text;
        let mut req_create = generic_invocation(
            "req",
            DocumentVerb::Create,
            None,
            Some("R".to_string()),
            Some("desc".to_string()),
            vec![],
            vec![],
            None,
        );
        req_create.args.insert("group", vec![req_group]);
        let rid = run(&c, &access(&c).0, &reg, req_create).unwrap().text;
        let out = run(
            &c,
            &access(&c).0,
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
            &access(&c).0,
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
        let reg = registry(&c);
        let gid = run(
            &c,
            &access(&c).0,
            &reg,
            generic_invocation(
                "req",
                DocumentVerb::CreateGroup,
                None,
                Some("G".to_string()),
                None,
                vec![],
                vec![],
                None,
            ),
        )
        .unwrap()
        .text;
        assert!(!gid.is_empty());
        let mock_scope = scope_for(&c, "ui-mockup");
        let id = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            &access(&c).1,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M1"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        let id2 = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            &access(&c).1,
            DocumentVerb::Create,
            &typed_args(&[("title", &["M2"]), ("task", &["T"]), ("spec", &["S"])]),
        )
        .unwrap()
        .text;
        crate::ui_mockup::handle(
            &c,
            &mock_scope,
            &access(&c).1,
            DocumentVerb::Edit,
            &typed_args(&[("id", &[id.as_str()]), ("superseded_by", &[id2.as_str()])]),
        )
        .unwrap();
        let out = crate::ui_mockup::handle(
            &c,
            &mock_scope,
            &access(&c).1,
            DocumentVerb::View,
            &typed_args(&[("id", &[id.as_str()])]),
        )
        .unwrap();
        assert!(out.text.contains(&id2));
    }

    #[test]
    fn handler_mapping_keeps_the_view_trailing_newline_quirk() {
        let dir = tmpdir("newline-quirk");
        default_config(&dir);
        let c = ctx(&dir);
        let reg = registry(&c);
        for slug in ["glossary", "req", "nfr"] {
            let descriptor = reg.descriptor(slug).unwrap_or_else(|| panic!("{slug}"));
            assert!(preserves_trailing_newline(descriptor), "{slug}");
        }
        for slug in ["rule", "ui-spec", "tech-spec", "audit", "use-case"] {
            let descriptor = reg.descriptor(slug).unwrap_or_else(|| panic!("{slug}"));
            assert!(!preserves_trailing_newline(descriptor), "{slug}");
        }
    }
}
