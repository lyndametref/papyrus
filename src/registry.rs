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
use crate::record::descriptor::RecordDescriptor;

use std::borrow::Cow;
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
    Close,
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
            Verb::Close => "close",
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
///
/// Every member is owned: the schema is built from the same literal facts the
/// former constants held, but the values live on the descriptor rather than in
/// process-global storage (TECH-071 section 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSchema {
    pub handler: TypedHandler,
    /// Body sections in written order; empty for HTML mockups.
    pub sections: Vec<Cow<'static, str>>,
    /// Extension verbs this family owns; must equal the descriptor's extensions.
    pub extensions: Vec<Verb>,
    /// Ordered per-verb command metadata (ADR-0016 decision 4).
    pub verbs: Vec<VerbSpec>,
}

/// How a command argument consumes values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgAction {
    /// One value.
    Set,
    /// Repeatable value.
    Append,
    /// Boolean switch.
    SetTrue,
    /// Optional value: present with zero or one value.
    OptionalValue,
}

/// One command argument, transcribed from the former `clap` derive field.
///
/// Argument declarations keep the derive order because `clap` derives help
/// display order from insertion order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgSpec {
    pub id: Cow<'static, str>,
    pub long: Option<Cow<'static, str>>,
    pub short: Option<char>,
    /// Explicit value name; `""` for switches.
    pub value_name: Cow<'static, str>,
    pub help: Cow<'static, str>,
    pub positional: bool,
    pub required: bool,
    pub required_unless_present: Option<Cow<'static, str>>,
    pub action: ArgAction,
    pub conflicts_with: Option<Cow<'static, str>>,
}

/// One verb of a document command family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerbSpec {
    pub verb: Verb,
    /// Verb help about-line; empty for `audit` and `ui-mockup`.
    pub about: Cow<'static, str>,
    pub args: Vec<ArgSpec>,
}

/// The canonical presentation order of document verbs (ADR-0016 decision 4).
pub const CANONICAL_DOCUMENT_VERB_ORDER: &[Verb] = &[
    Verb::Create,
    Verb::CreateGroup,
    Verb::List,
    Verb::View,
    Verb::Edit,
    Verb::Archive,
    Verb::Delete,
    Verb::Supersede,
    Verb::Status,
];

/// The body model of a document type (ADR-036 decision 5, REQ-07.004).
///
/// The model is seed data and never a configuration key: a seed slug takes its
/// model from the seed table, and every user-defined slug is `SingleMarkdown`
/// (ADR-036 decisions 6 and 31).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentModel {
    /// One markdown record per plain ID.
    SingleMarkdown,
    /// Group and member records with dotted member IDs.
    GroupedMarkdown,
    /// One HTML record per plain ID with metadata-held state.
    SingleHtmlMockup,
}

impl DocumentModel {
    /// The model a slug resolves to from the seed alone.
    ///
    /// `req` and `nfr` are grouped, `ui-mockup` is an HTML mockup, every other
    /// seed slug and every slug with no seed entry is single markdown.
    pub fn for_slug(slug: &str) -> DocumentModel {
        DOCUMENT_TEMPLATES
            .iter()
            .find(|template| template.slug == slug)
            .map_or(DocumentModel::SingleMarkdown, |template| {
                template.content.document_model()
            })
    }
}

impl TypeDescriptor {
    /// The ordered verb metadata used to build this type's command.
    ///
    /// A generic document carries no schema, so its metadata comes from the
    /// shared document baseline; a built-in type owns its copy.
    pub fn verb_specs(&self) -> Vec<VerbSpec> {
        match &self.schema {
            Some(schema) => schema.verbs.clone(),
            None => generic_document_verbs(),
        }
    }

    /// The module that owns typed validation and patch building, if any.
    pub fn handler(&self) -> Option<TypedHandler> {
        self.schema.as_ref().map(|schema| schema.handler)
    }

    /// Whether this descriptor is the config-defined document path that the
    /// seed-derived model replaces (REQ-07.004, MS-033 AC-7).
    ///
    /// Transitional bridge until MS-034/MS-035 read the seed-derived model and
    /// `reference_visibility` instead.
    pub fn is_config_defined_document(&self) -> bool {
        self.category == ArtifactCategory::Document && !is_builtin_document_slug(&self.slug)
    }

    /// The verb spec for `verb`, when the descriptor declares one.
    pub fn verb_spec(&self, verb: Verb) -> Option<VerbSpec> {
        self.verb_specs().into_iter().find(|spec| spec.verb == verb)
    }

    /// Every declared spec verb, in canonical presentation order.
    pub fn ordered_verb_specs(&self) -> Vec<VerbSpec> {
        let specs = self.verb_specs();
        CANONICAL_DOCUMENT_VERB_ORDER
            .iter()
            .filter(|verb| self.has_verb(**verb))
            .filter_map(|verb| specs.iter().find(|spec| spec.verb == *verb).cloned())
            .collect()
    }
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
    pub document_model: DocumentModel,
    /// Transitional typed schema carried by a built-in type until TASK-249
    /// derives fields, sections, and verbs from the seed (ADR-036 decision 29).
    ///
    /// `None` is the generic config-defined document path retained until the
    /// same task.
    pub schema: Option<TypedSchema>,
    pub frontmatter_keys: Vec<String>,
    pub id_shape: IdShape,
    pub archive_policy: ArchivePolicy,
    /// The record-layer descriptor for a document type (ADR-0017 decision 1).
    ///
    /// `Some` for every document descriptor, `None` for backlog and decision
    /// descriptors, whose record descriptors stay static.
    pub record: Option<RecordDescriptor>,
}

impl TypeDescriptor {
    pub fn has_verb(&self, verb: Verb) -> bool {
        self.verbs.contains(&verb)
    }

