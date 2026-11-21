//! TASK-240 / MS-032 AC-5, AC-13: replay the characterization corpus.
//!
//! Part of the default `cargo test` run. Every case from `cases.json` runs
//! against a throwaway copy of the committed seed project and is compared
//! byte-for-byte with `expected/<case>/`. HTML is compared byte-for-byte in
//! MS-032 because no behaviour may change; later milestones may use the
//! semantic page comparison of TC-184.

#[path = "support/characterization.rs"]
mod characterization;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use characterization::{Case, Ctx, Runner, compare_case, corpus_root, load_manifest};

fn minimal_case(id: &str, artifacts: &[&str]) -> Case {
    Case {
        id: id.to_string(),
        kind: "http".to_string(),
        argv: Vec::new(),
        request: None,
        document_type: None,
        model: None,
        renderer: None,
        record: None,
        project: None,
        artifacts: artifacts.iter().map(|name| (*name).to_string()).collect(),
    }
}

fn replay(kind: &'static str, tag: &str) {
    let root = corpus_root();
    let manifest = load_manifest(&root);
    assert_eq!(manifest.schema, "papyrus.characterization-cases/1");
    assert!(
        manifest
            .unreachable
            .iter()
            .any(|entry| entry["renderer"] == "conflict_page_test_case")
    );
    assert!(
        manifest
            .unreachable
            .iter()
            .any(|entry| entry["renderer"] == "conflict_page_test_report")
    );
    assert!(
        manifest.cases.iter().any(|case| case.kind == kind),
        "the manifest declares {kind} cases"
    );
    let ctx = Ctx::new(tag);
    Runner {
        root,
        ctx: &ctx,
        write: false,
        only: Some(kind),
    }
    .run();
}

#[test]
fn every_cli_case_replays_byte_for_byte() {
    replay("cli", "replay-cli");
}

#[test]
fn every_http_case_replays_byte_for_byte() {
    replay("http", "replay-http");
}

#[test]
fn a_single_byte_change_fails_the_comparison() {
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("characterization-negative-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).expect("create negative workspace");

    let mut observed: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    observed.insert("page.html".to_string(), b"<h1>Stable</h1>\n".to_vec());
    let case = minimal_case("http/negative", &["page.html"]);
    fs::write(scratch.join("page.html"), b"<h1>Stable</h1>\n").expect("write baseline");
    assert!(
        compare_case(&scratch, &observed, &case).is_ok(),
        "the unchanged artifact must compare equal"
    );

    fs::write(scratch.join("page.html"), b"<h1>Stabke</h1>\n").expect("write one-byte change");
    let error = compare_case(&scratch, &observed, &case)
        .expect_err("a one-byte change must fail the comparison");
    assert!(
        error.contains("page.html"),
        "the failure names the artifact"
    );

    fs::write(scratch.join("page.html"), b"<h1>Stable</h1>\n").expect("restore baseline");
    fs::write(scratch.join("extra.txt"), b"extra").expect("write extra artifact");
    assert!(
        compare_case(&scratch, &observed, &case).is_err(),
        "an extra expected artifact must fail the comparison"
    );
    let _ = fs::remove_dir_all(&scratch);
}
