//! TASK-121: two command trees and two registries in one process share no state.
//!
//! The descriptor cache and the leaked slug commands are gone, so building a
//! tree from registry B can never change the tree or the record descriptors of
//! registry A. The help output for one registry must be identical no matter
//! which other registry was built before it (ADR-0016 decision 1, ADR-0017
//! decision 10).

use papyrus::cli::tree;
use papyrus::config::{Config, DocTypeEntry};
use papyrus::document::DocumentVerb;
use papyrus::registry::ArtifactRegistry;
use std::fs;
use std::path::PathBuf;

fn entry(prefix: &str, directory: &str, enabled: bool) -> DocTypeEntry {
    DocTypeEntry {
        prefix: prefix.to_string(),
        directory: directory.to_string(),
        enabled,
        statuses: None,
        default_status: None,
        ..Default::default()
    }
}

/// Registry A: the default set plus the config-defined slug `note`.
fn config_a() -> Config {
    let mut cfg = Config::default();
    cfg.document_types
        .insert("note".to_string(), entry("NOTE", "notes/", true));
    cfg
}

/// Registry B: a different config-defined slug (`memo`) and `glossary` off.
fn config_b() -> Config {
    let mut cfg = Config::default();
    cfg.document_types
        .insert("memo".to_string(), entry("MEMO", "memos/", true));
    cfg.document_types
        .get_mut("glossary")
        .expect("default glossary entry")
        .enabled = false;
    cfg
}

fn help(registry: &ArtifactRegistry) -> String {
    let mut command = tree(Some(registry)).expect("tree builds");
    command.render_help().to_string()
}

fn subcommands(registry: &ArtifactRegistry) -> Vec<String> {
    tree(Some(registry))
        .expect("tree builds")
        .get_subcommands()
        .map(|command| command.get_name().to_string())
        .collect()
}

#[test]
fn help_for_one_registry_is_unchanged_by_a_second_registry() {
    let registry_a = ArtifactRegistry::build(&config_a()).expect("registry A");
    let before = help(&registry_a);

    // Build registry B and its tree between the two A builds.
    let registry_b = ArtifactRegistry::build(&config_b()).expect("registry B");
    let b_help = help(&registry_b);

    let after = help(&registry_a);
    assert_eq!(
        before, after,
        "building tree B must not change tree A help output"
    );
    assert_ne!(
        before, b_help,
        "the two registries expose different document sets"
    );
}

#[test]
fn each_tree_lists_only_its_own_document_slugs() {
    let registry_a = ArtifactRegistry::build(&config_a()).expect("registry A");
    let registry_b = ArtifactRegistry::build(&config_b()).expect("registry B");

    let a = subcommands(&registry_a);
    let b = subcommands(&registry_b);

    assert!(a.iter().any(|name| name == "note"), "{a:?}");
    assert!(!a.iter().any(|name| name == "memo"), "{a:?}");
    assert!(a.iter().any(|name| name == "glossary"), "{a:?}");

    assert!(b.iter().any(|name| name == "memo"), "{b:?}");
    assert!(!b.iter().any(|name| name == "note"), "{b:?}");
    assert!(
        !b.iter().any(|name| name == "glossary"),
        "a disabled type is absent from the tree: {b:?}"
    );
}

#[test]
fn config_defined_slug_still_parses_after_a_second_registry_was_built() {
    let registry_a = ArtifactRegistry::build(&config_a()).expect("registry A");
    let registry_b = ArtifactRegistry::build(&config_b()).expect("registry B");
    let _ = tree(Some(&registry_b)).expect("tree B builds");

    let command = tree(Some(&registry_a)).expect("tree A rebuilds");
    let matches = command
        .try_get_matches_from(["papyrus", "note", "create", "--title", "x"])
        .expect("note create parses");
    let (slug, sub) = matches.subcommand().expect("slug subcommand");
    assert_eq!(slug, "note");

    let invocation = tree::parse_document_invocation(sub, slug, true).expect("invocation");
    assert_eq!(invocation.slug, "note");
    assert_eq!(invocation.verb, DocumentVerb::Create);
    assert_eq!(invocation.title.as_deref(), Some("x"));
}