    /// `Some("status")` when the type stores a front-matter status.
    pub fn frontmatter_status_key(&self) -> Option<&str> {
        self.frontmatter_keys
            .iter()
            .map(String::as_str)
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
    /// I17: a document verb and its command metadata disagree.
    VerbSpecMismatch { slug: String, verb: &'static str },
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
            RegistryError::VerbSpecMismatch { slug, verb } => write!(
                f,
                "document_types.{}: verb {} has no matching command metadata",
                slug, verb
            ),
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

/// Build one argument spec with the common defaults.
#[allow(clippy::too_many_arguments)]
fn arg(
    id: &'static str,
    long: Option<&'static str>,
    short: Option<char>,
    value_name: &'static str,
    help: &'static str,
    positional: bool,
    required: bool,
    action: ArgAction,
) -> ArgSpec {
    ArgSpec {
        id: Cow::Borrowed(id),
        long: long.map(Cow::Borrowed),
        short,
        value_name: Cow::Borrowed(value_name),
        help: Cow::Borrowed(help),
        positional,
        required,
        required_unless_present: None,
        action,
        conflicts_with: None,
    }
}

/// Help shown by every option that accepts multiline standard input.
const MULTILINE_STDIN_HELP: &str = "Use \"-\" to read multiline text from standard input.";

fn arg_id() -> ArgSpec {
    arg("id", None, None, "ID", "", true, true, ArgAction::Set)
}
fn arg_value() -> ArgSpec {
    arg("value", None, None, "VALUE", "", true, true, ArgAction::Set)
}
fn arg_force() -> ArgSpec {
    arg(
        "force",
        Some("force"),
        None,
        "",
        "",
        false,
        false,
        ArgAction::SetTrue,
    )
}
fn arg_editor() -> ArgSpec {
    arg(
        "editor",
        Some("editor"),
        None,
        "",
        "Open the document body in $VISUAL or $EDITOR",
        false,
        false,
        ArgAction::SetTrue,
    )
}
fn arg_label() -> ArgSpec {
    arg(
        "label",
        Some("label"),
        None,
        "LABEL",
        "Label",
        false,
        false,
        ArgAction::Append,
    )
}
fn arg_reference() -> ArgSpec {
    arg(
        "reference",
        Some("reference"),
        None,
        "REFERENCE",
        "Reference",
        false,
        false,
        ArgAction::Append,
    )
}

fn generic_document_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a document.".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "title",
                    "",
                    true,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "title_flag",
                    Some("title"),
                    None,
                    "title",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "description",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "label",
                    Some("label"),
                    None,
                    "label",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
                arg(
                    "reference",
                    Some("reference"),
                    None,
                    "reference",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List documents.".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a document.".into(),
            args: vec![arg("id", None, None, "id", "", true, true, ArgAction::Set)],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a document.".into(),
            args: vec![
                arg("id", None, None, "id", "", true, true, ArgAction::Set),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "title",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "description",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "editor",
                    Some("editor"),
                    None,
                    "",
                    "",
                    false,
                    false,
                    ArgAction::SetTrue,
                ),
                arg(
                    "label",
                    Some("label"),
                    None,
                    "label",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
                arg(
                    "reference",
                    Some("reference"),
                    None,
                    "reference",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive a document.".into(),
            args: vec![arg("id", None, None, "id", "", true, true, ArgAction::Set)],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete a document.".into(),
            args: vec![
                arg("id", None, None, "id", "", true, true, ArgAction::Set),
                arg(
                    "force",
                    Some("force"),
                    None,
                    "",
                    "",
                    false,
                    false,
                    ArgAction::SetTrue,
                ),
            ],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Set document status.".into(),
            args: vec![
                arg("id", None, None, "id", "", true, true, ArgAction::Set),
                arg("value", None, None, "value", "", true, true, ArgAction::Set),
            ],
        },
    ]
}

fn requirement_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a business requirement".into(),
            args: vec![
                ArgSpec {
                    required_unless_present: Some("title_flag".into()),
                    ..arg(
                        "title",
                        None,
                        None,
                        "title",
                        "Requirement title (positional)",
                        true,
                        false,
                        ArgAction::Set,
                    )
                },
                ArgSpec {
                    required_unless_present: Some("title".into()),
                    ..arg(
                        "title_flag",
                        Some("title"),
                        None,
                        "TITLE_FLAG",
                        "Requirement title (flag)",
                        false,
                        false,
                        ArgAction::Set,
                    )
                },
                arg(
                    "story",
                    Some("story"),
                    None,
                    "STORY",
                    "User story",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "group",
                    Some("group"),
                    None,
                    "GROUP",
                    "Owning business-requirement group",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::CreateGroup,
            about: "Create a business-requirement group".into(),
            args: vec![
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Group title",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "story",
                    Some("story"),
                    None,
                    "STORY",
                    "User story",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "DESCRIPTION",
                    "Description",
                    false,
                    false,
                    ArgAction::Set,
                ),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List business requirements".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a business requirement".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a business requirement".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "story",
                    Some("story"),
                    None,
                    "STORY",
                    "Update the user story",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive a business requirement".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete a business requirement".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Set business requirement status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn glossary_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a glossary term".into(),
            args: vec![
                arg(
                    "term",
                    Some("term"),
                    None,
                    "TERM",
                    "Glossary term",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "definition",
                    Some("definition"),
                    None,
                    "DEFINITION",
                    "Definition of the term. Use \"-\" to read multiline text from standard input.",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List glossary terms".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a glossary term".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a glossary term".into(),
            args: vec![
                arg_id(),
                arg(
                    "term",
                    Some("term"),
                    None,
                    "TERM",
                    "Update the glossary term",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "definition",
                    Some("definition"),
                    None,
                    "DEFINITION",
                    "Update the definition. Use \"-\" to read multiline text from standard input.",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive a glossary term".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete a glossary term".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Set glossary term status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn rule_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a business rule".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "Business rule title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "rationale",
                    Some("rationale"),
                    None,
                    "RATIONALE",
                    "Explain why the rule exists",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Describe when and where the rule applies",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "requirement",
                    Some("requirement"),
                    None,
                    "REQUIREMENT",
                    "Reference a business requirement. Repeatable",
                    false,
                    false,
                    ArgAction::Append,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List business rules".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a business rule".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a business rule".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    Some('d'),
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "requirement",
                    Some("requirement"),
                    None,
                    "REQUIREMENT",
                    "Add a business requirement. Repeatable",
                    false,
                    false,
                    ArgAction::Append,
                ),
                arg(
                    "no_requirement",
                    Some("no-requirement"),
                    None,
                    "NO_REQUIREMENT",
                    "Remove a business requirement; without an ID removes all",
                    false,
                    false,
                    ArgAction::OptionalValue,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive a business rule".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete a business rule".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Set business rule status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn ui_spec_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a UI specification".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "UI specification title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "view",
                    Some("view"),
                    None,
                    "VIEW",
                    "Associate the specification with a view",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List UI specifications".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a UI specification".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a UI specification".into(),
            args: vec![
                arg_id(),
                arg(
                    "view",
                    Some("view"),
                    None,
                    "VIEW",
                    "Update the associated view",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn audit_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "".into(),
            args: vec![
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "",
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "content",
                    Some("content"),
                    None,
                    "CONTENT",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "audit_type",
                    Some("audit-type"),
                    None,
                    "AUDIT_TYPE",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                ArgSpec {
                    help: "".into(),
                    ..arg_label()
                },
                ArgSpec {
                    help: "".into(),
                    ..arg_reference()
                },
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "content",
                    Some("content"),
                    None,
                    "CONTENT",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "audit_type",
                    Some("audit-type"),
                    None,
                    "AUDIT_TYPE",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                ArgSpec {
                    help: "".into(),
                    ..arg_editor()
                },
                ArgSpec {
                    help: "".into(),
                    ..arg_label()
                },
                ArgSpec {
                    help: "".into(),
                    ..arg_reference()
                },
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn tech_spec_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a technical specification".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "Technical specification title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List technical specifications".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a technical specification".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a technical specification".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn use_case_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a use case".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "Use case title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Optional applicability",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List use cases".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a use case".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a use case".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Update the applicability (empty clears it)",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn test_case_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a test case".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "Test case title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Optional applicability",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List test cases".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a test case".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a test case".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Update the applicability (empty clears it)",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn test_report_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "Create a test report".into(),
            args: vec![
                arg(
                    "title",
                    None,
                    None,
                    "TITLE",
                    "Test report title",
                    true,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    true,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Optional applicability",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "List test reports".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "View a test report".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "Edit a test report".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "Update the title",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "description",
                    Some("description"),
                    None,
                    "DESCRIPTION",
                    MULTILINE_STDIN_HELP,
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "applicability",
                    Some("applicability"),
                    None,
                    "APPLICABILITY",
                    "Update the applicability (empty clears it)",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg_editor(),
                arg_label(),
                arg_reference(),
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "Archive".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "Delete".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Status,
            about: "Status".into(),
            args: vec![arg_id(), arg_value()],
        },
    ]
}

fn ui_mockup_verbs() -> Vec<VerbSpec> {
    vec![
        VerbSpec {
            verb: Verb::Create,
            about: "".into(),
            args: vec![
                arg("title", None, None, "TITLE", "", true, true, ArgAction::Set),
                arg(
                    "task",
                    Some("task"),
                    None,
                    "TASK",
                    "",
                    false,
                    true,
                    ArgAction::Set,
                ),
                ArgSpec {
                    required: true,
                    ..arg(
                        "spec",
                        Some("spec"),
                        None,
                        "SPEC",
                        "",
                        false,
                        false,
                        ArgAction::Append,
                    )
                },
                arg(
                    "ref",
                    Some("ref"),
                    None,
                    "REF",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
            ],
        },
        VerbSpec {
            verb: Verb::List,
            about: "".into(),
            args: vec![],
        },
        VerbSpec {
            verb: Verb::View,
            about: "".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Edit,
            about: "".into(),
            args: vec![
                arg_id(),
                arg(
                    "title",
                    Some("title"),
                    None,
                    "TITLE",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "task",
                    Some("task"),
                    None,
                    "TASK",
                    "",
                    false,
                    false,
                    ArgAction::Set,
                ),
                arg(
                    "spec",
                    Some("spec"),
                    None,
                    "SPEC",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
                arg(
                    "ref",
                    Some("ref"),
                    None,
                    "REF",
                    "",
                    false,
                    false,
                    ArgAction::Append,
                ),
                ArgSpec {
                    conflicts_with: Some("superseded_by".into()),
                    ..arg(
                        "status",
                        Some("status"),
                        None,
                        "STATUS",
                        "",
                        false,
                        false,
                        ArgAction::Set,
                    )
                },
                ArgSpec {
                    conflicts_with: Some("status".into()),
                    ..arg(
                        "superseded_by",
                        Some("superseded-by"),
                        None,
                        "SUPERSEDED_BY",
                        "",
                        false,
                        false,
                        ArgAction::Set,
                    )
                },
                ArgSpec {
                    help: "".into(),
                    ..arg_editor()
                },
            ],
        },
        VerbSpec {
            verb: Verb::Archive,
            about: "".into(),
            args: vec![arg_id()],
        },
        VerbSpec {
            verb: Verb::Delete,
            about: "".into(),
            args: vec![arg_id(), arg_force()],
        },
        VerbSpec {
            verb: Verb::Supersede,
            about: "".into(),
            args: vec![
                arg("old", None, None, "OLD", "", true, true, ArgAction::Set),
                arg("new", None, None, "NEW", "", true, true, ArgAction::Set),
            ],
        },
    ]
}

/// Build the owned section list of a schema from its written-order literals.
fn owned_sections(sections: &'static [&'static str]) -> Vec<Cow<'static, str>> {
    sections.iter().copied().map(Cow::Borrowed).collect()
}

fn task_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Task,
        sections: owned_sections(TASK_SECTIONS),
        extensions: Vec::new(),
        verbs: Vec::new(),
    }
}
fn milestone_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Milestone,
        sections: owned_sections(MILESTONE_SECTIONS),
        extensions: Vec::new(),
        verbs: Vec::new(),
    }
}
fn release_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Release,
        sections: owned_sections(RELEASE_SECTIONS),
        extensions: Vec::new(),
        verbs: Vec::new(),
    }
}
fn decision_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Decision,
        sections: owned_sections(DECISION_SECTIONS),
        extensions: Vec::new(),
        verbs: Vec::new(),
    }
}
fn requirement_business_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Requirement(RequirementKind::Business),
        sections: owned_sections(REQUIREMENT_SECTIONS),
        extensions: vec![Verb::CreateGroup],
        verbs: requirement_verbs(),
    }
}
fn requirement_non_functional_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Requirement(RequirementKind::NonFunctional),
        sections: owned_sections(REQUIREMENT_SECTIONS),
        extensions: vec![Verb::CreateGroup],
        verbs: requirement_verbs(),
    }
}
fn glossary_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Glossary,
        sections: owned_sections(GLOSSARY_SECTIONS),
        extensions: Vec::new(),
        verbs: glossary_verbs(),
    }
}
fn rule_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Rule,
        sections: owned_sections(RULE_SECTIONS),
        extensions: Vec::new(),
        verbs: rule_verbs(),
    }
}
fn ui_spec_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::UiSpec,
        sections: owned_sections(UI_SPEC_SECTIONS),
        extensions: Vec::new(),
        verbs: ui_spec_verbs(),
    }
}
fn ui_mockup_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::UiMockup,
        sections: Vec::new(),
        extensions: vec![Verb::Supersede],
        verbs: ui_mockup_verbs(),
    }
}
fn audit_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::Audit,
        sections: owned_sections(AUDIT_SECTIONS),
        extensions: Vec::new(),
        verbs: audit_verbs(),
    }
}
fn tech_spec_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::TechSpec,
        sections: owned_sections(TECH_SPEC_SECTIONS),
        extensions: Vec::new(),
        verbs: tech_spec_verbs(),
    }
}
fn use_case_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::UseCase,
        sections: owned_sections(USE_CASE_SECTIONS),
        extensions: Vec::new(),
        verbs: use_case_verbs(),
    }
}
fn test_case_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::TestCase,
        sections: owned_sections(TEST_CASE_SECTIONS),
        extensions: Vec::new(),
        verbs: test_case_verbs(),
    }
}
fn test_report_schema() -> TypedSchema {
    TypedSchema {
        handler: TypedHandler::TestReport,
        sections: owned_sections(TEST_REPORT_SECTIONS),
        extensions: Vec::new(),
        verbs: test_report_verbs(),
    }
}

