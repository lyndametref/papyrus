use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("rule-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn init(d: &std::path::Path) {
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

fn run(d: &std::path::Path, args: &[&str]) -> std::process::Output {
    bin()
        .args(["--dir", d.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn create(
    d: &std::path::Path,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
) -> std::process::Output {
    run(
        d,
        &[
            "rule",
            "create",
            title,
            "--description",
            description,
            "--rationale",
            rationale,
            "--applicability",
            applicability,
        ],
    )
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn record(d: &std::path::Path, id: &str) -> PathBuf {
    d.join("docs/business-rules").join(format!("{}.md", id))
}

#[test]
fn rule_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);

    let out = create(
        &d,
        "First Rule",
        "desc text",
        "rationale text",
        "applies here",
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out).trim(), "RUL-001");

    let out = run(&d, &["rule", "list"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("RUL-001 First Rule"));

    let out = run(&d, &["rule", "view", "RUL-001"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("# RUL-001 First Rule"));
    assert!(s.contains("## Description\ndesc text"));
    assert!(s.contains("## Rationale\nrationale text"));
    assert!(s.contains("## Applicability\napplies here"));

    // edit title only
    let out = run(
        &d,
        &["rule", "edit", "RUL-001", "--title", "First Rule Updated"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(s.contains("# RUL-001 First Rule Updated"));
    assert!(s.contains("## Description\ndesc text"));

    // edit description only
    let out = run(&d, &["rule", "edit", "RUL-001", "-d", "desc2"]);
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(s.contains("## Description\ndesc2"));
    assert!(s.contains("## Rationale\nrationale text"));

    // deprecate
    let out = run(&d, &["rule", "deprecate", "RUL-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["rule", "list"]);
    assert!(!stdout(&out).contains("RUL-001"));
    let out = run(&d, &["rule", "view", "RUL-001"]);
    assert!(out.status.success());

    // already deprecated
    let out = run(&d, &["rule", "deprecate", "RUL-001"]);
    assert_eq!(out.status.code(), Some(4));

    // restore
    let out = run(&d, &["rule", "restore", "RUL-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["rule", "list"]);
    assert!(stdout(&out).contains("RUL-001"));

    // already active
    let out = run(&d, &["rule", "restore", "RUL-001"]);
    assert_eq!(out.status.code(), Some(4));

    // unknown id
    let out = run(&d, &["rule", "view", "RUL-999"]);
    assert_eq!(out.status.code(), Some(3));
    let out = run(&d, &["rule", "deprecate", "RUL-999"]);
    assert_eq!(out.status.code(), Some(3));
    let out = run(&d, &["rule", "restore", "RUL-999"]);
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_id_generation_not_reused() {
    let d = tmpdir("ids");
    init(&d);
    create(&d, "A", "d", "r", "a");
    create(&d, "B", "d", "r", "a");
    let out = run(&d, &["rule", "deprecate", "RUL-001"]);
    assert!(out.status.success());
    let out = create(&d, "C", "d", "r", "a");
    assert_eq!(stdout(&out).trim(), "RUL-003");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_requirement_references() {
    let d = tmpdir("reqs");
    init(&d);

    let out = run(
        &d,
        &[
            "rule",
            "create",
            "R",
            "--description",
            "d",
            "--rationale",
            "r",
            "--applicability",
            "a",
            "--requirement",
            "REQ-001",
            "--requirement",
            "REQ-002",
            "--requirement",
            "REQ-001",
        ],
    );
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(s.contains("## Requirements"));
    assert!(s.contains("- REQ-001"));
    assert!(s.contains("- REQ-002"));
    // dedup: only once
    assert_eq!(s.matches("- REQ-001").count(), 1);

    // add
    let out = run(&d, &["rule", "edit", "RUL-001", "--requirement", "REQ-003"]);
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(s.contains("- REQ-003"));

    // remove one
    let out = run(
        &d,
        &["rule", "edit", "RUL-001", "--no-requirement", "REQ-001"],
    );
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(!s.contains("- REQ-001"));
    assert!(s.contains("- REQ-002"));

    // remove all
    let out = run(&d, &["rule", "edit", "RUL-001", "--no-requirement"]);
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-001"]);
    let s = stdout(&out);
    assert!(!s.contains("## Requirements"));

    // add + remove in one invocation, additions before removals
    let out = run(
        &d,
        &[
            "rule",
            "create",
            "R2",
            "--description",
            "d",
            "--rationale",
            "r",
            "--applicability",
            "a",
            "--requirement",
            "REQ-010",
        ],
    );
    assert!(out.status.success());
    let out = run(
        &d,
        &[
            "rule",
            "edit",
            "RUL-002",
            "--requirement",
            "REQ-010",
            "--no-requirement",
            "REQ-010",
        ],
    );
    assert!(out.status.success());
    let out = run(&d, &["rule", "view", "RUL-002"]);
    let s = stdout(&out);
    // added then removed => absent
    assert!(!s.contains("- REQ-010"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_sanitization_rejects_bad_input() {
    let d = tmpdir("san");
    init(&d);

    for bad in ["   ", "a/b", "x\\y", "a\nb", "a\rb", "a\tb", "a\u{1}b"] {
        let out = create(&d, bad, "d", "r", "a");
        assert_eq!(out.status.code(), Some(4), "title bad={:?}", bad);
        let out = create(&d, "T", bad, "r", "a");
        assert_eq!(out.status.code(), Some(4), "desc bad={:?}", bad);
        let out = create(&d, "T", "d", bad, "a");
        assert_eq!(out.status.code(), Some(4), "rationale bad={:?}", bad);
        let out = create(&d, "T", "d", "r", bad);
        assert_eq!(out.status.code(), Some(4), "applicability bad={:?}", bad);
    }
    assert!(!record(&d, "RUL-001").exists());

    create(&d, "OK", "d", "r", "a");

    for bad in ["a/b", "x\\y", "a\nb", "a\tb", "  "] {
        let out = run(&d, &["rule", "view", bad]);
        assert_eq!(out.status.code(), Some(4), "id bad={:?}", bad);
    }

    let before = fs::read_to_string(record(&d, "RUL-001")).unwrap();
    let out = run(&d, &["rule", "edit", "RUL-001", "--title", "x/y"]);
    assert_eq!(out.status.code(), Some(4));
    let out = run(&d, &["rule", "edit", "RUL-001", "-d", "a\tb"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "RUL-001")).unwrap(), before);

    // requirement sanitization
    let out = run(&d, &["rule", "edit", "RUL-001", "--requirement", "a/b"]);
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_missing_required_args_exit_2() {
    let d = tmpdir("usage");
    init(&d);
    let cases: &[&[&str]] = &[
        &["rule", "create", "T"],
        &["rule", "create", "T", "--description", "d"],
        &[
            "rule",
            "create",
            "T",
            "--description",
            "d",
            "--rationale",
            "r",
        ],
        &["rule", "create", "T", "--foo"],
    ];
    for args in cases {
        let out = run(&d, args);
        assert_eq!(out.status.code(), Some(2), "args={:?}", args);
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_disabled_type() {
    let d = tmpdir("disabled");
    init(&d);
    create(&d, "Existing", "d", "r", "a");

    let out = run(
        &d,
        &["config", "set", "document_types.rule.enabled", "false"],
    );
    assert!(out.status.success());

    let out = create(&d, "Blocked", "d", "r", "a");
    assert_eq!(out.status.code(), Some(4));
    assert!(!record(&d, "RUL-002").exists());

    let out = run(&d, &["rule", "list"]);
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_project_directory_boundary() {
    let root = tmpdir("bnd");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    init(&proj);

    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "--config",
            outside.join("evil.yml").to_str().unwrap(),
            "rule",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    create(&proj, "Boundary", "d", "r", "a");
    let target = record(&proj, "RUL-001");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "# secret\n").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }

    let out = run(&proj, &["rule", "view", "RUL-001"]);
    assert_eq!(out.status.code(), Some(1));

    let out = run(&proj, &["rule", "deprecate", "RUL-001"]);
    assert_eq!(out.status.code(), Some(1));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rule_json_output_shapes() {
    let d = tmpdir("json");
    init(&d);

    let out = run(
        &d,
        &[
            "--json",
            "rule",
            "create",
            "JSON title",
            "--description",
            "d",
            "--rationale",
            "r",
            "--applicability",
            "a",
            "--requirement",
            "REQ-001",
        ],
    );
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "RUL-001");

    let out = run(&d, &["--json", "rule", "list"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v[0]["id"], "RUL-001");
    assert_eq!(v[0]["title"], "JSON title");

    let out = run(&d, &["--json", "rule", "view", "RUL-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "RUL-001");
    assert!(v["path"].as_str().unwrap().ends_with("RUL-001.md"));
    assert!(
        v["content"]
            .as_str()
            .unwrap()
            .contains("# RUL-001 JSON title")
    );

    let out = run(&d, &["--json", "rule", "edit", "RUL-001", "--title", "T2"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "RUL-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "rule", "deprecate", "RUL-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "RUL-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "rule", "restore", "RUL-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "RUL-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "rule", "view", "NOPE"]);
    assert_eq!(out.status.code(), Some(3));
    let err: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stderr).trim()).unwrap();
    assert_eq!(err["error"]["code"], 3);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_editor_with_script() {
    let d = tmpdir("editscript");
    init(&d);
    create(&d, "Editor Rule", "d", "r", "a");

    let script = d.join("editor.sh");
    fs::write(&script, "#!/bin/sh\nprintf '# RUL-001 Edited title\\n\\n## Description\\nd-edited\\n\\n## Rationale\\nr-edited\\n\\n## Applicability\\na-edited\\n' > \"$1\"\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "rule",
            "edit",
            "RUL-001",
            "--editor",
        ])
        .env("VISUAL", script.to_str().unwrap())
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(record(&d, "RUL-001").exists());
    let content = fs::read_to_string(record(&d, "RUL-001")).unwrap();
    assert!(content.starts_with("# RUL-001 Edited title"));
    assert!(content.contains("## Description\nd-edited"));
    assert!(content.contains("## Rationale\nr-edited"));
    assert!(content.contains("## Applicability\na-edited"));

    // missing editor leaves file unchanged
    let before2 = fs::read_to_string(record(&d, "RUL-001")).unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "rule",
            "edit",
            "RUL-001",
            "--title",
            "ShouldNotApply",
            "--editor",
        ])
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(record(&d, "RUL-001")).unwrap(), before2);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_non_recursive_scan() {
    let d = tmpdir("nonrec");
    init(&d);
    // Create a record in a subdirectory that should be ignored.
    let subdir = d.join("docs/business-rules/sub");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("RUL-999.md"), "# RUL-999 Hidden\n").unwrap();
    // Also create a non-record file at top level.
    fs::write(
        d.join("docs/business-rules/BR-001.md"),
        "# BR-001 canonical\n",
    )
    .unwrap();

    create(&d, "Top", "d", "r", "a");
    let out = run(&d, &["rule", "list"]);
    let s = stdout(&out);
    assert!(s.contains("RUL-001 Top"));
    assert!(!s.contains("RUL-999"));
    assert!(!s.contains("BR-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_help_is_available() {
    let d = tmpdir("help");
    init(&d);
    let out = run(&d, &["rule", "--help"]);
    assert!(out.status.success());
    let s = stdout(&out);
    for sub in ["create", "list", "view", "edit", "deprecate", "restore"] {
        assert!(s.contains(sub), "help missing {}", sub);
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_deprecated_edit_keeps_deprecated() {
    let d = tmpdir("depedit");
    init(&d);
    create(&d, "Dep", "d", "r", "a");
    let out = run(&d, &["rule", "deprecate", "RUL-001"]);
    assert!(out.status.success());
    let out = run(&d, &["rule", "edit", "RUL-001", "--title", "Dep Updated"]);
    assert!(out.status.success());
    // still in deprecated folder
    assert!(!record(&d, "RUL-001").exists());
    assert!(d.join("docs/business-rules/deprecated/RUL-001.md").exists());
    let out = run(&d, &["rule", "list"]);
    assert!(!stdout(&out).contains("RUL-001"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_editor_sanitization_failure_writes_nothing() {
    let d = tmpdir("edsan");
    init(&d);
    create(&d, "San", "d", "r", "a");
    let before = fs::read_to_string(record(&d, "RUL-001")).unwrap();
    let out = run(&d, &["rule", "edit", "RUL-001", "--title", "a/b"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "RUL-001")).unwrap(), before);
    let _ = fs::remove_dir_all(&d);
}
