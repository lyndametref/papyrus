//! Integration tests for `papyrus milestone archive` (TC-015..TC-023).

use std::fs;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-msarchive-{}", name));
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

fn run(d: &std::path::Path, args: &[&str]) -> Output {
    let mut command = bin();
    command.arg("--dir").arg(d);
    command.args(args);
    command.output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).trim().to_string()
}

fn create_milestone(d: &std::path::Path, title: &str) -> String {
    let out = run(d, &["milestone", "create", title]);
    assert!(out.status.success(), "{}", stderr(&out));
    stdout(&out)
}

fn age_updated_date(path: &std::path::Path) {
    let content = fs::read_to_string(path).unwrap();
    let date_line = content
        .lines()
        .find(|line| line.starts_with("updated_date:"))
        .expect("updated_date line")
        .to_string();
    let aged = content.replacen(&date_line, "updated_date: '2020-01-01T00:00:00Z'", 1);
    fs::write(path, aged).unwrap();
}

#[test]
fn archive_open_milestone_closes_and_moves_it() {
    let d = tmpdir("open");
    init(&d);
    let id = create_milestone(&d, "Open M");
    let active = d.join("backlog/milestones/MS-001.md");
    let archived = d.join("backlog/archive/milestones/MS-001.md");
    age_updated_date(&active);

    let out = run(&d, &["milestone", "archive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), id);

    assert!(!active.exists(), "archive must move the source");
    assert!(archived.exists(), "archive must write the destination");
    let content = fs::read_to_string(&archived).unwrap();
    assert!(content.contains("status: archived"), "got: {content}");
    assert!(
        !content.contains("updated_date: '2020-01-01T00:00:00Z'"),
        "archive must refresh updated_date: {content}"
    );
    assert!(content.contains("Open M"), "got: {content}");
}

#[test]
fn archive_closed_milestone_moves_it() {
    let d = tmpdir("closed");
    init(&d);
    let id = create_milestone(&d, "Closed M");
    let close = run(&d, &["milestone", "close", &id]);
    assert!(close.status.success(), "{}", stderr(&close));

    let out = run(&d, &["milestone", "archive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    let archived = d.join("backlog/archive/milestones/MS-001.md");
    assert!(archived.exists());
    assert!(
        fs::read_to_string(&archived)
            .unwrap()
            .contains("status: archived")
    );
    assert!(!d.join("backlog/milestones/MS-001.md").exists());
}

#[test]
fn archive_already_archived_is_a_noop_success() {
    let d = tmpdir("noop");
    init(&d);
    let id = create_milestone(&d, "Noop M");
    let first = run(&d, &["milestone", "archive", &id]);
    assert!(first.status.success(), "{}", stderr(&first));
    let archived = d.join("backlog/archive/milestones/MS-001.md");
    let before = fs::read(&archived).unwrap();

    let second = run(&d, &["milestone", "archive", &id]);
    assert!(second.status.success(), "{}", stderr(&second));
    assert_eq!(stdout(&second), id);
    assert_eq!(fs::read(&archived).unwrap(), before, "no-op must not write");
}

#[test]
fn archive_rejects_an_invalid_status_with_exit_4() {
    let d = tmpdir("invalid");
    init(&d);
    let id = create_milestone(&d, "Invalid M");
    let active = d.join("backlog/milestones/MS-001.md");
    let content = fs::read_to_string(&active).unwrap();
    let invalid = content.replacen("status: open", "status: proposed", 1);
    fs::write(&active, &invalid).unwrap();

    let out = run(&d, &["milestone", "archive", &id]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    let message = stderr(&out);
    assert!(message.contains("proposed"), "stderr: {message}");
    assert!(message.contains(&id), "stderr: {message}");
    assert_eq!(fs::read_to_string(&active).unwrap(), invalid);
    assert!(!d.join("backlog/archive/milestones/MS-001.md").exists());
}

#[test]
fn archive_unknown_id_exits_3() {
    let d = tmpdir("unknown");
    init(&d);
    let out = run(&d, &["milestone", "archive", "MS-999"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(stderr(&out).contains("not found: MS-999"));
}

#[test]
fn archive_filesystem_failure_exits_1() {
    let d = tmpdir("io");
    init(&d);
    let id = create_milestone(&d, "IO M");
    fs::create_dir_all(d.join("backlog/archive")).unwrap();
    fs::write(d.join("backlog/archive/milestones"), "not a directory").unwrap();

    let out = run(&d, &["milestone", "archive", &id]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(d.join("backlog/milestones/MS-001.md").exists());
}

#[test]
fn archived_milestone_is_hidden_from_list() {
    let d = tmpdir("list");
    init(&d);
    let id = create_milestone(&d, "Hidden M");
    let out = run(&d, &["milestone", "archive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));

    let list = run(&d, &["milestone", "list"]);
    assert!(list.status.success());
    assert!(!stdout(&list).contains(&id), "got: {}", stdout(&list));
}

#[test]
fn archived_milestone_remains_viewable_and_editable_in_place() {
    let d = tmpdir("view-edit");
    init(&d);
    let id = create_milestone(&d, "Legacy M");
    let out = run(&d, &["milestone", "archive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    let archived = d.join("backlog/archive/milestones/MS-001.md");

    let view = run(&d, &["milestone", "view", &id]);
    assert!(view.status.success(), "{}", stderr(&view));
    assert!(stdout(&view).contains("status: archived"));

    let edit = run(&d, &["milestone", "edit", &id, "--title", "Renamed"]);
    assert!(edit.status.success(), "{}", stderr(&edit));
    assert!(
        fs::read_to_string(&archived)
            .unwrap()
            .contains("title: Renamed")
    );
    assert!(!d.join("backlog/milestones/MS-001.md").exists());
}

#[test]
fn archive_preserves_release_assignment_and_leaves_release_untouched() {
    let d = tmpdir("release");
    init(&d);
    let release = run(&d, &["release", "create", "R1"]);
    assert!(release.status.success(), "{}", stderr(&release));
    let out = run(&d, &["milestone", "create", "M1", "--release", "REL-001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let id = stdout(&out);
    let release_path = d.join("backlog/releases/REL-001.md");
    let release_before = fs::read(&release_path).unwrap();

    let archive = run(&d, &["milestone", "archive", &id]);
    assert!(archive.status.success(), "{}", stderr(&archive));
    let archived = d.join("backlog/archive/milestones/MS-001.md");
    assert!(
        fs::read_to_string(&archived)
            .unwrap()
            .contains("release: REL-001")
    );
    assert_eq!(fs::read(&release_path).unwrap(), release_before);
}
