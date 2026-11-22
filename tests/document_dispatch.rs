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
