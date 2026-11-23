//! Old-configuration load identity for the MS-033 descriptor refactor.
//!
//! Every configuration that loaded before MS-033 must still load without an
//! edit and project to the same effective document-type descriptor facts:
//! prefix, directory, enabled flag, statuses, default status, document model,
//! ID shape, verb set, and the ordered per-role field tuples
//! `(heading, frontmatter_key, option, kind, required, verbs)`.
//!
//! The expected table below is test data derived from ADR-036 decision 29 and
//! the frozen `tests/golden/cli-help` snapshots; it is not a production table.
//! The single permitted projected difference is the reference-visibility
//! default of a user-defined type: the former `ContentModel::Generic` hidden
//! rule becomes `Visible` (ADR-036 decision 33, REQ-006.019, TC-185).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use papyrus::commands::load_effective_config;
use papyrus::config::{Config, FieldSpec, FieldVerb, is_list_valued_key, yaml_get};
use papyrus::registry::{
    ArtifactCategory, ArtifactRegistry, DocumentModel, StatusModel, TypeDescriptor, Verb,
};

const CREATE_AND_EDIT: &[&str] = &["create", "edit"];
const CREATE_ONLY: &[&str] = &["create"];

/// One projected field of a role, in ADR-036 decision 29 order.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldView {
    heading: Option<String>,
    frontmatter_key: Option<String>,
    option: String,
    kind: String,
    required: bool,
    verbs: Vec<String>,
}

impl FieldView {
    fn new(
        heading: Option<&str>,
        frontmatter_key: Option<&str>,
        option: &str,
        kind: &str,
        required: bool,
        verbs: &[&str],
    ) -> Self {
        Self {
            heading: heading.map(str::to_string),
            frontmatter_key: frontmatter_key.map(str::to_string),
            option: option.to_string(),
            kind: kind.to_string(),
            required,
            verbs: verbs.iter().map(|verb| (*verb).to_string()).collect(),
        }
    }
}

/// One projected role: its name and ordered field sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RoleView {
    name: &'static str,
    fields: Vec<FieldView>,
}

impl RoleView {
    fn single(fields: &[FieldView]) -> Self {
        Self {
            name: "single",
            fields: fields.to_vec(),
        }
    }

    fn new(name: &'static str, fields: &[FieldView]) -> Self {
        Self {
            name,
            fields: fields.to_vec(),
        }
    }
}

/// The complete effective descriptor projection compared across configs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Projection {
    prefix: String,
    directory_key: String,
    directory: String,
    enabled: bool,
    statuses: Vec<String>,
    default_status: Option<String>,
    model: &'static str,
    id_shape: &'static str,
    verbs: Vec<String>,
    roles: Vec<RoleView>,
    reference_visibility: Option<String>,
}

fn project(descriptor: &TypeDescriptor) -> Projection {
    let (statuses, default_status) = project_status(&descriptor.status_model);
    let mut verbs: Vec<String> = descriptor
        .verbs
        .iter()
        .map(|verb| verb.as_str().to_string())
        .collect();
    verbs.sort();
    Projection {
        prefix: descriptor.prefix.clone(),
        directory_key: descriptor.directory.config_key.to_string(),
        directory: descriptor.directory.relative.clone(),
        enabled: descriptor.enabled,
        statuses,
        default_status,
        model: match descriptor.document_model {
            DocumentModel::SingleMarkdown => "single-markdown",
            DocumentModel::GroupedMarkdown => "grouped-markdown",
            DocumentModel::SingleHtmlMockup => "single-html-mockup",
        },
        id_shape: match descriptor.id_shape {
            papyrus::registry::IdShape::Plain => "plain",
            papyrus::registry::IdShape::Dotted => "dotted",
            papyrus::registry::IdShape::Grouped => "grouped",
        },
        verbs,
        roles: project_roles(descriptor),
        reference_visibility: descriptor
            .reference_visibility
            .map(|visibility| visibility.as_str().to_string()),
    }
}

