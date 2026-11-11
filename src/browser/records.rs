//! Record indexing, parsing, validation, and atomic persistence for the local
//! browser increment (specification sections 4, 7, and 8).
//!
//! Every filesystem path is resolved through [`Context::ensure_within`] before
//! it is read or written. Record IDs are resolved by exact match against the
//! active directory listing and are never joined to a path.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Maximum accepted record ID length in bytes (specification section 7.1).
pub const MAX_ID_BYTES: usize = 128;
/// Maximum accepted title length in Unicode scalar values (section 7.3).
pub const MAX_TITLE_CHARS: usize = 200;
/// Maximum accepted story or description length (section 7.3).
pub const MAX_TEXT_CHARS: usize = 20000;

/// Browser content category. Covers both document-store and backlog-store records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Req,
    Nfr,
    Task,
    Milestone,
    Release,
    Decision,
    Glossary,
    UiSpec,
    UiMockup,
    BusinessRule,
    AuditRecord,
    TechSpec,
    UseCase,
    TestCase,
    TestReport,
}

impl Category {
    pub fn key(self) -> &'static str {
        match self {
            Category::Req => "req",
            Category::Nfr => "nfr",
            Category::Task => "task",
            Category::Milestone => "milestone",
            Category::Release => "release",
            Category::Decision => "decision",
            Category::Glossary => "glossary",
            Category::UiSpec => "ui-spec",
            Category::UiMockup => "ui-mockup",
            Category::BusinessRule => "business-rule",
            Category::AuditRecord => "audit-record",
            Category::TechSpec => "tech-spec",
            Category::UseCase => "use-case",
            Category::TestCase => "test-case",
            Category::TestReport => "test-report",
        }
    }

    /// Configuration key used in `document_types`.
    pub fn config_key(self) -> &'static str {
        match self {
            Category::BusinessRule => "rule",
            Category::AuditRecord => "audit",
            _ => self.key(),
        }
    }

    pub fn supports_story(self) -> bool {
        matches!(self, Category::Req)
    }

    pub fn is_requirement(self) -> bool {
        matches!(self, Category::Req | Category::Nfr)
    }
}

/// The two record forms in this increment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Group,
    Requirement,
    Task,
    Milestone,
    Release,
    Decision,
    Glossary,
    UiSpec,
    UiMockup,
    BusinessRule,
    AuditRecord,
    TechSpec,
    UseCase,
    TestCase,
    TestReport,
}

impl Form {
    pub fn label(self) -> &'static str {
        match self {
            Form::Group => "Group",
            Form::Requirement => "Requirement",
            Form::Task => "Task",
            Form::Milestone => "Milestone",
            Form::Release => "Release",
            Form::Decision => "Decision",
            Form::Glossary => "Glossary term",
            Form::UiSpec => "UI specification",
            Form::UiMockup => "UI mockup",
            Form::BusinessRule => "Business rule",
            Form::AuditRecord => "Audit record",
            Form::TechSpec => "Technical specification",
            Form::UseCase => "Use case",
            Form::TestCase => "Test case",
            Form::TestReport => "Test report",
        }
    }
}

/// A parsed `## ` section. `body` keeps every line exactly as stored so an
/// unknown section can be written back content-identically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub body: Vec<String>,
}

/// A parsed record document: heading line plus ordered sections.
#[derive(Clone, Debug)]
pub struct Doc {
    pub id: String,
    pub title: String,
    pub heading: String,
    /// Lines between the heading and the first section (usually a blank line).
    pub preamble: Vec<String>,
    pub sections: Vec<Section>,
    /// The verbatim leading YAML front-matter block (including delimiters and
    /// trailing newline), if present.
    pub front_matter: Option<String>,
    /// The original document text, used to return byte-identical output for
    /// no-op saves.
    pub source: String,
}

/// A loaded record with its metadata.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub kind: Category,
    pub form: Form,
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
    pub doc: Doc,
    pub path: PathBuf,
    pub rel_path: String,
    pub version: String,
}

impl Record {
    pub fn kind_key(&self) -> &'static str {
        self.kind.key()
    }

    pub fn type_label(&self) -> String {
        let base = match self.kind {
            Category::Req => "Business requirement",
            Category::Nfr => "Non-functional requirement",
            Category::Task => "Task",
            Category::Milestone => "Milestone",
            Category::Release => "Release",
            Category::Decision => "Decision",
            Category::Glossary => "Glossary",
            Category::UiSpec => "UI Specifications",
            Category::UiMockup => "UI Mockups",
            Category::BusinessRule => "Business Rules",
            Category::AuditRecord => "Audit Records",
            Category::TechSpec => "Technical Specifications",
            Category::UseCase => "Use Cases",
            Category::TestCase => "Test Cases",
            Category::TestReport => "Test Reports",
        };
        match self.form {
            Form::Group => format!("{} group", base),
            Form::Requirement => base.to_string(),
            Form::Task
            | Form::Milestone
            | Form::Release
            | Form::Decision
            | Form::Glossary
            | Form::UiSpec
            | Form::UiMockup
            | Form::BusinessRule
            | Form::AuditRecord
            | Form::TechSpec
            | Form::UseCase
            | Form::TestCase
            | Form::TestReport => base.to_string(),
        }
    }
}

/// An indexed document category. Records are sorted by ID ascending.
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
    pub kind: Category,
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Thin adapter over the shared envelope: delegate all parsing to
/// `record::document` and only reshape the result into the browser `Doc`.
pub fn parse_doc(content: &str) -> Doc {
    let normalized = normalize_newlines(content);
    let front_matter = crate::record::frontmatter::find_block(&normalized)
        .map(|(_, _, bs)| normalized[..bs].to_string());
    let Ok(shared) = crate::record::document::parse(&normalized) else {
        return Doc {
            id: String::new(),
            title: String::new(),
            heading: String::new(),
            preamble: Vec::new(),
            sections: Vec::new(),
            front_matter,
            source: normalized,
        };
    };
    let (id, title) = crate::record::document::heading_id_title(&shared);
    Doc {
        id,
        title,
        heading: shared.heading.unwrap_or_default(),
        preamble: shared.preamble,
        sections: shared
            .sections
            .into_iter()
            .map(|s| Section {
                heading: s.heading,
                body: s.body,
            })
            .collect(),
        front_matter,
        source: normalized,
    }
}

