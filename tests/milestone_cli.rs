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
    assert!(s.contains("release: REL-001"), "got: {}", s);

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
    assert!(!s.contains("## Release"), "got: {}", s);
    assert!(!s.contains("release: REL-001"), "got: {}", s);

    // close open -> closed, in place
    let out = bin()
        .args(["--dir", p, "milestone", "close", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "MS-001");
    let out = bin()
        .args(["--dir", p, "milestone", "view", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("status: closed"));

    // a closed milestone stays listed
    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("MS-001"));

    // delete closed with force
    let out = bin()
        .args(["--dir", p, "milestone", "delete", "MS-001", "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // list hides the deleted milestone
    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("MS-001"));

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

    // 4: close of a closed milestone
    let _ = bin()
        .args(["--dir", p, "milestone", "create", "M2"])
        .output();
    let _ = bin()
        .args(["--dir", p, "milestone", "close", "MS-001"])
        .output();
    let out = bin()
        .args(["--dir", p, "milestone", "close", "MS-001"])
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

#[test]
fn milestone_close_error_paths() {
    let d = tmpdir("close-errors");
    init(&d);
    let p = d.to_str().unwrap();

    let out = bin()
        .args(["--dir", p, "milestone", "create", "M1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "MS-001");

    let path = d.join("backlog/milestones/MS-001.md");
    let created = fs::read_to_string(&path).unwrap();
    assert!(
        created.contains("status: open"),
        "create must seed status: open, got: {created}"
    );
    let date_line = created
        .lines()
        .find(|line| line.starts_with("updated_date:"))
        .expect("updated_date line")
        .to_string();
    let aged = created.replacen(&date_line, "updated_date: '2020-01-01T00:00:00Z'", 1);
    fs::write(&path, &aged).unwrap();

    let out = bin()
        .args(["--dir", p, "milestone", "close", "MS-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "MS-001");
    let closed = fs::read_to_string(&path).unwrap();
    assert!(closed.contains("status: closed"), "got: {closed}");
    assert!(
        !closed.contains("updated_date: '2020-01-01T00:00:00Z'"),
        "close must refresh updated_date, got: {closed}"
    );
    assert!(closed.contains("created_date:"), "got: {closed}");

    // JSON form for an open milestone.
    let out = bin()
        .args(["--dir", p, "milestone", "create", "M2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "--json", "milestone", "close", "MS-002"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["id"], "MS-002");
    assert_eq!(v["status"], "closed");

    // Closing a closed milestone exits 4, names the status, writes nothing.
    let before = fs::read(&path).unwrap();
    let out = bin()
        .args(["--dir", p, "milestone", "close", "MS-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("closed"), "stderr: {stderr}");
    assert!(stderr.contains("MS-001"), "stderr: {stderr}");
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "closed close must not write"
    );

    // Unknown ID exits 3 with the standard not-found message.
    let out = bin()
        .args(["--dir", p, "milestone", "close", "MS-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not found: MS-999"));

    // Removed milestone verbs are unknown subcommands.
    for verb in ["archive", "restore"] {
        let out = bin()
            .args(["--dir", p, "milestone", verb, "MS-001"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "milestone {verb} must exit 2");
    }
    let help =
        String::from_utf8(bin().args(["milestone", "--help"]).output().unwrap().stdout).unwrap();
    assert!(help.contains("close"), "help: {help}");
    assert!(!help.contains("archive"), "help: {help}");
    assert!(!help.contains("restore"), "help: {help}");

    let _ = fs::remove_dir_all(&d);
}
