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
            "--dir", p, "decision", "edit", "ADR-002", "--status", "archived",
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