const DECISION_STATES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
];
const MILESTONE_STATES: &[&str] = &["open", "closed", "archived"];
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

/// The typed schema shape a template names.
///
/// The template table is the static source of the current section and verb
/// facts. Building a descriptor materialises an owned [`TypedSchema`] from
/// this kind, so no descriptor borrows the table (TECH-071 section 1).
#[derive(Clone, Copy, Debug)]
enum SchemaKind {
    Task,
    Milestone,
    Release,
    Decision,
    RequirementBusiness,
    RequirementNonFunctional,
    Glossary,
    Rule,
    UiSpec,
    UiMockup,
    Audit,
    TechSpec,
    UseCase,
    TestCase,
    TestReport,
}

impl SchemaKind {
    fn schema(self) -> TypedSchema {
        match self {
            SchemaKind::Task => task_schema(),
            SchemaKind::Milestone => milestone_schema(),
            SchemaKind::Release => release_schema(),
            SchemaKind::Decision => decision_schema(),
            SchemaKind::RequirementBusiness => requirement_business_schema(),
            SchemaKind::RequirementNonFunctional => requirement_non_functional_schema(),
            SchemaKind::Glossary => glossary_schema(),
            SchemaKind::Rule => rule_schema(),
            SchemaKind::UiSpec => ui_spec_schema(),
            SchemaKind::UiMockup => ui_mockup_schema(),
            SchemaKind::Audit => audit_schema(),
            SchemaKind::TechSpec => tech_spec_schema(),
            SchemaKind::UseCase => use_case_schema(),
            SchemaKind::TestCase => test_case_schema(),
            SchemaKind::TestReport => test_report_schema(),
        }
    }

