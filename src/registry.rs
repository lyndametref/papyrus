//! Runtime artifact registry and per-type behavior descriptors (ADR-0017).
//!
//! The registry is the single runtime model of the artifact set: three
//! categories, one [`TypeDescriptor`] per managed type, and one exact,
//! case-sensitive slug lookup. It is assembled from [`Config`] alone and has no
//! filesystem, path, or environment access (ADR-0017 decision 9).
//!
//! No module consumes the registry yet. Wiring lands in TASK-107 (record
//! layer), TASK-108 (command tree), TASK-110 (generic handler), TASK-113
//! (search), and TASK-114 (browser).

use crate::config::{Config, DocTypeEntry, TypeEntry};
use crate::error::AppError;

use std::collections::BTreeMap;
use std::fmt;

/// The three artifact categories fixed by REQ-07.001.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactCategory {
    Backlog,
    Decision,
    Document,
}

/// The identity shape a type's records use (REQ-01.005, REQ-05.001/002).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdShape {
    /// `<PREFIX>-<sequence>`: the only shape the family claims.
    Plain,
    /// `<PREFIX>-<group>.<sequence>` without group records.
    Dotted,
    /// A grouped family: group records `<PREFIX>-<sequence>` and member
    /// records `<PREFIX>-<group>.<sequence>`. Only `req`/`nfr` use it.
    Grouped,
}

/// How a type's records reach their inactive location (REQ-05.007, REQ-04.002).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchivePolicy {
    /// The record type is never archived (decisions, releases).
    None,
    /// Archive relocates the file and may carry a front-matter state.
    ArchivedFolder { state_field: Option<&'static str> },
}

/// The fixed main statuses of every front-matter-backed document type.
pub const MAIN_DOCUMENT_STATUSES: [&str; 2] = ["active", "archived"];

const MAIN_STATUS_ACTIVE: &str = MAIN_DOCUMENT_STATUSES[0];
const MAIN_STATUS_ARCHIVED: &str = MAIN_DOCUMENT_STATUSES[1];

/// The status model of a type (REQ-07.001, REQ-01.002, REQ-04.001).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusModel {
    /// Tasks only; mirrors top-level `statuses`/`default_status` (REQ-01.002).
    TaskConfigurable {
        statuses: Vec<String>,
        default: String,
    },
    /// Milestones, releases, decisions (REQ-07.001, REQ-04.001).
    Fixed { states: &'static [&'static str] },
    /// Every front-matter-backed document type (REQ-07.001, REQ-07.002).
    ///
    /// `main` is always [`MAIN_DOCUMENT_STATUSES`]; `custom` is owned, ordered
    /// data taken from the validated configuration; `default` is
    /// `default_status` or `active`.
    DocumentMain {
        custom: Vec<String>,
        default: String,
    },
    /// UI mockups only: state lives in HTML metadata (REQ-05.008).
    HtmlMetadata { states: &'static [&'static str] },
}

/// The verbs a descriptor exposes (REQ-07.003).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Create,
    List,
    View,
    Edit,
    Archive,
    Restore,
    Delete,
    Status,
    CreateGroup,
    Supersede,
}

impl Verb {
    pub fn as_str(self) -> &'static str {
        match self {
            Verb::Create => "create",
            Verb::List => "list",
            Verb::View => "view",
            Verb::Edit => "edit",
            Verb::Archive => "archive",
            Verb::Restore => "restore",
            Verb::Delete => "delete",
            Verb::Status => "status",
            Verb::CreateGroup => "create-group",
            Verb::Supersede => "supersede",
        }
    }
}

/// Extra arguments a `create` command accepts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreateOptions {
    pub group: GroupRequirement,
}

/// Whether `create` requires a `--group` declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupRequirement {
    #[default]
    Absent,
    Required,
}

/// The requirement family a typed handler belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequirementKind {
    Business,
    NonFunctional,
}

/// Names the module family that owns validation and patch building for a typed
/// descriptor. Dispatch matches on this value, never on a slug string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedHandler {
    Task,
    Milestone,
    Release,
    Decision,
    Requirement(RequirementKind),
    Glossary,
    UiSpec,
    Rule,
    UiMockup,
    Audit,
    TechSpec,
    UseCase,
    TestCase,
    TestReport,
}

/// The typed content model of a built-in type (ADR-0018).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedSchema {
    pub handler: TypedHandler,
    /// Body sections in written order; empty for HTML mockups.
    pub sections: &'static [&'static str],
    /// Extension verbs this family owns; must equal the descriptor's extensions.
    pub extensions: &'static [Verb],
}

/// The body model of a document type (REQ-07.004, REQ-05.008).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentModel {
    /// Config-defined document type (REQ-07.004, ADR-0018).
    Generic,
    Typed(&'static TypedSchema),
    HtmlMockup(&'static TypedSchema),
}

/// A descriptor's directory: which configuration key to read and its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectorySpec {
    /// `"backlog_directory" | "decision_directory" | "doc_directory"`.
    pub config_key: &'static str,
    /// `"tasks" | "milestones" | "releases" | "" | <entry.directory>`.
    pub relative: String,
}

/// One managed artifact type and its behavior (ADR-0017 decision 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub slug: String,
    pub category: ArtifactCategory,
    pub label: String,
    pub prefix: String,
    pub directory: DirectorySpec,
    pub enabled: bool,
    pub status_model: StatusModel,
    pub verbs: Vec<Verb>,
    pub create_options: CreateOptions,
    pub content_model: ContentModel,
    pub frontmatter_keys: Vec<&'static str>,
    pub id_shape: IdShape,
    pub archive_policy: ArchivePolicy,
}

impl TypeDescriptor {
    pub fn has_verb(&self, verb: Verb) -> bool {
        self.verbs.contains(&verb)
    }