fn project_status(status: &StatusModel) -> (Vec<String>, Option<String>) {
    match status {
        StatusModel::TaskConfigurable { statuses, default } => {
            (statuses.clone(), Some(default.clone()))
        }
        StatusModel::Fixed { states } => (
            states.iter().map(|state| (*state).to_string()).collect(),
            None,
        ),
        StatusModel::DocumentMain { custom, default } => {
            let mut statuses = vec!["active".to_string(), "archived".to_string()];
            statuses.extend(custom.iter().cloned());
            (statuses, Some(default.clone()))
        }
        StatusModel::HtmlMetadata { states } => (
            states.iter().map(|state| (*state).to_string()).collect(),
            None,
        ),
    }
}

fn project_fields(fields: &[FieldSpec]) -> Vec<FieldView> {
    fields
        .iter()
        .map(|field| {
            let verbs: Vec<&str> = field.verbs.iter().map(|verb| verb.as_str()).collect();
            // A field without an explicit option derives its long name from
            // the key (ADR-036 decision 15).
            let option = field
                .option
                .clone()
                .unwrap_or_else(|| papyrus::sanitize::kebab_slug(&field.key));
            FieldView::new(
                field.heading.as_deref(),
                field.frontmatter_key.as_deref(),
                &option,
                field.kind.as_str(),
                field.required,
                &verbs,
            )
        })
        .collect()
}

fn project_roles(descriptor: &TypeDescriptor) -> Vec<RoleView> {
    match descriptor.document_model {
        DocumentModel::GroupedMarkdown => vec![
            RoleView::new(
                "group",
                &project_fields(
                    descriptor
                        .group
                        .as_ref()
                        .and_then(|role| role.fields.as_deref())
                        .unwrap_or(&[]),
                ),
            ),
            RoleView::new(
                "member",
                &project_fields(
                    descriptor
                        .member
                        .as_ref()
                        .and_then(|role| role.fields.as_deref())
                        .unwrap_or(&[]),
                ),
            ),
        ],
        DocumentModel::SingleMarkdown | DocumentModel::SingleHtmlMockup => {
            vec![RoleView::single(&project_fields(&descriptor.fields))]
        }
    }
}

/// A field tuple with its resolved long option name.
struct EField {
    heading: Option<&'static str>,
    frontmatter_key: Option<&'static str>,
    option: &'static str,
    kind: &'static str,
    required: bool,
    verbs: &'static [&'static str],
}

impl EField {
    fn view(&self) -> FieldView {
        FieldView::new(
            self.heading,
            self.frontmatter_key,
            self.option,
            self.kind,
            self.required,
            self.verbs,
        )
    }
}

type ERoles = &'static [(&'static str, &'static [EField])];

struct SeedOracle {
    model: &'static str,
    reference_visibility: Option<&'static str>,
    roles: ERoles,
}

