use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

/// Project-local scratch directory (never a system temp directory).
fn tmpdir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("ui-spec-{}-{}", name, std::process::id()));
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

fn create(d: &std::path::Path, title: &str, view: &str, description: &str) -> std::process::Output {
    run(
        d,
        &[
            "ui-spec",
            "create",
            title,
            "--view",
            view,
            "--description",
            description,
        ],
    )
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn record(d: &std::path::Path, id: &str) -> PathBuf {
    d.join("docs/specifications/ui").join(format!("{}.md", id))
}

#[test]
fn ui_spec_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);

    let out = create(&d, "Login screen", "auth-login", "Sign-in form");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out).trim(), "UI-001");

    let path = record(&d, "UI-001");
    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("# UI-001 Login screen\n"));
    assert!(content.contains("## View\nauth-login\n"));
    assert!(content.contains("## Description\nSign-in form\n"));
    assert!(content.ends_with('\n') && !content.ends_with("\n\n"));

    let out = run(&d, &["ui-spec", "list"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("UI-001 Login screen"));

    let out = run(&d, &["ui-spec", "view", "UI-001"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), content);

    // Edit view only: description is preserved.
    let out = run(&d, &["ui-spec", "edit", "UI-001", "--view", "auth-reset"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["ui-spec", "view", "UI-001"]);
    let s = stdout(&out);
    assert!(s.contains("## View\nauth-reset"));
    assert!(s.contains("## Description\nSign-in form"));
    assert!(s.contains("# UI-001 Login screen"));

    // Edit description only: view is preserved.
    let out = run(
        &d,
        &["ui-spec", "edit", "UI-001", "--description", "Reset form"],
    );
    assert!(out.status.success());
    let out = run(&d, &["ui-spec", "view", "UI-001"]);
    let s = stdout(&out);
    assert!(s.contains("## View\nauth-reset"));
    assert!(s.contains("## Description\nReset form"));

    // Deprecate: file moves, list hides it, view still works.
    let out = run(&d, &["ui-spec", "deprecate", "UI-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dep = d.join("docs/specifications/ui/deprecated/UI-001.md");
    assert!(dep.exists());
    assert!(!path.exists());

    let out = run(&d, &["ui-spec", "list"]);
    assert!(out.status.success());
    assert!(!stdout(&out).contains("UI-001"));

    let out = run(&d, &["ui-spec", "view", "UI-001"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Reset form"));

    // Deprecating again is a validation error.
    let out = run(&d, &["ui-spec", "deprecate", "UI-001"]);
    assert_eq!(out.status.code(), Some(4));

    // Restore preserves ID and content.
    let before = fs::read_to_string(&dep).unwrap();
    let out = run(&d, &["ui-spec", "restore", "UI-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(path.exists());
    assert!(!dep.exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), before);

    let out = run(&d, &["ui-spec", "list"]);
    assert!(stdout(&out).contains("UI-001 Login screen"));

    // Restoring an active spec is a validation error.
    let out = run(&d, &["ui-spec", "restore", "UI-001"]);
    assert_eq!(out.status.code(), Some(4));

    for args in [
        vec!["ui-spec", "view", "NOPE"],
        vec!["ui-spec", "edit", "NOPE", "--view", "v"],
        vec!["ui-spec", "deprecate", "NOPE"],
        vec!["ui-spec", "restore", "NOPE"],
    ] {
        let out = run(&d, &args);
        assert_eq!(out.status.code(), Some(3), "args={:?}", args);
    }

    // IDs are matched case-sensitively.
    let out = run(&d, &["ui-spec", "view", "ui-001"]);
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_create_requires_all_arguments() {
    let d = tmpdir("missing");
    init(&d);

    let cases: Vec<Vec<&str>> = vec![
        vec!["ui-spec", "create", "--view", "v", "--description", "d"],
        vec!["ui-spec", "create", "T", "--description", "d"],
        vec!["ui-spec", "create", "T", "--view", "v"],
        // --title is not part of the command.
        vec![
            "ui-spec",
            "create",
            "T",
            "--title",
            "T",
            "--view",
            "v",
            "--description",
            "d",
        ],
    ];
    for args in cases {
        let out = run(&d, &args);
        assert_eq!(out.status.code(), Some(2), "args={:?}", args);
    }
    assert!(!d.join("docs/specifications/ui/UI-001.md").exists());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_id_generation_and_no_reuse() {
    let d = tmpdir("ids");
    init(&d);

    let out = create(&d, "One", "v", "d");
    assert_eq!(stdout(&out).trim(), "UI-001");
    let out = run(&d, &["ui-spec", "deprecate", "UI-001"]);
    assert!(out.status.success());

    // The deprecated folder still counts for ID generation.
    let out = create(&d, "Two", "v", "d");
    assert_eq!(stdout(&out).trim(), "UI-002");

    let out = run(&d, &["ui-spec", "restore", "UI-001"]);
    assert!(out.status.success());
    let out = create(&d, "Three", "v", "d");
    assert_eq!(stdout(&out).trim(), "UI-003");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_respects_configured_prefix_padding_and_directory() {
    let d = tmpdir("config");
    init(&d);

    let out = run(
        &d,
        &["config", "set", "document_types.ui-spec.prefix", "SPEC"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["config", "set", "id_padding", "4"]);
    assert!(out.status.success());

    let out = create(&d, "Configurable", "v", "d");
    assert_eq!(stdout(&out).trim(), "SPEC-0001");
    assert!(record(&d, "SPEC-0001").exists());

    // Custom type directory.
    let out = run(
        &d,
        &[
            "config",
            "set",
            "document_types.ui-spec.directory",
            "custom-ui",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = create(&d, "Custom", "v", "d");
    assert_eq!(stdout(&out).trim(), "SPEC-0001");
    assert!(d.join("docs/custom-ui/SPEC-0001.md").exists());

    let out = run(&d, &["ui-spec", "list"]);
    assert!(stdout(&out).contains("SPEC-0001 Custom"));
    assert!(!record(&d, "SPEC-0002").exists());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_list_is_non_recursive_and_filters_non_records() {
    let d = tmpdir("scan");
    init(&d);

    create(&d, "One", "v", "d");
    create(&d, "Two", "v", "d");

    let dir = d.join("docs/specifications/ui");
    fs::create_dir_all(dir.join("mockups")).unwrap();
    fs::write(dir.join("mockups/UI-999.md"), "# not a record\n").unwrap();
    fs::create_dir_all(dir.join("ui-extra")).unwrap();
    fs::write(dir.join("ui-extra/UI-998.md"), "# not a record\n").unwrap();
    fs::write(dir.join("README.md"), "# UI description document\n").unwrap();
    fs::write(dir.join("UI-ABC.md"), "# UI-ABC bogus\n").unwrap();
    fs::write(
        dir.join("UI-010.md"),
        "# UI-010 Later\n\n## View\nv\n\n## Description\nd\n",
    )
    .unwrap();
    fs::write(dir.join("notes.txt"), "ignored\n").unwrap();

    let out = run(&d, &["ui-spec", "list"]);
    assert!(out.status.success());
    let lines: Vec<String> = stdout(&out)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        lines,
        vec![
            "UI-001 One".to_string(),
            "UI-002 Two".to_string(),
            "UI-010 Later".to_string()
        ]
    );

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_edit_deprecated_allowed_and_title_preserved() {
    let d = tmpdir("editdep");
    init(&d);
    create(&d, "Original title", "v1", "d1");
    let out = run(&d, &["ui-spec", "deprecate", "UI-001"]);
    assert!(out.status.success());

    let out = run(
        &d,
        &[
            "ui-spec",
            "edit",
            "UI-001",
            "--view",
            "v2",
            "--description",
            "d2",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = run(&d, &["ui-spec", "view", "UI-001"]);
    let s = stdout(&out);
    assert!(s.contains("# UI-001 Original title"));
    assert!(s.contains("## View\nv2"));
    assert!(s.contains("## Description\nd2"));

    assert!(
        d.join("docs/specifications/ui/deprecated/UI-001.md")
            .exists()
    );
    assert!(!d.join("docs/specifications/ui/UI-001.md").exists());

    let out = run(&d, &["ui-spec", "list"]);
    assert!(!stdout(&out).contains("UI-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_editor_rewrites_without_renaming() {
    let d = tmpdir("editor");
    init(&d);
    create(&d, "Editor title", "v", "d");

    let script = d.join("editor.sh");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '# UI-001 Edited title\\n\\n## View\\nv-edited\\n\\n## Description\\nd-edited\\n' > \"$1\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
    }

    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "ui-spec",
            "edit",
            "UI-001",
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

    assert!(record(&d, "UI-001").exists());
    let content = fs::read_to_string(record(&d, "UI-001")).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("## View\nv-edited"));
    assert!(content.contains("## Description\nd-edited"));

    // Missing editor is a runtime error (exit 1) and changes nothing.
    let before = fs::read_to_string(record(&d, "UI-001")).unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "ui-spec",
            "edit",
            "UI-001",
            "--view",
            "should-not-apply",
            "--editor",
        ])
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(record(&d, "UI-001")).unwrap(), before);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_sanitization_rejects_bad_input_and_writes_nothing() {
    let d = tmpdir("san");
    init(&d);

    for bad in ["   ", "a/b", "x\\y", "a\nb", "a\rb", "a\tb", "a\u{1}b"] {
        let out = create(&d, bad, "v", "d");
        assert_eq!(out.status.code(), Some(4), "title bad={:?}", bad);
        let out = create(&d, "T", bad, "d");
        assert_eq!(out.status.code(), Some(4), "view bad={:?}", bad);
        let out = create(&d, "T", "v", bad);
        assert_eq!(out.status.code(), Some(4), "description bad={:?}", bad);
    }
    assert!(!d.join("docs/specifications/ui/UI-001.md").exists());

    create(&d, "OK", "v", "d");

    for bad in ["a/b", "x\\y", "a\nb", "a\tb", "  "] {
        let out = run(&d, &["ui-spec", "view", bad]);
        assert_eq!(out.status.code(), Some(4), "id bad={:?}", bad);
    }

    let before = fs::read_to_string(record(&d, "UI-001")).unwrap();
    let out = run(&d, &["ui-spec", "edit", "UI-001", "--view", "x/y"]);
    assert_eq!(out.status.code(), Some(4));
    let out = run(&d, &["ui-spec", "edit", "UI-001", "--description", "a\tb"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "UI-001")).unwrap(), before);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_project_directory_boundary() {
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
            "ui-spec",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    create(&proj, "Boundary", "v", "d");
    let target = record(&proj, "UI-001");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "# secret\n").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }

    let out = run(&proj, &["ui-spec", "view", "UI-001"]);
    assert_eq!(out.status.code(), Some(1));

    let out = run(&proj, &["ui-spec", "deprecate", "UI-001"]);
    assert_eq!(out.status.code(), Some(1));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ui_spec_disabled_type_blocks_create_and_list() {
    let d = tmpdir("disabled");
    init(&d);
    create(&d, "Existing", "v", "d");

    let out = run(
        &d,
        &["config", "set", "document_types.ui-spec.enabled", "false"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = create(&d, "Blocked", "v", "d");
    assert_eq!(out.status.code(), Some(4));
    assert!(!record(&d, "UI-002").exists());

    let out = run(&d, &["ui-spec", "list"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(!stdout(&out).contains("UI-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_json_output_shapes() {
    let d = tmpdir("json");
    init(&d);

    let out = run(
        &d,
        &[
            "--json",
            "ui-spec",
            "create",
            "JSON title",
            "--view",
            "v",
            "--description",
            "d",
        ],
    );
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "UI-001");
    assert_eq!(v.as_object().unwrap().len(), 1);

    let out = run(&d, &["--json", "ui-spec", "list"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v[0]["id"], "UI-001");
    assert_eq!(v[0]["title"], "JSON title");

    let out = run(&d, &["--json", "ui-spec", "view", "UI-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "UI-001");
    assert!(v["path"].as_str().unwrap().ends_with("UI-001.md"));
    assert!(
        v["content"]
            .as_str()
            .unwrap()
            .contains("# UI-001 JSON title")
    );

    let out = run(&d, &["--json", "ui-spec", "edit", "UI-001", "--view", "v2"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "UI-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "ui-spec", "deprecate", "UI-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "UI-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "ui-spec", "restore", "UI-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "UI-001");
    assert_eq!(v["status"], "ok");

    // Errors in JSON mode carry the exit-relevant code.
    let out = run(&d, &["--json", "ui-spec", "view", "NOPE"]);
    assert_eq!(out.status.code(), Some(3));
    let err: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stderr).trim()).unwrap();
    assert_eq!(err["error"]["code"], 3);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn ui_spec_help_is_available() {
    let d = tmpdir("help");
    init(&d);
    let out = run(&d, &["ui-spec", "--help"]);
    assert!(out.status.success());
    let s = stdout(&out);
    for sub in ["create", "list", "view", "edit", "deprecate", "restore"] {
        assert!(s.contains(sub), "help missing {}", sub);
    }
    let _ = fs::remove_dir_all(&d);
}
