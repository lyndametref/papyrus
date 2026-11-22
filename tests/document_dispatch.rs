//! TASK-119: every document slug enters `document::run`.
//!
//! A config-defined generic slug and a built-in typed slug are invoked through
//! the same entry point with the same `DocumentInvocation` contract.

use papyrus::config::{Config, DocTypeEntry};
use papyrus::context::Context;
use papyrus::document::{self, DocumentArgs, DocumentInvocation, DocumentVerb};
use papyrus::registry::ArtifactRegistry;
use std::fs;
use std::path::{Path, PathBuf};

fn project(name: &str) -> (PathBuf, Context, Config, ArtifactRegistry) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("document-dispatch-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let mut cfg = Config::default();
    cfg.document_types.insert(
        "note".to_string(),
        DocTypeEntry {
            prefix: "NOTE".to_string(),
            directory: "notes/".to_string(),
            enabled: true,
            statuses: None,
            default_status: None,
            ..Default::default()
        },
    );
    fs::write(
        dir.join("papyrus.config.yml"),
        yaml_serde::to_string(&cfg).unwrap(),
    )
    .unwrap();
    let ctx = Context {
        project_dir: dir.clone(),
        config_path: dir.join("papyrus.config.yml"),
        json: false,
        no_color: true,
    };
    let registry = ArtifactRegistry::build(&cfg).unwrap();
    (dir, ctx, cfg, registry)
}

fn invocation(slug: &str, verb: DocumentVerb, args: DocumentArgs) -> DocumentInvocation {
    let mut invocation = DocumentInvocation::new(slug, verb);
    invocation.id = args.get("id").map(str::to_string);
    invocation.title = args
        .get("title")
        .or_else(|| args.get("title_flag"))
        .map(str::to_string);
    invocation.description = args.get("description").map(str::to_string);
    invocation.labels = args.get_all("label").to_vec();
    invocation.references = args.get_all("reference").to_vec();
    invocation.editor = args.flag("editor");
    invocation.status_value = args.get("value").map(str::to_string);
    invocation.force = args.flag("force");
    invocation.args = args;
    invocation
}

fn create(
    cfg: &Config,
    registry: &ArtifactRegistry,
    ctx: &Context,
    slug: &str,
    args: DocumentArgs,
) -> String {
    document::run(
        ctx,
        cfg,
        registry,
        invocation(slug, DocumentVerb::Create, args),
    )
    .unwrap()
    .text
}