    /// The document model this seed kind resolves to (ADR-036 decisions 6, 31).
    fn document_model(self) -> DocumentModel {
        match self {
            SchemaKind::RequirementBusiness | SchemaKind::RequirementNonFunctional => {
                DocumentModel::GroupedMarkdown
            }
            SchemaKind::UiMockup => DocumentModel::SingleHtmlMockup,
            _ => DocumentModel::SingleMarkdown,
        }
    }
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
    content: SchemaKind,
    /// `Some("status")` when the type stores its state in front matter.
    frontmatter_status: Option<&'static str>,
    frontmatter_extra: &'static [&'static str],
    archive: ArchivePolicy,
    create_options: CreateOptions,
    /// Record-layer facts of a document type. Backlog and decision templates
    /// leave this `None`; their record descriptors stay static.
    record: Option<RecordFacts>,
}

/// The record-level facts a document template contributes to the record layer.
///
/// These are the values that used to live in the parallel static document
/// table in `record::descriptor` (ADR-0017 decision 1).
#[derive(Clone, Copy, Debug)]
struct RecordFacts {
    record_name: &'static str,
    record_type: &'static str,
    type_name: &'static str,
    display_id_pattern: &'static str,
    /// Type-specific front-matter keys only, in written order.
    frontmatter_keys: &'static [&'static str],
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
    Verb::Close,
    Verb::Archive,
    Verb::Delete,
];
const RELEASE_VERBS: &[Verb] = &[
    Verb::Create,
    Verb::List,
    Verb::View,
    Verb::Edit,
    Verb::Close,
];
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
        content: SchemaKind::Task,
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
        record: None,
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
        content: SchemaKind::Milestone,
        frontmatter_status: Some("status"),
        frontmatter_extra: &["release"],
        // The milestone lifecycle is `open -> closed -> archived` (ADR-032).
        // `archive` closes an open milestone if needed, then moves the file to
        // the backlog archive milestones folder, which is both the lifecycle
        // archive destination and the legacy read/import location.
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: None,
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
        content: SchemaKind::Release,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: ArchivePolicy::None,
        create_options: NO_CREATE_OPTIONS,
        record: None,
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
    content: SchemaKind::Decision,
    // Decisions keep `status` in the document core, not in front matter.
    frontmatter_status: None,
    frontmatter_extra: &["superseded_by"],
    archive: ArchivePolicy::None,
    create_options: NO_CREATE_OPTIONS,
    record: None,
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
        content: SchemaKind::RequirementBusiness,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: REQUIRED_GROUP,
        record: Some(RecordFacts {
            record_name: "business requirement",
            record_type: "requirement",
            type_name: "requirement",
            display_id_pattern: "REQ-###.###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::RequirementNonFunctional,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: REQUIRED_GROUP,
        record: Some(RecordFacts {
            record_name: "non-functional requirement",
            record_type: "nfr",
            type_name: "nfr",
            display_id_pattern: "NFR-###.###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::Glossary,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "glossary term",
            record_type: "glossary",
            type_name: "glossary",
            display_id_pattern: "GLO-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::Rule,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "business rule",
            record_type: "rule",
            type_name: "rule",
            display_id_pattern: "RUL-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::UiSpec,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "ui specification",
            record_type: "ui-spec",
            type_name: "ui_spec",
            display_id_pattern: "UI-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::UiMockup,
        frontmatter_status: None,
        frontmatter_extra: &[],
        archive: HTML_METADATA_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "ui mockup",
            record_type: "ui-mockup",
            type_name: "ui_mockup",
            display_id_pattern: "UI-###",
            frontmatter_keys: &[],
        }),
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
        content: SchemaKind::Audit,
        frontmatter_status: Some("status"),
        frontmatter_extra: &["type"],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "audit record",
            record_type: "audit",
            type_name: "audit",
            display_id_pattern: "AUD-###",
            frontmatter_keys: &["status", "type"],
        }),
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
        content: SchemaKind::TechSpec,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "technical specification",
            record_type: "tech-spec",
            type_name: "tech_spec",
            display_id_pattern: "TECH-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::UseCase,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "use case",
            record_type: "use-case",
            type_name: "use_case",
            display_id_pattern: "UC-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::TestCase,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "test case",
            record_type: "test-case",
            type_name: "test_case",
            display_id_pattern: "TC-###",
            frontmatter_keys: &["status"],
        }),
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
        content: SchemaKind::TestReport,
        frontmatter_status: Some("status"),
        frontmatter_extra: &[],
        archive: FRONTMATTER_ARCHIVE,
        create_options: NO_CREATE_OPTIONS,
        record: Some(RecordFacts {
            record_name: "test report",
            record_type: "test-report",
            type_name: "test_report",
            display_id_pattern: "TR-###",
            frontmatter_keys: &["status"],
        }),
    },
];