    /// `Some("status")` when the type stores a front-matter status.
    pub fn frontmatter_status_key(&self) -> Option<&'static str> {
        self.frontmatter_keys
            .iter()
            .copied()
            .find(|key| *key == "status")
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Why a registry could not be assembled (ADR-0019 decision 11).
///
/// Every variant names the configuration key in the form
/// `document_types.<slug>.<field>` (or `document_types.<slug>` for a
/// slug-level violation) and converts to [`AppError::Validation`] (exit 4).
#[derive(Debug)]
pub enum RegistryError {
    /// I1: no two descriptors share a slug.
    DuplicateSlug { slug: String },
    /// I2: a configured document slug collides with a built-in type.
    ReservedSlug { slug: String, builtin: String },
    /// I3: a prefix is empty.
    EmptyPrefix { slug: String },
    /// I4: a prefix is already used, including by a disabled entry.
    DuplicatePrefix {
        slug: String,
        prefix: String,
        other: String,
    },
    /// I5: a document directory is empty.
    EmptyDirectory { slug: String },
    /// I6: a document directory is absolute or traverses.
    InvalidDirectory { slug: String, value: String },
    /// I7: an explicitly declared status list is empty.
    EmptyStatusList { slug: String },
    /// I8: a status value is repeated.
    DuplicateStatus { slug: String, value: String },
    /// I9/I10: `active` or `archived` is missing or repeated.
    MainStatusCount {
        slug: String,
        value: &'static str,
        count: usize,
    },
    /// I11: a type without front-matter status declares status data.
    StatusesNotAllowed { slug: String, field: &'static str },
    /// I12: `default_status` is not a declared status.
    DefaultStatusNotDeclared { slug: String, value: String },
    /// I13: `default_status` is `archived`.
    ArchivedDefaultStatus { slug: String },
    /// I14: the `status` verb and the front-matter status key disagree.
    StatusVerbMismatch { slug: String },
    /// I15: an extension verb is not declared by the content model.
    ExtensionVerbMismatch { slug: String, verb: &'static str },
    /// I16: a document declares the `restore` verb.
    RestoreVerbOnDocument { slug: String },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::DuplicateSlug { slug } => {
                write!(f, "document_types.{}: duplicate type slug", slug)
            }
            RegistryError::ReservedSlug { slug, builtin } => write!(
                f,
                "document_types.{}: slug conflicts with built-in type {}",
                slug, builtin
            ),
            RegistryError::EmptyPrefix { slug } => {
                write!(f, "document_types.{}.prefix: must not be empty", slug)
            }
            RegistryError::DuplicatePrefix {
                slug,
                prefix,
                other,
            } => write!(
                f,
                "document_types.{}.prefix: {} is already used by {}",
                slug, prefix, other
            ),
            RegistryError::EmptyDirectory { slug } => {
                write!(f, "document_types.{}.directory: must not be empty", slug)
            }
            RegistryError::InvalidDirectory { slug, value } => write!(
                f,
                "document_types.{}.directory: must be relative and inside doc_directory: {}",
                slug, value
            ),
            RegistryError::EmptyStatusList { slug } => {
                write!(f, "document_types.{}.statuses: must not be empty", slug)
            }
            RegistryError::DuplicateStatus { slug, value } => write!(
                f,
                "document_types.{}.statuses: duplicate value {}",
                slug, value
            ),
            RegistryError::MainStatusCount { slug, value, .. } => write!(
                f,
                "document_types.{}.statuses: \"{}\" must be declared exactly once",
                slug, value
            ),
            RegistryError::StatusesNotAllowed { slug, field } => write!(
                f,
                "document_types.{}.{}: not allowed for a type without a front-matter status",
                slug, field
            ),
            RegistryError::DefaultStatusNotDeclared { slug, value } => write!(
                f,
                "document_types.{}.default_status: {} is not a declared status",
                slug, value
            ),
            RegistryError::ArchivedDefaultStatus { slug } => write!(
                f,
                "document_types.{}.default_status: must not be \"archived\"",
                slug
            ),
            RegistryError::StatusVerbMismatch { slug } => write!(
                f,
                "document_types.{}: verbs and status model disagree about \"status\"",
                slug
            ),
            RegistryError::ExtensionVerbMismatch { slug, verb } => write!(
                f,
                "document_types.{}: verb {} is not declared by the content model",
                slug, verb
            ),
            RegistryError::RestoreVerbOnDocument { slug } => {
                write!(f, "document_types.{}: documents have no restore verb", slug)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<RegistryError> for AppError {
    fn from(error: RegistryError) -> Self {
        AppError::Validation(error.to_string())
    }
}

const BACKLOG_DIRECTORY_KEY: &str = "backlog_directory";
const DECISION_DIRECTORY_KEY: &str = "decision_directory";
const DOC_DIRECTORY_KEY: &str = "doc_directory";

/// General front-matter keys in canonical order (ADR-0023 decision 5).
pub const GENERAL_FRONTMATTER_KEYS: &[&str] = &[
    "id",
    "title",
    "created_date",
    "updated_date",
    "labels",
    "references",
];

/// The document baseline verb family (REQ-07.003).
pub const DOCUMENT_BASELINE_VERBS: &[Verb] = &[
    Verb::Create,
    Verb::List,
    Verb::View,
    Verb::Edit,
    Verb::Archive,
    Verb::Delete,
];

const TASK_SECTIONS: &[&str] = &["## Description", "## Acceptance Criteria", "## Comments"];
const MILESTONE_SECTIONS: &[&str] = &["## Description"];
const RELEASE_SECTIONS: &[&str] = &["## Description"];
const DECISION_SECTIONS: &[&str] = &["## Status", "## Decision", "## Context", "## Consequences"];
const REQUIREMENT_SECTIONS: &[&str] = &["## User Story", "## Group", "## Description"];
const GLOSSARY_SECTIONS: &[&str] = &["## Definition"];
const RULE_SECTIONS: &[&str] = &[
    "## Description",
    "## Rationale",
    "## Applicability",
    "## Requirements",
];
const UI_SPEC_SECTIONS: &[&str] = &["## View", "## Description"];
const AUDIT_SECTIONS: &[&str] = &["## Audit Type", "## Content"];
const TECH_SPEC_SECTIONS: &[&str] = &["## Description"];
const USE_CASE_SECTIONS: &[&str] = &["## Description", "## Applicability"];
const TEST_CASE_SECTIONS: &[&str] = &["## Description", "## Applicability"];
const TEST_REPORT_SECTIONS: &[&str] = &["## Description", "## Applicability"];

const TASK_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Task,
    sections: TASK_SECTIONS,
    extensions: &[],
};
const MILESTONE_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Milestone,
    sections: MILESTONE_SECTIONS,
    extensions: &[],
};
const RELEASE_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Release,
    sections: RELEASE_SECTIONS,
    extensions: &[],
};
const DECISION_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Decision,
    sections: DECISION_SECTIONS,
    extensions: &[],
};
const REQUIREMENT_BUSINESS_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Requirement(RequirementKind::Business),
    sections: REQUIREMENT_SECTIONS,
    extensions: &[Verb::CreateGroup],
};
const REQUIREMENT_NON_FUNCTIONAL_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Requirement(RequirementKind::NonFunctional),
    sections: REQUIREMENT_SECTIONS,
    extensions: &[Verb::CreateGroup],
};
const GLOSSARY_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Glossary,
    sections: GLOSSARY_SECTIONS,
    extensions: &[],
};
const RULE_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Rule,
    sections: RULE_SECTIONS,
    extensions: &[],
};
const UI_SPEC_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::UiSpec,
    sections: UI_SPEC_SECTIONS,
    extensions: &[],
};
const UI_MOCKUP_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::UiMockup,
    sections: &[],
    extensions: &[Verb::Supersede],
};
const AUDIT_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::Audit,
    sections: AUDIT_SECTIONS,
    extensions: &[],
};
const TECH_SPEC_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::TechSpec,
    sections: TECH_SPEC_SECTIONS,
    extensions: &[],
};
const USE_CASE_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::UseCase,
    sections: USE_CASE_SECTIONS,
    extensions: &[],
};
const TEST_CASE_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::TestCase,
    sections: TEST_CASE_SECTIONS,
    extensions: &[],
};
const TEST_REPORT_SCHEMA: TypedSchema = TypedSchema {
    handler: TypedHandler::TestReport,
    sections: TEST_REPORT_SECTIONS,
    extensions: &[],
};

const DECISION_STATES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
];
const MILESTONE_STATES: &[&str] = &["open", "closed"];
const RELEASE_STATES: &[&str] = &["planned", "released"];
const MOCKUP_STATES: &[&str] = &["pending", "approved", "archived"];

const NO_CREATE_OPTIONS: CreateOptions = CreateOptions {
    group: GroupRequirement::Absent,
};
const REQUIRED_GROUP: CreateOptions = CreateOptions {
    group: GroupRequirement::Required,
};

const FRONTMATTER_ARCHIVE: ArchivePolicy = ArchivePolicy::ArchivedFolder {
    state_field: Some("status"),
};
const HTML_METADATA_ARCHIVE: ArchivePolicy = ArchivePolicy::ArchivedFolder { state_field: None };

/// Where a template's prefix comes from.
#[derive(Clone, Copy, Debug)]
enum PrefixSource {
    Task,
    Milestone,
    Release,
    Decision,
    Entry,
}

/// The status model a template declares, before configuration is applied.
#[derive(Clone, Copy, Debug)]
enum TemplateStatus {
    TaskConfigurable,
    Fixed(&'static [&'static str]),
    DocumentMain,
    HtmlMetadata(&'static [&'static str]),
}