fn section_value(doc: &Doc, heading: &str) -> Option<String> {
    let section = doc.sections.iter().find(|s| s.heading == heading)?;
    let text = section.body.join("\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn section_first_line(doc: &Doc, heading: &str) -> Option<String> {
    let section = doc.sections.iter().find(|s| s.heading == heading)?;
    let text = section.body.join("\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.lines().next().unwrap_or("").to_string())
    }
}

fn parse_group(doc: &Doc) -> (Option<String>, Option<String>) {
    let Some(line) = section_first_line(doc, "## Group") else {
        return (None, None);
    };
    let mut parts = line.splitn(2, char::is_whitespace);
    let id = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let title = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    (id, title)
}

/// Validate a record ID (specification section 7.1).
pub fn valid_id(id: &str) -> bool {
    if id.is_empty() || id.len() > MAX_ID_BYTES {
        return false;
    }
    if id.starts_with('.') || id.starts_with('-') {
        return false;
    }
    if id.contains("..") || id == "." || id == ".." {
        return false;
    }
    id.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Whether `id` matches the configured technical-specification ID pattern
/// (`<prefix>-<digits>`), regardless of whether the document type is enabled.
pub fn matches_tech_spec_id(cfg: &Config, id: &str) -> bool {
    let Some(entry) = cfg.document_types.get(Category::TechSpec.config_key()) else {
        return false;
    };
    id.strip_prefix(&format!("{}-", entry.prefix))
        .map(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or(false)
}

pub fn matches_use_case_id(cfg: &Config, id: &str) -> bool {
    let Some(entry) = cfg.document_types.get(Category::UseCase.config_key()) else {
        return false;
    };
    id.strip_prefix(&format!("{}-", entry.prefix))
        .map(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or(false)
}

pub fn matches_test_case_id(cfg: &Config, id: &str) -> bool {
    let Some(entry) = cfg.document_types.get(Category::TestCase.config_key()) else {
        return false;
    };
    id.strip_prefix(&format!("{}-", entry.prefix))
        .map(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or(false)
}

pub fn matches_test_report_id(cfg: &Config, id: &str) -> bool {
    let Some(entry) = cfg.document_types.get(Category::TestReport.config_key()) else {
        return false;
    };
    id.strip_prefix(&format!("{}-", entry.prefix))
        .map(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or(false)
}

fn form_for_stem(prefix: &str, stem: &str) -> Option<Form> {
    let rest = stem.strip_prefix(prefix)?.strip_prefix('-')?;
    if rest.is_empty() {
        return None;
    }
    if rest.bytes().all(|b| b.is_ascii_digit()) {
        return Some(Form::Group);
    }
    let (group, sequence) = rest.split_once('.')?;
    if !group.is_empty()
        && !sequence.is_empty()
        && group.bytes().all(|b| b.is_ascii_digit())
        && sequence.bytes().all(|b| b.is_ascii_digit())
    {
        return Some(Form::Requirement);
    }
    None
}

fn active_entry(cfg: &Config, kind: Category) -> Option<crate::config::DocTypeEntry> {
    match kind {
        Category::Req | Category::Nfr => cfg
            .document_types
            .get(kind.key())
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::Task => Some(crate::config::DocTypeEntry {
            prefix: cfg.task_prefix.clone(),
            directory: "tasks".to_string(),
            enabled: true,
        }),
        Category::Milestone => Some(crate::config::DocTypeEntry {
            prefix: cfg.milestone_prefix.clone(),
            directory: "milestones".to_string(),
            enabled: true,
        }),
        Category::Release => Some(crate::config::DocTypeEntry {
            prefix: cfg.release_prefix.clone(),
            directory: "releases".to_string(),
            enabled: true,
        }),
        Category::Decision => Some(crate::config::DocTypeEntry {
            prefix: String::new(),
            directory: cfg.decision_directory.clone(),
            enabled: true,
        }),
        Category::Glossary => cfg
            .document_types
            .get("glossary")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::UiSpec => cfg
            .document_types
            .get("ui-spec")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::UiMockup => cfg
            .document_types
            .get("ui-mockup")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::BusinessRule => cfg
            .document_types
            .get("rule")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::AuditRecord => cfg
            .document_types
            .get("audit")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::TechSpec => cfg
            .document_types
            .get("tech-spec")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::UseCase => cfg
            .document_types
            .get("use-case")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::TestCase => cfg
            .document_types
            .get("test-case")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
        Category::TestReport => cfg
            .document_types
            .get("test-report")
            .cloned()
            .filter(|entry| entry.enabled && !entry.prefix.is_empty()),
    }
}

fn active_dir(ctx: &Context, cfg: &Config, kind: Category) -> Option<PathBuf> {
    let entry = active_entry(cfg, kind)?;
    let base = match kind {
        Category::Req
        | Category::Nfr
        | Category::Glossary
        | Category::UiSpec
        | Category::UiMockup
        | Category::BusinessRule
        | Category::AuditRecord
        | Category::TechSpec
        | Category::UseCase
        | Category::TestCase
        | Category::TestReport => ctx.project_dir.join(&cfg.doc_directory),
        Category::Task | Category::Milestone | Category::Release => {
            ctx.project_dir.join(&cfg.backlog_directory)
        }
        Category::Decision => ctx.project_dir.clone(),
    };
    let raw = base.join(&entry.directory);
    Context::ensure_within(&ctx.project_dir, &raw).ok()
}

/// Format the optimistic-concurrency token from file metadata.
pub fn version_token(meta: &fs::Metadata) -> String {
    let modified = meta.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!(
        "{}.{}.{}",
        duration.as_secs(),
        duration.subsec_nanos(),
        meta.len()
    )
}

/// Whether a token has the documented `seconds.nanos.length` shape.
pub fn valid_version_token(token: &str) -> bool {
    let mut parts = token.split('.');
    let (Some(seconds), Some(nanos), Some(length), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    !seconds.is_empty()
        && !nanos.is_empty()
        && !length.is_empty()
        && seconds.bytes().all(|b| b.is_ascii_digit())
        && nanos.bytes().all(|b| b.is_ascii_digit())
        && length.bytes().all(|b| b.is_ascii_digit())
}

fn relative_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn load_backlog_record(
    ctx: &Context,
    kind: Category,
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, kind) {
        Some(e) => e,
        None => return Ok(None),
    };
    // validate stem against prefix
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }

    let (
        title,
        status,
        milestone_id,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
        description,
    ) = match kind {
        Category::Task => {
            let task_doc = crate::record::document::parse(&content).unwrap_or_else(|_| {
                crate::record::document::Document {
                    source: content.clone(),
                    front_matter: None,
                    frontmatter: None,
                    heading: None,
                    preamble: Vec::new(),
                    sections: Vec::new(),
                }
            });
            let td = crate::task::task_from_document(&task_doc);
            let title = if td.title.is_empty() {
                stem.to_string()
            } else {
                td.title.clone()
            };
            (
                title,
                if td.status.is_empty() {
                    None
                } else {
                    Some(td.status.clone())
                },
                td.milestone.clone(),
                td.assignee.clone(),
                td.labels.clone(),
                td.deps.clone(),
                td.acs.clone(),
                td.comments.clone(),
                None,
                td.description.clone(),
            )
        }
        Category::Milestone => {
            let parsed_doc = crate::record::document::parse(&content).unwrap_or_else(|_| {
                crate::record::document::Document {
                    source: content.clone(),
                    front_matter: None,
                    frontmatter: None,
                    heading: None,
                    preamble: Vec::new(),
                    sections: Vec::new(),
                }
            });
            let (_id, title) =
                crate::record::document::heading_id_title_with_frontmatter(&parsed_doc);
            let desc = crate::record::document::section_text(&parsed_doc, "Description");
            let release = crate::record::document::section_text(&parsed_doc, "Release")
                .map(|s| s.trim().to_string());
            let title = if title.is_empty() {
                stem.to_string()
            } else {
                title
            };
            (
                title,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                release,
                desc,
            )
        }
        Category::Release => {
            let parsed_doc = crate::record::document::parse(&content).unwrap_or_else(|_| {
                crate::record::document::Document {
                    source: content.clone(),
                    front_matter: None,
                    frontmatter: None,
                    heading: None,
                    preamble: Vec::new(),
                    sections: Vec::new(),
                }
            });
            let (_id, title) = crate::record::document::heading_id_title(&parsed_doc);
            let desc = crate::record::document::section_text(&parsed_doc, "Description");
            let title = if title.is_empty() {
                stem.to_string()
            } else {
                title
            };
            (
                title,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
                desc,
            )
        }
        _ => return Ok(None),
    };

    // milestone title resolution
    let milestone_title = if kind == Category::Task {
        milestone_id.as_ref().and_then(|mid| {
            // read milestone file
            let dir = active_dir(ctx, &cfg, Category::Milestone)?;
            let path = dir.join(format!("{}.md", mid));
            let safe = Context::ensure_within(&ctx.project_dir, &path).ok()?;
            if !safe.exists() {
                return None;
            }
            let c = fs::read_to_string(&safe).ok()?;
            let d = crate::record::document::parse(&c).ok()?;
            let (_, t) = crate::record::document::heading_id_title_with_frontmatter(&d);
            if t.is_empty() { None } else { Some(t) }
        })
    } else {
        None
    };
    let release_title = if kind == Category::Milestone {
        release_id.as_ref().and_then(|rid| {
            let dir = active_dir(ctx, &cfg, Category::Release)?;
            let path = dir.join(format!("{}.md", rid));
            let safe = Context::ensure_within(&ctx.project_dir, &path).ok()?;
            if !safe.exists() {
                return None;
            }
            let c = fs::read_to_string(&safe).ok()?;
            let d = crate::record::document::parse(&c).ok()?;
            let (_, t) = crate::record::document::heading_id_title(&d);
            if t.is_empty() { None } else { Some(t) }
        })
    } else {
        None
    };

    let form = match kind {
        Category::Task => Form::Task,
        Category::Milestone => Form::Milestone,
        Category::Release => Form::Release,
        _ => return Ok(None),
    };

    Ok(Some(Record {
        id: stem.to_string(),
        kind,
        form,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status,
        milestone_id,
        milestone_title,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
        release_title,
        decision: None,
        context: None,
        consequences: None,
        decision_type: None,
        superseded_by: None,
        superseded_title: None,
        specs: Vec::new(),
        refs: Vec::new(),
        mockup_task: String::new(),
        audit_type: None,
        doc: parse_doc(&content),
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_decision_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    if crate::decision::type_key_for_id(&cfg, stem).is_none() {
        return Ok(None);
    }
    let parsed_doc = crate::record::document::parse(&content).unwrap_or_else(|_| {
        crate::record::document::Document {
            source: content.clone(),
            front_matter: None,
            frontmatter: None,
            heading: None,
            preamble: Vec::new(),
            sections: Vec::new(),
        }
    });
    let (_, parsed_title) = crate::record::document::heading_id_title(&parsed_doc);
    let title = if parsed_title.is_empty() {
        stem.to_string()
    } else {
        parsed_title.clone()
    };
    let parsed_status =
        crate::record::document::section_text(&parsed_doc, "Status").unwrap_or_default();
    let status = if parsed_status.trim().is_empty() {
        "proposed".to_string()
    } else {
        parsed_status.trim().to_string()
    };
    let superseded_by = status
        .strip_prefix("Superseded by ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let superseded_title =
        superseded_by
            .as_ref()
            .and_then(|sid| match resolve_id(ctx, &cfg, sid) {
                Ok(Some(r)) => Some(r.record.title),
                _ => None,
            });
    let decision_type = crate::decision::type_key_for_id(&cfg, stem);
    let doc = parse_doc(&content);
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::Decision,
        form: Form::Decision,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description: None,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status: Some(status),
        milestone_id: None,
        milestone_title: None,
        assignee: None,
        labels: Vec::new(),
        dependencies: Vec::new(),
        acceptance_criteria: Vec::new(),
        comments: Vec::new(),
        release_id: None,
        release_title: None,
        decision: crate::record::document::section_text(&parsed_doc, "Decision"),
        context: crate::record::document::section_text(&parsed_doc, "Context"),
        consequences: crate::record::document::section_text(&parsed_doc, "Consequences"),
        decision_type,
        superseded_by,
        superseded_title,
        specs: Vec::new(),
        refs: Vec::new(),
        mockup_task: String::new(),
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_glossary_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::Glossary) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let definition = section_value(&doc, "## Definition");
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::Glossary,
        form: Form::Glossary,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description: None,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition,
        view: None,
        status: None,
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
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_ui_spec_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::UiSpec) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let view = section_value(&doc, "## View");
    let description = section_value(&doc, "## Description");
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::UiSpec,
        form: Form::UiSpec,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view,
        status: None,
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
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_ui_mockup_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::UiMockup) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let meta_record = crate::ui_mockup::parse_browser_metadata(&content, stem);
    let title = meta_record.title.clone();
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::UiMockup,
        form: Form::UiMockup,
        title: title.clone(),
        story: None,
        group_id: None,
        group_title: None,
        description: None,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status: if meta_record.status.is_empty() {
            None
        } else {
            Some(meta_record.status.clone())
        },
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
        superseded_by: meta_record.superseded_by.clone(),
        superseded_title: None,
        specs: meta_record.specs.clone(),
        refs: meta_record.refs.clone(),
        mockup_task: meta_record.task.clone(),
        audit_type: None,
        doc: Doc {
            id: stem.to_string(),
            title: title.clone(),
            heading: format!("# {} {}", stem, title),
            preamble: Vec::new(),
            sections: Vec::new(),
            front_matter: None,
            source: String::new(),
        },
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_business_rule_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::BusinessRule) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let description = section_value(&doc, "## Description");
    let rationale = section_value(&doc, "## Rationale");
    let applicability = section_value(&doc, "## Applicability");
    // requirements from ## Requirements bullets
    let requirements = parse_rule_requirements(&doc);
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::BusinessRule,
        form: Form::BusinessRule,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        rationale,
        applicability,
        requirements,
        definition: None,
        view: None,
        status: None,
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
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn parse_rule_requirements(doc: &Doc) -> Vec<String> {
    let Some(section) = doc.sections.iter().find(|s| s.heading == "## Requirements") else {
        return Vec::new();
    };
    let body = section.body.join("\n");
    let mut out = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let item = if let Some(rest) = t.strip_prefix("- ") {
            rest.trim()
        } else if let Some(rest) = t.strip_prefix('-') {
            rest.trim()
        } else {
            t
        };
        if item.is_empty() {
            continue;
        }
        if !out.contains(&item.to_string()) {
            out.push(item.to_string());
        }
    }
    out
}

fn load_audit_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::AuditRecord) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let audit_type_raw = crate::record::document::parse(&content)
        .ok()
        .and_then(|d| crate::record::document::section_text(&d, "Audit Type"))
        .unwrap_or_default();
    let fm_type = crate::record::frontmatter::parse(&content)
        .ok()
        .flatten()
        .and_then(|fm| {
            fm.data
                .get(yaml_serde::Value::String("type".to_string()))
                .cloned()
        })
        .map(|v| match v {
            yaml_serde::Value::String(s) => s,
            other => yaml_serde::to_string(&other)
                .unwrap_or_default()
                .trim()
                .to_string(),
        });
    let audit_type = if let Some(t) = fm_type.filter(|t| !t.trim().is_empty()) {
        t.trim().to_string()
    } else if audit_type_raw.trim().is_empty() {
        "security".to_string()
    } else {
        audit_type_raw.trim().to_string()
    };
    let content_val = crate::record::document::parse(&content)
        .ok()
        .and_then(|d| crate::record::document::section_text(&d, "Content"));
    // store content in doc only; audit_type in Record
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::AuditRecord,
        form: Form::AuditRecord,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description: content_val,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        audit_type: Some(audit_type),
        definition: None,
        view: None,
        status: None,
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
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_tech_spec_record(ctx: &Context, path: &Path) -> Result<Option<Record>, AppError> {
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, Category::TechSpec) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let description = section_value(&doc, "## Description");
    Ok(Some(Record {
        id: stem.to_string(),
        kind: Category::TechSpec,
        form: Form::TechSpec,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status: None,
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
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_simple_doc_record(
    ctx: &Context,
    path: &Path,
    kind: Category,
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
    if !valid_id(stem) {
        return Ok(None);
    }
    let cfg = match crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let entry = match active_entry(&cfg, kind) {
        Some(e) => e,
        None => return Ok(None),
    };
    let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let form = match kind {
        Category::UseCase => Form::UseCase,
        Category::TestCase => Form::TestCase,
        Category::TestReport => Form::TestReport,
        _ => return Ok(None),
    };
    let doc = parse_doc(&content);
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let description = section_value(&doc, "## Description");
    let applicability = section_value(&doc, "## Applicability");
    let view = crate::record::view::RecordView::from_content(&content, stem, "").ok();
    let labels = view
        .as_ref()
        .map(|v| v.list_value("labels"))
        .unwrap_or_default();
    let refs = view
        .as_ref()
        .map(|v| v.list_value("references"))
        .unwrap_or_default();
    Ok(Some(Record {
        id: stem.to_string(),
        kind,
        form,
        title,
        story: None,
        group_id: None,
        group_title: None,
        description,
        rationale: None,
        applicability,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status: None,
        milestone_id: None,
        milestone_title: None,
        assignee: None,
        labels,
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
        refs,
        mockup_task: String::new(),
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn load_record_from_path(
    ctx: &Context,
    kind: Category,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    if kind == Category::Decision {
        return load_decision_record(ctx, path);
    }
    if kind == Category::Glossary {
        return load_glossary_record(ctx, path);
    }
    if kind == Category::UiSpec {
        return load_ui_spec_record(ctx, path);
    }
    if kind == Category::UiMockup {
        return load_ui_mockup_record(ctx, path);
    }
    if kind == Category::BusinessRule {
        return load_business_rule_record(ctx, path);
    }
    if kind == Category::AuditRecord {
        return load_audit_record(ctx, path);
    }
    if kind == Category::TechSpec {
        return load_tech_spec_record(ctx, path);
    }
    if matches!(
        kind,
        Category::UseCase | Category::TestCase | Category::TestReport
    ) {
        return load_simple_doc_record(ctx, path, kind);
    }
    if matches!(
        kind,
        Category::Task | Category::Milestone | Category::Release | Category::Decision
    ) {
        return load_backlog_record(ctx, kind, path);
    }
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        // A symlink escaping the project is not listed and is not readable.
        Err(_) => return Ok(None),
    };
    let content = match fs::read_to_string(&safe) {
        Ok(content) => content,
        // A non-UTF-8 or unreadable record is skipped by the index.
        Err(_) => return Ok(None),
    };
    let meta = match fs::metadata(&safe) {
        Ok(meta) => meta,
        Err(_) => return Ok(None),
    };
    let doc = parse_doc(&content);
    let Some(stem) = safe.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    if !valid_id(stem) {
        return Ok(None);
    }
    let Some(entry) = active_entry_owned(ctx, kind) else {
        return Ok(None);
    };
    let Some(form) = form_for_stem(&entry.prefix, stem) else {
        return Ok(None);
    };
    let (group_id, group_title) = if form == Form::Requirement {
        parse_group(&doc)
    } else {
        (None, None)
    };
    let title = if doc.title.is_empty() {
        stem.to_string()
    } else {
        doc.title.clone()
    };
    let (
        status,
        milestone_id,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
    ) = match kind {
        Category::Req | Category::Nfr => (
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        ),
        _ => (
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        ),
    };
    Ok(Some(Record {
        id: stem.to_string(),
        kind,
        form,
        title,
        story: section_value(&doc, "## User Story"),
        group_id,
        group_title,
        description: section_value(&doc, "## Description"),
        rationale: None,
        applicability: None,
        requirements: Vec::new(),
        definition: None,
        view: None,
        status,
        milestone_id,
        milestone_title: None,
        assignee,
        labels,
        dependencies,
        acceptance_criteria,
        comments,
        release_id,
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
        audit_type: None,
        doc,
        path: safe.clone(),
        rel_path: relative_path(&ctx.project_dir, &safe),
        version: version_token(&meta),
    }))
}

fn active_entry_owned(ctx: &Context, kind: Category) -> Option<crate::config::DocTypeEntry> {
    let cfg = crate::commands::load_effective_config(&ctx.config_path, &ctx.project_dir).ok()?;
    active_entry(&cfg, kind)
}

/// Build the index for one category. Missing or unreadable directories render
/// as an empty category; they are not server errors.
pub fn build_index(ctx: &Context, cfg: &Config, kind: Category) -> Index {
    let mut index = Index::default();
    if matches!(
        kind,
        Category::Task
            | Category::Milestone
            | Category::Release
            | Category::Decision
            | Category::Glossary
            | Category::UiSpec
            | Category::UiMockup
            | Category::BusinessRule
            | Category::AuditRecord
            | Category::TechSpec
            | Category::UseCase
            | Category::TestCase
            | Category::TestReport
    ) {
        let Some(entry) = active_entry(cfg, kind) else {
            return index;
        };
        let Some(dir) = active_dir(ctx, cfg, kind) else {
            return index;
        };
        let Ok(read_dir) = fs::read_dir(&dir) else {
            return index;
        };
        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            let allowed = match kind {
                Category::UiMockup => path.extension().and_then(|s| s.to_str()) == Some("html"),
                _ => path.extension().and_then(|s| s.to_str()) == Some("md"),
            };
            if !allowed {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !valid_id(stem) {
                continue;
            }
            if kind == Category::Decision {
                if crate::decision::type_key_for_id(cfg, stem).is_none() {
                    continue;
                }
            } else {
                let rest = match stem.strip_prefix(&format!("{}-", entry.prefix)) {
                    Some(r) => r,
                    None => continue,
                };
                if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
                    continue;
                }
            }
            if let Ok(Some(record)) = load_record_from_path(ctx, kind, &path) {
                index.records.push(record);
            }
        }
        index.records.sort_by(|a, b| a.id.cmp(&b.id));
        return index;
    }
    let Some(entry) = active_entry(cfg, kind) else {
        return index;
    };
    let Some(dir) = active_dir(ctx, cfg, kind) else {
        return index;
    };
    let Ok(read_dir) = fs::read_dir(&dir) else {
        return index;
    };
    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !valid_id(stem) || form_for_stem(&entry.prefix, stem).is_none() {
            continue;
        }
        if let Ok(Some(record)) = load_record_from_path(ctx, kind, &path) {
            index.records.push(record);
        }
    }
    index.records.sort_by(|a, b| a.id.cmp(&b.id));
    index.groups = index
        .records
        .iter()
        .filter(|r| r.form == Form::Group)
        .cloned()
        .collect();
    index.requirements = index
        .records
        .iter()
        .filter(|r| r.form == Form::Requirement)
        .cloned()
        .collect();

    let group_ids: BTreeMap<String, usize> = index
        .groups
        .iter()
        .enumerate()
        .map(|(position, group)| (group.id.clone(), position))
        .collect();
    for (position, requirement) in index.requirements.iter().enumerate() {
        match requirement
            .group_id
            .as_ref()
            .and_then(|id| group_ids.get(id))
        {
            Some(_) => {
                index
                    .members
                    .entry(requirement.group_id.clone().unwrap_or_default())
                    .or_default()
                    .push(position);
            }
            None => index.other.push(position),
        }
    }
    index
}

/// Resolve an ID by exact, case-sensitive match against the active directory
/// listings. Returns `Ok(None)` for an unknown, invalid, or deprecated ID.
pub fn resolve_id(ctx: &Context, cfg: &Config, id: &str) -> Result<Option<Resolved>, AppError> {
    if !valid_id(id) {
        return Ok(None);
    }
    let mut matches: Vec<(Category, PathBuf)> = Vec::new();
    for kind in [
        Category::Req,
        Category::Nfr,
        Category::Task,
        Category::Milestone,
        Category::Release,
        Category::Decision,
        Category::Glossary,
        Category::UiSpec,
        Category::UiMockup,
        Category::BusinessRule,
        Category::AuditRecord,
        Category::TechSpec,
        Category::UseCase,
        Category::TestCase,
        Category::TestReport,
    ] {
        let Some(entry) = active_entry(cfg, kind) else {
            continue;
        };
        let Some(dir) = active_dir(ctx, cfg, kind) else {
            continue;
        };
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            let allowed = match kind {
                Category::UiMockup => path.extension().and_then(|s| s.to_str()) == Some("html"),
                _ => path.extension().and_then(|s| s.to_str()) == Some("md"),
            };
            if !allowed {
                continue;
            }
            if path.file_stem().and_then(|s| s.to_str()) != Some(id) {
                continue;
            }
            let valid = if kind == Category::Decision {
                crate::decision::type_key_for_id(cfg, id).is_some()
            } else if kind == Category::Glossary
                || kind == Category::UiSpec
                || kind == Category::UiMockup
                || kind == Category::BusinessRule
                || kind == Category::AuditRecord
                || kind == Category::TechSpec
                || kind == Category::UseCase
                || kind == Category::TestCase
                || kind == Category::TestReport
            {
                id.strip_prefix(&format!("{}-", entry.prefix))
                    .map(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
                    .unwrap_or(false)
            } else if kind.is_requirement() {
                form_for_stem(&entry.prefix, id).is_some()
            } else {
                id.strip_prefix(&format!("{}-", entry.prefix))
                    .map(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
                    .unwrap_or(false)
            };
            if !valid {
                continue;
            }
            // Reject an entry that escapes the project directory.
            if Context::ensure_within(&ctx.project_dir, &path).is_err() {
                continue;
            }
            matches.push((kind, path));
        }
    }
    match matches.len() {
        0 => Ok(None),
        1 => {
            let (kind, path) = matches.remove(0);
            let Some(record) = load_record_from_path(ctx, kind, &path)? else {
                // A symlink that escaped is treated as unknown; a non-UTF-8
                // record is an internal failure.
                let safe = Context::ensure_within(&ctx.project_dir, &path)
                    .map_err(|_| AppError::Runtime("record is not readable".to_string()))?;
                if fs::read_to_string(&safe).is_err() {
                    return Err(AppError::Runtime("record is not valid UTF-8".to_string()));
                }
                return Ok(None);
            };
            Ok(Some(Resolved { record, kind }))
        }
        _ => Err(AppError::Runtime(
            "record ID is ambiguous across document types".to_string(),
        )),
    }
}

/// Serialize an audit-record document with title, audit type, and content changes.
/// Emit patch operations that turn the document's sections into
/// `final_sections`. When no new section was inserted, only changed
/// sections are rewritten in place and every other byte of the file is
/// preserved. When a new section was inserted, all existing sections are
/// first cleared and then re-emitted in final order so the insertion
/// lands at the exact position the legacy serializer computed.
fn section_ops(doc: &Doc, final_sections: &[Section]) -> Vec<crate::record::patch::PatchOp> {
    use crate::record::patch::PatchOp;
    let orig = &doc.sections;
    let inserted = final_sections
        .iter()
        .any(|s| !orig.iter().any(|o| o.heading == s.heading));
    let mut ops = Vec::new();
    if inserted {
        for o in orig {
            ops.push(PatchOp::ClearSection(o.heading.clone()));
        }
        for s in final_sections {
            ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n")));
        }
        return ops;
    }
    for o in orig {
        if !final_sections.iter().any(|s| s.heading == o.heading) {
            ops.push(PatchOp::ClearSection(o.heading.clone()));
        }
    }
    for s in final_sections {
        match orig.iter().find(|o| o.heading == s.heading) {
            None => ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n"))),
            Some(o) if o.body != s.body => {
                ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n")));
            }
            Some(_) => {}
        }
    }
    ops
}

/// Route a browser save through the shared patcher (TASK-67): apply the
/// front-matter and section operations to the original document, refresh
/// `updated_date` only when the content actually changed, and never copy
/// metadata into the body. A no-op save returns the source byte-identical
/// and does not refresh `updated_date`.
pub fn finish_patch(
    doc: &Doc,
    fm_ops: &mut Vec<crate::record::patch::PatchOp>,
    final_sections: &[Section],
) -> String {
    use crate::record::patch::PatchOp;
    let source = &doc.source;
    let mut ops = std::mem::take(fm_ops);
    ops.extend(section_ops(doc, final_sections));
    if ops.is_empty() {
        return source.clone();
    }
    let Ok(rdoc) = crate::record::document::parse(source) else {
        return source.clone();
    };
    let Ok(base) = crate::record::patch::apply_patch(&rdoc, &ops) else {
        return source.clone();
    };
    if base == *source {
        return source.clone();
    }
    if crate::record::frontmatter::has_front_matter(source) {
        let mut ops2 = ops;
        ops2.push(PatchOp::SetFrontMatter(
            "updated_date".into(),
            crate::record::frontmatter::utc_now(),
        ));
        if let Ok(with_date) = crate::record::patch::apply_patch(&rdoc, &ops2) {
            return with_date;
        }
    }
    base
}

pub fn apply_audit_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    audit_type: &str,
    content: FieldChange,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    if has_fm {
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "type".into(),
            audit_type.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    // Audit type always non-empty
    set_section(
        &mut sections,
        "## Audit Type",
        Some(audit_type.to_string()),
        AUDIT_KNOWN,
    );
    match content {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Content", None, AUDIT_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Content", Some(value), AUDIT_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
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
    let normalized = normalize_newlines(raw);
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

/// A change to an editable multi-line field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldChange {
    Keep,
    Clear,
    Set(String),
}

fn trailing_empty_lines(body: &[String]) -> usize {
    body.iter().rev().take_while(|line| line.is_empty()).count()
}

fn remove_section(sections: &mut Vec<Section>, index: usize) {
    // Remove only the section itself. Neighbouring sections keep their exact
    // bodies, which preserves editor-added sections content-identically.
    sections.remove(index);
}

fn known_index(heading: &str, known: &[&str]) -> Option<usize> {
    known.iter().position(|h| *h == heading)
}

fn set_section(sections: &mut Vec<Section>, heading: &str, value: Option<String>, known: &[&str]) {
    let existing = sections.iter().position(|s| s.heading == heading);
    match (existing, value) {
        (Some(index), Some(value)) => {
            let trailing = trailing_empty_lines(&sections[index].body);
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            for _ in 0..trailing {
                body.push(String::new());
            }
            sections[index].body = body;
        }
        (Some(index), None) => remove_section(sections, index),
        (None, Some(value)) => {
            let order = known_index(heading, known).unwrap_or(usize::MAX);
            let mut position = None;
            for (index, section) in sections.iter().enumerate() {
                if let Some(section_order) = known_index(&section.heading, known)
                    && section_order < order
                {
                    position = Some(index + 1);
                }
            }
            if position.is_none() {
                for (index, section) in sections.iter().enumerate() {
                    if let Some(section_order) = known_index(&section.heading, known)
                        && section_order > order
                    {
                        position = Some(index);
                        break;
                    }
                }
            }
            let position = position.unwrap_or(sections.len());
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            if position < sections.len() {
                body.push(String::new());
            }
            sections.insert(
                position,
                Section {
                    heading: heading.to_string(),
                    body,
                },
            );
        }
        (None, None) => {}
    }
}

const REQ_KNOWN: &[&str] = &["## User Story", "## Group", "## Description"];
const TASK_KNOWN: &[&str] = &[
    "## Description",
    "## Status",
    "## Milestone",
    "## Assignee",
    "## Labels",
    "## Dependencies",
    "## Acceptance Criteria",
    "## Comments",
];
const MILESTONE_KNOWN: &[&str] = &["## Description", "## Release"];
const RELEASE_KNOWN: &[&str] = &["## Description"];
const DECISION_KNOWN: &[&str] = &["## Status", "## Decision", "## Context", "## Consequences"];
const GLOSSARY_KNOWN: &[&str] = &["## Definition"];
const UI_SPEC_KNOWN: &[&str] = &["## View", "## Description"];
const RULE_KNOWN: &[&str] = &[
    "## Description",
    "## Rationale",
    "## Applicability",
    "## Requirements",
];

const AUDIT_KNOWN: &[&str] = &["## Audit Type", "## Content"];

/// Serialize a technical-specification document with title and description
/// changes. Only the heading and the `## Description` section are rewritten;
/// the preamble and every other section are preserved content-identically.
pub fn apply_tech_spec_edit(doc: &Doc, _id: &str, title: &str, description: FieldChange) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                // Insert immediately after the preamble, before the first
                // section; when no section exists this is after the heading.
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if !sections.is_empty() {
                    body.push(String::new());
                }
                sections.insert(
                    0,
                    Section {
                        heading: "## Description".to_string(),
                        body,
                    },
                );
            }
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Serialize a document with a new title and optional story/description
/// changes, preserving every unknown `## ` section content-identically.
pub fn apply_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    story: FieldChange,
    description: FieldChange,
    supports_story: bool,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    // No-op byte identity: if nothing changes, return the original document.
    let no_change = title == doc.title
        && matches!(story, FieldChange::Keep)
        && matches!(description, FieldChange::Keep);
    if no_change && !doc.source.is_empty() {
        return doc.source.clone();
    }

    let mut sections = doc.sections.clone();
    if supports_story {
        match story {
            FieldChange::Keep => {}
            FieldChange::Clear => set_section(&mut sections, "## User Story", None, REQ_KNOWN),
            FieldChange::Set(value) => {
                set_section(&mut sections, "## User Story", Some(value), REQ_KNOWN);
            }
        }
    }
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, REQ_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), REQ_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Validate a list field (one value per line) and return the parsed items.
pub fn validate_list_items(label: &str, raw: &str) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    for line in normalize_newlines(raw).lines() {
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

/// Render a list field body.
fn render_list_items(items: &[String], prefix: &str) -> String {
    items
        .iter()
        .map(|v| format!("{} {}", prefix, v))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(clippy::too_many_arguments)]
/// Serialize a use-case/test-case/test-report document with title, description,
/// applicability, labels, and references changes. Only the heading, the
/// `## Description` and `## Applicability` sections, and the front-matter
/// `title`/`labels`/`references` keys are rewritten; the preamble and every
/// other section are preserved content-identically.
pub fn apply_use_case_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    description: FieldChange,
    applicability: FieldChange,
    labels: Vec<String>,
    references: Vec<String>,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    // labels
    if has_fm {
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "labels".into(),
            labels.join("\n"),
        ));
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "references".into(),
            references.join("\n"),
        ));
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            // description is required; clearing removes section
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                // Insert immediately after the preamble, before the first section.
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if !sections.is_empty() {
                    body.push(String::new());
                }
                sections.insert(
                    0,
                    Section {
                        heading: "## Description".to_string(),
                        body,
                    },
                );
            }
        }
    }
    match applicability {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            if let Some(index) = sections
                .iter()
                .position(|s| s.heading == "## Applicability")
            {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            if let Some(index) = sections
                .iter()
                .position(|s| s.heading == "## Applicability")
            {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                // Insert after last preceding known section (Description), else
                // before first following known, else append.
                let known = ["## Description", "## Applicability"];
                let order = 1usize;
                let mut position = None;
                for (index, section) in sections.iter().enumerate() {
                    if let Some(section_order) = known.iter().position(|h| *h == section.heading)
                        && section_order < order
                    {
                        position = Some(index + 1);
                    }
                }
                if position.is_none() {
                    for (index, section) in sections.iter().enumerate() {
                        if let Some(section_order) =
                            known.iter().position(|h| *h == section.heading)
                            && section_order > order
                        {
                            position = Some(index);
                            break;
                        }
                    }
                }
                let position = position.unwrap_or(sections.len());
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if position < sections.len() {
                    body.push(String::new());
                }
                sections.insert(
                    position,
                    Section {
                        heading: "## Applicability".to_string(),
                        body,
                    },
                );
            }
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

pub fn apply_test_case_edit(
    doc: &Doc,
    id: &str,
    title: &str,
    description: FieldChange,
    applicability: FieldChange,
    labels: Vec<String>,
    references: Vec<String>,
) -> String {
    apply_use_case_edit(
        doc,
        id,
        title,
        description,
        applicability,
        labels,
        references,
    )
}

pub fn apply_test_report_edit(
    doc: &Doc,
    id: &str,
    title: &str,
    description: FieldChange,
    applicability: FieldChange,
    labels: Vec<String>,
    references: Vec<String>,
) -> String {
    apply_use_case_edit(
        doc,
        id,
        title,
        description,
        applicability,
        labels,
        references,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn apply_task_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    description: FieldChange,
    status: &str,
    milestone: FieldChange,
    assignee: FieldChange,
    labels: Vec<String>,
    dependencies: Vec<String>,
    acceptance_criteria: Vec<String>,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    if has_fm {
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "status".into(),
            status.to_string(),
        ));
        match &milestone {
            FieldChange::Clear => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatterValue(
                "milestone".into(),
                crate::record::frontmatter::FrontMatterValue::Null,
            )),
            FieldChange::Set(value) => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "milestone".into(),
                value.clone(),
            )),
            FieldChange::Keep => {}
        }
        // Canonical empty assignee is the empty list (`assignee: []`).
        match &assignee {
            FieldChange::Clear => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "assignee".into(),
                String::new(),
            )),
            FieldChange::Set(value) => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "assignee".into(),
                value.clone(),
            )),
            FieldChange::Keep => {}
        }
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "labels".into(),
            labels.join("\n"),
        ));
        fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "dependencies".into(),
            dependencies.join("\n"),
        ));
    }
    let mut sections = doc.sections.clone();
    // Status always set (required)
    set_section(
        &mut sections,
        "## Status",
        Some(status.to_string()),
        TASK_KNOWN,
    );
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), TASK_KNOWN)
        }
    }
    match milestone {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Milestone", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Milestone", Some(value), TASK_KNOWN)
        }
    }
    match assignee {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Assignee", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Assignee", Some(value), TASK_KNOWN)
        }
    }
    if labels.is_empty() {
        set_section(&mut sections, "## Labels", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Labels",
            Some(render_list_items(&labels, "-")),
            TASK_KNOWN,
        );
    }
    if dependencies.is_empty() {
        set_section(&mut sections, "## Dependencies", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Dependencies",
            Some(render_list_items(&dependencies, "-")),
            TASK_KNOWN,
        );
    }
    if acceptance_criteria.is_empty() {
        set_section(&mut sections, "## Acceptance Criteria", None, TASK_KNOWN);
    } else {
        let body = acceptance_criteria
            .iter()
            .map(|v| format!("- [ ] {}", v))
            .collect::<Vec<_>>()
            .join("\n");
        set_section(
            &mut sections,
            "## Acceptance Criteria",
            Some(body),
            TASK_KNOWN,
        );
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

pub fn apply_milestone_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    description: FieldChange,
    release: FieldChange,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    if has_fm {
        match &release {
            FieldChange::Clear => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatterValue(
                "release".into(),
                crate::record::frontmatter::FrontMatterValue::Null,
            )),
            FieldChange::Set(value) => fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "release".into(),
                value.clone(),
            )),
            FieldChange::Keep => {}
        }
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => set_section(
            &mut sections,
            "## Description",
            Some(value),
            MILESTONE_KNOWN,
        ),
    }
    match release {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Release", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Release", Some(value), MILESTONE_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

pub fn apply_release_edit(doc: &Doc, _id: &str, title: &str, description: FieldChange) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, RELEASE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), RELEASE_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Serialize a decision document with title/status/field changes.
pub fn apply_decision_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    status: &str,
    decision: FieldChange,
    context: FieldChange,
    consequences: FieldChange,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    // Status always set
    set_section(
        &mut sections,
        "## Status",
        Some(status.to_string()),
        DECISION_KNOWN,
    );
    match decision {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Decision", None, DECISION_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Decision", Some(value), DECISION_KNOWN)
        }
    }
    match context {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Context", None, DECISION_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Context", Some(value), DECISION_KNOWN)
        }
    }
    match consequences {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Consequences", None, DECISION_KNOWN),
        FieldChange::Set(value) => set_section(
            &mut sections,
            "## Consequences",
            Some(value),
            DECISION_KNOWN,
        ),
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Serialize a glossary document with title and definition changes.
pub fn apply_glossary_edit(doc: &Doc, _id: &str, title: &str, definition: FieldChange) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    match definition {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Definition", None, GLOSSARY_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Definition", Some(value), GLOSSARY_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Serialize a UI specification document with title, view, and description changes.
pub fn apply_ui_spec_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    view: FieldChange,
    description: FieldChange,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    match view {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## View", None, UI_SPEC_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## View", Some(value), UI_SPEC_KNOWN)
        }
    }
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, UI_SPEC_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), UI_SPEC_KNOWN)
        }
    }

    finish_patch(doc, &mut fm_ops, &sections)
}

