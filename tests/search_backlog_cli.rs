//! Integration tests for `papyrus search` over task, milestone, release records (TASK-10).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-search-bl-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn init(d: &Path) {
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(out.status.success());
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn run(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn search(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["search"];
    full.extend_from_slice(args);
    run(project, &full)
}

#[test]
fn basic_task_milestone_release_search() {
    let d = tmpdir("basic");
    init(&d);
    run(&d, &["task", "create", "Task title", "-d", "needle task"]);
    run(
        &d,
        &["milestone", "create", "Milestone title", "-d", "needle ms"],
    );
    run(
        &d,
        &["release", "create", "Release title", "-d", "needle rel"],
    );

    let out = search(&d, &["needle"]);
    assert_eq!(out.status.code(), Some(0));
    let s = stdout(&out);
    assert!(s.contains("TASK-001 task Task title"));
    assert!(s.contains("MS-001 milestone Milestone title"));
    assert!(s.contains("REL-001 release Release title"));
}

#[test]
fn type_filters_restrict() {
    let d = tmpdir("filters");
    init(&d);
    run(&d, &["task", "create", "Task title", "-d", "needle task"]);
    run(
        &d,
        &["milestone", "create", "Milestone title", "-d", "needle ms"],
    );
    run(
        &d,
        &["release", "create", "Release title", "-d", "needle rel"],
    );

    let out = search(&d, &["needle", "--type", "task"]);
    assert_eq!(stdout(&out).trim(), "TASK-001 task Task title");

    let out = search(&d, &["needle", "--type", "milestone"]);
    assert_eq!(stdout(&out).trim(), "MS-001 milestone Milestone title");

    let out = search(&d, &["needle", "--type", "release"]);
    assert_eq!(stdout(&out).trim(), "REL-001 release Release title");

    let out = search(&d, &["needle", "--type", "decision"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

#[test]
fn archive_hides_task_and_milestone() {
    let d = tmpdir("archive");
    init(&d);
    run(&d, &["task", "create", "Task title", "-d", "needle task"]);
    run(
        &d,
        &["milestone", "create", "Milestone title", "-d", "needle ms"],
    );
    run(
        &d,
        &["release", "create", "Release title", "-d", "needle rel"],
    );

    run(&d, &["task", "archive", "TASK-001"]);
    run(&d, &["milestone", "archive", "MS-001"]);

    let out = search(&d, &["needle"]);
    assert_eq!(out.status.code(), Some(0));
    let s = stdout(&out);
    assert!(!s.contains("TASK-001"));
    assert!(!s.contains("MS-001"));
    assert!(s.contains("REL-001"));
}

#[test]
fn json_output_shape() {
    let d = tmpdir("json");
    init(&d);
    run(&d, &["task", "create", "Task title", "-d", "needle task"]);

    let out = search(&d, &["needle", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "TASK-001");
    assert_eq!(arr[0]["type"], "task");
    assert_eq!(arr[0]["title"], "Task title");
    assert!(
        arr[0]["path"]
            .as_str()
            .unwrap()
            .ends_with("backlog/tasks/TASK-001.md")
    );
}

#[test]
fn quoted_phrase_and_metacharacters() {
    let d = tmpdir("phrase");
    init(&d);
    run(
        &d,
        &["task", "create", "Task title", "-d", "quick brown fox"],
    );

    let out = search(&d, &["quick brown"]);
    assert_eq!(stdout(&out).trim(), "TASK-001 task Task title");

    let out = search(&d, &["quick fox"]);
    assert_eq!(stdout(&out), "");

    let out = search(&d, &["TASK-001"]);
    assert_eq!(stdout(&out).trim(), "TASK-001 task Task title");
}

#[test]
fn missing_store_dirs_no_error() {
    let d = tmpdir("missing");
    init(&d);
    // remove backlog entirely
    let _ = fs::remove_dir_all(d.join("backlog"));
    let out = search(&d, &["anything"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

#[test]
fn non_utf8_skipped() {
    let d = tmpdir("nonutf8");
    init(&d);
    run(&d, &["task", "create", "Task title", "-d", "needle task"]);
    fs::write(d.join("backlog/tasks/BROKEN.md"), [0xffu8, 0xfe, 0x00]).unwrap();

    let out = search(&d, &["needle"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "TASK-001 task Task title");
}

#[test]
fn limit_caps_combined() {
    let d = tmpdir("limit");
    init(&d);
    for _ in 0..3 {
        run(&d, &["task", "create", "Task title", "-d", "needle task"]);
    }
    // ids TASK-001..003
    let out = search(&d, &["needle", "--limit", "2"]);
    assert_eq!(out.status.code(), Some(0));
    let out_stdout = stdout(&out);
    let lines: Vec<&str> = out_stdout.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn decision_search_basic() {
    let d = tmpdir("decision");
    init(&d);
    run(
        &d,
        &[
            "decision",
            "create",
            "Decision title",
            "-d",
            "needle decision",
        ],
    );

    let out = search(&d, &["needle"]);
    assert_eq!(out.status.code(), Some(0));
    let s = stdout(&out);
    assert!(s.contains("ADR-001 decision Decision title"));

    let out = search(&d, &["needle", "--type", "decision"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("ADR-001 decision Decision title"));

    // archived decision is hidden
    run(&d, &["decision", "archive", "ADR-001"]);
    let out = search(&d, &["needle", "--type", "decision"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(!stdout(&out).contains("ADR-001"));
}
