use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("audit-{}-{}", name, std::process::id()));
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
    content: &str,
    audit_type: Option<&str>,
) -> std::process::Output {
    let mut args = vec!["audit", "create", "--title", title, "--content", content];
    if let Some(t) = audit_type {
        args.push("--audit-type");
        args.push(t);
    }
    run(d, &args)
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn record(d: &std::path::Path, id: &str) -> PathBuf {
    d.join("docs/audit-records").join(format!("{}.md", id))
}

#[test]
fn audit_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);

    let out = create(&d, "First Audit", "content text", None);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out).trim(), "AUD-001");

    let out = run(&d, &["audit", "list"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("AUD-001 First Audit"));

    let out = run(&d, &["audit", "view", "AUD-001"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("# AUD-001 First Audit"));
    assert!(s.contains("## Audit Type\nsecurity"));
    assert!(s.contains("## Content\ncontent text"));

    let out = run(
        &d,
        &["audit", "edit", "AUD-001", "--title", "First Audit Updated"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["audit", "view", "AUD-001"]);
    let s = stdout(&out);
    assert!(s.contains("# AUD-001 First Audit Updated"));
    assert!(s.contains("## Content\ncontent text"));

    let out = run(&d, &["audit", "edit", "AUD-001", "--content", "content2"]);
    assert!(out.status.success());
    let out = run(&d, &["audit", "view", "AUD-001"]);
    let s = stdout(&out);
    assert!(s.contains("## Content\ncontent2"));
    assert!(s.contains("## Audit Type\nsecurity"));

    let out = run(&d, &["audit", "edit", "AUD-001", "--audit-type", "licence"]);
    assert!(out.status.success());
    let out = run(&d, &["audit", "view", "AUD-001"]);
    let s = stdout(&out);
    assert!(s.contains("## Audit Type\nlicence"));

    let out = run(&d, &["audit", "deprecate", "AUD-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("docs/audit-records/deprecated/AUD-001.md").exists());
    let out = run(&d, &["audit", "list"]);
    assert!(!stdout(&out).contains("AUD-001"));
    let out = run(&d, &["audit", "view", "AUD-001"]);
    assert!(out.status.success());

    let out = run(&d, &["audit", "deprecate", "AUD-001"]);
    assert_eq!(out.status.code(), Some(4));

    let out = run(&d, &["audit", "restore", "AUD-001"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("docs/audit-records/AUD-001.md").exists());
    assert!(!d.join("docs/audit-records/deprecated/AUD-001.md").exists());
    let out = run(&d, &["audit", "list"]);
    assert!(stdout(&out).contains("AUD-001"));

    let out = run(&d, &["audit", "restore", "AUD-001"]);
    assert_eq!(out.status.code(), Some(4));

    let out = run(&d, &["audit", "view", "AUD-999"]);
    assert_eq!(out.status.code(), Some(3));
    let out = run(&d, &["audit", "deprecate", "AUD-999"]);
    assert_eq!(out.status.code(), Some(3));
    let out = run(&d, &["audit", "restore", "AUD-999"]);
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_id_not_reused() {
    let d = tmpdir("reuse");
    init(&d);
    create(&d, "A", "d", None);
    create(&d, "B", "d", None);
    let out = run(&d, &["audit", "deprecate", "AUD-001"]);
    assert!(out.status.success());
    let out = create(&d, "C", "d", None);
    assert_eq!(stdout(&out).trim(), "AUD-003");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_types_and_custom() {
    let d = tmpdir("types");
    init(&d);
    assert!(create(&d, "Sec", "d", Some("security")).status.success());
    assert!(create(&d, "Lic", "d", Some("licence")).status.success());
    assert!(
        create(&d, "Arch", "d", Some("architecture"))
            .status
            .success()
    );
    assert!(
        create(&d, "Custom", "d", Some("custom-type"))
            .status
            .success()
    );

    let out = run(&d, &["audit", "view", "AUD-004"]);
    let s = stdout(&out);
    assert!(s.contains("## Audit Type\ncustom-type"));

    let out = run(&d, &["audit", "view", "AUD-001"]);
    let s = stdout(&out);
    assert!(s.contains("## Audit Type\nsecurity"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_edit_deprecated_allowed() {
    let d = tmpdir("editdep");
    init(&d);
    create(&d, "T", "d", None);
    let _ = run(&d, &["audit", "deprecate", "AUD-001"]);
    let out = run(
        &d,
        &[
            "audit",
            "edit",
            "AUD-001",
            "--title",
            "T2",
            "--content",
            "d2",
            "--audit-type",
            "architecture",
        ],
    );
    assert!(out.status.success());
    let out = run(&d, &["audit", "view", "AUD-001"]);
    let s = stdout(&out);
    assert!(s.contains("# AUD-001 T2"));
    assert!(s.contains("## Audit Type\narchitecture"));
    assert!(s.contains("## Content\nd2"));
    let out = run(&d, &["audit", "list"]);
    assert!(!stdout(&out).contains("AUD-001"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_sanitization() {
    let d = tmpdir("san");
    init(&d);

    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = create(&d, bad, "ok", None);
        assert_eq!(out.status.code(), Some(4), "title bad={}", bad);
    }

    let out = create(&d, "OK", "x/y", None);
    assert_eq!(out.status.code(), Some(4));

    let out = create(&d, "OK", "ok", Some("a/b"));
    assert_eq!(out.status.code(), Some(4));

    let out = create(&d, "OK", "ok", Some("security"));
    assert!(out.status.success());

    let out = run(&d, &["audit", "view", "a/b"]);
    assert_eq!(out.status.code(), Some(4));

    let out = run(&d, &["audit", "edit", "AUD-001", "--title", "x\\y"]);
    assert_eq!(out.status.code(), Some(4));

    let out = run(&d, &["audit", "edit", "AUD-001", "--content", "a/b"]);
    assert_eq!(out.status.code(), Some(4));

    let out = run(&d, &["audit", "edit", "AUD-001", "--audit-type", "a/b"]);
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_project_directory_boundary() {
    let root = tmpdir("bnd");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let _ = bin()
        .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();

    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "--config",
            outside.join("evil.yml").to_str().unwrap(),
            "audit",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let p = proj.to_str().unwrap();
    let _ = create(&proj, "D", "d", None);

    let target = proj.join("docs/audit-records/AUD-001.md");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "secret").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }
    let out = bin()
        .args(["--dir", p, "audit", "view", "AUD-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let out = bin()
        .args(["--dir", p, "audit", "deprecate", "AUD-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn audit_json_output() {
    let d = tmpdir("json");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "--json",
            "audit",
            "create",
            "--title",
            "Foo",
            "--content",
            "def",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        r#"{"id":"AUD-001"}"#
    );

    let out = bin()
        .args(["--dir", p, "--json", "audit", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"id\":\"AUD-001\""));

    let out = bin()
        .args(["--dir", p, "--json", "audit", "view", "AUD-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"content\""));

    let out = bin()
        .args([
            "--dir", p, "--json", "audit", "edit", "AUD-001", "--title", "Foo2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        r#"{"status":"ok","id":"AUD-001"}"#
    );

    let out = bin()
        .args(["--dir", p, "--json", "audit", "deprecate", "AUD-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        r#"{"status":"ok","id":"AUD-001"}"#
    );

    let out = bin()
        .args(["--dir", p, "--json", "audit", "restore", "AUD-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        r#"{"status":"ok","id":"AUD-001"}"#
    );

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_missing_args_exit_2() {
    let d = tmpdir("missing");
    init(&d);
    let out = run(&d, &["audit", "create", "--content", "d"]);
    assert_eq!(out.status.code(), Some(2));
    let out = run(&d, &["audit", "create", "--title", "t"]);
    assert_eq!(out.status.code(), Some(2));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_disabled_type_exit_4() {
    let d = tmpdir("disabled");
    init(&d);
    let out = run(
        &d,
        &["config", "set", "document_types.audit.enabled", "false"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&d, &["audit", "list"]);
    assert_eq!(out.status.code(), Some(4));
    let out = run(&d, &["audit", "create", "--title", "t", "--content", "c"]);
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_editor_with_script() {
    let d = tmpdir("editscript");
    init(&d);
    create(&d, "Editor Audit", "d", None);

    let script = d.join("editor.sh");
    fs::write(&script, "#!/bin/sh\nprintf '# AUD-001 Edited title\\n\\n## Audit Type\\nsecurity\\n\\n## Content\\nd-edited\\n' > \"$1\"\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "audit",
            "edit",
            "AUD-001",
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
    let content = fs::read_to_string(record(&d, "AUD-001")).unwrap();
    assert!(content.starts_with("# AUD-001 Edited title"));
    assert!(content.contains("## Audit Type\nsecurity"));
    assert!(content.contains("## Content\nd-edited"));

    let before2 = fs::read_to_string(record(&d, "AUD-001")).unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "audit",
            "edit",
            "AUD-001",
            "--title",
            "ShouldNotApply",
            "--editor",
        ])
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(record(&d, "AUD-001")).unwrap(), before2);

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_non_recursive_scan() {
    let d = tmpdir("nonrec");
    init(&d);
    let subdir = d.join("docs/audit-records/sub");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("AUD-999.md"), "# AUD-999 Hidden\n").unwrap();
    fs::write(
        d.join("docs/audit-records/BR-001.md"),
        "# BR-001 canonical\n",
    )
    .unwrap();

    create(&d, "Top", "d", None);
    let out = run(&d, &["audit", "list"]);
    let s = stdout(&out);
    assert!(s.contains("AUD-001 Top"));
    assert!(!s.contains("AUD-999"));
    assert!(!s.contains("BR-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn audit_restore_unknown_exit_3() {
    let d = tmpdir("restunk");
    init(&d);
    let out = run(&d, &["audit", "restore", "NOPE"]);
    assert_eq!(out.status.code(), Some(3));
    let out = run(&d, &["audit", "deprecate", "NOPE"]);
    assert_eq!(out.status.code(), Some(3));
    let _ = fs::remove_dir_all(&d);
}