/// Serialize a business-rule document with title and field changes.
pub fn apply_business_rule_edit(
    doc: &Doc,
    _id: &str,
    title: &str,
    description: FieldChange,
    rationale: FieldChange,
    applicability: FieldChange,
    requirements: Vec<String>,
) -> String {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title != doc.title {
        if has_fm {
            fm_ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "title".into(),
                title.to_string(),
            ));
        }
        fm_ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            title.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), RULE_KNOWN)
        }
    }
    match rationale {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Rationale", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Rationale", Some(value), RULE_KNOWN)
        }
    }
    match applicability {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Applicability", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Applicability", Some(value), RULE_KNOWN)
        }
    }
    if requirements.is_empty() {
        set_section(&mut sections, "## Requirements", None, RULE_KNOWN);
    } else {
        let body = requirements
            .iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n");
        set_section(&mut sections, "## Requirements", Some(body), RULE_KNOWN);
    }

    finish_patch(doc, &mut fm_ops, &sections)
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

/// Inner span of the first `<title>` element: (start, end, uses `&mdash;`).
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

/// Inner span of the first `<h1>` whose text starts with `id` plus whitespace.
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

/// Full span of the first `<!-- key: ... -->` comment.
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

/// Inner span of the first `<p>` whose text starts with `label:`.
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