fn list(cfg: &Config, registry: &ArtifactRegistry, ctx: &Context, slug: &str) -> String {
    document::run(
        ctx,
        cfg,
        registry,
        DocumentInvocation::new(slug, DocumentVerb::List),
    )
    .unwrap()
    .text
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn generic_and_typed_slugs_share_the_document_run_entry() {
    let (dir, ctx, cfg, registry) = project("shared");

    let mut generic_args = DocumentArgs::default();
    generic_args.insert("title", vec!["Generic note".to_string()]);
    let generic_id = create(&cfg, &registry, &ctx, "note", generic_args);

    let mut typed_args = DocumentArgs::default();
    typed_args.insert("term", vec!["Alpha".to_string()]);
    typed_args.insert("definition", vec!["First letter.".to_string()]);
    let typed_id = create(&cfg, &registry, &ctx, "glossary", typed_args);

    assert_eq!(generic_id, "NOTE-001");
    assert_eq!(typed_id, "GLO-001");
    assert!(dir.join("docs/notes/NOTE-001.md").exists());
    assert!(dir.join("docs/glossary/GLO-001.md").exists());

    let generic_list = list(&cfg, &registry, &ctx, "note");
    let typed_list = list(&cfg, &registry, &ctx, "glossary");
    assert!(generic_list.contains("NOTE-001"));
    assert!(typed_list.contains("GLO-001"));

    cleanup(&dir);
}

#[test]
fn typed_and_generic_verbs_delegate_through_the_same_invocation() {
    let (dir, ctx, cfg, registry) = project("verbs");

    let mut typed_args = DocumentArgs::default();
    typed_args.insert("term", vec!["Beta".to_string()]);
    typed_args.insert("definition", vec!["Second letter.".to_string()]);
    let typed_id = create(&cfg, &registry, &ctx, "glossary", typed_args);

    let mut view_args = DocumentArgs::default();
    view_args.insert("id", vec![typed_id.clone()]);
    let view = invocation("glossary", DocumentVerb::View, view_args);
    let viewed = document::run(&ctx, &cfg, &registry, view).unwrap();
    assert!(viewed.text.contains("Beta"));

    let mut archive_args = DocumentArgs::default();
    archive_args.insert("id", vec![typed_id.clone()]);
    let archive = invocation("glossary", DocumentVerb::Archive, archive_args);
    document::run(&ctx, &cfg, &registry, archive).unwrap();
    assert!(!dir.join("docs/glossary/GLO-001.md").exists());
    assert!(dir.join("docs/glossary/archived/GLO-001.md").exists());

    let mut generic_args = DocumentArgs::default();
    generic_args.insert("title", vec!["Deploy".to_string()]);
    let generic_id = create(&cfg, &registry, &ctx, "note", generic_args);
    let mut status_args = DocumentArgs::default();
    status_args.insert("id", vec![generic_id.clone()]);
    status_args.insert("value", vec!["active".to_string()]);
    let status = invocation("note", DocumentVerb::Status, status_args);
    document::run(&ctx, &cfg, &registry, status).unwrap();

    cleanup(&dir);
}

fn project_with(
    name: &str,
    entry: Option<(&str, DocTypeEntry)>,
) -> (PathBuf, Context, Config, ArtifactRegistry) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("document-engine-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let mut cfg = Config::default();
    if let Some((slug, entry)) = entry {
        cfg.document_types.insert(slug.to_string(), entry);
    }
    fs::write(
        dir.join("papyrus.config.yml"),
        yaml_serde::to_string(&cfg).unwrap(),
    )
    .unwrap();
    let ctx = Context {
        project_dir: dir.clone(),
        config_path: dir.join("papyrus.config.yml"),
        json: false,
        no_color: true,
    };
    let registry = ArtifactRegistry::build(&cfg).unwrap();
    (dir, ctx, cfg, registry)
}

fn args(pairs: &[(&str, &[&str])]) -> DocumentArgs {
    let mut args = DocumentArgs::default();
    for (key, values) in pairs {
        args.insert(
            (*key).to_string(),
            values.iter().map(|v| (*v).to_string()).collect(),
        );
    }
    args
}

#[test]
fn audit_create_applies_the_seed_default_and_edit_leaves_an_absent_field_unchanged() {
    let (dir, ctx, cfg, registry) = project_with("audit-default", None);

    let id = create(
        &cfg,
        &registry,
        &ctx,
        "audit",
        args(&[("title", &["Defaulted"]), ("content", &["Body"])]),
    );
    let path = dir.join("docs/audit-records/AUD-001.md");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("type: security\n"), "{content}");
    assert!(content.contains("## Audit Type\nsecurity\n"), "{content}");

    let mut edit_args = args(&[("id", &[&id]), ("title", &["Renamed"])]);
    document::run(
        &ctx,
        &cfg,
        &registry,
        invocation("audit", DocumentVerb::Edit, edit_args),
    )
    .unwrap();
    let unchanged = fs::read_to_string(&path).unwrap();
    assert!(unchanged.contains("type: security\n"), "{unchanged}");
    assert!(
        unchanged.contains("## Audit Type\nsecurity\n"),
        "{unchanged}"
    );

    edit_args = args(&[("id", &[&id]), ("audit_type", &["quality"])]);
    document::run(
        &ctx,
        &cfg,
        &registry,
        invocation("audit", DocumentVerb::Edit, edit_args),
    )
    .unwrap();
    let changed = fs::read_to_string(&path).unwrap();
    assert!(changed.contains("type: quality\n"), "{changed}");
    assert!(changed.contains("## Audit Type\nquality\n"), "{changed}");
    assert!(!changed.contains("security"), "{changed}");

    cleanup(&dir);
}

#[test]
fn create_omitting_required_fields_reports_every_key_and_writes_nothing() {
    let (dir, ctx, cfg, registry) = project_with("required", None);

    let out = document::run(
        &ctx,
        &cfg,
        &registry,
        invocation(
            "glossary",
            DocumentVerb::Create,
            args(&[("term", &["Term"])]),
        ),
    )
    .err()
    .expect("a missing required field is a validation error");
    let message = out.to_string();
    assert!(message.contains("definition"), "{message}");
    assert!(!dir.join("docs/glossary/GLO-001.md").exists());

    let out = document::run(
        &ctx,
        &cfg,
        &registry,
        invocation("rule", DocumentVerb::Create, args(&[("title", &["Rule"])])),
    )
    .err()
    .expect("missing required fields are one validation outcome");
    let message = out.to_string();
    for key in ["description", "rationale", "applicability"] {
        assert!(message.contains(key), "{message}");
    }
    assert!(!dir.join("docs/business-rules/RUL-001.md").exists());

    cleanup(&dir);
}

