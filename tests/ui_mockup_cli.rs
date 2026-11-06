use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("ui-mockup-{}-{}", name, std::process::id()));
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
    task: &str,
    specs: &[&str],
    refs: &[&str],
) -> std::process::Output {
    let mut args = vec!["ui-mockup", "create", title, "--task", task];
    for s in specs {
        args.push("--spec");
        args.push(s);
    }
    for r in refs {
        args.push("--ref");
        args.push(r);
    }
    run(d, &args)
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn record(d: &std::path::Path, id: &str) -> PathBuf {
    d.join("docs/specifications/ui/mockups")
        .join(format!("{}.html", id))
}

#[test]
fn basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);

    let out = create(&d, "Login screen", "TASK-41", &["UI-001"], &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out).trim(), "MOCK-001");

    let path = record(&d, "MOCK-001");
    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("<!-- status: pending -->"));
    assert!(content.contains("<!-- task: TASK-41 -->"));
    assert!(content.contains("<!-- specs: UI-001 -->"));
    assert!(content.ends_with('\n') && !content.ends_with("\n\n"));

    let out = run(&d, &["ui-mockup", "list"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("MOCK-001 Login screen"));

    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), content);

    // Edit task/spec only
    let out = run(
        &d,
        &[
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--task",
            "TASK-99",
            "--spec",
            "UI-002",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    let s = stdout(&out);
    assert!(s.contains("<!-- task: TASK-99 -->"));
    assert!(s.contains("<!-- specs: UI-002 -->"));
    assert!(s.contains("<h1>MOCK-001 Login screen</h1>"));

    // Deprecate
    let out = run(&d, &["ui-mockup", "deprecate", "MOCK-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dep = d.join("docs/specifications/ui/mockups/deprecated/MOCK-001.html");
    assert!(dep.exists());
    assert!(!path.exists());
    let out = run(&d, &["ui-mockup", "list"]);
    assert!(!stdout(&out).contains("MOCK-001"));
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    assert!(out.status.success());

    // deprecate again
    let out = run(&d, &["ui-mockup", "deprecate", "MOCK-001"]);
    assert_eq!(out.status.code(), Some(4));

    // restore
    let out = run(&d, &["ui-mockup", "restore", "MOCK-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(path.exists());
    assert!(!dep.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("<p>Status: pending</p>"));

    let out = run(&d, &["ui-mockup", "restore", "MOCK-001"]);
    assert_eq!(out.status.code(), Some(4));

    for args in [
        vec!["ui-mockup", "view", "NOPE"],
        vec!["ui-mockup", "edit", "NOPE", "--title", "x"],
        vec!["ui-mockup", "deprecate", "NOPE"],
        vec!["ui-mockup", "restore", "NOPE"],
    ] {
        let out = run(&d, &args);
        assert_eq!(out.status.code(), Some(3), "args={:?}", args);
    }

    // case-sensitive
    let out = run(&d, &["ui-mockup", "view", "mock-001"]);
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn create_requires_args() {
    let d = tmpdir("missing");
    init(&d);
    let cases: Vec<Vec<&str>> = vec![
        vec!["ui-mockup", "create", "--task", "T", "--spec", "S"],
        vec!["ui-mockup", "create", "T", "--spec", "S"],
        vec!["ui-mockup", "create", "T", "--task", "T"],
    ];
    for args in cases {
        let out = run(&d, &args);
        assert_eq!(out.status.code(), Some(2), "args={:?}", args);
    }
    assert!(!record(&d, "MOCK-001").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn id_generation_no_reuse() {
    let d = tmpdir("ids");
    init(&d);
    let out = create(&d, "One", "T", &["S"], &[]);
    assert_eq!(stdout(&out).trim(), "MOCK-001");
    let out = run(&d, &["ui-mockup", "deprecate", "MOCK-001"]);
    assert!(out.status.success());
    let out = create(&d, "Two", "T", &["S"], &[]);
    assert_eq!(stdout(&out).trim(), "MOCK-002");
    let out = run(&d, &["ui-mockup", "restore", "MOCK-001"]);
    assert!(out.status.success());
    let out = create(&d, "Three", "T", &["S"], &[]);
    assert_eq!(stdout(&out).trim(), "MOCK-003");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_prefix_padding_directory() {
    let d = tmpdir("config");
    init(&d);
    let out = run(
        &d,
        &["config", "set", "document_types.ui-mockup.prefix", "SPEC"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["config", "set", "id_padding", "4"]);
    assert!(out.status.success());
    let out = create(&d, "Configurable", "T", &["S"], &[]);
    assert_eq!(stdout(&out).trim(), "SPEC-0001");
    assert!(
        d.join("docs/specifications/ui/mockups/SPEC-0001.html")
            .exists()
    );

    let out = run(
        &d,
        &[
            "config",
            "set",
            "document_types.ui-mockup.directory",
            "custom-ui",
        ],
    );
    assert!(out.status.success());
    let out = create(&d, "Custom", "T", &["S"], &[]);
    assert_eq!(stdout(&out).trim(), "SPEC-0001");
    assert!(d.join("docs/custom-ui/SPEC-0001.html").exists());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn list_non_recursive() {
    let d = tmpdir("scan");
    init(&d);
    create(&d, "One", "T", &["S"], &[]);
    create(&d, "Two", "T", &["S"], &[]);

    let dir = d.join("docs/specifications/ui/mockups");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/MOCK-999.html"), "<html></html>").unwrap();
    fs::write(dir.join("README.md"), "# readme").unwrap();
    fs::write(dir.join("wireframe.css"), "body{}").unwrap();
    fs::write(dir.join("MOCK-ABC.html"), "<html></html>").unwrap();
    fs::write(dir.join("MOCK-010.html"), "<!doctype html>\n<html><body><h1>MOCK-010 Later</h1><p>Status: pending</p><p>Task: T</p><p>UI specifications: S</p><p>References: </p><p>Superseded by: </p></body></html>\n").unwrap();

    let out = run(&d, &["ui-mockup", "list"]);
    assert!(out.status.success());
    let lines: Vec<String> = stdout(&out)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    assert_eq!(
        lines,
        vec!["MOCK-001 One", "MOCK-002 Two", "MOCK-010 Later"]
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn edit_deprecated_allowed() {
    let d = tmpdir("editdep");
    init(&d);
    create(&d, "Original", "T", &["S"], &[]);
    let out = run(&d, &["ui-mockup", "deprecate", "MOCK-001"]);
    assert!(out.status.success());
    let out = run(&d, &["ui-mockup", "edit", "MOCK-001", "--task", "T2"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    let s = stdout(&out);
    assert!(s.contains("<h1>MOCK-001 Original</h1>"));
    assert!(s.contains("<!-- task: T2 -->"));
    assert!(
        d.join("docs/specifications/ui/mockups/deprecated/MOCK-001.html")
            .exists()
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn editor_rewrites() {
    let d = tmpdir("editor");
    init(&d);
    create(&d, "Editor title", "T", &["S"], &[]);

    let script = d.join("editor.sh");
    fs::write(&script, "#!/bin/sh\nprintf '<!doctype html>\\n<html><body><h1>MOCK-001 Edited</h1><p>Status: pending</p><p>Task: T</p><p>UI specifications: S</p><p>References: </p><p>Superseded by: </p></body></html>\\n' > \"$1\"\n").unwrap();
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
            "ui-mockup",
            "edit",
            "MOCK-001",
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
    let content = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    assert!(content.contains("<h1>MOCK-001 Edited</h1>"));

    // missing editor
    let before = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--editor",
        ])
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(record(&d, "MOCK-001")).unwrap(), before);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn sanitization() {
    let d = tmpdir("san");
    init(&d);
    for bad in ["   ", "a/b", "x\\y", "a\nb", "a\rb", "a\tb", "a\u{1}b"] {
        let out = create(&d, bad, "T", &["S"], &[]);
        assert_eq!(out.status.code(), Some(4), "title bad={:?}", bad);
        let out = create(&d, "T", bad, &["S"], &[]);
        assert_eq!(out.status.code(), Some(4), "task bad={:?}", bad);
        let out = create(&d, "T", "T", &[bad], &[]);
        assert_eq!(out.status.code(), Some(4), "spec bad={:?}", bad);
    }
    assert!(!record(&d, "MOCK-001").exists());

    create(&d, "OK", "T", &["S"], &[]);

    for bad in ["a/b", "x\\y", "a\nb", "a\tb", "  "] {
        let out = run(&d, &["ui-mockup", "view", bad]);
        assert_eq!(out.status.code(), Some(4), "id bad={:?}", bad);
    }

    let before = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "edit", "MOCK-001", "--task", "x/y"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "MOCK-001")).unwrap(), before);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn boundary() {
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
            "ui-mockup",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    create(&proj, "Boundary", "T", &["S"], &[]);
    let target = record(&proj, "MOCK-001");
    let outside_file = outside.join("evil.html");
    fs::write(&outside_file, "<html></html>").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }
    let out = run(&proj, &["ui-mockup", "view", "MOCK-001"]);
    assert_eq!(out.status.code(), Some(1));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn disabled_type() {
    let d = tmpdir("disabled");
    init(&d);
    create(&d, "Existing", "T", &["S"], &[]);
    let out = run(
        &d,
        &["config", "set", "document_types.ui-mockup.enabled", "false"],
    );
    assert!(out.status.success());
    let out = create(&d, "Blocked", "T", &["S"], &[]);
    assert_eq!(out.status.code(), Some(4));
    assert!(!record(&d, "MOCK-002").exists());
    let out = run(&d, &["ui-mockup", "list"]);
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn supersession() {
    let d = tmpdir("sup");
    init(&d);
    create(&d, "D1", "T", &["S"], &[]);
    create(&d, "D2", "T", &["S"], &[]);

    let out = run(
        &d,
        &[
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--superseded-by",
            "MOCK-002",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    let s = stdout(&out);
    assert!(s.contains("<p>Status: deprecated</p>"));
    assert!(s.contains("<p>Superseded by: MOCK-002</p>"));
    assert!(!record(&d, "MOCK-001").exists());
    assert!(
        d.join("docs/specifications/ui/mockups/deprecated/MOCK-001.html")
            .exists()
    );

    // self
    // need active mockup
    create(&d, "D3", "T", &["S"], &[]);
    let out = run(
        &d,
        &[
            "ui-mockup",
            "edit",
            "MOCK-003",
            "--superseded-by",
            "MOCK-003",
        ],
    );
    assert_eq!(out.status.code(), Some(4));

    // unknown successor
    let out = run(
        &d,
        &[
            "ui-mockup",
            "edit",
            "MOCK-003",
            "--superseded-by",
            "MOCK-999",
        ],
    );
    assert_eq!(out.status.code(), Some(3));

    // blocked transitions
    let out = run(&d, &["ui-mockup", "restore", "MOCK-001"]);
    assert_eq!(out.status.code(), Some(4));
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "pending"],
    );
    assert_eq!(out.status.code(), Some(4));

    // overwrite superseded-by allowed
    create(&d, "D4", "T", &["S"], &[]);
    let out = run(
        &d,
        &[
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--superseded-by",
            "MOCK-004",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    assert!(stdout(&out).contains("<p>Superseded by: MOCK-004</p>"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn status_transitions() {
    let d = tmpdir("trans");
    init(&d);
    create(&d, "A", "T", &["S"], &[]);
    // pending -> approved
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "approved"],
    );
    assert!(out.status.success());
    // approved -> deprecated via edit
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "deprecated"],
    );
    assert!(out.status.success());
    // deprecated -> pending via restore? first deprecate? already deprecated status but active folder? edit didn't move; restore not applicable. Use deprecate+restore
    let _ = fs::remove_dir_all(&d);
    let d = tmpdir("trans2");
    init(&d);
    create(&d, "A", "T", &["S"], &[]);
    let out = run(&d, &["ui-mockup", "deprecate", "MOCK-001"]);
    assert!(out.status.success());
    let out = run(&d, &["ui-mockup", "restore", "MOCK-001"]);
    assert!(out.status.success());
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    assert!(stdout(&out).contains("<p>Status: pending</p>"));
    // invalid: approved -> pending
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "approved"],
    );
    assert!(out.status.success());
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "pending"],
    );
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn json_shapes() {
    let d = tmpdir("json");
    init(&d);
    let out = run(
        &d,
        &[
            "--json",
            "ui-mockup",
            "create",
            "JSON title",
            "--task",
            "T",
            "--spec",
            "S",
        ],
    );
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "MOCK-001");

    let out = run(&d, &["--json", "ui-mockup", "list"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v[0]["id"], "MOCK-001");
    assert_eq!(v[0]["title"], "JSON title");

    let out = run(&d, &["--json", "ui-mockup", "view", "MOCK-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "MOCK-001");
    assert!(v["path"].as_str().unwrap().ends_with("MOCK-001.html"));
    assert!(v["content"].as_str().unwrap().contains("MOCK-001"));

    let out = run(
        &d,
        &["--json", "ui-mockup", "edit", "MOCK-001", "--task", "T2"],
    );
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "MOCK-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "ui-mockup", "deprecate", "MOCK-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "MOCK-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "ui-mockup", "restore", "MOCK-001"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(v["id"], "MOCK-001");
    assert_eq!(v["status"], "ok");

    let out = run(&d, &["--json", "ui-mockup", "view", "NOPE"]);
    assert_eq!(out.status.code(), Some(3));
    let err: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stderr).trim()).unwrap();
    assert_eq!(err["error"]["code"], 3);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn help() {
    let d = tmpdir("help");
    init(&d);
    let out = run(&d, &["ui-mockup", "--help"]);
    assert!(out.status.success());
    let s = stdout(&out);
    for sub in ["create", "list", "view", "edit", "deprecate", "restore"] {
        assert!(s.contains(sub), "help missing {}", sub);
    }
    let _ = fs::remove_dir_all(&d);
}
