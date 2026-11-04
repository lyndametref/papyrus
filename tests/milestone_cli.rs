use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-mscli-{}", name));
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
fn milestone_and_release_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);
    let p = d.to_str().unwrap();

    // release create
    let out = bin()
        .args(["--dir", p, "release", "create", "R1", "-d", "rdesc"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REL-001");

    // milestone create with release
    let out = bin()
        .args([
            "--dir",
            p,
            "milestone",
            "create",
            "M1",
            "-d",
            "mdesc",
            "--release",
            "REL-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "MS-001");

    // list
    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("MS-001 M1"));

    let out = bin()
        .args(["--dir", p, "release", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("REL-001 R1"));

    // view
    let out = bin()
        .args(["--dir", p, "milestone", "view", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("## Description\nmdesc"));
    assert!(s.contains("## Release\nREL-001"));

    // edit remove release
    let out = bin()
        .args([
            "--dir",
            p,
            "milestone",
            "edit",
            "MS-001",
            "--no-release",
            "REL-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "milestone", "view", "MS-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("## Release"));

    // archive + view archived
    let out = bin()
        .args(["--dir", p, "milestone", "archive", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "milestone", "view", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // list hides archived
    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("MS-001"));

    // delete archived with force
    let out = bin()
        .args(["--dir", p, "milestone", "delete", "MS-001", "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // release edit
    let out = bin()
        .args([
            "--dir", p, "release", "edit", "REL-001", "--title", "R2", "-d", "d2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "release", "view", "REL-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# REL-001 R2"));
    assert!(s.contains("## Description\nd2"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn milestone_exit_codes() {
    let d = tmpdir("exit");
    init(&d);
    let p = d.to_str().unwrap();

    // 2: missing title (usage)
    let out = bin()
        .args(["--dir", p, "milestone", "create"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    // 3: unknown
    let out = bin()
        .args(["--dir", p, "milestone", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    // 4: sanitization
    let out = bin()
        .args(["--dir", p, "milestone", "create", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: mutual exclusion
    let _ = bin().args(["--dir", p, "release", "create", "R"]).output();
    let _ = bin()
        .args(["--dir", p, "milestone", "create", "M"])
        .output();
    let out = bin()
        .args([
            "--dir",
            p,
            "milestone",
            "edit",
            "MS-001",
            "--release",
            "REL-001",
            "--no-release",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: already archived
    let _ = bin()
        .args(["--dir", p, "milestone", "create", "M2"])
        .output();
    let _ = bin()
        .args(["--dir", p, "milestone", "archive", "MS-001"])
        .output();
    let out = bin()
        .args(["--dir", p, "milestone", "archive", "MS-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn release_unknown_and_json() {
    let d = tmpdir("reljson");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin().args(["--dir", p, "release", "create", "R"]).output();
    let out = bin()
        .args(["--dir", p, "--json", "release", "view", "REL-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["id"], "REL-001");

    let out = bin()
        .args(["--dir", p, "release", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}