#[test]
fn a_required_field_cannot_be_cleared_but_an_optional_field_can() {
    let (dir, ctx, cfg, registry) = project_with("clearing", None);

    let glossary_id = create(
        &cfg,
        &registry,
        &ctx,
        "glossary",
        args(&[("term", &["Term"]), ("definition", &["Definition"])]),
    );
    let glossary_path = dir.join("docs/glossary/GLO-001.md");
    let before = fs::read_to_string(&glossary_path).unwrap();
    let out = document::run(
        &ctx,
        &cfg,
        &registry,
        invocation(
            "glossary",
            DocumentVerb::Edit,
            args(&[("id", &[&glossary_id]), ("definition", &[""])]),
        ),
    )
    .err()
    .expect("a required field cannot be cleared");
    assert!(out.to_string().contains("definition"), "{out}");
    assert_eq!(fs::read_to_string(&glossary_path).unwrap(), before);

    let use_case_id = create(
        &cfg,
        &registry,
        &ctx,
        "use-case",
        args(&[
            ("title", &["Case"]),
            ("description", &["Description"]),
            ("applicability", &["When browsing"]),
        ]),
    );
    document::run(
        &ctx,
        &cfg,
        &registry,
        invocation(
            "use-case",
            DocumentVerb::Edit,
            args(&[("id", &[&use_case_id]), ("applicability", &[""])]),
        ),
    )
    .unwrap();
    let content = fs::read_to_string(dir.join("docs/use-cases/UC-001.md")).unwrap();
    assert!(!content.contains("## Applicability"), "{content}");

    cleanup(&dir);
}

#[test]
fn command_options_follow_the_role_field_sequence() {
    let (_dir, _ctx, _cfg, registry) = project_with("options", None);
    let descriptor = registry.descriptor("rule").unwrap();
    let create = descriptor
        .verb_spec(papyrus::registry::Verb::Create)
        .unwrap();
    let longs: Vec<&str> = create
        .args
        .iter()
        .filter_map(|arg| arg.long.as_deref())
        .collect();
    assert_eq!(
        longs,
        vec![
            "description",
            "rationale",
            "applicability",
            "requirement",
            "label",
            "reference"
        ]
    );
    let edit = descriptor.verb_spec(papyrus::registry::Verb::Edit).unwrap();
    assert!(
        edit.args
            .iter()
            .any(|arg| arg.long.as_deref() == Some("no-requirement"))
    );
}

#[test]
fn dual_storage_and_list_fields_write_sections_and_yaml_sequences() {
    use papyrus::config::{FieldKind, FieldVerb};
    let summary = papyrus::config::FieldSpec {
        key: "summary".to_string(),
        heading: Some("## Summary".to_string()),
        frontmatter_key: Some("summary".to_string()),
        option: None,
        label: Some("Summary".to_string()),
        kind: FieldKind::Text,
        required: false,
        verbs: vec![FieldVerb::Create, FieldVerb::Edit],
        help: None,
    };
    let tags = papyrus::config::FieldSpec {
        key: "tags".to_string(),
        heading: None,
        frontmatter_key: Some("tags".to_string()),
        option: None,
        label: Some("Tags".to_string()),
        kind: FieldKind::List,
        required: false,
        verbs: vec![FieldVerb::Create, FieldVerb::Edit],
        help: None,
    };
    let entry = DocTypeEntry {
        prefix: "CUS".to_string(),
        directory: "custom/".to_string(),
        enabled: true,
        fields: Some(vec![summary, tags]),
        ..Default::default()
    };
    let (dir, ctx, cfg, registry) = project_with("dual", Some(("custom", entry)));

    let id = create(
        &cfg,
        &registry,
        &ctx,
        "custom",
        args(&[
            ("title", &["Custom"]),
            ("summary", &["Dual stored"]),
            ("tags", &["single"]),
        ]),
    );
    let path = dir.join("docs/custom/CUS-001.md");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("summary: \"Dual stored\"\n"), "{content}");
    assert!(content.contains("## Summary\nDual stored\n"), "{content}");
    assert!(content.contains("tags:\n  - single\n"), "{content}");

    document::run(
        &ctx,
        &cfg,
        &registry,
        invocation(
            "custom",
            DocumentVerb::Edit,
            args(&[("id", &[&id]), ("no_tags", &[])]),
        ),
    )
    .unwrap();
    let cleared = fs::read_to_string(&path).unwrap();
    assert!(cleared.contains("tags: []\n"), "{cleared}");

    cleanup(&dir);
}
