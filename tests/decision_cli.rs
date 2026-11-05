use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-dectest-{}", name));
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

#[test]
fn decision_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);
    let p = d.to_str().unwrap();

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "First Decision",
            "-d",
            "dec text",
            "--context",
            "ctx text",
            "--consequences",
            "cons text",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ADR-001");

    assert!(!d.join("decisions/archived").exists());

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "Biz Decision",
            "--decision-type",
            "bdr",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "BDR-001");

    let out = bin()
        .args(["--dir", p, "decision", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ADR-001 First Decision"));
    assert!(s.contains("BDR-001 Biz Decision"));

    assert!(!d.join("decisions/archived").exists());

    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# ADR-001 First Decision"));
    assert!(s.contains("## Status\nproposed"));
    assert!(s.contains("## Decision\ndec text"));
    assert!(s.contains("## Context\nctx text"));
    assert!(s.contains("## Consequences\ncons text"));

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-001",
            "--title",
            "First Decision Updated",
            "-d",
            "dec2",
            "--context",
            "ctx2",
            "--consequences",
            "cons2",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(!d.join("decisions/archived").exists());

    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# ADR-001 First Decision Updated"));
    assert!(s.contains("## Decision\ndec2"));
    assert!(s.contains("## Context\nctx2"));
    assert!(s.contains("## Consequences\ncons2"));

    let out = bin()
        .args(["--dir", p, "decision", "archive", "ADR-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(d.join("decisions/archived").exists());
    assert!(d.join("decisions/archived/ADR-001.md").exists());

    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = bin()
        .args(["--dir", p, "decision", "list"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("ADR-001"));

    let out = bin()
        .args(["--dir", p, "decision", "archive", "ADR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "decision", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_status_transitions() {
    let d = tmpdir("status");
    init(&d);
    let p = d.to_str().unwrap();

    let _ = bin()
        .args(["--dir", p, "decision", "create", "D"])
        .output()
        .unwrap();

    let out = bin()
        .args([
            "--dir", p, "decision", "edit", "ADR-001", "--status", "accepted",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-001",
            "--status",
            "deprecated",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("## Status\ndeprecated"));

    let _ = bin()
        .args(["--dir", p, "decision", "create", "D2"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-002",
            "--status",
            "deprecated",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args([
            "--dir", p, "decision", "edit", "ADR-002", "--status", "bogus",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args([
            "--dir", p, "decision", "edit", "ADR-002", "--status", "proposed",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_supersession() {
    let d = tmpdir("sup");
    init(&d);
    let p = d.to_str().unwrap();

    let _ = bin()
        .args(["--dir", p, "decision", "create", "D1"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "decision", "create", "D2"])
        .output()
        .unwrap();

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-001",
            "--superseded-by",
            "ADR-002",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("## Status\nSuperseded by ADR-002"));

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-002",
            "--superseded-by",
            "ADR-002",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-002",
            "--superseded-by",
            "ADR-999",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "B",
            "--decision-type",
            "bdr",
        ])
        .output()
        .unwrap();
    let _ = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "B2",
            "--decision-type",
            "bdr",
        ])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-002",
            "--superseded-by",
            "BDR-001",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "edit",
            "ADR-002",
            "--status",
            "accepted",
            "--superseded-by",
            "ADR-001",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_type_filtering() {
    let d = tmpdir("filter");
    init(&d);
    let p = d.to_str().unwrap();

    let _ = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "A",
            "--decision-type",
            "adr",
        ])
        .output()
        .unwrap();
    let _ = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "B",
            "--decision-type",
            "bdr",
        ])
        .output()
        .unwrap();

    let out = bin()
        .args(["--dir", p, "decision", "list", "--decision-type", "adr"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ADR-001"));
    assert!(!s.contains("BDR-001"));

    let out = bin()
        .args(["--dir", p, "decision", "list", "--decision-type", "bdr"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("BDR-001"));
    assert!(!s.contains("ADR-001"));

    let out = bin()
        .args(["--dir", p, "decision", "list", "--decision-type", "nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "decision", "list"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ADR-001"));
    assert!(s.contains("BDR-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_sanitization() {
    let d = tmpdir("san");
    init(&d);
    let p = d.to_str().unwrap();

    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args(["--dir", p, "decision", "create", bad])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "title bad={}", bad);
    }

    let out = bin()
        .args([
            "--dir",
            p,
            "decision",
            "create",
            "OK",
            "--decision-type",
            "a/b",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = bin()
        .args(["--dir", p, "decision", "create", "OK"])
        .output()
        .unwrap();

    let out = bin()
        .args(["--dir", p, "decision", "view", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "decision", "edit", "ADR-001", "--status", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "decision", "edit", "ADR-001", "--title", "x\\y"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_id_not_reused() {
    let d = tmpdir("reuse");
    init(&d);
    let p = d.to_str().unwrap();

    let _ = bin()
        .args(["--dir", p, "decision", "create", "D1"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "decision", "archive", "ADR-001"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "decision", "create", "D2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ADR-002");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn decision_project_directory_boundary() {
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
            "decision",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let p = proj.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "decision", "create", "D"])
        .output()
        .unwrap();

    let target = proj.join("decisions/ADR-001.md");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "secret").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }
    let out = bin()
        .args(["--dir", p, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let out = bin()
        .args(["--dir", p, "decision", "archive", "ADR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let _ = fs::remove_dir_all(&root);
}