/// The verb family a template declares.
#[derive(Clone, Copy, Debug)]
enum TemplateVerbs {
    Fixed(&'static [Verb]),
    DocumentBaseline { extensions: &'static [Verb] },
}

/// A static, configuration-free description of a built-in type.
///
/// Templates carry no prefix, directory value, or status list; everything
/// configuration-derived is filled by `assemble`, so a template cannot embed a
/// project path or a stale prefix.
#[derive(Clone, Copy, Debug)]
struct DescriptorTemplate {
    slug: &'static str,
    category: ArtifactCategory,
    label: &'static str,
    prefix: PrefixSource,
    directory_key: &'static str,
    /// `Some("tasks")` for backlog families; `None` for document types, whose
    /// directory comes from the `document_types` entry.
    subpath: Option<&'static str>,
    id_shape: IdShape,
    status: TemplateStatus,
    verbs: TemplateVerbs,
    content: ContentModel,
    /// `Some("status")` when the type stores its state in front matter.
    frontmatter_status: Option<&'static str>,
    frontmatter_extra: &'static [&'static str],
    archive: ArchivePolicy,
    create_options: CreateOptions,
}

const TASK_VERBS: &[Verb] = &[
    Verb::Create,
    Verb::List,
    Verb::View,
    Verb::Edit,
    Verb::Archive,
    Verb::Restore,
    Verb::Delete,
];
const MILESTONE_VERBS: &[Verb] = &[
    Verb::Create,
    Verb::List,
    Verb::View,
    Verb::Edit,
    Verb::Archive,
    Verb::Delete,
];
const RELEASE_VERBS: &[Verb] = &[Verb::Create, Verb::List, Verb::View, Verb::Edit];
const DECISION_VERBS: &[Verb] = &[Verb::Create, Verb::List, Verb::View, Verb::Edit];

const BACKLOG_TEMPLATES: &[DescriptorTemplate] = &[
    DescriptorTemplate {
        slug: "task",
        category: ArtifactCategory::Backlog,
        label: "Task",
        prefix: PrefixSource::Task,
        directory_key: BACKLOG_DIRECTORY_KEY,
        subpath: Some("tasks"),
        id_shape: IdShape::Plain,
        status: TemplateStatus::TaskConfigurable,
        verbs: TemplateVerbs::Fixed(TASK_VERBS),
        content: ContentModel::Typed(&TASK_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[
            "assignee",
            "milestone",
            "dependencies",
            "ordinal",
            "documentation",
            "type",
        ],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "milestone",
        category: ArtifactCategory::Backlog,
        label: "Milestone",
        prefix: PrefixSource::Milestone,
        directory_key: BACKLOG_DIRECTORY_KEY,
        subpath: Some("milestones"),
        id_shape: IdShape::Plain,
        status: TemplateStatus::Fixed(MILESTONE_STATES),
        verbs: TemplateVerbs::Fixed(MILESTONE_VERBS),
        content: ContentModel::Typed(&MILESTONE_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &["release"],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "release",
        category: ArtifactCategory::Backlog,
        label: "Release",
        prefix: PrefixSource::Release,
        directory_key: BACKLOG_DIRECTORY_KEY,
        subpath: Some("releases"),
        id_shape: IdShape::Plain,
        status: TemplateStatus::Fixed(RELEASE_STATES),
        verbs: TemplateVerbs::Fixed(RELEASE_VERBS),
        content: ContentModel::Typed(&RELEASE_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: ArchivePolicy::None,
        create_options: NO_CREATE_OPTIONS,
    },
];

const DECISION_TEMPLATE: DescriptorTemplate = DescriptorTemplate {
    slug: "decision",
    category: ArtifactCategory::Decision,
    label: "Decision",
    prefix: PrefixSource::Decision,
    directory_key: DECISION_DIRECTORY_KEY,
    subpath: None,
    id_shape: IdShape::Plain,
    status: TemplateStatus::Fixed(DECISION_STATES),
    verbs: TemplateVerbs::Fixed(DECISION_VERBS),
    content: ContentModel::Typed(&DECISION_SCHEMA),
    // Decisions keep `status` in the document core, not in front matter.
    frontmatter_status: None,
    frontmatter_extra: &["superseded_by"],
    archive: ArchivePolicy::None,
    create_options: NO_CREATE_OPTIONS,
};

const DOCUMENT_TEMPLATES: &[DescriptorTemplate] = &[
    DescriptorTemplate {
        slug: "req",
        category: ArtifactCategory::Document,
        label: "Business requirement",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Grouped,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline {
            extensions: &[Verb::CreateGroup],
        },
        content: ContentModel::Typed(&REQUIREMENT_BUSINESS_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: REQUIRED_GROUP,
    },
    DescriptorTemplate {
        slug: "nfr",
        category: ArtifactCategory::Document,
        label: "Non-functional requirement",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Grouped,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline {
            extensions: &[Verb::CreateGroup],
        },
        content: ContentModel::Typed(&REQUIREMENT_NON_FUNCTIONAL_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: REQUIRED_GROUP,
    },
    DescriptorTemplate {
        slug: "glossary",
        category: ArtifactCategory::Document,
        label: "Glossary term",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&GLOSSARY_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "rule",
        category: ArtifactCategory::Document,
        label: "Business rule",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&RULE_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "ui-spec",
        category: ArtifactCategory::Document,
        label: "UI specification",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&UI_SPEC_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "ui-mockup",
        category: ArtifactCategory::Document,
        label: "UI mockup",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::HtmlMetadata(MOCKUP_STATES),
        verbs: TemplateVerbs::DocumentBaseline {
            extensions: &[Verb::Supersede],
        },
        content: ContentModel::HtmlMockup(&UI_MOCKUP_SCHEMA),
        frontmatter_status: None,
        frontmatter_extra: &[],
        archive: HTML_METADATA_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "audit",
        category: ArtifactCategory::Document,
        label: "Audit record",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&AUDIT_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &["type"],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "tech-spec",
        category: ArtifactCategory::Document,
        label: "Technical specification",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&TECH_SPEC_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "use-case",
        category: ArtifactCategory::Document,
        label: "Use case",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&USE_CASE_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "test-case",
        category: ArtifactCategory::Document,
        label: "Test case",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&TEST_CASE_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
    DescriptorTemplate {
        slug: "test-report",
        category: ArtifactCategory::Document,
        label: "Test report",
        prefix: PrefixSource::Entry,
        directory_key: DOC_DIRECTORY_KEY,
        subpath: None,
        id_shape: IdShape::Plain,
        status: TemplateStatus::DocumentMain,
        verbs: TemplateVerbs::DocumentBaseline { extensions: &[] },
        content: ContentModel::Typed(&TEST_REPORT_SCHEMA),
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
    },
];

/// The built-in registry: every configured type with its behavior.
#[derive(Debug)]
pub struct ArtifactRegistry {
    /// Canonical order: task, milestone, release, decision types in key order,
    /// document types in slug order.
    descriptors: Vec<TypeDescriptor>,
    by_slug: BTreeMap<String, usize>,
}

impl ArtifactRegistry {
    /// Assemble and validate the registry from configuration alone.
    ///
    /// # Errors
    ///
    /// Returns the first violated [`RegistryError`] in a deterministic order.
    /// Every error names `document_types.<slug>.<field>` and converts to
    /// [`AppError::Validation`] (exit code 4).
    pub fn build(cfg: &Config) -> Result<ArtifactRegistry, RegistryError> {
        let descriptors = assemble(cfg)?;
        check_invariants(&descriptors)?;
        let mut by_slug = BTreeMap::new();
        for (index, descriptor) in descriptors.iter().enumerate() {
            by_slug.insert(descriptor.slug.clone(), index);
        }
        Ok(ArtifactRegistry {
            descriptors,
            by_slug,
        })
    }

    /// Exact, case-sensitive slug lookup. No alias, prefix, or label matching.
    pub fn descriptor(&self, slug: &str) -> Option<&TypeDescriptor> {
        self.by_slug
            .get(slug)
            .and_then(|index| self.descriptors.get(*index))
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &TypeDescriptor> + '_ {
        self.descriptors.iter()
    }

    pub fn in_category(
        &self,
        category: ArtifactCategory,
    ) -> impl Iterator<Item = &TypeDescriptor> + '_ {
        self.descriptors
            .iter()
            .filter(move |descriptor| descriptor.category == category)
    }

    pub fn enabled(
        &self,
        category: ArtifactCategory,
    ) -> impl Iterator<Item = &TypeDescriptor> + '_ {
        self.in_category(category)
            .filter(|descriptor| descriptor.is_enabled())
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }
}

/// Build descriptors in the canonical order, without validating invariants.
fn assemble(cfg: &Config) -> Result<Vec<TypeDescriptor>, RegistryError> {
    let mut descriptors = Vec::new();
    for template in BACKLOG_TEMPLATES {
        descriptors.push(backlog_descriptor(cfg, template));
    }
    for (slug, entry) in &cfg.decision_types {
        descriptors.push(decision_descriptor(slug, entry));
    }
    for (slug, entry) in &cfg.document_types {
        descriptors.push(document_descriptor(slug, entry)?);
    }
    Ok(descriptors)
}

fn frontmatter_keys(template: &DescriptorTemplate) -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = GENERAL_FRONTMATTER_KEYS.to_vec();
    if let Some(key) = template.frontmatter_status {
        keys.push(key);
    }
    keys.extend(template.frontmatter_extra.iter().copied());
    keys
}

fn backlog_descriptor(cfg: &Config, template: &DescriptorTemplate) -> TypeDescriptor {
    let prefix = match template.prefix {
        PrefixSource::Task => cfg.task_prefix.clone(),
        PrefixSource::Milestone => cfg.milestone_prefix.clone(),
        PrefixSource::Release => cfg.release_prefix.clone(),
        // Backlog templates never read a decision or entry prefix.
        PrefixSource::Decision | PrefixSource::Entry => String::new(),
    };
    let status_model = match template.status {
        TemplateStatus::TaskConfigurable => StatusModel::TaskConfigurable {
            statuses: cfg.statuses.clone(),
            default: cfg.default_status.clone(),
        },
        TemplateStatus::Fixed(states) => StatusModel::Fixed { states },
        // Document status models are built by `document_descriptor`.
        TemplateStatus::DocumentMain => StatusModel::Fixed { states: &[] },
        TemplateStatus::HtmlMetadata(states) => StatusModel::Fixed { states },
    };
    let verbs = match template.verbs {
        TemplateVerbs::Fixed(verbs) => verbs.to_vec(),
        TemplateVerbs::DocumentBaseline { .. } => DOCUMENT_BASELINE_VERBS.to_vec(),
    };
    TypeDescriptor {
        slug: template.slug.to_string(),
        category: template.category,
        label: template.label.to_string(),
        prefix,
        directory: DirectorySpec {
            config_key: template.directory_key,
            relative: template.subpath.unwrap_or_default().to_string(),
        },
        enabled: true,
        status_model,
        verbs,
        create_options: template.create_options,
        content_model: template.content,
        frontmatter_keys: frontmatter_keys(template),
        id_shape: template.id_shape,
        archive_policy: template.archive,
    }
}

fn decision_descriptor(slug: &str, entry: &TypeEntry) -> TypeDescriptor {
    TypeDescriptor {
        slug: slug.to_string(),
        category: ArtifactCategory::Decision,
        label: slug.to_string(),
        prefix: entry.prefix.clone(),
        directory: DirectorySpec {
            config_key: DECISION_DIRECTORY_KEY,
            relative: String::new(),
        },
        enabled: entry.enabled,
        status_model: StatusModel::Fixed {
            states: DECISION_STATES,
        },
        verbs: DECISION_VERBS.to_vec(),
        create_options: NO_CREATE_OPTIONS,
        content_model: ContentModel::Typed(&DECISION_SCHEMA),
        frontmatter_keys: frontmatter_keys(&DECISION_TEMPLATE),
        id_shape: IdShape::Plain,
        archive_policy: ArchivePolicy::None,
    }
}

/// Build one document descriptor, typed when a template exists and generic
/// otherwise (REQ-07.004, ADR-0018).
fn document_descriptor(slug: &str, entry: &DocTypeEntry) -> Result<TypeDescriptor, RegistryError> {
    let template = DOCUMENT_TEMPLATES
        .iter()
        .find(|template| template.slug == slug);

    let (
        status_model,
        verbs,
        content_model,
        id_shape,
        create_options,
        archive_policy,
        frontmatter_keys,
    ) = match template {
        Some(template) => {
            let status_model = document_status_model(slug, entry, template.status)?;
            let mut verbs = DOCUMENT_BASELINE_VERBS.to_vec();
            if matches!(status_model, StatusModel::DocumentMain { .. }) {
                verbs.push(Verb::Status);
            }
            if let TemplateVerbs::DocumentBaseline { extensions } = template.verbs {
                verbs.extend(extensions.iter().copied());
            }
            let keys = match template.content {
                // UI mockups are HTML records and carry no front matter.
                ContentModel::HtmlMockup(_) => Vec::new(),
                _ => frontmatter_keys(template),
            };
            (
                status_model,
                verbs,
                template.content,
                template.id_shape,
                template.create_options,
                template.archive,
                keys,
            )
        }
        None => {
            let status_model = document_status_model(slug, entry, TemplateStatus::DocumentMain)?;
            let mut verbs = DOCUMENT_BASELINE_VERBS.to_vec();
            verbs.push(Verb::Status);
            let mut keys: Vec<&'static str> = GENERAL_FRONTMATTER_KEYS.to_vec();
            keys.push("status");
            (
                status_model,
                verbs,
                ContentModel::Generic,
                IdShape::Plain,
                NO_CREATE_OPTIONS,
                FRONTMATTER_ARCHIVE,
                keys,
            )
        }
    };

    Ok(TypeDescriptor {
        slug: slug.to_string(),
        category: ArtifactCategory::Document,
        label: match template {
            Some(template) => template.label.to_string(),
            None => slug.to_string(),
        },
        prefix: entry.prefix.clone(),
        directory: DirectorySpec {
            config_key: DOC_DIRECTORY_KEY,
            relative: entry.directory.clone(),
        },
        enabled: entry.enabled,
        status_model,
        verbs,
        create_options,
        content_model,
        frontmatter_keys,
        id_shape,
        archive_policy,
    })
}

/// Resolve a document descriptor's status model from its entry (REQ-07.001).
///
/// Status-list invariants that need the raw declaration are checked here: an
/// explicit empty list, duplicates, the two fixed main statuses, and status
/// data on a type without front-matter status cannot be recovered from
/// [`StatusModel`] afterwards.
fn document_status_model(
    slug: &str,
    entry: &DocTypeEntry,
    template_status: TemplateStatus,
) -> Result<StatusModel, RegistryError> {
    if let TemplateStatus::HtmlMetadata(states) = template_status {
        if entry.statuses.is_some() {
            return Err(RegistryError::StatusesNotAllowed {
                slug: slug.to_string(),
                field: "statuses",
            });
        }
        if entry.default_status.is_some() {
            return Err(RegistryError::StatusesNotAllowed {
                slug: slug.to_string(),
                field: "default_status",
            });
        }
        return Ok(StatusModel::HtmlMetadata { states });
    }

    let custom = match &entry.statuses {
        None => Vec::new(),
        Some(declared) => {
            if declared.is_empty() {
                return Err(RegistryError::EmptyStatusList {
                    slug: slug.to_string(),
                });
            }
            let active = declared
                .iter()
                .filter(|value| value.as_str() == MAIN_STATUS_ACTIVE)
                .count();
            if active != 1 {
                return Err(RegistryError::MainStatusCount {
                    slug: slug.to_string(),
                    value: MAIN_STATUS_ACTIVE,
                    count: active,
                });
            }
            let archived = declared
                .iter()
                .filter(|value| value.as_str() == MAIN_STATUS_ARCHIVED)
                .count();
            if archived != 1 {
                return Err(RegistryError::MainStatusCount {
                    slug: slug.to_string(),
                    value: MAIN_STATUS_ARCHIVED,
                    count: archived,
                });
            }
            let mut seen: Vec<&str> = Vec::new();
            for value in declared {
                if seen.contains(&value.as_str()) {
                    return Err(RegistryError::DuplicateStatus {
                        slug: slug.to_string(),
                        value: value.clone(),
                    });
                }
                seen.push(value.as_str());
            }
            declared
                .iter()
                .filter(|value| {
                    value.as_str() != MAIN_STATUS_ACTIVE && value.as_str() != MAIN_STATUS_ARCHIVED
                })
                .cloned()
                .collect()
        }
    };

    Ok(StatusModel::DocumentMain {
        custom,
        default: entry
            .default_status
            .clone()
            .unwrap_or_else(|| MAIN_STATUS_ACTIVE.to_string()),
    })
}

/// Check every registry invariant, returning the first failure in a fixed
/// order (ADR-0019 decision 11).
fn check_invariants(descriptors: &[TypeDescriptor]) -> Result<(), RegistryError> {
    let documents = || {
        descriptors
            .iter()
            .filter(|descriptor| descriptor.category == ArtifactCategory::Document)
    };

    // I2 (checked before I1 so a reserved slug reports its own cause).
    for descriptor in documents() {
        if let Some(builtin) = descriptors.iter().find(|other| {
            other.category != ArtifactCategory::Document
                && other.slug.eq_ignore_ascii_case(&descriptor.slug)
        }) {
            return Err(RegistryError::ReservedSlug {
                slug: descriptor.slug.clone(),
                builtin: builtin.slug.clone(),
            });
        }
    }

    // I1.
    let mut seen_slugs: Vec<&str> = Vec::new();
    for descriptor in descriptors {
        if seen_slugs.contains(&descriptor.slug.as_str()) {
            return Err(RegistryError::DuplicateSlug {
                slug: descriptor.slug.clone(),
            });
        }
        seen_slugs.push(descriptor.slug.as_str());
    }

    // I3 and I4.
    let mut seen_prefixes: Vec<(String, String)> = Vec::new();
    for descriptor in descriptors {
        if descriptor.prefix.is_empty() {
            return Err(RegistryError::EmptyPrefix {
                slug: descriptor.slug.clone(),
            });
        }
        let lower = descriptor.prefix.to_lowercase();
        if let Some((_, other)) = seen_prefixes.iter().find(|(prefix, _)| *prefix == lower) {
            return Err(RegistryError::DuplicatePrefix {
                slug: descriptor.slug.clone(),
                prefix: descriptor.prefix.clone(),
                other: other.clone(),
            });
        }
        seen_prefixes.push((lower, descriptor.slug.clone()));
    }

    // I5 and I6.
    for descriptor in documents() {
        let value = &descriptor.directory.relative;
        if value.is_empty() {
            return Err(RegistryError::EmptyDirectory {
                slug: descriptor.slug.clone(),
            });
        }
        if !is_relative_directory(value) {
            return Err(RegistryError::InvalidDirectory {
                slug: descriptor.slug.clone(),
                value: value.clone(),
            });
        }
    }

    // I12 and I13.
    for descriptor in descriptors {
        if let StatusModel::DocumentMain { custom, default } = &descriptor.status_model {
            let declared = default == MAIN_STATUS_ACTIVE
                || default == MAIN_STATUS_ARCHIVED
                || custom.iter().any(|value| value == default);
            if !declared {
                return Err(RegistryError::DefaultStatusNotDeclared {
                    slug: descriptor.slug.clone(),
                    value: default.clone(),
                });
            }
            if default == MAIN_STATUS_ARCHIVED {
                return Err(RegistryError::ArchivedDefaultStatus {
                    slug: descriptor.slug.clone(),
                });
            }
        }
    }

    // I14.
    for descriptor in documents() {
        let has_status_verb = descriptor.verbs.contains(&Verb::Status);
        let has_status_key = descriptor.frontmatter_keys.contains(&"status");
        if has_status_verb != has_status_key {
            return Err(RegistryError::StatusVerbMismatch {
                slug: descriptor.slug.clone(),
            });
        }
    }

    // I16 (checked before I15: `restore` is reported as the specific mismatch).
    for descriptor in documents() {
        if descriptor.verbs.contains(&Verb::Restore) {
            return Err(RegistryError::RestoreVerbOnDocument {
                slug: descriptor.slug.clone(),
            });
        }
    }

    // I15.
    for descriptor in descriptors {
        let expected: &[Verb] = match descriptor.content_model {
            ContentModel::Typed(schema) | ContentModel::HtmlMockup(schema) => schema.extensions,
            // A generic document has exactly the baseline plus `status`.
            ContentModel::Generic => &[],
        };
        let declared = extension_verbs(descriptor);
        if declared.len() != expected.len() || !declared.iter().all(|verb| expected.contains(verb))
        {
            let verb = declared
                .iter()
                .find(|verb| !expected.contains(verb))
                .or_else(|| expected.iter().find(|verb| !declared.contains(verb)))
                .copied()
                .unwrap_or(Verb::Status);
            return Err(RegistryError::ExtensionVerbMismatch {
                slug: descriptor.slug.clone(),
                verb: verb.as_str(),
            });
        }
    }

    Ok(())
}

/// The verbs a descriptor adds beyond its category's baseline.
fn extension_verbs(descriptor: &TypeDescriptor) -> Vec<Verb> {
    match descriptor.category {
        ArtifactCategory::Document => descriptor
            .verbs
            .iter()
            .copied()
            .filter(|verb| *verb != Verb::Status && !DOCUMENT_BASELINE_VERBS.contains(verb))
            .collect(),
        // Backlog and decision verb sets are fixed; they declare no extensions.
        ArtifactCategory::Backlog | ArtifactCategory::Decision => Vec::new(),
    }
}

/// Whether a document directory value is relative and free of traversal.
///
/// Checked with string inspection only: the registry must not touch
/// [`std::path`] (ADR-0017 decision 9). A trailing separator is allowed.
fn is_relative_directory(value: &str) -> bool {
    if value.starts_with('/') || value.contains('\\') {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    let trimmed = value.trim_end_matches('/');
    if trimmed.is_empty() {
        return false;
    }
    trimmed
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TYPED_DOCUMENT_SLUGS: &[&str] = &[
        "req",
        "nfr",
        "glossary",
        "rule",
        "ui-spec",
        "ui-mockup",
        "audit",
        "tech-spec",
        "use-case",
        "test-case",
        "test-report",
    ];

    fn entry(prefix: &str, directory: &str) -> DocTypeEntry {
        DocTypeEntry {
            prefix: prefix.to_string(),
            directory: directory.to_string(),
            enabled: true,
            statuses: None,
            default_status: None,
        }
    }

    fn config_with(slug: &str, value: DocTypeEntry) -> Config {
        let mut cfg = Config::default();
        cfg.document_types.insert(slug.to_string(), value);
        cfg
    }

    fn default_registry() -> ArtifactRegistry {
        ArtifactRegistry::build(&Config::default()).expect("default configuration builds")
    }

    fn find<'a>(descriptors: &'a [TypeDescriptor], slug: &str) -> &'a TypeDescriptor {
        descriptors
            .iter()
            .find(|descriptor| descriptor.slug == slug)
            .unwrap_or_else(|| panic!("descriptor missing: {}", slug))
    }

    fn find_mut<'a>(descriptors: &'a mut [TypeDescriptor], slug: &str) -> &'a mut TypeDescriptor {
        descriptors
            .iter_mut()
            .find(|descriptor| descriptor.slug == slug)
            .unwrap_or_else(|| panic!("descriptor missing: {}", slug))
    }

    fn assembled(cfg: &Config) -> Vec<TypeDescriptor> {
        assemble(cfg).expect("assembly succeeds")
    }

    fn mutated(
        cfg: &Config,
        slug: &str,
        mutate: impl FnOnce(&mut TypeDescriptor),
    ) -> Vec<TypeDescriptor> {
        let mut descriptors = assembled(cfg);
        mutate(find_mut(&mut descriptors, slug));
        descriptors
    }

    fn invariant_error(descriptors: &[TypeDescriptor]) -> RegistryError {
        check_invariants(descriptors).expect_err("invariant must fail")
    }

    fn schema(descriptor: &TypeDescriptor) -> &'static TypedSchema {
        match descriptor.content_model {
            ContentModel::Typed(schema) | ContentModel::HtmlMockup(schema) => schema,
            ContentModel::Generic => panic!("descriptor has no typed schema: {}", descriptor.slug),
        }
    }