/// The built-in registry: every configured type with its behavior.
#[derive(Clone, Debug)]
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

    /// The record-layer descriptor of a document type.
    ///
    /// This is the only slug-to-record lookup: record facts for document types
    /// live on the registry descriptor (ADR-0017 decisions 1 and 4).
    pub fn record_descriptor(&self, slug: &str) -> Option<&RecordDescriptor> {
        self.descriptor(slug)
            .and_then(|descriptor| descriptor.record.as_ref())
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

/// Whether a document slug is declared by a built-in template.
pub fn is_builtin_document_slug(slug: &str) -> bool {
    DOCUMENT_TEMPLATES
        .iter()
        .any(|template| template.slug == slug)
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

fn frontmatter_keys(template: &DescriptorTemplate) -> Vec<String> {
    let mut keys: Vec<String> = GENERAL_FRONTMATTER_KEYS
        .iter()
        .map(|key| (*key).to_string())
        .collect();
    if let Some(key) = template.frontmatter_status {
        keys.push(key.to_string());
    }
    keys.extend(
        template
            .frontmatter_extra
            .iter()
            .map(|key| (*key).to_string()),
    );
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
        document_model: template.content.document_model(),
        schema: Some(template.content.schema()),
        frontmatter_keys: frontmatter_keys(template),
        id_shape: template.id_shape,
        archive_policy: template.archive,
        record: None,
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
        document_model: SchemaKind::Decision.document_model(),
        schema: Some(SchemaKind::Decision.schema()),
        frontmatter_keys: frontmatter_keys(&DECISION_TEMPLATE),
        id_shape: IdShape::Plain,
        archive_policy: ArchivePolicy::None,
        record: None,
    }
}

/// Build one document descriptor, typed when a template exists and generic
/// otherwise (REQ-07.004, ADR-0018).
fn document_descriptor(slug: &str, entry: &DocTypeEntry) -> Result<TypeDescriptor, RegistryError> {
    let template = DOCUMENT_TEMPLATES
        .iter()
        .find(|template| template.slug == slug);

    let document_model = template.map_or(DocumentModel::SingleMarkdown, |template| {
        template.content.document_model()
    });
    let schema = template.map(|template| template.content.schema());

    let (status_model, verbs, id_shape, create_options, archive_policy, frontmatter_keys) =
        match template {
            Some(template) => {
                let status_model = document_status_model(slug, entry, template.status)?;
                let mut verbs = DOCUMENT_BASELINE_VERBS.to_vec();
                if matches!(status_model, StatusModel::DocumentMain { .. }) {
                    verbs.push(Verb::Status);
                }
                if let TemplateVerbs::DocumentBaseline { extensions } = template.verbs {
                    verbs.extend(extensions.iter().copied());
                }
                let keys = if document_model == DocumentModel::SingleHtmlMockup {
                    // UI mockups are HTML records and carry no front matter.
                    Vec::new()
                } else {
                    frontmatter_keys(template)
                };
                (
                    status_model,
                    verbs,
                    template.id_shape,
                    template.create_options,
                    template.archive,
                    keys,
                )
            }
            None => {
                let status_model =
                    document_status_model(slug, entry, TemplateStatus::DocumentMain)?;
                let mut verbs = DOCUMENT_BASELINE_VERBS.to_vec();
                verbs.push(Verb::Status);
                let mut keys: Vec<String> = GENERAL_FRONTMATTER_KEYS
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect();
                keys.push("status".to_string());
                (
                    status_model,
                    verbs,
                    IdShape::Plain,
                    NO_CREATE_OPTIONS,
                    FRONTMATTER_ARCHIVE,
                    keys,
                )
            }
        };

    let record = match template {
        Some(template) => template
            .record
            .map(|facts| document_record(template, schema.as_ref(), facts)),
        // A config-defined type has no static template, so the registry builds
        // its record facts inline from the type it already owns. The registry
        // owns the descriptor; nothing is leaked or cached (ADR-0017 D10).
        None => Some(RecordDescriptor {
            name: Cow::Owned(slug.to_string()),
            record_type: Cow::Borrowed("document"),
            type_name: Cow::Borrowed("document"),
            label: Cow::Owned(slug.to_string()),
            display_id_pattern: Cow::Borrowed("GEN-###"),
            directory: Cow::Borrowed("doc"),
            subpath: Cow::Borrowed(""),
            doc_type_key: Some(Cow::Owned(slug.to_string())),
            frontmatter_keys: vec![Cow::Borrowed("status")],
            sections: vec![Cow::Borrowed("## Description")],
        }),
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
        document_model,
        schema,
        frontmatter_keys,
        id_shape,
        archive_policy,
        record,
    })
}