const REQ_FIELDS: &[EField] = &[
    EField {
        heading: Some("## User Story"),
        frontmatter_key: None,
        option: "story",
        kind: "text",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
    EField {
        heading: Some("## Description"),
        frontmatter_key: None,
        option: "description",
        kind: "multiline",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
];

const GLOSSARY_FIELDS: &[EField] = &[EField {
    heading: Some("## Definition"),
    frontmatter_key: None,
    option: "definition",
    kind: "multiline",
    required: true,
    verbs: CREATE_AND_EDIT,
}];

const RULE_FIELDS: &[EField] = &[
    EField {
        heading: Some("## Description"),
        frontmatter_key: None,
        option: "description",
        kind: "multiline",
        required: true,
        verbs: CREATE_AND_EDIT,
    },
    EField {
        heading: Some("## Rationale"),
        frontmatter_key: None,
        option: "rationale",
        kind: "text",
        required: true,
        verbs: CREATE_ONLY,
    },
    EField {
        heading: Some("## Applicability"),
        frontmatter_key: None,
        option: "applicability",
        kind: "text",
        required: true,
        verbs: CREATE_ONLY,
    },
    EField {
        heading: Some("## Requirements"),
        frontmatter_key: None,
        option: "requirement",
        kind: "list",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
];

const UI_SPEC_FIELDS: &[EField] = &[
    EField {
        heading: Some("## View"),
        frontmatter_key: None,
        option: "view",
        kind: "text",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
    EField {
        heading: Some("## Description"),
        frontmatter_key: None,
        option: "description",
        kind: "multiline",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
];

const AUDIT_FIELDS: &[EField] = &[
    EField {
        heading: Some("## Audit Type"),
        frontmatter_key: Some("type"),
        option: "audit-type",
        kind: "text",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
    EField {
        heading: Some("## Content"),
        frontmatter_key: None,
        option: "content",
        kind: "multiline",
        required: true,
        verbs: CREATE_AND_EDIT,
    },
];

const TECH_SPEC_FIELDS: &[EField] = &[EField {
    heading: Some("## Description"),
    frontmatter_key: None,
    option: "description",
    kind: "multiline",
    required: false,
    verbs: CREATE_AND_EDIT,
}];

const DESCRIPTION_APPLICABILITY_FIELDS: &[EField] = &[
    EField {
        heading: Some("## Description"),
        frontmatter_key: None,
        option: "description",
        kind: "multiline",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
    EField {
        heading: Some("## Applicability"),
        frontmatter_key: None,
        option: "applicability",
        kind: "text",
        required: false,
        verbs: CREATE_AND_EDIT,
    },
];

const USER_DEFINED_FIELDS: &[EField] = &[EField {
    heading: Some("## Description"),
    frontmatter_key: None,
    option: "description",
    kind: "multiline",
    required: false,
    verbs: CREATE_AND_EDIT,
}];

fn seed_oracle(slug: &str) -> Option<SeedOracle> {
    let (model, reference_visibility, roles): (&'static str, Option<&'static str>, ERoles) =
        match slug {
            "req" | "nfr" => (
                "grouped-markdown",
                None,
                &[("group", REQ_FIELDS), ("member", REQ_FIELDS)],
            ),
            "glossary" => (
                "single-markdown",
                Some("visible"),
                &[("single", GLOSSARY_FIELDS)],
            ),
            "rule" => (
                "single-markdown",
                Some("visible"),
                &[("single", RULE_FIELDS)],
            ),
            "ui-spec" => (
                "single-markdown",
                Some("visible"),
                &[("single", UI_SPEC_FIELDS)],
            ),
            "ui-mockup" => ("single-html-mockup", Some("visible"), &[("single", &[])]),
            "audit" => (
                "single-markdown",
                Some("visible"),
                &[("single", AUDIT_FIELDS)],
            ),
            "tech-spec" => (
                "single-markdown",
                Some("visible"),
                &[("single", TECH_SPEC_FIELDS)],
            ),
            "use-case" | "test-case" | "test-report" => (
                "single-markdown",
                Some("visible"),
                &[("single", DESCRIPTION_APPLICABILITY_FIELDS)],
            ),
            _ => return None,
        };
    Some(SeedOracle {
        model,
        reference_visibility,
        roles,
    })
}

fn seed_roles(roles: &[(&'static str, &'static [EField])]) -> Vec<RoleView> {
    roles
        .iter()
        .map(|(name, fields)| {
            RoleView::new(name, &fields.iter().map(EField::view).collect::<Vec<_>>())
        })
        .collect()
}

fn user_defined_roles() -> Vec<RoleView> {
    vec![RoleView::single(
        &USER_DEFINED_FIELDS
            .iter()
            .map(EField::view)
            .collect::<Vec<_>>(),
    )]
}

/// Independent fixture facts for the fields under the M-6 oracle.
///
/// These values come from the raw fixture declaration plus the literal seed
/// facts below, never from the effective `Config` entry or the descriptor, so a
/// fill or replace bug cannot move both sides of the comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FactEntry {
    prefix: String,
    directory: String,
    enabled: bool,
    statuses: Option<Vec<String>>,
    default_status: Option<String>,
}

impl FactEntry {
    fn new(
        prefix: &str,
        directory: &str,
        enabled: bool,
        statuses: Option<&[&str]>,
        default_status: Option<&str>,
    ) -> Self {
        Self {
            prefix: prefix.to_string(),
            directory: directory.to_string(),
            enabled,
            statuses: statuses.map(|values| values.iter().map(|v| (*v).to_string()).collect()),
            default_status: default_status.map(str::to_string),
        }
    }
}

const SEED_SLUGS: &[&str] = &[
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

const ACTIVE_ARCHIVED: &[&str] = &["active", "archived"];

fn seed_facts(slug: &str) -> Option<FactEntry> {
    let facts = match slug {
        "req" => FactEntry::new(
            "REQ",
            "requirements/business/",
            true,
            Some(ACTIVE_ARCHIVED),
            None,
        ),
        "nfr" => FactEntry::new(
            "NFR",
            "requirements/non-functional/",
            true,
            Some(ACTIVE_ARCHIVED),
            None,
        ),
        "glossary" => FactEntry::new("GLO", "glossary/", true, Some(ACTIVE_ARCHIVED), None),
        "rule" => FactEntry::new("RUL", "business-rules/", true, Some(ACTIVE_ARCHIVED), None),
        "ui-spec" => FactEntry::new(
            "UI",
            "specifications/ui/",
            true,
            Some(ACTIVE_ARCHIVED),
            None,
        ),
        "ui-mockup" => FactEntry::new("MOCK", "specifications/ui/mockups/", true, None, None),
        "audit" => FactEntry::new("AUD", "audit-records/", true, Some(ACTIVE_ARCHIVED), None),
        "tech-spec" => FactEntry::new("TECH", "technical/", true, Some(ACTIVE_ARCHIVED), None),
        "use-case" => FactEntry::new("UC", "use-cases/", true, Some(ACTIVE_ARCHIVED), None),
        "test-case" => FactEntry::new("TC", "test-cases/", true, Some(ACTIVE_ARCHIVED), None),
        "test-report" => FactEntry::new("TR", "test-reports/", true, Some(ACTIVE_ARCHIVED), None),
        _ => return None,
    };
    Some(facts)
}

fn user_facts() -> FactEntry {
    FactEntry::new("", "", true, Some(ACTIVE_ARCHIVED), None)
}

fn fixture_facts(path: &Path) -> BTreeMap<String, FactEntry> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let value: yaml_serde::Value =
        yaml_serde::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut facts: BTreeMap<String, FactEntry> = SEED_SLUGS
        .iter()
        .map(|slug| ((*slug).to_string(), seed_facts(slug).expect("seed facts")))
        .collect();
    let Some(yaml_serde::Value::Mapping(entries)) = yaml_get(&value, "document_types") else {
        return facts;
    };
    for (key, declared) in entries {
        let yaml_serde::Value::String(slug) = key else {
            continue;
        };
        if !matches!(declared, yaml_serde::Value::Mapping(_)) {
            continue;
        }
        let mut entry = seed_facts(slug).unwrap_or_else(user_facts);
        if let Some(yaml_serde::Value::String(prefix)) = yaml_get(declared, "prefix") {
            entry.prefix = prefix.clone();
        }
        if let Some(yaml_serde::Value::String(directory)) = yaml_get(declared, "directory") {
            entry.directory = directory.clone();
        }
        if let Some(yaml_serde::Value::Bool(enabled)) = yaml_get(declared, "enabled") {
            entry.enabled = *enabled;
        }
        if let Some(yaml_serde::Value::Sequence(statuses)) = yaml_get(declared, "statuses") {
            let values: Vec<String> = statuses
                .iter()
                .filter_map(|status| match status {
                    yaml_serde::Value::String(status) => Some(status.clone()),
                    _ => None,
                })
                .collect();
            entry.statuses = Some(values);
        }
        if let Some(yaml_serde::Value::String(default_status)) =
            yaml_get(declared, "default_status")
        {
            entry.default_status = Some(default_status.clone());
        }
        facts.insert(slug.clone(), entry);
    }
    facts
}

fn expected_document(slug: &str, facts: &FactEntry) -> Projection {
    let seed = seed_oracle(slug);
    let (model, roles, reference_visibility) = match &seed {
        Some(seed) => (
            seed.model,
            seed_roles(seed.roles),
            seed.reference_visibility.map(str::to_string),
        ),
        // The documented delta: a user-defined type now resolves reference
        // visibility to `Visible` instead of the former generic `Hidden`.
        None => (
            "single-markdown",
            user_defined_roles(),
            Some("visible".to_string()),
        ),
    };
    let (statuses, default_status) = if model == "single-html-mockup" {
        (
            vec![
                "pending".to_string(),
                "approved".to_string(),
                "archived".to_string(),
            ],
            None,
        )
    } else {
        let declared = facts
            .statuses
            .clone()
            .unwrap_or_else(|| vec!["active".to_string(), "archived".to_string()]);
        let mut statuses = vec!["active".to_string(), "archived".to_string()];
        statuses.extend(
            declared
                .into_iter()
                .filter(|status| status != "active" && status != "archived"),
        );
        (
            statuses,
            Some(
                facts
                    .default_status
                    .clone()
                    .unwrap_or_else(|| "active".to_string()),
            ),
        )
    };
    Projection {
        prefix: facts.prefix.clone(),
        directory_key: "doc_directory".to_string(),
        directory: facts.directory.clone(),
        enabled: facts.enabled,
        statuses,
        default_status,
        model,
        id_shape: if model == "grouped-markdown" {
            "grouped"
        } else {
            "plain"
        },
        verbs: model_verbs(model),
        roles,
        reference_visibility,
    }
}

fn model_verbs(model: &str) -> Vec<String> {
    let mut verbs: Vec<String> = ["create", "list", "view", "edit", "archive", "delete"]
        .iter()
        .map(|verb| (*verb).to_string())
        .collect();
    match model {
        "grouped-markdown" => {
            verbs.push("status".to_string());
            verbs.push("create-group".to_string());
        }
        "single-html-mockup" => verbs.push("supersede".to_string()),
        _ => verbs.push("status".to_string()),
    }
    verbs.sort();
    verbs
}

fn expected_backlog(slug: &str, cfg: &Config) -> Projection {
    let (prefix, directory, statuses, default_status, verbs): (
        String,
        &str,
        Vec<String>,
        Option<String>,
        &[&str],
    ) = match slug {
        "task" => (
            cfg.task_prefix.clone(),
            "tasks",
            cfg.statuses.clone(),
            Some(cfg.default_status.clone()),
            &[
                "create", "list", "view", "edit", "archive", "restore", "delete",
            ],
        ),
        "milestone" => (
            cfg.milestone_prefix.clone(),
            "milestones",
            vec![
                "open".to_string(),
                "closed".to_string(),
                "archived".to_string(),
            ],
            None,
            &[
                "create", "list", "view", "edit", "close", "archive", "delete",
            ],
        ),
        "release" => (
            cfg.release_prefix.clone(),
            "releases",
            vec!["planned".to_string(), "released".to_string()],
            None,
            &["create", "list", "view", "edit", "close"],
        ),
        other => panic!("unexpected backlog descriptor: {other}"),
    };
    let mut verbs: Vec<String> = verbs.iter().map(|verb| (*verb).to_string()).collect();
    verbs.sort();
    Projection {
        prefix,
        directory_key: "backlog_directory".to_string(),
        directory: directory.to_string(),
        enabled: true,
        statuses,
        default_status,
        model: "single-markdown",
        id_shape: "plain",
        verbs,
        roles: vec![RoleView::single(&[])],
        reference_visibility: None,
    }
}

fn expected_decision(entry: &papyrus::config::TypeEntry) -> Projection {
    Projection {
        prefix: entry.prefix.clone(),
        directory_key: "decision_directory".to_string(),
        directory: String::new(),
        enabled: entry.enabled,
        statuses: [
            "proposed",
            "accepted",
            "rejected",
            "deprecated",
            "superseded",
        ]
        .iter()
        .map(|state| (*state).to_string())
        .collect(),
        default_status: None,
        model: "single-markdown",
        id_shape: "plain",
        verbs: {
            let mut verbs: Vec<String> = ["create", "list", "view", "edit"]
                .iter()
                .map(|verb| (*verb).to_string())
                .collect();
            verbs.sort();
            verbs
        },
        roles: vec![RoleView::single(&[])],
        reference_visibility: None,
    }
}

fn expected_projection(
    cfg: &Config,
    descriptor: &TypeDescriptor,
    facts: &BTreeMap<String, FactEntry>,
) -> Projection {
    match descriptor.category {
        ArtifactCategory::Backlog => expected_backlog(&descriptor.slug, cfg),
        ArtifactCategory::Decision => expected_decision(&cfg.decision_types[&descriptor.slug]),
        ArtifactCategory::Document => {
            let entry = facts
                .get(&descriptor.slug)
                .unwrap_or_else(|| panic!("missing fixture facts for {}", descriptor.slug));
            expected_document(&descriptor.slug, entry)
        }
    }
}

fn projection_mismatches(
    cfg: &Config,
    registry: &ArtifactRegistry,
    facts: &BTreeMap<String, FactEntry>,
    label: &str,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    let expected: Vec<&str> = facts.keys().map(String::as_str).collect();
    let projected: Vec<&str> = registry
        .in_category(ArtifactCategory::Document)
        .map(|descriptor| descriptor.slug.as_str())
        .collect();
    if expected != projected {
        mismatches.push(format!(
            "{label}: every effective document type must project exactly once, in order; expected {expected:?}, got {projected:?}"
        ));
    }
    for descriptor in registry.descriptors() {
        let actual = project(descriptor);
        let expected = expected_projection(cfg, descriptor, facts);
        if actual != expected {
            mismatches.push(format!(
                "{label}: descriptor projection drifted for {}: expected {expected:?}, got {actual:?}",
                descriptor.slug
            ));
        }
    }
    mismatches
}

fn assert_registry_projects(
    cfg: &Config,
    registry: &ArtifactRegistry,
    facts: &BTreeMap<String, FactEntry>,
    label: &str,
) {
    let mismatches = projection_mismatches(cfg, registry, facts, label);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

const CONFIG_FIXTURES: &[&str] = &[
    "papyrus.config.yml",
    "tests/fixtures/cli-help/papyrus.config.yml",
    "tests/fixtures/characterization/corpus/project/papyrus.config.yml",
    "tests/fixtures/characterization/corpus/project-empty/papyrus.config.yml",
    "tests/fixtures/characterization/lists/project/papyrus.config.yml",
    "tests/fixtures/characterization/lists/project-empty/papyrus.config.yml",
];

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn snapshot_tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
            .map(|entry| entry.expect("directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let bytes =
                    fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                files.push((path, bytes));
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

#[test]
fn every_repository_configuration_loads_unedited_and_projects_identically() {
    for relative in CONFIG_FIXTURES {
        let path = manifest().join(relative);
        let project_dir = path.parent().expect("config has a parent");
        let before = fs::read(&path).expect("config bytes readable");
        let cfg = load_effective_config(&path, project_dir)
            .unwrap_or_else(|error| panic!("{relative} must load unedited: {error}"));
        let registry = ArtifactRegistry::build(&cfg)
            .unwrap_or_else(|error| panic!("{relative} must build a registry: {error}"));
        let facts = fixture_facts(&path);
        assert_registry_projects(&cfg, &registry, &facts, relative);
        let after = fs::read(&path).expect("config bytes readable after load");
        assert_eq!(before, after, "{relative} was modified by the load");
    }
}

#[test]
fn corpus_configuration_and_records_are_byte_identical_after_the_load() {
    for fixture in ["project", "project-empty"] {
        let root = manifest()
            .join("tests/fixtures/characterization/corpus")
            .join(fixture);
        let path = root.join("papyrus.config.yml");
        let before = snapshot_tree(&root);
        let cfg = load_effective_config(&path, &root).expect("corpus configuration loads");
        ArtifactRegistry::build(&cfg).expect("corpus registry builds");
        let after = snapshot_tree(&root);
        assert_eq!(
            before, after,
            "corpus/{fixture} configuration and records must stay byte-identical"
        );
    }
}

#[test]
fn only_user_defined_reference_visibility_is_a_projected_delta() {
    let path = manifest().join("tests/fixtures/cli-help/papyrus.config.yml");
    let project_dir = path.parent().expect("config has a parent");
    let cfg = load_effective_config(&path, project_dir).expect("cli-help configuration loads");
    let registry = ArtifactRegistry::build(&cfg).expect("cli-help registry builds");

    let mut user_defined = 0usize;
    for descriptor in registry.in_category(ArtifactCategory::Document) {
        if seed_oracle(&descriptor.slug).is_some() {
            continue;
        }
        user_defined += 1;
        assert_eq!(
            descriptor.reference_visibility.map(|v| v.as_str()),
            Some("visible"),
            "{}: the only projected delta is user-defined reference visibility \
             moving from the former Generic hidden rule to Visible (TC-185)",
            descriptor.slug
        );
    }
    assert!(
        user_defined >= 2,
        "the cli-help configuration must exercise user-defined slugs (note, legacy)"
    );

    // The pre-change seed visibility is unchanged.
    for (slug, expected) in [
        ("req", None),
        ("nfr", None),
        ("glossary", Some("visible")),
        ("rule", Some("visible")),
        ("ui-mockup", Some("visible")),
    ] {
        assert_eq!(
            registry
                .descriptor(slug)
                .expect(slug)
                .reference_visibility
                .map(|v| v.as_str()),
            expected,
            "{slug}: seed reference visibility must not move"
        );
    }
}

// ---------------------------------------------------------------------------
// cli-help goldens: root type order, verb lists, and long-option order.
// ---------------------------------------------------------------------------

fn read_golden(name: &str) -> String {
    let path = manifest().join(format!("tests/golden/cli-help/{name}.json"));
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let value: serde_json::Value = serde_json::from_str(&text).expect("golden is JSON");
    assert_eq!(value["exit"], serde_json::json!(0), "{name} golden exit");
    value["stdout"]
        .as_str()
        .expect("golden stdout is a string")
        .to_string()
}

fn commands_in(stdout: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_commands = false;
    for line in stdout.lines() {
        if line.trim() == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line.trim().is_empty() {
            break;
        }
        if let Some(word) = line.split_whitespace().next() {
            commands.push(word.to_string());
        }
    }
    commands
}

fn long_options_in(stdout: &str) -> Vec<String> {
    let mut options = Vec::new();
    let mut in_options = false;
    for line in stdout.lines() {
        if line.trim() == "Options:" {
            in_options = true;
            continue;
        }
        if !in_options {
            continue;
        }
        let trimmed = line.trim_start();
        if !trimmed.starts_with('-') {
            continue;
        }
        for token in trimmed.split_whitespace() {
            let token = token.trim_end_matches(',');
            if let Some(long) = token.strip_prefix("--") {
                let name = long.split(['=', '[', '<']).next().unwrap_or("");
                if !name.is_empty() {
                    options.push(name.to_string());
                }
            }
        }
    }
    options
}

const GLOBAL_OPTIONS: &[&str] = &["json", "dir", "config", "no-color", "help"];

fn type_long_options(stdout: &str) -> Vec<String> {
    long_options_in(stdout)
        .into_iter()
        .filter(|name| !GLOBAL_OPTIONS.contains(&name.as_str()))
        .collect()
}

fn descriptor_field_options(descriptor: &TypeDescriptor, verb: Verb) -> Vec<String> {
    let mut options = Vec::new();
    let mut push = |fields: &[FieldSpec]| {
        for field in fields {
            let offered = field.verbs.iter().any(|field_verb| {
                matches!(
                    (field_verb, verb),
                    (&FieldVerb::Create, Verb::Create) | (&FieldVerb::Edit, Verb::Edit)
                )
            });
            if offered {
                options.push(
                    field
                        .option
                        .clone()
                        .unwrap_or_else(|| papyrus::sanitize::kebab_slug(&field.key)),
                );
            }
        }
    };
    push(&descriptor.fields);
    if let Some(group) = &descriptor.group {
        push(group.fields.as_deref().unwrap_or(&[]));
    }
    if let Some(member) = &descriptor.member {
        push(member.fields.as_deref().unwrap_or(&[]));
    }
    options
}

#[test]
fn cli_help_projection_matches_the_frozen_goldens() {
    let path = manifest().join("tests/fixtures/cli-help/papyrus.config.yml");
    let project_dir = path.parent().expect("config has a parent");
    let cfg = load_effective_config(&path, project_dir).expect("cli-help configuration loads");
    let registry = ArtifactRegistry::build(&cfg).expect("cli-help registry builds");

    let top = commands_in(&read_golden("top"));
    let mut expected_top: Vec<String> = [
        "init",
        "config",
        "check",
        "milestone",
        "release",
        "task",
        "decision",
        "import",
        "browser",
        "search",
    ]
    .iter()
    .map(|command| (*command).to_string())
    .collect();
    expected_top.extend(
        registry
            .enabled(ArtifactCategory::Document)
            .map(|descriptor| descriptor.slug.clone()),
    );
    expected_top.push("help".to_string());
    assert_eq!(
        top, expected_top,
        "root type order must match the cli-help top golden (disabled legacy excluded)"
    );

    for descriptor in registry.enabled(ArtifactCategory::Document) {
        let slug = descriptor.slug.as_str();
        let slug_help = read_golden(&format!("slug-{slug}"));
        let mut expected_verbs: Vec<String> = descriptor
            .ordered_verb_specs()
            .iter()
            .map(|spec| spec.verb.as_str().to_string())
            .collect();
        expected_verbs.push("help".to_string());
        assert_eq!(
            commands_in(&slug_help),
            expected_verbs,
            "{slug}: verb list drifted from the cli-help golden"
        );

        for spec in descriptor.ordered_verb_specs() {
            let name = format!("verb-{slug}-{}", spec.verb.as_str());
            let golden = type_long_options(&read_golden(&name));
            let spec_longs: Vec<String> = spec
                .args
                .iter()
                .filter_map(|arg| arg.long.as_deref())
                .map(str::to_string)
                .collect();
            // Field-sequence option order is generated and TASK-260
            // regenerates the goldens; every generated option must still be
            // present in the frozen help.
            for option in &spec_longs {
                assert!(
                    golden.contains(option),
                    "{name}: generated option --{option} missing from {golden:?}"
                );
            }
            // Every projected field option for the verb must survive in the
            // golden. Field-sequence order is not the CLI argument order, so
            // presence is the assertion; argument order is covered above.
            for option in descriptor_field_options(descriptor, spec.verb) {
                assert!(
                    golden.contains(&option),
                    "{name}: projected field option --{option} missing from {golden:?}"
                );
            }
        }
    }
}

#[test]
fn equivalence_check_fails_when_one_effective_field_is_mutated() {
    let path = manifest().join("tests/fixtures/cli-help/papyrus.config.yml");
    let project_dir = path.parent().expect("config has a parent");
    let facts = fixture_facts(&path);
    let cfg = load_effective_config(&path, project_dir).expect("cli-help configuration loads");
    let registry = ArtifactRegistry::build(&cfg).expect("cli-help registry builds");
    assert!(
        projection_mismatches(&cfg, &registry, &facts, "baseline").is_empty(),
        "the unmutated fixture must satisfy the independent oracle"
    );

    let mut prefix_drift = cfg.clone();
    prefix_drift
        .document_types
        .get_mut("glossary")
        .expect("glossary")
        .prefix = "WRONG".to_string();
    let drifted = ArtifactRegistry::build(&prefix_drift).expect("drifted registry builds");
    assert!(
        !projection_mismatches(&prefix_drift, &drifted, &facts, "prefix").is_empty(),
        "a mutated prefix must fail the independent oracle"
    );

    let mut status_drift = cfg;
    status_drift
        .document_types
        .get_mut("glossary")
        .expect("glossary")
        .statuses = Some(vec![
        "active".to_string(),
        "archived".to_string(),
        "drifted".to_string(),
    ]);
    let drifted = ArtifactRegistry::build(&status_drift).expect("drifted registry builds");
    assert!(
        !projection_mismatches(&status_drift, &drifted, &facts, "statuses").is_empty(),
        "a mutated status list must fail the independent oracle"
    );
}

#[test]
fn list_key_classifier_keeps_field_sequences_scalar() {
    // Emitter guard: the new nested field sequences are structural, never
    // comma-separated list-valued configuration keys.
    for key in [
        "document_types.deploy.fields",
        "document_types.req.group.fields",
        "document_types.req.member.fields",
    ] {
        assert!(
            !is_list_valued_key(key),
            "{key} must not be treated as a list-valued key"
        );
    }

    // The projection's verb set is a set: a duplicate would change its size.
    let path = manifest().join("papyrus.config.yml");
    let project_dir = path.parent().expect("config has a parent");
    let cfg = load_effective_config(&path, project_dir).expect("live configuration loads");
    let registry = ArtifactRegistry::build(&cfg).expect("live registry builds");
    for descriptor in registry.descriptors() {
        for (index, verb) in descriptor.verbs.iter().enumerate() {
            assert!(
                !descriptor.verbs[..index].contains(verb),
                "{} declares a duplicate verb",
                descriptor.slug
            );
        }
    }
}