#[test]
fn record_descriptors_are_stable_and_registry_local() {
    let registry_a = ArtifactRegistry::build(&config_a()).expect("registry A");
    let registry_b = ArtifactRegistry::build(&config_b()).expect("registry B");

    let first = registry_a.record_descriptor("note").expect("A note record");
    let second = registry_a
        .record_descriptor("note")
        .expect("A note record again");
    assert_eq!(first, second, "repeated calls are field-equal");
    assert_eq!(first.doc_type_key.as_deref(), Some("note"));
    assert_eq!(
        first.configured_prefixes(&config_a()),
        vec!["NOTE".to_string()]
    );

    // Build B between the two A calls: no cross-registry state.
    assert!(registry_b.record_descriptor("note").is_none());
    assert!(
        registry_a.record_descriptor("memo").is_none(),
        "A must not see B's config-defined type"
    );
    let third = registry_a.record_descriptor("note").expect("A note record");
    assert_eq!(first, third, "registry A is not affected by registry B");
    assert_eq!(
        third.configured_prefixes(&config_a()),
        vec!["NOTE".to_string()]
    );

    let memo = registry_b.record_descriptor("memo").expect("B memo record");
    assert_eq!(memo.doc_type_key.as_deref(), Some("memo"));
    assert_eq!(
        memo.configured_prefixes(&config_b()),
        vec!["MEMO".to_string()]
    );
    assert_eq!(memo.label, "memo");
    assert_eq!(memo.sections, &["## Description"]);
    assert_eq!(memo.frontmatter_keys, &["status"]);
    assert!(
        memo.has_frontmatter_key("status"),
        "the status key is carried"
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("tree-rebuild");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("tmp project");
    let project = fs::canonicalize(&root).expect("canonical project");
    assert_eq!(
        memo.resolve_directory(&config_b(), &project)
            .expect("memo directory"),
        project.join("docs").join("memos"),
        "the configured directory is carried by the record descriptor"
    );
    let _ = fs::remove_dir_all(&root);

    // A disabled type keeps its descriptor (record I/O and migration read it)
    // but stays out of the command tree.
    assert!(registry_b.record_descriptor("glossary").is_some());
    assert_eq!(
        registry_b
            .record_descriptor("glossary")
            .expect("B glossary record")
            .configured_prefixes(&config_b()),
        vec!["GLO".to_string()]
    );
}

#[test]
fn registry_free_tree_exposes_the_full_check_surface() {
    let command = tree(None).expect("registry-free tree builds");
    assert!(
        command.find_subcommand("check").is_some(),
        "check is a built-in command"
    );
    let error = command
        .try_get_matches_from(["papyrus", "check", "--help"])
        .expect_err("--help renders the command help");
    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    let help = error.to_string();

    assert!(help.contains("Usage: papyrus check [OPTIONS]"), "{help}");
    for needle in [
        "--fix",
        "--force",
        "duplicate-id",
        "self-dependency",
        "dependency-cycle",
        "dangling-milestone",
        "dangling-release",
        "frontmatter-conflict",
        "config-invalid",
        "Nothing is written unless --fix is given",
        "an invalid configuration is reported as one config-invalid finding per",
        "Exit status: 0 when nothing was found; 4 when at least one finding was reported,",
        "1 on an I/O failure; 2 on a usage error",
    ] {
        assert!(help.contains(needle), "help is missing {needle:?}:\n{help}");
    }
}

#[test]
fn registry_tree_lists_check_once_and_never_as_a_document_type() {
    let registry = ArtifactRegistry::build(&Config::default()).expect("default registry");
    let command = tree(Some(&registry)).expect("tree builds");
    let names: Vec<&str> = command
        .get_subcommands()
        .map(|command| command.get_name())
        .collect();
    assert_eq!(
        names.iter().filter(|name| **name == "check").count(),
        1,
        "check is listed exactly once: {names:?}"
    );

    let check = command.find_subcommand("check").expect("check built-in");
    assert_eq!(
        check.get_subcommands().count(),
        0,
        "check is not a document family"
    );
}

#[test]
fn check_document_slug_is_rejected_by_the_tree_guard() {
    for slug in ["check", "Check", "CHECK"] {
        let mut cfg = Config::default();
        cfg.document_types
            .insert(slug.to_string(), entry("CHECK", "checks/", true));
        let registry = ArtifactRegistry::build(&cfg).expect("registry build accepts the slug");

        let err = tree(Some(&registry)).expect_err("the tree guard rejects the slug");
        assert_eq!(err.code(), 4);
        assert_eq!(
            err.to_string(),
            format!(
                "document_types.{}: slug conflicts with built-in type check",
                slug
            )
        );
    }
}