/// Project a document template's record facts into the record-layer shape.
///
/// `directory` and `subpath` stay at the record-layer defaults: the concrete
/// location comes from the `document_types.<slug>.directory` entry that
/// `RecordDescriptor::resolve_directory` reads through `doc_type_key`.
fn document_record(
    template: &DescriptorTemplate,
    schema: Option<&TypedSchema>,
    facts: RecordFacts,
) -> RecordDescriptor {
    RecordDescriptor {
        name: Cow::Owned(facts.record_name.to_string()),
        record_type: Cow::Borrowed(facts.record_type),
        type_name: Cow::Borrowed(facts.type_name),
        label: Cow::Borrowed(template.label),
        display_id_pattern: Cow::Borrowed(facts.display_id_pattern),
        directory: Cow::Borrowed("doc"),
        subpath: Cow::Borrowed(""),
        doc_type_key: Some(Cow::Borrowed(template.slug)),
        frontmatter_keys: facts
            .frontmatter_keys
            .iter()
            .copied()
            .map(Cow::Borrowed)
            .collect(),
        sections: match schema {
            Some(schema) => schema.sections.clone(),
            // Unreachable for a template, but keeps the projection total.
            None => vec![Cow::Borrowed("## Description")],
        },
    }
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
        let has_status_key = descriptor
            .frontmatter_keys
            .iter()
            .any(|key| key == "status");
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
        let expected: &[Verb] = match &descriptor.schema {
            Some(schema) => &schema.extensions,
            // A generic document has exactly the baseline plus `status`.
            None => &[],
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

    // I17: every document verb has a spec and every spec verb is declared.
    for descriptor in documents() {
        let specs = descriptor.verb_specs();
        if let Some(verb) = descriptor
            .verbs
            .iter()
            .find(|verb| !specs.iter().any(|spec| spec.verb == **verb))
        {
            return Err(RegistryError::VerbSpecMismatch {
                slug: descriptor.slug.clone(),
                verb: verb.as_str(),
            });
        }
        if let Some(spec) = specs
            .iter()
            .find(|spec| !descriptor.verbs.contains(&spec.verb))
        {
            return Err(RegistryError::VerbSpecMismatch {
                slug: descriptor.slug.clone(),
                verb: spec.verb.as_str(),
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
    use std::fs;
    use std::path::Path;

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
            ..Default::default()
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

    fn schema(descriptor: &TypeDescriptor) -> &TypedSchema {
        descriptor
            .schema
            .as_ref()
            .unwrap_or_else(|| panic!("descriptor has no typed schema: {}", descriptor.slug))
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
        assert_eq!(
            milestone.verbs,
            vec![
                Verb::Create,
                Verb::List,
                Verb::View,
                Verb::Edit,
                Verb::Close,
                Verb::Archive,
                Verb::Delete
            ],
            "milestone verbs are exactly create, list, view, edit, close, archive, delete"
        );
        assert!(milestone.has_verb(Verb::Close));
        assert!(milestone.has_verb(Verb::Archive));
        for absent in [Verb::Restore, Verb::Status] {
            assert!(
                !milestone.has_verb(absent),
                "milestone must not expose {}",
                absent.as_str()
            );
        }

        let release = registry.descriptor("release").expect("release");
        assert_eq!(release.verbs, RELEASE_VERBS.to_vec());
        assert!(release.has_verb(Verb::Close));
        for absent in [Verb::Archive, Verb::Restore, Verb::Delete, Verb::Status] {
            assert!(
                !release.has_verb(absent),
                "release must not expose {}",
                absent.as_str()
            );
        }
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
            assert!(
                !descriptor
                    .frontmatter_keys
                    .iter()
                    .any(|key| key == "status")
            );
        }
    }

    // 4
    #[test]
    fn default_document_set_has_the_eleven_typed_slugs() {
        let registry = default_registry();
        for slug in TYPED_DOCUMENT_SLUGS {
            let descriptor = registry.descriptor(slug).expect(slug);
            assert_eq!(descriptor.category, ArtifactCategory::Document);
            assert_eq!(
                descriptor.document_model,
                match *slug {
                    "req" | "nfr" => DocumentModel::GroupedMarkdown,
                    "ui-mockup" => DocumentModel::SingleHtmlMockup,
                    _ => DocumentModel::SingleMarkdown,
                }
            );
            assert!(descriptor.schema.is_some(), "{slug} schema");
        }
        assert_eq!(
            registry.in_category(ArtifactCategory::Document).count(),
            11,
            "the default map seeds the eleven configured document types"
        );
        assert!(registry.descriptor("deploy").is_none());
    }

    #[test]
    fn document_model_is_seed_only_and_user_defined_slugs_default_to_single_markdown() {
        for (slug, expected) in [
            ("req", DocumentModel::GroupedMarkdown),
            ("nfr", DocumentModel::GroupedMarkdown),
            ("ui-mockup", DocumentModel::SingleHtmlMockup),
            ("glossary", DocumentModel::SingleMarkdown),
            ("rule", DocumentModel::SingleMarkdown),
            ("ui-spec", DocumentModel::SingleMarkdown),
            ("audit", DocumentModel::SingleMarkdown),
            ("tech-spec", DocumentModel::SingleMarkdown),
            ("use-case", DocumentModel::SingleMarkdown),
            ("test-case", DocumentModel::SingleMarkdown),
            ("test-report", DocumentModel::SingleMarkdown),
            ("deploy", DocumentModel::SingleMarkdown),
            ("", DocumentModel::SingleMarkdown),
        ] {
            assert_eq!(DocumentModel::for_slug(slug), expected, "{slug}");
        }
        assert!(!is_builtin_document_slug("deploy"));
        assert!(is_builtin_document_slug("req"));

        let registry = ArtifactRegistry::build(&config_with(
            "deploy",
            entry("DEP", "architecture/deployment/"),
        ))
        .expect("registry");
        let deploy = registry.descriptor("deploy").expect("deploy");
        assert_eq!(deploy.document_model, DocumentModel::SingleMarkdown);
        assert!(deploy.schema.is_none());
        assert!(deploy.is_config_defined_document());
        assert!(
            !registry
                .descriptor("req")
                .expect("req")
                .is_config_defined_document()
        );
    }

    // 5
    #[test]
    fn deploy_resolves_to_a_generic_descriptor_when_declared() {
        let registry = ArtifactRegistry::build(&config_with(
            "deploy",
            entry("DEP", "architecture/deployment/"),
        ))
        .expect("explicit deploy configuration builds");
        let deploy = registry.descriptor("deploy").expect("deploy");
        assert_eq!(deploy.category, ArtifactCategory::Document);
        assert_eq!(deploy.label, "deploy");
        assert_eq!(deploy.prefix, "DEP");
        assert_eq!(deploy.directory.config_key, "doc_directory");
        assert_eq!(deploy.directory.relative, "architecture/deployment/");
        assert_eq!(deploy.document_model, DocumentModel::SingleMarkdown);
        assert!(deploy.schema.is_none());
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
        assert_eq!(mockup.document_model, DocumentModel::SingleHtmlMockup);
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
            FRONTMATTER_ARCHIVE,
            "milestones archive to the backlog archive milestones folder"
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
        assert_eq!(registry.len(), 16);
        assert_eq!(registry.in_category(ArtifactCategory::Backlog).count(), 3);
        assert_eq!(registry.in_category(ArtifactCategory::Decision).count(), 2);
        assert_eq!(registry.in_category(ArtifactCategory::Document).count(), 11);
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
    fn every_document_descriptor_has_a_record_and_no_other_slug_does() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        let documents: Vec<&TypeDescriptor> =
            registry.in_category(ArtifactCategory::Document).collect();
        assert_eq!(documents.len(), TYPED_DOCUMENT_SLUGS.len());

        let with_record = documents
            .iter()
            .filter(|descriptor| registry.record_descriptor(&descriptor.slug).is_some())
            .count();
        assert_eq!(
            with_record,
            documents.len(),
            "every document descriptor must carry a record descriptor"
        );

        for slug in ["task", "milestone", "release", "adr", "bdr"] {
            assert!(
                registry.record_descriptor(slug).is_none(),
                "{slug} must not carry a document record descriptor"
            );
        }
        assert!(registry.record_descriptor("deploy").is_none());
        assert!(registry.record_descriptor("unknown").is_none());
    }

    // 20
    #[test]
    fn record_descriptor_fields_derive_from_their_registry_descriptor() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("registry-record-fields");
        fs::create_dir_all(&root).expect("tmp project");
        let project = fs::canonicalize(&root).expect("canonical project");

        for descriptor in registry.in_category(ArtifactCategory::Document) {
            let slug = descriptor.slug.as_str();
            let record = registry
                .record_descriptor(slug)
                .unwrap_or_else(|| panic!("record descriptor missing: {slug}"));

            assert_eq!(
                record.doc_type_key.as_deref(),
                Some(slug),
                "{slug} doc-type key"
            );
            assert_eq!(record.label, descriptor.label, "{slug} label");
            assert_eq!(
                record.configured_prefixes(&cfg),
                vec![descriptor.prefix.clone()],
                "{slug} prefix"
            );
            assert_eq!(
                record.is_dotted_id(),
                descriptor.id_shape != IdShape::Plain,
                "{slug} id shape"
            );
            // The registry also stores the general keys; the record keeps only
            // the type-specific tail, in the same written order.
            let expected_keys: Vec<&str> = descriptor
                .frontmatter_keys
                .iter()
                .skip(GENERAL_FRONTMATTER_KEYS.len())
                .map(String::as_str)
                .collect();
            assert_eq!(
                record.frontmatter_keys, expected_keys,
                "{slug} front-matter keys"
            );
            assert_eq!(
                record.has_frontmatter_key("status"),
                descriptor.frontmatter_status_key().is_some(),
                "{slug} status key"
            );

            let expected_sections: &[Cow<'static, str>] =
                &descriptor.schema.as_ref().expect("typed schema").sections;
            assert_eq!(record.sections, expected_sections, "{slug} sections");

            let expected_dir = project
                .join(&cfg.doc_directory)
                .join(&descriptor.directory.relative);
            assert_eq!(
                record
                    .resolve_directory(&cfg, &project)
                    .expect("record directory"),
                expected_dir,
                "{slug} directory"
            );
        }
        let _ = fs::remove_dir_all(&root);
    }

    // 21
    #[test]
    fn record_descriptor_resolves_builtin_and_config_defined_types_alike() {
        let mut cfg = Config::default();
        cfg.document_types
            .insert("note".to_string(), entry("NOTE", "notes/"));
        let registry = ArtifactRegistry::build(&cfg).expect("registry builds");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("registry-record-parity");
        fs::create_dir_all(&root).expect("tmp project");
        let project = fs::canonicalize(&root).expect("canonical project");

        let glossary = registry
            .record_descriptor("glossary")
            .expect("glossary record");
        assert_eq!(glossary.doc_type_key.as_deref(), Some("glossary"));
        assert_eq!(glossary.configured_prefixes(&cfg), vec!["GLO"]);
        assert!(glossary.has_frontmatter_key("status"));
        assert_eq!(glossary.sections, GLOSSARY_SECTIONS);
        assert_eq!(glossary.record_type, "glossary");

        let note = registry.record_descriptor("note").expect("note record");
        assert_eq!(note.doc_type_key.as_deref(), Some("note"));
        assert_eq!(note.configured_prefixes(&cfg), vec!["NOTE"]);
        assert!(note.has_frontmatter_key("status"));
        assert_eq!(note.sections, &["## Description"]);
        assert_eq!(note.record_type, "document");
        assert_eq!(note.label, "note");

        let glossary_dir = project
            .join(&cfg.doc_directory)
            .join(&cfg.document_types["glossary"].directory);
        let note_dir = project
            .join(&cfg.doc_directory)
            .join(&cfg.document_types["note"].directory);
        assert_eq!(
            glossary.resolve_directory(&cfg, &project).unwrap(),
            glossary_dir
        );
        assert_eq!(note.resolve_directory(&cfg, &project).unwrap(), note_dir);
        assert_ne!(glossary_dir, note_dir);
        let _ = fs::remove_dir_all(&root);
    }

    // 22
    #[test]
    fn no_parallel_document_record_table_remains() {
        use crate::record::descriptor::descriptors as static_records;

        let static_records = static_records();
        assert_eq!(static_records.len(), 4, "only backlog families are static");
        assert!(
            static_records
                .iter()
                .all(|descriptor| descriptor.doc_type_key.is_none()),
            "static records must not describe document types"
        );

        let source = fs::read_to_string("src/record/descriptor.rs").expect("descriptor source");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production lines");
        // A per-slug literal would be a new parallel document table, whether
        // borrowed or owned (`doc_type_key: Some(` matches both forms).
        for forbidden in [
            concat!("descriptor_for", "_doc_type"),
            "doc_type_key: Some(",
        ] {
            assert!(
                !production.contains(forbidden),
                "src/record/descriptor.rs must not contain `{forbidden}`"
            );
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
    // 41
    #[test]
    fn every_document_verb_has_a_spec_and_every_spec_verb_is_declared() {
        let registry = default_registry();
        for descriptor in registry.in_category(ArtifactCategory::Document) {
            let specs = descriptor.verb_specs();
            for verb in &descriptor.verbs {
                assert!(
                    specs.iter().any(|spec| spec.verb == *verb),
                    "{}: verb {} has no spec",
                    descriptor.slug,
                    verb.as_str()
                );
            }
            for spec in specs {
                assert!(
                    descriptor.verbs.contains(&spec.verb),
                    "{}: spec verb {} is not declared",
                    descriptor.slug,
                    spec.verb.as_str()
                );
            }
        }
        // A declared verb without a spec is a startup defect.
        let no_spec_schema = TypedSchema {
            handler: TypedHandler::Glossary,
            sections: Vec::new(),
            extensions: Vec::new(),
            verbs: Vec::new(),
        };
        let cfg = Config::default();
        let mut descriptors = assembled(&cfg);
        let glossary = find_mut(&mut descriptors, "glossary");
        glossary.schema = Some(no_spec_schema);
        let error = invariant_error(&descriptors);
        assert!(matches!(
            error,
            RegistryError::VerbSpecMismatch { ref slug, verb: "create" } if slug == "glossary"
        ));
    }

    // 42
    #[test]
    fn canonical_verb_order_and_typed_value_names_are_preserved() {
        let registry = default_registry();

        let glossary = registry.descriptor("glossary").unwrap();
        let order: Vec<&str> = glossary
            .ordered_verb_specs()
            .iter()
            .map(|spec| spec.verb.as_str())
            .collect();
        assert_eq!(
            order,
            vec![
                "create", "list", "view", "edit", "archive", "delete", "status"
            ]
        );

        let req = registry.descriptor("req").unwrap();
        let order: Vec<&str> = req
            .ordered_verb_specs()
            .iter()
            .map(|spec| spec.verb.as_str())
            .collect();
        assert_eq!(
            order,
            vec![
                "create",
                "create-group",
                "list",
                "view",
                "edit",
                "archive",
                "delete",
                "status"
            ]
        );
        let create = req.verb_spec(Verb::Create).unwrap();
        let title_flag = create
            .args
            .iter()
            .find(|arg| arg.id == "title_flag")
            .unwrap();
        assert_eq!(title_flag.value_name, "TITLE_FLAG");
        assert_eq!(title_flag.long.as_deref(), Some("title"));
        let title = create.args.iter().find(|arg| arg.id == "title").unwrap();
        assert_eq!(title.value_name, "title");
        assert_eq!(title.required_unless_present.as_deref(), Some("title_flag"));

        let mockup = registry.descriptor("ui-mockup").unwrap();
        let edit = mockup.verb_spec(Verb::Edit).unwrap();
        assert_eq!(
            edit.args
                .iter()
                .find(|arg| arg.id == "status")
                .unwrap()
                .conflicts_with
                .as_deref(),
            Some("superseded_by")
        );
        let create = mockup.verb_spec(Verb::Create).unwrap();
        assert!(
            create
                .args
                .iter()
                .find(|arg| arg.id == "spec")
                .unwrap()
                .required
        );

        let built = ArtifactRegistry::build(&config_with("deploy", entry("DEP", "deploy/")))
            .expect("generic registry");
        let generic = built.descriptor("deploy").unwrap();
        let create = generic.verb_spec(Verb::Create).unwrap();
        assert_eq!(
            create
                .args
                .iter()
                .find(|arg| arg.id == "title_flag")
                .unwrap()
                .value_name,
            "title"
        );
    }

    // 43
    #[test]
    fn every_seed_type_keeps_the_pre_change_frontmatter_key_order() {
        let registry = default_registry();
        let general = [
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
        ];
        let cases: &[(&str, bool, &[&str])] = &[
            ("req", true, &[]),
            ("nfr", true, &[]),
            ("glossary", true, &[]),
            ("rule", true, &[]),
            ("ui-spec", true, &[]),
            ("ui-mockup", false, &[]),
            ("audit", true, &["type"]),
            ("tech-spec", true, &[]),
            ("use-case", true, &[]),
            ("test-case", true, &[]),
            ("test-report", true, &[]),
            (
                "task",
                true,
                &[
                    "assignee",
                    "milestone",
                    "dependencies",
                    "ordinal",
                    "documentation",
                    "type",
                ],
            ),
            ("milestone", true, &["release"]),
            ("release", true, &[]),
            ("adr", false, &["superseded_by"]),
        ];
        for (slug, has_frontmatter, extra) in cases {
            let descriptor = registry
                .descriptor(slug)
                .unwrap_or_else(|| panic!("{slug}"));
            if !has_frontmatter {
                if *slug == "ui-mockup" {
                    assert!(descriptor.frontmatter_keys.is_empty(), "{slug}");
                } else {
                    let mut expected: Vec<String> =
                        general.iter().map(|key| (*key).to_string()).collect();
                    expected.extend(extra.iter().map(|key| (*key).to_string()));
                    assert_eq!(descriptor.frontmatter_keys, expected, "{slug}");
                }
                continue;
            }
            let mut expected: Vec<String> = general.iter().map(|key| (*key).to_string()).collect();
            expected.push("status".to_string());
            expected.extend(extra.iter().map(|key| (*key).to_string()));
            assert_eq!(descriptor.frontmatter_keys, expected, "{slug}");
        }
    }

    // 44
    #[test]
    fn config_defined_record_descriptor_is_owned_and_registry_local() {
        let cfg = config_with("note", entry("NOTE", "notes/"));
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let record = registry.record_descriptor("note").expect("note record");

        assert_eq!(record.name, "note");
        assert_eq!(record.label, "note");
        assert_eq!(record.doc_type_key.as_deref(), Some("note"));
        assert_eq!(record.frontmatter_keys, &["status"]);
        assert_eq!(record.sections, &["## Description"]);
        assert_eq!(record.configured_prefixes(&cfg), vec!["NOTE".to_string()]);
        assert_eq!(record.directory.as_ref(), "doc");
        assert_eq!(record.subpath.as_ref(), "");

        // A second registry built from another map is independent.
        let mut other = Config::default();
        other
            .document_types
            .insert("memo".to_string(), entry("MEMO", "memos/"));
        let other_registry = ArtifactRegistry::build(&other).expect("other registry");
        assert!(other_registry.record_descriptor("note").is_none());
        assert_eq!(
            other_registry.record_descriptor("memo").expect("memo").name,
            "memo"
        );

        // Repeating the lookup returns a field-equal value.
        assert_eq!(registry.record_descriptor("note"), Some(record));
    }

    // 45
    #[test]
    fn verb_specs_and_arguments_are_owned_and_cloneable() {
        let registry = default_registry();
        let descriptor = registry.descriptor("req").expect("req");
        let specs: Vec<VerbSpec> = descriptor.ordered_verb_specs();
        let create = specs
            .into_iter()
            .find(|spec| spec.verb == Verb::Create)
            .expect("create spec");
        // Moving and cloning the owned values proves they carry no lifetime.
        let moved = create.clone();
        assert_eq!(moved.about.as_ref(), "Create a business requirement");
        assert_eq!(moved.args.len(), 7);
        assert_eq!(moved.args[0].id.as_ref(), "title");
    }
}