    // 1
    #[test]
    fn backlog_descriptors_match_the_current_command_families() {
        let registry = default_registry();

        let task = registry.descriptor("task").expect("task");
        assert_eq!(task.category, ArtifactCategory::Backlog);
        assert_eq!(task.label, "Task");
        assert_eq!(task.prefix, "TASK");
        assert_eq!(task.directory.config_key, "backlog_directory");
        assert_eq!(task.directory.relative, "tasks");
        assert_eq!(task.verbs, TASK_VERBS.to_vec());
        assert!(task.has_verb(Verb::Archive));
        assert!(task.has_verb(Verb::Restore));

        let milestone = registry.descriptor("milestone").expect("milestone");
        assert_eq!(milestone.verbs, MILESTONE_VERBS.to_vec());
        assert!(!milestone.has_verb(Verb::Restore));

        let release = registry.descriptor("release").expect("release");
        assert_eq!(release.verbs, RELEASE_VERBS.to_vec());
        assert!(!release.has_verb(Verb::Archive));
    }

    // 2
    #[test]
    fn decision_descriptors_follow_configured_decision_types() {
        let registry = default_registry();
        let decisions: Vec<&str> = registry
            .in_category(ArtifactCategory::Decision)
            .map(|descriptor| descriptor.slug.as_str())
            .collect();
        assert_eq!(decisions, vec!["adr", "bdr"]);

        let adr = registry.descriptor("adr").expect("adr");
        assert_eq!(adr.category, ArtifactCategory::Decision);
        assert_eq!(adr.label, "adr");
        assert_eq!(adr.prefix, "ADR");
        assert_eq!(adr.directory.config_key, "decision_directory");
        assert_eq!(adr.directory.relative, "");

        let mut cfg = Config::default();
        cfg.decision_types.remove("bdr");
        cfg.decision_types.insert(
            "policy".to_string(),
            TypeEntry {
                prefix: "POL".to_string(),
                enabled: false,
            },
        );
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        assert!(registry.descriptor("bdr").is_none());
        let policy = registry.descriptor("policy").expect("policy");
        assert_eq!(policy.prefix, "POL");
        assert!(!policy.enabled);
        assert!(
            registry
                .enabled(ArtifactCategory::Decision)
                .all(|descriptor| descriptor.slug != "policy")
        );
    }

