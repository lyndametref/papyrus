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
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dep = d.join("docs/specifications/ui/mockups/archived/MOCK-001.html");
    assert!(dep.exists());
    assert!(!path.exists());
    let out = run(&d, &["ui-mockup", "list"]);
    assert!(!stdout(&out).contains("MOCK-001"));
    let out = run(&d, &["ui-mockup", "view", "MOCK-001"]);
    assert!(out.status.success());

    // archive again is idempotent
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success());

    for args in [
        vec!["ui-mockup", "view", "NOPE"],
        vec!["ui-mockup", "edit", "NOPE", "--title", "x"],
        vec!["ui-mockup", "archive", "NOPE"],
        vec!["ui-mockup", "restore", "NOPE"],
    ] {
        let out = run(&d, &args);
        let expected = if args.contains(&"restore") { 2 } else { 3 };
        assert_eq!(out.status.code(), Some(expected), "args={:?}", args);
    }

    // UI mockups are HTML records outside the Markdown ID layout.
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
    assert_eq!(stdout(&out).trim(), "SPEC-0002");
    assert!(d.join("docs/custom-ui/SPEC-0002.html").exists());

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
fn edit_archived_allowed() {
    let d = tmpdir("editdep");
    init(&d);
    create(&d, "Original", "T", &["S"], &[]);
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
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
        d.join("docs/specifications/ui/mockups/archived/MOCK-001.html")
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
    assert_eq!(out.status.code(), Some(2));
    assert!(!record(&d, "MOCK-002").exists());
    let out = run(&d, &["ui-mockup", "list"]);
    assert_eq!(out.status.code(), Some(2));
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
    assert!(s.contains("<p>Status: archived</p>"));
    assert!(s.contains("<p>Superseded by: MOCK-002</p>"));
    assert!(!record(&d, "MOCK-001").exists());
    assert!(
        d.join("docs/specifications/ui/mockups/archived/MOCK-001.html")
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

    // blocked transitions (restore is no longer a command)
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success());
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

    let out = run(&d, &["--json", "ui-mockup", "archive", "MOCK-001"]);
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
    for sub in [
        "create",
        "list",
        "view",
        "edit",
        "archive",
        "delete",
        "supersede",
    ] {
        assert!(s.contains(sub), "help missing {}", sub);
    }
    assert!(
        !s.contains("create-group"),
        "the grouped-only verb must not appear for the HTML model"
    );
    assert!(
        !s.contains("status"),
        "the fixed-metadata model has no status verb"
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn fixed_metadata_block_carries_the_six_keys_and_starts_pending() {
    let d = tmpdir("metadata");
    init(&d);
    let out = create(
        &d,
        "Metadata",
        "TASK-007",
        &["UI-001", "UI-002"],
        &["REF-1"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "MOCK-001");

    let content = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    assert!(
        !content.starts_with("---"),
        "a mockup carries no YAML front matter"
    );
    for needle in [
        "<!-- status: pending -->",
        "<!-- task: TASK-007 -->",
        "<!-- specs: UI-001 UI-002 -->",
        "<!-- refs: REF-1 -->",
        "<p>References: REF-1</p>",
        "<p>Superseded by: </p>",
    ] {
        assert!(content.contains(needle), "missing {needle} in {content}");
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn approval_is_recorded_by_edit_status_and_archive_is_the_only_archived_writer() {
    let d = tmpdir("approval");
    init(&d);
    create(&d, "Approve me", "T", &["S"], &[]);

    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "approved"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(record(&d, "MOCK-001").exists());
    let content = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    assert!(content.contains("<!-- status: approved -->"), "{content}");

    // `edit --status archived` is not a path to archived.
    let before = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "archived"],
    );
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "MOCK-001")).unwrap(), before);

    // A backwards transition is rejected.
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "pending"],
    );
    assert_eq!(out.status.code(), Some(4));

    // `archive` is the only path to archived for an approved mockup.
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<!-- status: archived -->"), "{archived}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn supersede_archives_the_old_mockup_and_leaves_the_successor_unchanged() {
    let d = tmpdir("supersede-successor");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let successor = fs::read_to_string(record(&d, "MOCK-002")).unwrap();

    let out = run(&d, &["ui-mockup", "supersede", "MOCK-001", "MOCK-002"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let old = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(old.contains("<!-- status: archived -->"), "{old}");
    assert!(old.contains("<p>Superseded by: MOCK-002</p>"), "{old}");
    assert!(!record(&d, "MOCK-001").exists());
    assert_eq!(
        fs::read_to_string(record(&d, "MOCK-002")).unwrap(),
        successor,
        "the successor is unchanged by supersession"
    );

    // A superseded mockup refuses further status transitions.
    let out = run(
        &d,
        &["ui-mockup", "edit", "MOCK-001", "--status", "pending"],
    );
    assert_eq!(out.status.code(), Some(4));
    // Re-archiving an already archived superseded mockup is an idempotent
    // no-op success; status transitions such as --status remain blocked.
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn title_is_required_single_line_and_never_cleared_by_edit() {
    let d = tmpdir("title");
    init(&d);
    let out = create(&d, "", "T", &["S"], &[]);
    assert_eq!(out.status.code(), Some(4));
    assert!(!record(&d, "MOCK-001").exists());
    let out = create(&d, "a\nb", "T", &["S"], &[]);
    assert_eq!(out.status.code(), Some(4));
    assert!(!record(&d, "MOCK-001").exists());

    create(&d, "Keep", "T", &["S"], &[]);
    let before = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "edit", "MOCK-001", "--title", ""]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read_to_string(record(&d, "MOCK-001")).unwrap(), before);
    let _ = fs::remove_dir_all(&d);
}

fn archived_record(d: &std::path::Path, id: &str) -> PathBuf {
    d.join("docs/specifications/ui/mockups/archived")
        .join(format!("{}.html", id))
}

fn mockups_dir(d: &std::path::Path) -> PathBuf {
    d.join("docs/specifications/ui/mockups")
}

fn assert_no_temp_files(dir: &std::path::Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "leftover temp file: {}", name);
    }
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn archive_supersede_and_edit_with_move_write_then_unlink() {
    // archive: only the archived copy remains, rendered with status archived.
    let d = tmpdir("seq-archive");
    init(&d);
    create(&d, "Old", "TASK-A", &["UI-001"], &[]);
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!record(&d, "MOCK-001").exists());
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<!-- status: archived -->"));
    assert!(archived.contains("<!-- task: TASK-A -->"));
    assert!(archived.contains("<!-- specs: UI-001 -->"));
    let _ = fs::remove_dir_all(&d);

    // supersede: back-reference is present in the archived destination.
    let d = tmpdir("seq-supersede");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let out = run(&d, &["ui-mockup", "supersede", "MOCK-001", "MOCK-002"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!record(&d, "MOCK-001").exists());
    assert!(record(&d, "MOCK-002").exists());
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<!-- status: archived -->"));
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    let _ = fs::remove_dir_all(&d);

    // edit --superseded-by: same destination-write-then-unlink sequence.
    let d = tmpdir("seq-edit-move");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
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
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!record(&d, "MOCK-001").exists());
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<!-- status: archived -->"));
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    let _ = fs::remove_dir_all(&d);

    // plain edit: one in-place write, nothing moved or unlinked.
    let d = tmpdir("seq-edit-plain");
    init(&d);
    create(&d, "Plain", "T", &["S"], &[]);
    let before = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "edit", "MOCK-001", "--task", "T2"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(record(&d, "MOCK-001").exists());
    assert!(!archived_record(&d, "MOCK-001").exists());
    let after = fs::read_to_string(record(&d, "MOCK-001")).unwrap();
    assert_ne!(after, before);
    assert!(after.contains("<!-- task: T2 -->"));
    let _ = fs::remove_dir_all(&d);

    // supersede of an already-archived record writes in place and unlinks nothing.
    let d = tmpdir("seq-supersede-archived");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    assert!(
        run(&d, &["ui-mockup", "archive", "MOCK-001"])
            .status
            .success()
    );
    let out = run(&d, &["ui-mockup", "supersede", "MOCK-001", "MOCK-002"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(archived_record(&d, "MOCK-001").exists());
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    let _ = fs::remove_dir_all(&d);

    // edit-with-move of an already-archived record writes in place.
    let d = tmpdir("seq-edit-archived");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    assert!(
        run(&d, &["ui-mockup", "archive", "MOCK-001"])
            .status
            .success()
    );
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
    assert!(out.status.success(), "{}", stderr(&out));
    let archived = fs::read_to_string(archived_record(&d, "MOCK-001")).unwrap();
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    assert!(!record(&d, "MOCK-001").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn destination_write_failure_keeps_source_intact() {
    // archive
    let d = tmpdir("dw-archive");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    let before = fs::read(record(&d, "MOCK-001")).unwrap();
    fs::create_dir_all(archived_record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(fs::read(record(&d, "MOCK-001")).unwrap(), before);
    assert!(archived_record(&d, "MOCK-001").is_dir());
    assert_no_temp_files(&mockups_dir(&d).join("archived"));
    let _ = fs::remove_dir_all(&d);

    // supersede
    let d = tmpdir("dw-supersede");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let before = fs::read(record(&d, "MOCK-001")).unwrap();
    fs::create_dir_all(archived_record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "supersede", "MOCK-001", "MOCK-002"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(fs::read(record(&d, "MOCK-001")).unwrap(), before);
    assert_no_temp_files(&mockups_dir(&d).join("archived"));
    let _ = fs::remove_dir_all(&d);

    // edit with --superseded-by
    let d = tmpdir("dw-edit");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let before = fs::read(record(&d, "MOCK-001")).unwrap();
    fs::create_dir_all(archived_record(&d, "MOCK-001")).unwrap();
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
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(fs::read(record(&d, "MOCK-001")).unwrap(), before);
    assert_no_temp_files(&mockups_dir(&d).join("archived"));
    let _ = fs::remove_dir_all(&d);

    // delete (archive-first step fails, so nothing is removed)
    let d = tmpdir("dw-delete");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    let before = fs::read(record(&d, "MOCK-001")).unwrap();
    fs::create_dir_all(archived_record(&d, "MOCK-001")).unwrap();
    let out = run(&d, &["ui-mockup", "delete", "MOCK-001", "--force"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(fs::read(record(&d, "MOCK-001")).unwrap(), before);
    assert_no_temp_files(&mockups_dir(&d).join("archived"));
    let _ = fs::remove_dir_all(&d);
}

#[cfg(unix)]
fn permission_bits_are_enforced(probe: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let dir = probe.join("ro-probe");
    fs::create_dir_all(&dir).unwrap();
    let mut perms = fs::metadata(&dir).unwrap().permissions();
    perms.set_mode(0o555);
    fs::set_permissions(&dir, perms).unwrap();
    let enforced = fs::write(dir.join("blocked"), b"x").is_err();
    let mut perms = fs::metadata(&dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dir, perms).unwrap();
    let _ = fs::remove_dir_all(&dir);
    enforced
}

#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms).unwrap();
}

#[cfg(unix)]
#[test]
fn unlink_failure_keeps_both_copies_and_names_both_paths() {
    let probe = tmpdir("ul-probe");
    if !permission_bits_are_enforced(&probe) {
        println!(
            "SKIP unlink-failure test: permission bits are not enforced (privileged/root run)"
        );
        let _ = fs::remove_dir_all(&probe);
        return;
    }
    let _ = fs::remove_dir_all(&probe);

    // archive
    let d = tmpdir("ul-archive");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    let source = record(&d, "MOCK-001");
    let dest = archived_record(&d, "MOCK-001");
    let before = fs::read(&source).unwrap();
    let active_dir = mockups_dir(&d);
    fs::create_dir_all(active_dir.join("archived")).unwrap();
    set_mode(&active_dir, 0o555);
    let out = run(&d, &["ui-mockup", "archive", "MOCK-001"]);
    set_mode(&active_dir, 0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains(&fs::canonicalize(&source).unwrap().display().to_string()),
        "stderr must name the source: {}",
        err
    );
    assert!(
        err.contains(&fs::canonicalize(&dest).unwrap().display().to_string()),
        "stderr must name the destination: {}",
        err
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    assert!(
        fs::read_to_string(&source)
            .unwrap()
            .contains("<!-- status: pending -->")
    );
    assert!(
        fs::read_to_string(&dest)
            .unwrap()
            .contains("<!-- status: archived -->")
    );
    assert_no_temp_files(&active_dir.join("archived"));
    let _ = fs::remove_dir_all(&d);

    // supersede
    let d = tmpdir("ul-supersede");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let source = record(&d, "MOCK-001");
    let dest = archived_record(&d, "MOCK-001");
    let before = fs::read(&source).unwrap();
    let active_dir = mockups_dir(&d);
    fs::create_dir_all(active_dir.join("archived")).unwrap();
    set_mode(&active_dir, 0o555);
    let out = run(&d, &["ui-mockup", "supersede", "MOCK-001", "MOCK-002"]);
    set_mode(&active_dir, 0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains(&fs::canonicalize(&source).unwrap().display().to_string()),
        "stderr must name the source: {}",
        err
    );
    assert!(
        err.contains(&fs::canonicalize(&dest).unwrap().display().to_string()),
        "stderr must name the destination: {}",
        err
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    let archived = fs::read_to_string(&dest).unwrap();
    assert!(archived.contains("<!-- status: archived -->"));
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    assert_no_temp_files(&active_dir.join("archived"));
    let _ = fs::remove_dir_all(&d);

    // edit with --superseded-by
    let d = tmpdir("ul-edit");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    create(&d, "New", "T", &["S"], &[]);
    let source = record(&d, "MOCK-001");
    let dest = archived_record(&d, "MOCK-001");
    let before = fs::read(&source).unwrap();
    let active_dir = mockups_dir(&d);
    fs::create_dir_all(active_dir.join("archived")).unwrap();
    set_mode(&active_dir, 0o555);
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
    set_mode(&active_dir, 0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains(&fs::canonicalize(&source).unwrap().display().to_string()),
        "stderr must name the source: {}",
        err
    );
    assert!(
        err.contains(&fs::canonicalize(&dest).unwrap().display().to_string()),
        "stderr must name the destination: {}",
        err
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    let archived = fs::read_to_string(&dest).unwrap();
    assert!(archived.contains("<p>Superseded by: MOCK-002</p>"));
    assert_no_temp_files(&active_dir.join("archived"));
    let _ = fs::remove_dir_all(&d);

    // delete --force (archive step fails, nothing is removed)
    let d = tmpdir("ul-delete");
    init(&d);
    create(&d, "Old", "T", &["S"], &[]);
    let source = record(&d, "MOCK-001");
    let dest = archived_record(&d, "MOCK-001");
    let before = fs::read(&source).unwrap();
    let active_dir = mockups_dir(&d);
    fs::create_dir_all(active_dir.join("archived")).unwrap();
    set_mode(&active_dir, 0o555);
    let out = run(&d, &["ui-mockup", "delete", "MOCK-001", "--force"]);
    set_mode(&active_dir, 0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains(&fs::canonicalize(&source).unwrap().display().to_string()),
        "stderr must name the source: {}",
        err
    );
    assert!(
        err.contains(&fs::canonicalize(&dest).unwrap().display().to_string()),
        "stderr must name the destination: {}",
        err
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    assert!(dest.exists(), "archive wrote the destination before unlink");
    assert_no_temp_files(&active_dir.join("archived"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn archive_of_archived_mockup_is_a_noop_success() {
    let d = tmpdir("noop");
    init(&d);
    create(&d, "Already archived", "T", &["S"], &[]);
    assert!(
        run(&d, &["ui-mockup", "archive", "MOCK-001"])
            .status
            .success()
    );

    let path = archived_record(&d, "MOCK-001");
    let before = fs::read(&path).unwrap();
    let before_mtime = fs::metadata(&path).unwrap().modified().unwrap();

    let out = run(&d, &["--json", "ui-mockup", "archive", "MOCK-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let value: serde_json::Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["id"], "MOCK-001");

    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        before_mtime,
        "no-op archive must not change the modification time"
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn delete_removes_only_the_archived_copy() {
    // active delete: archive first, then remove the archived copy.
    let d = tmpdir("del-active");
    init(&d);
    create(&d, "Active", "T", &["S"], &[]);
    let out = run(&d, &["ui-mockup", "delete", "MOCK-001", "--force"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!record(&d, "MOCK-001").exists());
    assert!(!archived_record(&d, "MOCK-001").exists());
    let _ = fs::remove_dir_all(&d);

    // archived delete removes only the archived copy.
    let d = tmpdir("del-archived");
    init(&d);
    create(&d, "Archived", "T", &["S"], &[]);
    assert!(
        run(&d, &["ui-mockup", "archive", "MOCK-001"])
            .status
            .success()
    );
    let out = run(&d, &["ui-mockup", "delete", "MOCK-001", "--force"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!archived_record(&d, "MOCK-001").exists());
    assert!(!record(&d, "MOCK-001").exists());
    let _ = fs::remove_dir_all(&d);

    // unknown id stays not-found.
    let d = tmpdir("del-unknown");
    init(&d);
    let out = run(&d, &["ui-mockup", "delete", "MOCK-404", "--force"]);
    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn source_guard_uses_only_the_shared_archive_sequence() {
    let source = include_str!("../src/ui_mockup.rs");
    let forbidden_move = concat!("move_", "record");
    let forbidden_rename = concat!("fs::", "rename");
    assert!(
        !source.contains(forbidden_move),
        "src/ui_mockup.rs must not call the move-then-write helper"
    );
    assert!(
        !source.contains(forbidden_rename),
        "src/ui_mockup.rs must not rename record files directly"
    );
    assert!(
        source.contains("archive_content"),
        "src/ui_mockup.rs must route moves through the shared archive sequence"
    );
}