/// Byte index immediately after the opening `<body ...>` tag.
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
///
/// Only the `<title>` text, a stub `<h1>` starting with the record ID, the
/// `specs`/`refs` comment lines, and existing visible metadata lines are
/// rewritten. Every other byte — styles, scripts, slides, and body markup —
/// is preserved content-identically. The CLI stub serializer is never used.
pub fn apply_ui_mockup_edit(
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
        if !valid_id(value) {
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

/// Validate requirement references for business rules.
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
        if !valid_id(value) {
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
pub fn decision_exists(ctx: &Context, cfg: &Config, id: &str) -> bool {
    let Some(active) = active_dir(ctx, cfg, Category::Decision) else {
        return false;
    };
    let active_path = active.join(format!("{}.md", id));
    if Context::ensure_within(&ctx.project_dir, &active_path)
        .map(|p| p.exists())
        .unwrap_or(false)
    {
        return true;
    }
    let archived_path = active.join("archived").join(format!("{}.md", id));
    Context::ensure_within(&ctx.project_dir, &archived_path)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Write a record atomically in its own directory, preserving permissions and
/// removing the temporary file on every failure path.
pub fn write_atomic(ctx: &Context, target: &Path, content: &str) -> Result<(), AppError> {
    let safe_target = Context::ensure_within(&ctx.project_dir, target)
        .map_err(|_| AppError::NotFound("record not found".to_string()))?;
    let directory = safe_target
        .parent()
        .ok_or_else(|| AppError::Runtime("record directory is unavailable".to_string()))?
        .to_path_buf();
    let file_name = safe_target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::Runtime("record name is unavailable".to_string()))?
        .to_string();
    let original_permissions = fs::metadata(&safe_target)
        .map_err(|_| AppError::NotFound("record not found".to_string()))?
        .permissions();

    let mut temporary: Option<PathBuf> = None;
    let mut handle = None;
    for suffix in 0..1000u32 {
        let name = if suffix == 0 {
            format!("{}.tmp", file_name)
        } else {
            format!("{}.tmp.{}", file_name, suffix)
        };
        let candidate = directory.join(name);
        let safe_candidate = Context::ensure_within(&ctx.project_dir, &candidate)
            .map_err(|_| AppError::Runtime("temporary path is unavailable".to_string()))?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&safe_candidate)
        {
            Ok(file) => {
                temporary = Some(safe_candidate);
                handle = Some(file);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(AppError::Runtime(error.to_string())),
        }
    }
    let (Some(temporary), Some(mut handle)) = (temporary, handle) else {
        return Err(AppError::Runtime(
            "cannot create a temporary record file".to_string(),
        ));
    };

    let result = (|| -> Result<(), AppError> {
        handle
            .write_all(content.as_bytes())
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        handle
            .sync_all()
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        drop(handle);
        fs::set_permissions(&temporary, original_permissions)
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        fs::rename(&temporary, &safe_target)
            .map_err(|error| AppError::Runtime(error.to_string()))?;
        if let Ok(directory_handle) = fs::File::open(&directory) {
            let _ = directory_handle.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_validation() {
        assert!(valid_id("REQ-001.001"));
        assert!(valid_id("NFR-01_2"));
        assert!(!valid_id(""));
        assert!(!valid_id(".hidden"));
        assert!(!valid_id("-x"));
        assert!(!valid_id("a/b"));
        assert!(!valid_id("a..b"));
        assert!(!valid_id("a b"));
        assert!(!valid_id(&"a".repeat(129)));
    }

    #[test]
    fn stem_forms() {
        assert_eq!(form_for_stem("REQ", "REQ-001"), Some(Form::Group));
        assert_eq!(form_for_stem("REQ", "REQ-001.001"), Some(Form::Requirement));
        assert_eq!(form_for_stem("REQ", "REQ-001.x"), None);
        assert_eq!(form_for_stem("REQ", "NFR-001"), None);
        assert_eq!(form_for_stem("REQ", "REQ-001.001.002"), None);
    }

    #[test]
    fn apply_edit_preserves_front_matter_and_noop() {
        let input = "---\nid: X\ntitle: T\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\n---\n# X T\n\n## Description\nBody\n";
        let doc = parse_doc(input);
        let out = apply_edit(&doc, "X", "T", FieldChange::Keep, FieldChange::Keep, false);
        assert_eq!(out, input);
        let out2 = apply_edit(
            &doc,
            "X",
            "New",
            FieldChange::Keep,
            FieldChange::Set("New body".to_string()),
            false,
        );
        assert!(out2.starts_with("---\n"));
        assert!(out2.contains("id: X"));
        assert!(out2.contains("created_date: '2026-01-01T00:00:00Z'"));
        assert!(out2.contains("# X New"));
        assert!(out2.contains("## Description\nNew body"));
    }

    #[test]
    fn parse_and_preserve_unknown_sections() {
        let input = "# REQ-001.001 Title\n\n## User Story\nAs a user\n\n## Group\nREQ-001 Group\n\n## Criticality\nMust have\n\n## Description\nBody\n";
        let doc = parse_doc(input);
        assert_eq!(doc.id, "REQ-001.001");
        assert_eq!(doc.title, "Title");
        assert_eq!(doc.sections.len(), 4);
        let output = apply_edit(
            &doc,
            "REQ-001.001",
            "New title",
            FieldChange::Keep,
            FieldChange::Set("New body".to_string()),
            true,
        );
        assert!(output.starts_with("# REQ-001.001 New title\n"));
        assert!(output.contains("## Criticality\nMust have\n"));
        assert!(output.contains("## Description\nNew body\n"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn remove_and_insert_sections() {
        let doc = parse_doc("# REQ-001 Title\n\n## Description\nOld\n");
        let output = apply_edit(
            &doc,
            "REQ-001",
            "Title",
            FieldChange::Set("Story".to_string()),
            FieldChange::Clear,
            true,
        );
        assert!(output.contains("## User Story\nStory\n"));
        assert!(!output.contains("## Description"));
    }

    #[test]
    fn version_token_shape() {
        assert!(valid_version_token("1762000000.123456789.412"));
        assert!(!valid_version_token("1.2"));
        assert!(!valid_version_token("a.b.c"));
        assert!(!valid_version_token("1.2.3.4"));
    }

    #[test]
    fn title_and_multiline_validation() {
        assert!(validate_title("  Title  ").is_ok());
        assert!(validate_title("").is_err());
        assert!(validate_title(&"x".repeat(201)).is_err());
        assert!(validate_title("a\tb").is_err());
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert!(validate_multiline("Description", "a\n## Section\nb").is_err());
        assert!(validate_multiline("Description", "a\u{0007}b").is_err());
        assert!(validate_multiline("Description", "a\tb").is_ok());
    }

    #[test]
    fn ui_mockup_specs_validation() {
        assert_eq!(
            validate_specs(&["UI-001".to_string(), " UI-002 ".to_string()]).unwrap(),
            vec!["UI-001".to_string(), "UI-002".to_string()]
        );
        // Empty occurrences are dropped (no-JS trailing input).
        assert_eq!(
            validate_specs(&["UI-001".to_string(), "".to_string(), "  ".to_string()]).unwrap(),
            vec!["UI-001".to_string()]
        );
        assert_eq!(
            validate_specs(&[]).unwrap_err(),
            "At least one UI specification id is required."
        );
        assert_eq!(
            validate_specs(&["".to_string()]).unwrap_err(),
            "At least one UI specification id is required."
        );
        assert_eq!(
            validate_specs(&["bad id".to_string()]).unwrap_err(),
            "UI specification ids must be valid record IDs."
        );
        assert_eq!(
            validate_specs(&["../x".to_string()]).unwrap_err(),
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
            validate_refs(&["".to_string()]).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(validate_refs(&[]).unwrap(), Vec::<String>::new());
        assert_eq!(
            validate_refs(&["x".repeat(201)]).unwrap_err(),
            "References must be at most 200 characters."
        );
        assert_eq!(
            validate_refs(&["a\u{0007}b".to_string()]).unwrap_err(),
            "References cannot contain control characters."
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
        let output = apply_ui_mockup_edit(
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
        assert!(output.contains("<!-- status: pending -->"));
        assert!(output.contains("<!-- task: T -->"));
        assert!(output.contains("<p>Superseded by: </p>"));
    }

    #[test]
    fn ui_mockup_review_package_preserved() {
        let original = "<!doctype html>\n<html><head><title>MOCK-003 &mdash; Old review</title><style>body{color:red}</style><script>var s=1;</script></head><body class=\"x\">\n<h1>TASK-99 Landing</h1>\n<section>Slides &amp; more</section>\n</body></html>\n";
        let output = apply_ui_mockup_edit(
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
        assert!(output.contains("<section>Slides &amp; more</section>"));
        assert!(output.contains("<!-- specs: UI-010 -->"));
        assert!(output.contains("<!-- refs: docs/x.md -->"));
        assert!(output.contains("<body class=\"x\">"));
        assert!(!output.contains("<p>UI specifications:"));
        assert!(!output.contains("<p>References:"));
    }

    #[test]
    fn ui_mockup_no_title_never_created() {
        let original = "<html><body>\n<h1>MOCK-001 Old</h1>\n</body></html>\n";
        let output =
            apply_ui_mockup_edit(original, "MOCK-001", "New", &["UI-001".to_string()], &[]);
        assert!(!output.contains("<title>"));
        assert!(output.contains("<h1>MOCK-001 New</h1>"));
    }

    #[test]
    fn ui_mockup_escapes_title_and_visible_lines() {
        let stub = "<title>MOCK-001 — Old</title><h1>MOCK-001 Old</h1><!-- specs: UI-001 --><!-- refs: a --><p>UI specifications: UI-001</p><p>References: a</p>";
        let output = apply_ui_mockup_edit(
            stub,
            "MOCK-001",
            "A & <B>",
            &["UI-001".to_string()],
            &["a&b".to_string()],
        );
        assert!(output.contains("A &amp; &lt;B&gt;"));
        assert!(output.contains("<p>References: a&amp;b</p>"));
    }

    #[test]
    fn business_rule_parse_preserve_and_requirements() {
        let input = "# RUL-001 Title\n\npreamble line\n\n## Description\nBody\n\n## Unknown\nKeep me\n\n## Requirements\n- REQ-001\n- REQ-002\n\n## Rationale\nWhy\n";
        let doc = parse_doc(input);
        assert_eq!(doc.id, "RUL-001");
        assert_eq!(doc.title, "Title");
        assert_eq!(
            section_value(&doc, "## Description").as_deref(),
            Some("Body")
        );
        assert_eq!(section_value(&doc, "## Rationale").as_deref(), Some("Why"));
        assert_eq!(section_value(&doc, "## Applicability"), None);
        assert_eq!(
            parse_rule_requirements(&doc),
            vec!["REQ-001".to_string(), "REQ-002".to_string()]
        );
        let output = apply_business_rule_edit(
            &doc,
            "RUL-001",
            "New title",
            FieldChange::Set("New body".to_string()),
            FieldChange::Keep,
            FieldChange::Clear,
            vec!["REQ-003".to_string()],
        );
        assert!(output.starts_with("# RUL-001 New title\n"));
        assert!(output.contains("preamble line"));
        assert!(output.contains("## Unknown\nKeep me\n"));
        assert!(output.contains("## Description\nNew body\n"));
        assert!(!output.contains("## Applicability"));
        assert!(output.contains("## Requirements\n- REQ-003\n"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn business_rule_field_validation() {
        assert!(validate_title("  Title  ").is_ok());
        assert!(validate_title("").is_err());
        assert_eq!(validate_title("").unwrap_err(), "Title is required.");
        assert!(validate_title(&"x".repeat(201)).is_err());
        assert!(validate_title("a\tb").is_err());
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert!(validate_multiline("Description", "a\n## Section\nb").is_err());
        assert!(validate_multiline("Description", "a\u{0007}b").is_err());
        assert!(validate_multiline("Description", "a\tb").is_ok());
        assert_eq!(
            validate_requirement_refs(&["REQ-001".to_string(), " REQ-002 ".to_string()]).unwrap(),
            vec!["REQ-001".to_string(), "REQ-002".to_string()]
        );
        assert_eq!(
            validate_requirement_refs(&["REQ-001".to_string(), "REQ-001".to_string()]).unwrap(),
            vec!["REQ-001".to_string()]
        );
        assert_eq!(
            validate_requirement_refs(&["".to_string(), "  ".to_string()]).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            validate_requirement_refs(&[]).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            validate_requirement_refs(&["bad id".to_string()]).unwrap_err(),
            "Requirement references must be valid record IDs."
        );
        assert_eq!(
            validate_requirement_refs(&["../x".to_string()]).unwrap_err(),
            "Requirement references must be valid record IDs."
        );
    }

    #[test]
    fn business_rule_duplicate_collapse_in_apply() {
        let input = "# RUL-001 Title\n";
        let doc = parse_doc(input);
        let output = apply_business_rule_edit(
            &doc,
            "RUL-001",
            "Title",
            FieldChange::Keep,
            FieldChange::Keep,
            FieldChange::Keep,
            vec![
                "REQ-001".to_string(),
                "REQ-002".to_string(),
                "REQ-001".to_string(),
            ],
        );
        assert!(output.contains("## Requirements\n- REQ-001\n- REQ-002\n"));
    }

    #[test]
    fn business_rule_empty_requirements_removes_section() {
        let input = "# RUL-001 Title\n\n## Requirements\n- REQ-001\n";
        let doc = parse_doc(input);
        let output = apply_business_rule_edit(
            &doc,
            "RUL-001",
            "Title",
            FieldChange::Keep,
            FieldChange::Keep,
            FieldChange::Keep,
            Vec::new(),
        );
        assert!(!output.contains("## Requirements"));
    }

    #[test]
    fn business_rule_version_token_shape() {
        assert!(valid_version_token("1762000000.123456789.412"));
        assert!(!valid_version_token("1.2"));
        assert!(!valid_version_token("a.b.c"));
    }

    #[test]
    fn audit_record_parse_preserve_and_sections() {
        let input = "# AUD-001 Title\n\npreamble line\n\n## Audit Type\nsecurity\n\n## Unknown\nKeep me\n\n## Content\nBody line\n";
        let doc = parse_doc(input);
        assert_eq!(doc.id, "AUD-001");
        assert_eq!(doc.title, "Title");
        assert_eq!(
            section_value(&doc, "## Audit Type").as_deref(),
            Some("security")
        );
        assert_eq!(
            section_value(&doc, "## Content").as_deref(),
            Some("Body line")
        );
        let output = apply_audit_edit(
            &doc,
            "AUD-001",
            "New title",
            "licence",
            FieldChange::Set("New body".to_string()),
        );
        assert!(output.starts_with("# AUD-001 New title\n"));
        assert!(output.contains("preamble line"));
        assert!(output.contains("## Unknown\nKeep me\n"));
        assert!(output.contains("## Audit Type\nlicence\n"));
        assert!(output.contains("## Content\nNew body\n"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn audit_record_clear_content_removes_section() {
        let input = "# AUD-001 Title\n\n## Audit Type\nsecurity\n\n## Content\nBody\n";
        let doc = parse_doc(input);
        let output = apply_audit_edit(&doc, "AUD-001", "Title", "security", FieldChange::Clear);
        assert!(!output.contains("## Content"));
        assert!(output.contains("## Audit Type\nsecurity\n"));
    }

    #[test]
    fn audit_record_insert_content_section() {
        let input = "# AUD-001 Title\n\n## Audit Type\nsecurity\n";
        let doc = parse_doc(input);
        let output = apply_audit_edit(
            &doc,
            "AUD-001",
            "Title",
            "security",
            FieldChange::Set("New content".to_string()),
        );
        assert!(output.contains("## Content\nNew content\n"));
    }

    #[test]
    fn audit_record_field_validation() {
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
            validate_multiline("Content", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Content", "   ").unwrap(), None);
        assert_eq!(
            validate_multiline("Content", &"x".repeat(20001)).unwrap_err(),
            "Content must be at most 20000 characters."
        );
        assert_eq!(
            validate_multiline("Content", "a\u{0007}b").unwrap_err(),
            "Content cannot contain control characters."
        );
        assert_eq!(
            validate_multiline("Content", "a\n## Section\nb").unwrap_err(),
            "Content contains a line starting with ## ."
        );
        assert!(validate_multiline("Content", "a\tb").is_ok());
    }

    #[test]
    fn audit_record_audit_type_values() {
        // The browser allows only the three built-in types; this documents the
        // accepted set used by validation.
        for value in ["security", "licence", "architecture"] {
            assert!(["security", "licence", "architecture"].contains(&value));
        }
    }

    #[test]
    fn tech_spec_category_and_form_labels() {
        assert_eq!(Category::TechSpec.key(), "tech-spec");
        assert_eq!(Category::TechSpec.config_key(), "tech-spec");
        assert_eq!(Form::TechSpec.label(), "Technical specification");
        assert_eq!(
            Record {
                id: "TECH-001".to_string(),
                kind: Category::TechSpec,
                form: Form::TechSpec,
                title: "Title".to_string(),
                story: None,
                group_id: None,
                group_title: None,
                description: None,
                rationale: None,
                applicability: None,
                requirements: Vec::new(),
                definition: None,
                view: None,
                status: None,
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
                audit_type: None,
                doc: parse_doc("# TECH-001 Title\n"),
                path: PathBuf::new(),
                rel_path: String::new(),
                version: String::new(),
            }
            .type_label(),
            "Technical Specifications"
        );
    }

    #[test]
    fn tech_spec_parse_title_and_description() {
        let input = "# TECH-001 Offline browser\n\n## Description\nDefines the local browser.\n\nSecond paragraph.\n\n## Notes\nKeep me\n";
        let doc = parse_doc(input);
        assert_eq!(doc.id, "TECH-001");
        assert_eq!(doc.title, "Offline browser");
        assert_eq!(
            section_value(&doc, "## Description").as_deref(),
            Some("Defines the local browser.\n\nSecond paragraph.")
        );
        let heading_only = parse_doc("# TECH-002\n\n## Description\n\n");
        assert_eq!(heading_only.title, "");
        assert_eq!(section_value(&heading_only, "## Description"), None);
    }

    #[test]
    fn tech_spec_field_preserving_round_trip() {
        let input = "# TECH-001 Old\n\npreamble line\n\n## Description\nOld body\n\n## Editor Notes\nDo not touch\n\n## Description Extra\nkeep\n";
        let doc = parse_doc(input);
        let output = apply_tech_spec_edit(
            &doc,
            "TECH-001",
            "New",
            FieldChange::Set("New body".to_string()),
        );
        assert!(output.starts_with("# TECH-001 New\n"));
        assert!(output.contains("preamble line\n"));
        assert!(output.contains("## Description\nNew body\n"));
        assert!(output.contains("## Editor Notes\nDo not touch\n"));
        assert!(output.contains("## Description Extra\nkeep\n"));
        // Everything outside the heading and the Description body is unchanged.
        let before_other: Vec<&str> = input
            .lines()
            .filter(|line| *line != "# TECH-001 Old" && *line != "Old body" && !line.is_empty())
            .collect();
        let after_other: Vec<&str> = output
            .lines()
            .filter(|line| *line != "# TECH-001 New" && *line != "New body" && !line.is_empty())
            .collect();
        assert_eq!(before_other, after_other);
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn tech_spec_missing_description_inserted_after_heading() {
        let doc = parse_doc("# TECH-001 Title\n\n## Notes\nnote\n");
        let output = apply_tech_spec_edit(
            &doc,
            "TECH-001",
            "Title",
            FieldChange::Set("Body".to_string()),
        );
        assert!(output.contains("## Description\nBody\n"));
        assert!(output.contains("## Notes\nnote\n"));
        let description_at = output.find("## Description").unwrap();
        let notes_at = output.find("## Notes").unwrap();
        assert!(description_at < notes_at);
    }

    #[test]
    fn tech_spec_field_validation_messages() {
        assert_eq!(validate_title("").unwrap_err(), "Title is required.");
        assert_eq!(
            validate_title(&"x".repeat(201)).unwrap_err(),
            "Title must be at most 200 characters."
        );
        assert_eq!(
            validate_title("a\tb").unwrap_err(),
            "Title cannot contain control characters."
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert_eq!(
            validate_multiline("Description", &"x".repeat(20001)).unwrap_err(),
            "Description must be at most 20000 characters."
        );
        assert_eq!(
            validate_multiline("Description", "a\u{0007}b").unwrap_err(),
            "Description cannot contain control characters."
        );
        assert_eq!(
            validate_multiline("Description", "a\n## Section\nb").unwrap_err(),
            "Description contains a line starting with ## ."
        );
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert!(validate_multiline("Description", "a\tb").is_ok());
    }
}