    // 3
    #[test]
    fn decision_descriptors_have_no_archive_policy_or_verb() {
        let registry = default_registry();
        for slug in ["adr", "bdr"] {
            let descriptor = registry.descriptor(slug).expect(slug);
            assert_eq!(descriptor.archive_policy, ArchivePolicy::None);
            assert!(!descriptor.has_verb(Verb::Archive));
            assert!(!descriptor.has_verb(Verb::Restore));
            assert_eq!(descriptor.frontmatter_status_key(), None);
            assert!(!descriptor.frontmatter_keys.contains(&"status"));
        }
    }

    // 4
    #[test]
    fn default_document_set_has_the_eleven_typed_slugs() {
        let registry = default_registry();
        for slug in TYPED_DOCUMENT_SLUGS {
            let descriptor = registry.descriptor(slug).expect(slug);
            assert_eq!(descriptor.category, ArtifactCategory::Document);
            assert!(matches!(
                descriptor.content_model,
                ContentModel::Typed(_) | ContentModel::HtmlMockup(_)
            ));
        }
        assert_eq!(
            registry.in_category(ArtifactCategory::Document).count(),
            12,
            "the default map still seeds deploy"
        );
    }

    // 5
    #[test]
    fn deploy_resolves_to_a_generic_descriptor() {
        let registry = default_registry();
        let deploy = registry.descriptor("deploy").expect("deploy");
        assert_eq!(deploy.category, ArtifactCategory::Document);
        assert_eq!(deploy.label, "deploy");
        assert_eq!(deploy.prefix, "DEP");
        assert_eq!(deploy.directory.config_key, "doc_directory");
        assert_eq!(deploy.directory.relative, "architecture/deployment/");
        assert_eq!(deploy.content_model, ContentModel::Generic);
        assert_eq!(deploy.id_shape, IdShape::Plain);
        assert_eq!(
            deploy.verbs,
            vec![
                Verb::Create,
                Verb::List,
                Verb::View,
                Verb::Edit,
                Verb::Archive,
                Verb::Delete,
                Verb::Status,
            ]
        );
        assert_eq!(deploy.create_options, NO_CREATE_OPTIONS);
        assert_eq!(
            deploy.archive_policy,
            ArchivePolicy::ArchivedFolder {
                state_field: Some("status")
            }
        );
        assert_eq!(
            deploy.frontmatter_keys,
            vec![
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "status"
            ]
        );
    }

