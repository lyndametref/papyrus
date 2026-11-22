//! Regression tests for the AUD-141 corrections (TASK-310, F1-F15).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use papyrus::config::Config;
use papyrus::registry::ArtifactRegistry;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("corrections-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn init(d: &Path) {
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run(d: &Path, args: &[&str]) -> Output {
    bin()
        .args(["--dir", d.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn run_stdin(d: &Path, args: &[&str], input: &str) -> Output {
    let mut child = bin()
        .args(["--dir", d.to_str().unwrap()])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn write_config(d: &Path, body: &str) {
    fs::write(d.join("papyrus.config.yml"), body).unwrap();
}

fn front_matter(content: &str) -> &str {
    let rest = content.strip_prefix("---\n").expect("front matter");
    let end = rest.find("\n---\n").expect("closing delimiter");
    &rest[..end]
}

fn frontmatter_value(content: &str, key: &str) -> yaml_serde::Value {
    let parsed: yaml_serde::Value = yaml_serde::from_str(front_matter(content)).unwrap();
    let yaml_serde::Value::Mapping(mapping) = parsed else {
        panic!("front matter is not a mapping");
    };
    mapping
        .get(yaml_serde::Value::String(key.to_string()))
        .cloned()
        .unwrap_or(yaml_serde::Value::Null)
}

fn record(d: &Path, relative: &str) -> PathBuf {
    d.join("docs").join(relative)
}

fn sequence(values: &[&str]) -> yaml_serde::Value {
    yaml_serde::Value::Sequence(
        values
            .iter()
            .map(|value| yaml_serde::Value::String((*value).to_string()))
            .collect(),
    )
}

// F1: create-group runs the shared pipeline, including stdin.
#[test]
fn f1_create_group_resolves_stdin_before_allocation() {
    let d = tmpdir("f1-stdin");
    init(&d);
    let out = run_stdin(
        &d,
        &[
            "req",
            "create-group",
            "--title",
            "Group",
            "--description",
            "-",
        ],
        "from standard input\n",
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "REQ-001");
    let content = fs::read_to_string(record(&d, "requirements/business/REQ-001.md")).unwrap();
    assert!(
        content.contains("## Description\nfrom standard input\n"),
        "{content}"
    );
    assert!(!content.contains("## Description\n-\n"), "{content}");
    let _ = fs::remove_dir_all(&d);
}

// F2: concurrent creates must reserve distinct IDs for every model.
fn concurrent_creates(d: &Path, args: &[&str]) -> Vec<String> {
    let exe = env!("CARGO_BIN_EXE_papyrus");
    let dir = d.to_str().unwrap().to_string();
    let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
    let handles: Vec<_> = (0..12)
        .map(|index| {
            let exe = exe.to_string();
            let dir = dir.clone();
            let args = args.clone();
            std::thread::spawn(move || {
                let out = Command::new(exe)
                    .args(["--dir", &dir])
                    .args(&args)
                    .arg(format!("Item {index}"))
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            })
        })
        .collect();
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

#[test]
fn f2_concurrent_creates_produce_one_file_per_create() {
    let d = tmpdir("f2-concurrency");
    init(&d);

    let ids = concurrent_creates(&d, &["tech-spec", "create", "--description", "body"]);
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(ids.len(), 12);
    assert_eq!(unique.len(), 12, "{ids:?}");
    let files: Vec<String> = fs::read_dir(record(&d, "technical"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md"))
        .collect();
    assert_eq!(files.len(), 12, "{files:?}");
    assert!(
        files.iter().all(|name| name.starts_with("TECH-")),
        "{files:?}"
    );

    let group = stdout(&run(&d, &["req", "create-group", "--title", "G"]));
    assert_eq!(group, "REQ-001");
    let ids = concurrent_creates(&d, &["req", "create", "--group", "REQ-001"]);
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 12, "{ids:?}");
    let count = fs::read_dir(record(&d, "requirements/business"))
        .unwrap()
        .filter(|e| {
            let name = e
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .to_string();
            name != "REQ-001.md" && name.starts_with("REQ-001.") && name.ends_with(".md")
        })
        .count();
    assert_eq!(count, 12);

    let ids = concurrent_creates(&d, &["ui-mockup", "create", "--task", "T", "--spec", "S"]);
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 12, "{ids:?}");
    let count = fs::read_dir(record(&d, "specifications/ui/mockups"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|s| s.to_str())
                == Some("html")
        })
        .count();
    assert_eq!(count, 12);
    let _ = fs::remove_dir_all(&d);
}

// F3: indicator-leading titles and bracketed values round-trip as strings.
#[test]
fn f3_yaml_unsafe_titles_round_trip() {
    let d = tmpdir("f3-titles");
    init(&d);
    for (index, title) in ["*bold", "[a]", "&x", "!x", "|x", "?x", "-x", "{}", "a[b]c"]
        .into_iter()
        .enumerate()
    {
        let out = run(
            &d,
            &["tech-spec", "create", "--description", "d", "--", title],
        );
        assert!(out.status.success(), "title {title:?}: {}", stderr(&out));
        let id = format!("TECH-{:03}", index + 1);
        let content = fs::read_to_string(record(&d, &format!("technical/{id}.md"))).unwrap();
        assert_eq!(
            frontmatter_value(&content, "title"),
            yaml_serde::Value::String(title.to_string()),
            "{content}"
        );
        let viewed = run(&d, &["tech-spec", "view", &id]);
        assert!(viewed.status.success(), "{}", stderr(&viewed));
    }
    let _ = fs::remove_dir_all(&d);
}

// F4: metadata comes from the fixed comment block and never from the title.
#[test]
fn f4_mockup_title_cannot_inject_metadata() {
    let d = tmpdir("f4-injection");
    init(&d);
    for (index, title) in ["Status: approved", "a --> b", "<script>alert(1)</script>"]
        .into_iter()
        .enumerate()
    {
        let out = run(
            &d,
            &["ui-mockup", "create", title, "--task", "T", "--spec", "S"],
        );
        assert!(out.status.success(), "title {title:?}: {}", stderr(&out));
        let id = format!("MOCK-{:03}", index + 1);
        let path = record(&d, &format!("specifications/ui/mockups/{id}.html"));
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("<!-- status: pending -->"), "{content}");
        assert!(!content.contains("<script>"), "{content}");
        let out = run(&d, &["ui-mockup", "edit", &id, "--task", "T2"]);
        assert!(out.status.success(), "{}", stderr(&out));
        let edited = fs::read_to_string(&path).unwrap();
        assert!(edited.contains("<!-- status: pending -->"), "{edited}");
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn f4_encoded_title_survives_an_unrelated_edit() {
    let d = tmpdir("f4-entity");
    init(&d);
    let out = run(
        &d,
        &[
            "ui-mockup",
            "create",
            "A & B <tag>",
            "--task",
            "T",
            "--spec",
            "S",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let path = record(&d, "specifications/ui/mockups/MOCK-001.html");
    let out = run(&d, &["ui-mockup", "edit", "MOCK-001", "--task", "T2"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("<title>MOCK-001 — A &amp; B &lt;tag&gt;</title>"),
        "{content}"
    );
    assert!(!content.contains("&amp;amp;"), "{content}");
    let _ = fs::remove_dir_all(&d);
}

// F5: every accepted group reference form resolves to the same canonical group.
#[test]
fn f5_group_references_are_canonicalized() {
    let d = tmpdir("f5-group");
    init(&d);
    assert_eq!(
        stdout(&run(&d, &["req", "create-group", "--title", "G"])),
        "REQ-001"
    );
    for (index, reference) in ["REQ-1", "req-001", "REQ-0001", "REQ-001.001"]
        .iter()
        .enumerate()
    {
        let id = if index == 3 {
            stdout(&run(&d, &["req", "create", "M3", "--group", reference]))
        } else {
            stdout(&run(
                &d,
                &["req", "create", &format!("M{index}"), "--group", reference],
            ))
        };
        assert!(id.starts_with("REQ-001."), "reference {reference}: {id}");
        let content =
            fs::read_to_string(record(&d, &format!("requirements/business/{id}.md"))).unwrap();
        assert!(
            content.contains("group: REQ-001\n"),
            "{reference}: {content}"
        );
    }
    let _ = fs::remove_dir_all(&d);
}

fn grouped_role_config(d: &Path) {
    write_config(
        d,
        r###"project_name: Test
document_types:
  req:
    prefix: REQ
    directory: requirements/business/
    group:
      fields:
        - key: group_note
          heading: "## Group Note"
          option: group-note
        - key: shared
          heading: "## Shared"
    member:
      fields:
        - key: member_note
          heading: "## Member Note"
          option: member-note
        - key: shared
          heading: "## Shared"
"###,
    );
}

// F6/F9: edit is role-scoped and both roles read the same field resolver as help.
#[test]
fn f6_edit_is_role_scoped_and_help_matches_the_role_fields() {
    let d = tmpdir("f6-role");
    init(&d);
    grouped_role_config(&d);

    assert_eq!(
        stdout(&run(
            &d,
            &["req", "create-group", "G", "--group-note", "gn"]
        )),
        "REQ-001"
    );
    assert_eq!(
        stdout(&run(
            &d,
            &[
                "req",
                "create",
                "M",
                "--group",
                "REQ-001",
                "--member-note",
                "mn"
            ]
        )),
        "REQ-001.001"
    );

    let group_path = record(&d, "requirements/business/REQ-001.md");
    let before = fs::read_to_string(&group_path).unwrap();
    let out = run(&d, &["req", "edit", "REQ-001", "--member-note", "x"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(
        stderr(&out).contains("not available for this record role"),
        "{}",
        stderr(&out)
    );
    assert_eq!(fs::read_to_string(&group_path).unwrap(), before);
    assert!(
        run(&d, &["req", "edit", "REQ-001", "--group-note", "gn2"])
            .status
            .success()
    );

    let member_path = record(&d, "requirements/business/REQ-001.001.md");
    let before = fs::read_to_string(&member_path).unwrap();
    let out = run(&d, &["req", "edit", "REQ-001.001", "--group-note", "x"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(&member_path).unwrap(), before);
    assert!(
        run(&d, &["req", "edit", "REQ-001.001", "--member-note", "mn2"])
            .status
            .success()
    );

    let create_group_help = stdout(&run(&d, &["req", "create-group", "--help"]));
    assert!(
        create_group_help.contains("--group-note"),
        "{create_group_help}"
    );
    assert!(
        !create_group_help.contains("--member-note"),
        "{create_group_help}"
    );
    let create_help = stdout(&run(&d, &["req", "create", "--help"]));
    assert!(create_help.contains("--member-note"), "{create_help}");
    assert!(!create_help.contains("--group-note"), "{create_help}");
    let edit_help = stdout(&run(&d, &["req", "edit", "--help"]));
    assert!(edit_help.contains("--group-note"), "{edit_help}");
    assert!(edit_help.contains("--member-note"), "{edit_help}");
    let _ = fs::remove_dir_all(&d);
}

// F7: every seed descriptor builds a command and help without a panic.
#[test]
fn f7_every_seed_descriptor_builds_its_command_and_help() {
    let registry = ArtifactRegistry::build(&Config::default()).unwrap();
    let mut tree = papyrus::cli::tree::tree(Some(&registry)).unwrap();
    tree.clone().debug_assert();
    let _ = tree.render_help();
    for descriptor in registry
        .descriptors()
        .filter(|d| d.category == papyrus::registry::ArtifactCategory::Document)
    {
        let mut command = papyrus::cli::tree::document_command(descriptor);
        command.clone().debug_assert();
        for spec in descriptor.ordered_verb_specs() {
            let verb = command
                .find_subcommand_mut(spec.verb.as_str())
                .unwrap_or_else(|| panic!("{} {}", descriptor.slug, spec.verb.as_str()));
            let _ = verb.render_help();
        }
    }
}

// F11: a config-defined archived record hits the reachable archived-edit guard.
#[test]
fn f11_archived_config_defined_edit_is_rejected_without_a_write() {
    let d = tmpdir("f11-archived");
    init(&d);
    write_config(
        &d,
        "project_name: Test\ndocument_types:\n  note:\n    prefix: NOTE\n    directory: notes/\n",
    );
    assert_eq!(stdout(&run(&d, &["note", "create", "N"])), "NOTE-001");
    assert!(run(&d, &["note", "archive", "NOTE-001"]).status.success());
    let path = record(&d, "notes/archived/NOTE-001.md");
    let before = fs::read_to_string(&path).unwrap();
    let out = run(&d, &["note", "edit", "NOTE-001", "--title", "Renamed"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("archived; edits are not allowed"),
        "{}",
        stderr(&out)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    let _ = fs::remove_dir_all(&d);
}

// F14: legacy `## Group` records can take new members; the decision is to accept.
#[test]
fn f14_legacy_group_section_accepts_new_members() {
    let d = tmpdir("f14-legacy");
    init(&d);
    let dir = record(&d, "requirements/business");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("REQ-001.md"),
        include_str!("fixtures/legacy-group/REQ-001.md"),
    )
    .unwrap();
    fs::write(
        dir.join("REQ-001.001.md"),
        include_str!("fixtures/legacy-group/REQ-001.001.md"),
    )
    .unwrap();

    let out = run(&d, &["req", "create", "New member", "--group", "REQ-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "REQ-001.002");
    let content = fs::read_to_string(dir.join("REQ-001.002.md")).unwrap();
    assert!(content.contains("group: REQ-001\n"), "{content}");
    assert!(run(&d, &["req", "view", "REQ-001.001"]).status.success());
    let _ = fs::remove_dir_all(&d);
}

// F15: label and reference lists use the sequence variant, including literal [].
#[test]
fn f15_labels_and_references_are_sequences() {
    let d = tmpdir("f15-lists");
    init(&d);
    let out = run(
        &d,
        &[
            "tech-spec",
            "create",
            "Lists",
            "--description",
            "d",
            "--label",
            "[]",
            "--label",
            "core",
            "--reference",
            "REQ-001",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let path = record(&d, "technical/TECH-001.md");
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        yaml_serde::Value::Sequence(vec![
            yaml_serde::Value::String("[]".to_string()),
            yaml_serde::Value::String("core".to_string()),
        ]),
        "{content}"
    );
    assert_eq!(
        frontmatter_value(&content, "references"),
        yaml_serde::Value::Sequence(vec![yaml_serde::Value::String("REQ-001".to_string())]),
        "{content}"
    );
    assert!(
        content.contains("labels:\n  - \"[]\"\n  - core\n"),
        "{content}"
    );

    assert!(
        run(&d, &["tech-spec", "edit", "TECH-001", "--label", "extra"])
            .status
            .success()
    );
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        yaml_serde::Value::Sequence(vec![
            yaml_serde::Value::String("[]".to_string()),
            yaml_serde::Value::String("core".to_string()),
            yaml_serde::Value::String("extra".to_string()),
        ])
    );

    assert!(
        run(&d, &["tech-spec", "create", "Empty", "--description", "d"])
            .status
            .success()
    );
    let content = fs::read_to_string(record(&d, "technical/TECH-002.md")).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        yaml_serde::Value::Sequence(Vec::new())
    );
    let _ = fs::remove_dir_all(&d);
}

// F15: backlog list fields use the same sequence variant.
#[test]
fn f15_backlog_list_fields_are_sequences() {
    let d = tmpdir("f15-backlog");
    init(&d);
    assert!(run(&d, &["task", "create", "Base"]).status.success());
    let out = run(
        &d,
        &[
            "task",
            "create",
            "Backlog lists",
            "--label",
            "[]",
            "--reference",
            "REQ-001",
            "--documentation",
            "docs/a.md",
            "--assignee",
            "alice",
            "bob",
            "--depends-on",
            "TASK-001",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(record(&d, "../backlog/tasks/TASK-002.md")).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        yaml_serde::Value::Sequence(vec![yaml_serde::Value::String("[]".to_string())])
    );
    for (key, expected) in [
        ("references", vec!["REQ-001"]),
        ("documentation", vec!["docs/a.md"]),
        ("assignee", vec!["alice", "bob"]),
        ("dependencies", vec!["TASK-001"]),
    ] {
        assert_eq!(
            frontmatter_value(&content, key),
            yaml_serde::Value::Sequence(
                expected
                    .into_iter()
                    .map(|value| yaml_serde::Value::String(value.to_string()))
                    .collect()
            ),
            "{key}: {content}"
        );
    }
    let _ = fs::remove_dir_all(&d);
}

// F7: the clap auto-generated help flag is reserved for every document command,
// and a config that declares it is rejected with an exit-4 selector-path
// diagnostic instead of panicking while clap builds the command tree.
#[test]
fn f7_reserved_help_option_rejects_every_document_command() {
    use papyrus::config::{DocTypeEntry, FieldKind, FieldSpec, FieldVerb, RoleFields};
    use papyrus::registry::DocumentModel;

    let field = |key: &str| FieldSpec {
        key: key.to_string(),
        heading: Some(format!("## {}", key)),
        frontmatter_key: None,
        option: Some("help".to_string()),
        label: None,
        kind: FieldKind::Text,
        required: false,
        verbs: vec![FieldVerb::Create, FieldVerb::Edit],
        help: None,
    };

    // Every seed descriptor: both validation layers reject the reserved option
    // before a command is built, so no descriptor ever reaches clap.
    let defaults = Config::default();
    for (slug, seeded) in &defaults.document_types {
        let mut entry: DocTypeEntry = seeded.clone();
        let model = DocumentModel::for_slug(slug);
        match model {
            DocumentModel::SingleHtmlMockup => {
                entry.fields = Some(vec![field("custom")]);
            }
            DocumentModel::GroupedMarkdown => {
                entry.group = Some(RoleFields {
                    fields: Some(vec![field("custom")]),
                    reference_visibility: entry
                        .group
                        .as_ref()
                        .and_then(|role| role.reference_visibility),
                });
            }
            DocumentModel::SingleMarkdown => {
                entry.fields = Some(vec![field("custom")]);
            }
        }
        let mut cfg = defaults.clone();
        cfg.document_types.insert(slug.clone(), entry);

        let config_error = cfg
            .validate(None)
            .expect_err("config layer must reject help");
        assert!(
            config_error.contains(&format!("document_types.{}.", slug)),
            "{slug}: {config_error}"
        );
        let registry_error =
            ArtifactRegistry::build(&cfg).expect_err("registry layer must reject help");
        assert!(
            registry_error
                .to_string()
                .contains(&format!("document_types.{}.", slug)),
            "{slug}: {registry_error}"
        );
        if model != DocumentModel::SingleHtmlMockup {
            assert!(
                config_error.contains("reserved option name: help"),
                "{slug}: {config_error}"
            );
            assert!(
                registry_error
                    .to_string()
                    .contains("reserved option name: help"),
                "{slug}: {registry_error}"
            );
        }
    }

    // A real project run reports exit 4 for both a verb call and its --help
    // flag, with no panic and no command build.
    let d = tmpdir("f7-help");
    init(&d);
    write_config(
        &d,
        "project_name: Test\ndocument_types:\n  deploy:\n    prefix: DEP\n    directory: deployment/\n    fields:\n      - key: description\n        heading: '## Description'\n        option: help\n",
    );
    for args in [
        vec!["deploy", "create", "T", "--description", "d"],
        vec!["deploy", "create", "--help"],
    ] {
        let out = run(&d, &args);
        assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
        assert!(
            stderr(&out).contains(
                "document_types.deploy.fields[description].option: reserved option name: help"
            ),
            "{}",
            stderr(&out)
        );
        assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
    }
    let _ = fs::remove_dir_all(&d);
}

// F15: every backlog edit path writes labels, references, documentation,
// dependencies and assignee as sequences, including a bracketed value.
#[test]
fn f15_backlog_edit_list_fields_are_sequences() {
    let d = tmpdir("f15-backlog-edit");
    init(&d);

    // Task: create with no values is an empty sequence.
    assert!(run(&d, &["task", "create", "Target"]).status.success());
    let task = record(&d, "../backlog/tasks/TASK-001.md");
    let content = fs::read_to_string(&task).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&[]));

    let out = run(&d, &["task", "edit", "TASK-001", "--label", "[]"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let content = fs::read_to_string(&task).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&["[]"]));
    assert!(content.contains("labels:\n  - \"[]\"\n"), "{content}");

    assert!(
        run(&d, &["task", "edit", "TASK-001", "--label", "single"])
            .status
            .success()
    );
    assert!(
        run(
            &d,
            &[
                "task", "edit", "TASK-001", "--label", "multi-a", "--label", "multi-b",
            ],
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&task).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        sequence(&["[]", "single", "multi-a", "multi-b"])
    );

    assert!(
        run(
            &d,
            &[
                "task",
                "edit",
                "TASK-001",
                "--reference",
                "[]",
                "--reference",
                "REQ-001",
                "--documentation",
                "[]",
                "--documentation",
                "docs/a.md",
                "--assignee",
                "[]",
                "--assignee",
                "alice",
            ],
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&task).unwrap();
    for (key, expected) in [
        ("references", vec!["[]", "REQ-001"]),
        ("documentation", vec!["[]", "docs/a.md"]),
        ("assignee", vec!["[]", "alice"]),
    ] {
        assert_eq!(
            frontmatter_value(&content, key),
            sequence(&expected),
            "{key}: {content}"
        );
    }

    assert!(run(&d, &["task", "create", "Dependency"]).status.success());
    assert!(
        run(
            &d,
            &["task", "edit", "TASK-001", "--depends-on", "TASK-002"]
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&task).unwrap();
    assert_eq!(
        frontmatter_value(&content, "dependencies"),
        sequence(&["TASK-002"])
    );

    // Empty values: an edit without list flags leaves every sequence unchanged.
    let before = fs::read_to_string(&task).unwrap();
    assert!(
        run(&d, &["task", "edit", "TASK-001", "--title", "Renamed"])
            .status
            .success()
    );
    let content = fs::read_to_string(&task).unwrap();
    for key in [
        "labels",
        "references",
        "documentation",
        "assignee",
        "dependencies",
    ] {
        assert_eq!(
            frontmatter_value(&content, key),
            frontmatter_value(&before, key),
            "{key}: {content}"
        );
    }

    // Milestone edit: bracketed, single, multiple and empty values.
    assert!(run(&d, &["milestone", "create", "M1"]).status.success());
    let milestone = record(&d, "../backlog/milestones/MS-001.md");
    let content = fs::read_to_string(&milestone).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&[]));
    assert!(
        run(&d, &["milestone", "edit", "MS-001", "--label", "[]"])
            .status
            .success()
    );
    let content = fs::read_to_string(&milestone).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&["[]"]));
    assert!(
        run(
            &d,
            &[
                "milestone",
                "edit",
                "MS-001",
                "--label",
                "single",
                "--label",
                "multi-a",
                "--label",
                "multi-b",
                "--reference",
                "[]",
                "--reference",
                "REQ-001",
            ],
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&milestone).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        sequence(&["[]", "single", "multi-a", "multi-b"])
    );
    assert_eq!(
        frontmatter_value(&content, "references"),
        sequence(&["[]", "REQ-001"])
    );
    let before = fs::read_to_string(&milestone).unwrap();
    assert!(
        run(&d, &["milestone", "edit", "MS-001", "--title", "Renamed"])
            .status
            .success()
    );
    let content = fs::read_to_string(&milestone).unwrap();
    for key in ["labels", "references"] {
        assert_eq!(
            frontmatter_value(&content, key),
            frontmatter_value(&before, key),
            "{key}: {content}"
        );
    }

    // Release edit: bracketed, single, multiple and empty values.
    assert!(run(&d, &["release", "create", "R1"]).status.success());
    let release = record(&d, "../backlog/releases/REL-001.md");
    let content = fs::read_to_string(&release).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&[]));
    assert!(
        run(&d, &["release", "edit", "REL-001", "--label", "[]"])
            .status
            .success()
    );
    let content = fs::read_to_string(&release).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&["[]"]));
    assert!(
        run(
            &d,
            &[
                "release",
                "edit",
                "REL-001",
                "--label",
                "single",
                "--label",
                "multi-a",
                "--label",
                "multi-b",
                "--reference",
                "[]",
                "--reference",
                "REQ-001",
            ],
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&release).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        sequence(&["[]", "single", "multi-a", "multi-b"])
    );
    assert_eq!(
        frontmatter_value(&content, "references"),
        sequence(&["[]", "REQ-001"])
    );
    let before = fs::read_to_string(&release).unwrap();
    assert!(
        run(&d, &["release", "edit", "REL-001", "--title", "Renamed"])
            .status
            .success()
    );
    let content = fs::read_to_string(&release).unwrap();
    for key in ["labels", "references"] {
        assert_eq!(
            frontmatter_value(&content, key),
            frontmatter_value(&before, key),
            "{key}: {content}"
        );
    }

    // Decision edit: bracketed, single, multiple and empty values.
    assert!(run(&d, &["decision", "create", "D"]).status.success());
    let decision = fs::read_dir(d.join("decisions"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "md"))
        .expect("decision record");
    let content = fs::read_to_string(&decision).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&[]));
    assert!(
        run(&d, &["decision", "edit", "ADR-001", "--label", "[]"])
            .status
            .success()
    );
    let content = fs::read_to_string(&decision).unwrap();
    assert_eq!(frontmatter_value(&content, "labels"), sequence(&["[]"]));
    assert!(
        run(
            &d,
            &[
                "decision",
                "edit",
                "ADR-001",
                "--label",
                "single",
                "--label",
                "multi-a",
                "--label",
                "multi-b",
                "--reference",
                "[]",
                "--reference",
                "REQ-001",
            ],
        )
        .status
        .success()
    );
    let content = fs::read_to_string(&decision).unwrap();
    assert_eq!(
        frontmatter_value(&content, "labels"),
        sequence(&["[]", "single", "multi-a", "multi-b"])
    );
    assert_eq!(
        frontmatter_value(&content, "references"),
        sequence(&["[]", "REQ-001"])
    );
    let before = fs::read_to_string(&decision).unwrap();
    assert!(
        run(&d, &["decision", "edit", "ADR-001", "--title", "Renamed"])
            .status
            .success()
    );
    let content = fs::read_to_string(&decision).unwrap();
    for key in ["labels", "references"] {
        assert_eq!(
            frontmatter_value(&content, key),
            frontmatter_value(&before, key),
            "{key}: {content}"
        );
    }

    let _ = fs::remove_dir_all(&d);
}
