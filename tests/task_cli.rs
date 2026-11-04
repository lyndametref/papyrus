use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-taskcli-{}", name));
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
fn task_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);
    let p = d.to_str().unwrap();

    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "create",
            "T1",
            "-d",
            "desc",
            "--ac",
            "ac1",
            "--label",
            "l",
            "--assignee",
            "alice",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "TASK-001");

    let out = bin()
        .args(["--dir", p, "milestone", "create", "M1"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "create",
            "T2",
            "--depends-on",
            "TASK-001",
            "--milestone",
            "MS-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "TASK-002");

    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("TASK-001 T1"));
    assert!(s.contains("TASK-002 T2"));

    let out = bin()
        .args(["--dir", p, "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("## Description\ndesc"));
    assert!(s.contains("## Assignee\nalice"));
    assert!(s.contains("## Labels\n- l"));
    assert!(s.contains("## Acceptance Criteria\n- [ ] ac1"));

    // shorthand
    let out = bin()
        .args(["--dir", p, "task", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // edit
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "edit",
            "TASK-001",
            "--title",
            "T1b",
            "--status",
            "Initiated",
            "--ac",
            "ac2",
            "--comment",
            "hi",
            "--comment-author",
            "bob",
            "--label",
            "x",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "task", "view", "TASK-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# TASK-001 T1b"));
    assert!(s.contains("## Status\nInitiated"));
    assert!(s.contains("- [ ] ac2"));
    assert!(s.contains("- **bob**: hi"));
    assert!(s.contains("- x"));

    // archive/restore
    let out = bin()
        .args(["--dir", p, "task", "archive", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("TASK-001"));
    let out = bin()
        .args(["--dir", p, "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "task", "restore", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("TASK-001"));

    // delete
    let out = bin()
        .args(["--dir", p, "task", "delete", "TASK-001", "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("TASK-001"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn task_exit_codes() {
    let d = tmpdir("exit");
    init(&d);
    let p = d.to_str().unwrap();

    // 2: missing title
    let out = bin().args(["--dir", p, "task", "create"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));

    // 3: unknown view
    let out = bin()
        .args(["--dir", p, "task", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    // 4: sanitization
    let out = bin()
        .args(["--dir", p, "task", "create", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: unknown status
    let _ = bin().args(["--dir", p, "task", "create", "T"]).output();
    let out = bin()
        .args(["--dir", p, "task", "edit", "TASK-001", "--status", "Nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: mutual exclusion
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "edit",
            "TASK-001",
            "--milestone",
            "MS-001",
            "--no-milestone",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: already archived
    let _ = bin()
        .args(["--dir", p, "task", "archive", "TASK-001"])
        .output();
    let out = bin()
        .args(["--dir", p, "task", "archive", "TASK-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: unknown sort
    let out = bin()
        .args(["--dir", p, "task", "list", "--sort", "foo"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 2: invalid limit
    let out = bin()
        .args(["--dir", p, "task", "list", "--limit", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn task_json_and_filters() {
    let d = tmpdir("json");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin().args(["--dir", p, "release", "create", "R"]).output();
    let _ = bin()
        .args([
            "--dir",
            p,
            "milestone",
            "create",
            "M",
            "--release",
            "REL-001",
        ])
        .output();
    let _ = bin()
        .args([
            "--dir",
            p,
            "task",
            "create",
            "Alpha",
            "--milestone",
            "MS-001",
            "--label",
            "bug",
            "--assignee",
            "alice",
        ])
        .output();
    let _ = bin()
        .args(["--dir", p, "task", "create", "Beta", "--label", "feat"])
        .output();

    let out = bin()
        .args(["--dir", p, "--json", "task", "list", "--release", "REL-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);

    let out = bin()
        .args(["--dir", p, "--json", "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["id"], "TASK-001");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn task_dependency_and_milestone_validation() {
    let d = tmpdir("dep");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin().args(["--dir", p, "task", "create", "T1"]).output();
    // unknown dep
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "create",
            "T2",
            "--depends-on",
            "TASK-999",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    // self dep via edit
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "edit",
            "TASK-001",
            "--depends-on",
            "TASK-001",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    // unknown milestone
    let out = bin()
        .args(["--dir", p, "task", "create", "T3", "--milestone", "MS-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn task_no_milestone_and_delete_confirm() {
    let d = tmpdir("nm");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "milestone", "create", "M"])
        .output();
    let _ = bin()
        .args(["--dir", p, "task", "create", "T", "--milestone", "MS-001"])
        .output();
    // remove specific
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "edit",
            "TASK-001",
            "--no-milestone",
            "MS-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("## Milestone"));
    // no-milestone without id on already none ok
    let out = bin()
        .args(["--dir", p, "task", "edit", "TASK-001", "--no-milestone"])
        .output()
        .unwrap();
    assert!(out.status.success());
    // delete confirm abort
    use std::io::Write;
    let mut child = bin()
        .args(["--dir", p, "task", "delete", "TASK-001"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"n\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("TASK-001"));
    let _ = fs::remove_dir_all(&d);
}