    // 6
    #[test]
    fn lookup_is_exact_and_case_sensitive() {
        let registry = default_registry();
        assert!(registry.descriptor("req").is_some());
        for slug in ["Req", "REQ", "non-functional requirement", "nfreq", ""] {
            assert!(registry.descriptor(slug).is_none(), "{}", slug);
        }
    }

    // 7
    #[test]
    fn req_and_nfr_declare_grouped_ids_create_group_and_required_group() {
        let registry = default_registry();
        for slug in ["req", "nfr"] {
            let descriptor = registry.descriptor(slug).expect(slug);
            assert_eq!(descriptor.id_shape, IdShape::Grouped);
            assert!(descriptor.has_verb(Verb::CreateGroup));
            assert!(descriptor.has_verb(Verb::Status));
            assert_eq!(descriptor.create_options.group, GroupRequirement::Required);
        }
        assert_eq!(
            registry.descriptor("req").unwrap().label,
            "Business requirement"
        );
        assert_eq!(
            registry.descriptor("nfr").unwrap().label,
            "Non-functional requirement"
        );
    }

    // 8
    #[test]
    fn ui_mockup_declares_html_states_supersede_and_no_status() {
        let registry = default_registry();
        let mockup = registry.descriptor("ui-mockup").expect("ui-mockup");
        assert_eq!(
            mockup.status_model,
            StatusModel::HtmlMetadata {
                states: MOCKUP_STATES
            }
        );
        assert!(mockup.has_verb(Verb::Supersede));
        assert!(!mockup.has_verb(Verb::Status));
        assert!(mockup.frontmatter_keys.is_empty());
        assert_eq!(mockup.frontmatter_status_key(), None);
        assert!(matches!(mockup.content_model, ContentModel::HtmlMockup(_)));
        assert!(schema(mockup).sections.is_empty());
        assert_eq!(schema(mockup).handler, TypedHandler::UiMockup);
        assert_eq!(mockup.create_options.group, GroupRequirement::Absent);
    }

    // 9
    #[test]
    fn document_status_model_contains_main_statuses_and_configured_order() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec![
            "active".to_string(),
            "review".to_string(),
            "archived".to_string(),
            "draft".to_string(),
        ]);
        cfg.document_types.insert("req".to_string(), req);
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        assert_eq!(
            registry.descriptor("req").unwrap().status_model,
            StatusModel::DocumentMain {
                custom: vec!["review".to_string(), "draft".to_string()],
                default: "active".to_string(),
            }
        );
        assert_eq!(MAIN_DOCUMENT_STATUSES, ["active", "archived"]);
    }

    // 10
    #[test]
    fn document_default_status_falls_back_to_active() {
        let registry = default_registry();
        assert_eq!(
            registry.descriptor("glossary").unwrap().status_model,
            StatusModel::DocumentMain {
                custom: Vec::new(),
                default: "active".to_string(),
            }
        );

        let mut cfg = Config::default();
        let mut glossary = entry("GLO", "glossary/");
        glossary.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        glossary.default_status = Some("active".to_string());
        cfg.document_types.insert("glossary".to_string(), glossary);
        assert_eq!(
            ArtifactRegistry::build(&cfg)
                .unwrap()
                .descriptor("glossary")
                .unwrap()
                .status_model,
            StatusModel::DocumentMain {
                custom: Vec::new(),
                default: "active".to_string(),
            }
        );
    }

    // 11
    #[test]
    fn frontmatter_keys_follow_the_field_map_order() {
        let registry = default_registry();

        assert_eq!(
            registry.descriptor("task").unwrap().frontmatter_keys,
            vec![
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "status",
                "assignee",
                "milestone",
                "dependencies",
                "ordinal",
                "documentation",
                "type",
            ]
        );
        assert_eq!(
            registry.descriptor("milestone").unwrap().frontmatter_keys,
            vec![
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "status",
                "release",
            ]
        );
        assert_eq!(
            registry.descriptor("audit").unwrap().frontmatter_keys,
            vec![
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "status",
                "type",
            ]
        );
        assert_eq!(
            registry.descriptor("adr").unwrap().frontmatter_keys,
            vec![
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "superseded_by",
            ]
        );
        assert!(
            registry
                .descriptor("ui-mockup")
                .unwrap()
                .frontmatter_keys
                .is_empty()
        );
    }

    // 12
    #[test]
    fn status_models_are_category_specific() {
        let registry = default_registry();
        for descriptor in registry.descriptors() {
            match descriptor.category {
                ArtifactCategory::Backlog => match descriptor.slug.as_str() {
                    "task" => assert!(matches!(
                        descriptor.status_model,
                        StatusModel::TaskConfigurable { .. }
                    )),
                    _ => assert!(matches!(descriptor.status_model, StatusModel::Fixed { .. })),
                },
                ArtifactCategory::Decision => assert!(matches!(
                    descriptor.status_model,
                    StatusModel::Fixed { states } if states == DECISION_STATES
                )),
                ArtifactCategory::Document => {
                    if descriptor.slug == "ui-mockup" {
                        assert!(matches!(
                            descriptor.status_model,
                            StatusModel::HtmlMetadata { .. }
                        ));
                    } else {
                        assert!(matches!(
                            descriptor.status_model,
                            StatusModel::DocumentMain { .. }
                        ));
                    }
                }
            }
        }
        assert_eq!(
            registry.descriptor("task").unwrap().status_model,
            StatusModel::TaskConfigurable {
                statuses: Config::default().statuses,
                default: Config::default().default_status,
            }
        );
    }

    // 13
    #[test]
    fn id_shapes_cover_plain_dotted_grouped() {
        let registry = default_registry();
        assert_eq!(
            registry.descriptor("glossary").unwrap().id_shape,
            IdShape::Plain
        );
        for slug in ["req", "nfr"] {
            assert_eq!(
                registry.descriptor(slug).unwrap().id_shape,
                IdShape::Grouped
            );
        }
        assert_ne!(IdShape::Plain, IdShape::Dotted);
        assert_ne!(IdShape::Dotted, IdShape::Grouped);
        assert_ne!(IdShape::Plain, IdShape::Grouped);
    }

    // 14
    #[test]
    fn archive_policies_follow_the_layout() {
        let registry = default_registry();
        assert_eq!(
            registry.descriptor("task").unwrap().archive_policy,
            FRONTMATTER_ARCHIVE
        );
        assert_eq!(
            registry.descriptor("milestone").unwrap().archive_policy,
            FRONTMATTER_ARCHIVE
        );
        assert_eq!(
            registry.descriptor("release").unwrap().archive_policy,
            ArchivePolicy::None
        );
        assert_eq!(
            registry.descriptor("adr").unwrap().archive_policy,
            ArchivePolicy::None
        );
        assert_eq!(
            registry.descriptor("ui-mockup").unwrap().archive_policy,
            ArchivePolicy::ArchivedFolder { state_field: None }
        );
        assert_eq!(
            registry.descriptor("glossary").unwrap().archive_policy,
            FRONTMATTER_ARCHIVE
        );
    }

    // 15
    #[test]
    fn disabled_descriptor_resolves_but_is_excluded_from_enabled() {
        let mut cfg = Config::default();
        let mut glossary = entry("GLO", "glossary/");
        glossary.enabled = false;
        cfg.document_types.insert("glossary".to_string(), glossary);
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        assert!(!registry.descriptor("glossary").unwrap().is_enabled());
        assert!(
            registry
                .enabled(ArtifactCategory::Document)
                .all(|descriptor| descriptor.slug != "glossary")
        );
        assert!(
            registry
                .in_category(ArtifactCategory::Document)
                .any(|descriptor| descriptor.slug == "glossary")
        );
    }

    // 16
    #[test]
    fn documents_are_enumerated_in_slug_order() {
        let registry = default_registry();
        let slugs: Vec<&str> = registry
            .in_category(ArtifactCategory::Document)
            .map(|descriptor| descriptor.slug.as_str())
            .collect();
        let mut sorted = slugs.clone();
        sorted.sort_unstable();
        assert_eq!(slugs, sorted);
    }

    // 17
    #[test]
    fn descriptor_total_equals_the_configured_set() {
        let registry = default_registry();
        assert_eq!(registry.len(), 17);
        assert_eq!(registry.in_category(ArtifactCategory::Backlog).count(), 3);
        assert_eq!(registry.in_category(ArtifactCategory::Decision).count(), 2);
        assert_eq!(registry.in_category(ArtifactCategory::Document).count(), 12);
        assert_eq!(
            Config::default().document_types.len() + Config::default().decision_types.len() + 3,
            registry.len()
        );
    }

    // 18
    #[test]
    fn build_uses_no_project_directory() {
        let cfg = Config {
            backlog_directory: "does/not/exist/".to_string(),
            decision_directory: "also/missing/".to_string(),
            doc_directory: "nowhere/".to_string(),
            ..Config::default()
        };
        assert!(ArtifactRegistry::build(&cfg).is_ok());
    }

    // 19
    #[test]
    fn registry_agrees_with_record_descriptor_for_default_documents() {
        use crate::record::descriptor::descriptor_for_doc_type;

        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        for slug in TYPED_DOCUMENT_SLUGS {
            let descriptor = registry.descriptor(slug).expect(slug);
            let entry = cfg.document_types.get(*slug).expect("entry");
            assert_eq!(descriptor.prefix, entry.prefix);
            assert_eq!(descriptor.directory.relative, entry.directory);

            match descriptor.content_model {
                ContentModel::Typed(schema) => {
                    let record = descriptor_for_doc_type(slug).expect("record descriptor");
                    assert_eq!(record.doc_type_key, Some(*slug));
                    assert_eq!(record.directory, "doc");
                    assert_eq!(schema.sections, record.sections);
                    assert_eq!(
                        record.is_dotted_id(),
                        descriptor.id_shape != IdShape::Plain,
                        "{} id shape drifted",
                        slug
                    );
                    assert_eq!(
                        record.configured_prefixes(&cfg),
                        vec![descriptor.prefix.clone()]
                    );
                }
                ContentModel::HtmlMockup(_) => {
                    assert_eq!(*slug, "ui-mockup");
                    assert!(descriptor_for_doc_type(slug).is_none());
                }
                ContentModel::Generic => panic!("default document must be typed: {}", slug),
            }
        }
    }

    // 20
    #[test]
    fn duplicate_slug_is_rejected() {
        let cfg = Config::default();
        let mut descriptors = assembled(&cfg);
        let duplicate = find(&descriptors, "glossary").clone();
        descriptors.push(duplicate);
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::DuplicateSlug { ref slug } if slug == "glossary"
        ));
    }

    // 21
    #[test]
    fn reserved_slug_is_rejected_case_insensitively() {
        for slug in ["task", "TASK", "Task", "adr", "ADR"] {
            let cfg = config_with(slug, entry("TSK", "tasks-docs/"));
            let error = ArtifactRegistry::build(&cfg).expect_err("reserved slug");
            assert!(
                matches!(error, RegistryError::ReservedSlug { ref slug, .. } if slug.eq_ignore_ascii_case("task") || slug.eq_ignore_ascii_case("adr")),
                "{:?}",
                error
            );
        }
    }

    // 22
    #[test]
    fn empty_prefix_is_rejected() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "task", |descriptor| descriptor.prefix.clear());
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::EmptyPrefix { ref slug } if slug == "task"
        ));
        assert!(error.to_string().contains("document_types.task.prefix"));
    }

    // 23
    #[test]
    fn duplicate_prefix_is_rejected_case_insensitively() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "glossary", |descriptor| {
            descriptor.prefix = "req".to_string();
        });
        let error = invariant_error(&descriptors);
        match error {
            RegistryError::DuplicatePrefix { slug, other, .. } => {
                // The later descriptor (`req`) reports the earlier owner.
                assert_eq!(slug, "req");
                assert_eq!(other, "glossary");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    // 24
    #[test]
    fn duplicate_prefix_check_includes_disabled_entries() {
        let mut cfg = Config::default();
        let mut deploy = entry("REQ", "architecture/deployment/");
        deploy.enabled = false;
        cfg.document_types.insert("deploy".to_string(), deploy);
        let error = ArtifactRegistry::build(&cfg).expect_err("duplicate prefix");
        assert!(matches!(error, RegistryError::DuplicatePrefix { .. }));
    }

    // 25
    #[test]
    fn empty_directory_is_rejected() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "glossary", |descriptor| {
            descriptor.directory.relative.clear();
        });
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::EmptyDirectory { ref slug } if slug == "glossary"
        ));
        assert!(
            error
                .to_string()
                .contains("document_types.glossary.directory")
        );
    }

    // 26
    #[test]
    fn absolute_directory_is_rejected() {
        for value in ["/etc/passwd", "C:/records"] {
            let cfg = Config::default();
            let descriptors = mutated(&cfg, "glossary", |descriptor| {
                descriptor.directory.relative = value.to_string();
            });
            let error = invariant_error(&descriptors);
            assert!(matches!(
                error,
                RegistryError::InvalidDirectory { ref slug, .. } if slug == "glossary"
            ));
        }
    }

    // 27
    #[test]
    fn traversing_directory_is_rejected() {
        for value in ["../shared", "a/../../b", "./records", "a//b"] {
            let cfg = Config::default();
            let descriptors = mutated(&cfg, "glossary", |descriptor| {
                descriptor.directory.relative = value.to_string();
            });
            let error = invariant_error(&descriptors);
            assert!(
                matches!(
                    error,
                    RegistryError::InvalidDirectory { ref slug, .. } if slug == "glossary"
                ),
                "{} should be invalid: {:?}",
                value,
                error
            );
        }
    }

    // 28
    #[test]
    fn empty_status_list_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(Vec::new());
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("empty status list");
        assert!(matches!(
            error,
            RegistryError::EmptyStatusList { ref slug } if slug == "req"
        ));
        assert!(error.to_string().contains("document_types.req.statuses"));
    }

    // 29
    #[test]
    fn duplicate_status_value_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec![
            "active".to_string(),
            "archived".to_string(),
            "review".to_string(),
            "review".to_string(),
        ]);
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("duplicate status");
        assert!(matches!(
            error,
            RegistryError::DuplicateStatus { ref value, .. } if value == "review"
        ));
    }

    // 30
    #[test]
    fn status_list_without_active_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec!["archived".to_string()]);
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("missing active");
        assert!(matches!(
            error,
            RegistryError::MainStatusCount {
                value: "active",
                count: 0,
                ..
            }
        ));
    }

    // 31
    #[test]
    fn status_list_without_archived_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec!["active".to_string()]);
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("missing archived");
        assert!(matches!(
            error,
            RegistryError::MainStatusCount {
                value: "archived",
                count: 0,
                ..
            }
        ));
    }

    // 32
    #[test]
    fn status_list_with_repeated_main_status_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec![
            "active".to_string(),
            "active".to_string(),
            "archived".to_string(),
        ]);
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("repeated active");
        assert!(matches!(
            error,
            RegistryError::MainStatusCount {
                value: "active",
                count: 2,
                ..
            }
        ));
    }

    // 33
    #[test]
    fn statuses_on_ui_mockup_is_rejected() {
        let mut cfg = Config::default();
        let mut mockup = entry("MOCK", "specifications/ui/mockups/");
        mockup.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        cfg.document_types.insert("ui-mockup".to_string(), mockup);
        let error = ArtifactRegistry::build(&cfg).expect_err("mockup statuses");
        assert!(matches!(
            error,
            RegistryError::StatusesNotAllowed {
                field: "statuses",
                ..
            }
        ));
        assert!(
            error
                .to_string()
                .contains("document_types.ui-mockup.statuses")
        );

        let mut cfg = Config::default();
        let mut mockup = entry("MOCK", "specifications/ui/mockups/");
        mockup.default_status = Some("pending".to_string());
        cfg.document_types.insert("ui-mockup".to_string(), mockup);
        let error = ArtifactRegistry::build(&cfg).expect_err("mockup default status");
        assert!(matches!(
            error,
            RegistryError::StatusesNotAllowed {
                field: "default_status",
                ..
            }
        ));
    }

    // 34
    #[test]
    fn undeclared_default_status_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        req.default_status = Some("review".to_string());
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("undeclared default");
        assert!(matches!(
            error,
            RegistryError::DefaultStatusNotDeclared { ref value, .. } if value == "review"
        ));
        assert!(
            error
                .to_string()
                .contains("document_types.req.default_status")
        );
    }

    // 35
    #[test]
    fn archived_default_status_is_rejected() {
        let mut cfg = Config::default();
        let mut req = entry("REQ", "requirements/business/");
        req.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        req.default_status = Some("archived".to_string());
        cfg.document_types.insert("req".to_string(), req);
        let error = ArtifactRegistry::build(&cfg).expect_err("archived default");
        assert!(matches!(
            error,
            RegistryError::ArchivedDefaultStatus { ref slug } if slug == "req"
        ));
    }

    // 36
    #[test]
    fn missing_status_verb_is_rejected() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "glossary", |descriptor| {
            descriptor.verbs.retain(|verb| *verb != Verb::Status);
        });
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::StatusVerbMismatch { ref slug } if slug == "glossary"
        ));
    }

    // 37
    #[test]
    fn status_verb_on_a_type_without_frontmatter_status_is_rejected() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "ui-mockup", |descriptor| {
            descriptor.verbs.push(Verb::Status);
        });
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::StatusVerbMismatch { ref slug } if slug == "ui-mockup"
        ));
    }

    // 38
    #[test]
    fn extension_verb_mismatch_is_rejected() {
        let cfg = Config::default();
        let descriptors = mutated(&cfg, "glossary", |descriptor| {
            descriptor.verbs.push(Verb::Supersede);
        });
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::ExtensionVerbMismatch {
                ref slug,
                verb: "supersede"
            } if slug == "glossary"
        ));
    }

    // 39
    #[test]
    fn restore_verb_on_a_document_is_rejected() {
        let cfg = config_with("policy", entry("POL", "policies/"));
        let descriptors = mutated(&cfg, "policy", |descriptor| {
            descriptor.verbs.push(Verb::Restore);
        });
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::RestoreVerbOnDocument { ref slug } if slug == "policy"
        ));
        assert!(error.to_string().contains("document_types.policy"));
    }

    // 40
    #[test]
    fn registry_error_maps_to_validation_outcome() {
        let error = RegistryError::EmptyPrefix {
            slug: "req".to_string(),
        };
        let app_error: AppError = error.into();
        assert_eq!(app_error.code(), 4);
        assert_eq!(
            app_error.message(),
            "document_types.req.prefix: must not be empty"
        );
        assert!(matches!(app_error, AppError::Validation(_)));
    }
}
