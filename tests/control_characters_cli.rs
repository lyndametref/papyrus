//! TASK-123 (F-117-1): single-line values carrying C0 control characters are
//! rejected by the shared sanitizers with the standard validation outcome.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VALIDATION_MESSAGE: &str = "input cannot be normalized";

const NOTE_BLOCK: &str = "  note:\n    prefix: NOTE\n    directory: notes/\n    enabled: true\n    statuses: [active, review, approved, archived]\n";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("t123-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn run(project: &Path, args: &[&str]) -> Output {
    let mut full = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn run_ok(project: &Path, args: &[&str]) -> String {
    let out = run(project, args);
    assert!(
        out.status.success(),
        "command failed: {args:?}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init(name: &str) -> PathBuf {
    let dir = root(name);
    run_ok(&dir, &["init", "Proj", "--no-git"]);
    let config = dir.join("papyrus.config.yml");
    let original = fs::read_to_string(&config).unwrap();
    let updated = original.replacen(
        "document_types:\n",
        &format!("document_types:\n{NOTE_BLOCK}"),
        1,
    );
    assert_ne!(original, updated);
    fs::write(&config, updated).unwrap();
    dir
}

fn md_files(dir: &Path) -> BTreeSet<PathBuf> {
    fn walk(dir: &Path, out: &mut BTreeSet<PathBuf>) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                out.insert(path);
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(dir, &mut out);
    out
}

fn find_md(dir: &Path, file_name: &str) -> PathBuf {
    md_files(dir)
        .into_iter()
        .find(|p| p.file_name().and_then(|n| n.to_str()) == Some(file_name))
        .unwrap_or_else(|| panic!("no {file_name} under {}", dir.display()))
}

fn assert_validation_failure(out: &Output, context: &str) {
    assert_eq!(out.status.code(), Some(4), "{context}: wrong exit code");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(VALIDATION_MESSAGE),
        "{context}: stderr missing {VALIDATION_MESSAGE:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stdout.is_empty(),
        "{context}: stdout should be empty, got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

fn control_titles() -> Vec<String> {
    vec!["Ti\ttle".to_string(), "Ti\u{1}tle".to_string()]
}

#[test]
fn create_rejects_control_characters_without_writing() {
    let dir = init("create");
    run_ok(&dir, &["req", "create-group", "--title", "Group"]);

    for title in control_titles() {
        let cases: &[(&[&str], &str)] = &[
            (
                &["req", "create", &title, "--group", "REQ-001"],
                "req create",
            ),
            (&["note", "create", &title], "note create"),
            (&["task", "create", &title], "task create"),
            (&["milestone", "create", &title], "milestone create"),
        ];
        for (args, context) in cases {
            let before = md_files(&dir);
            let out = run(&dir, args);
            assert_validation_failure(&out, context);
            assert_eq!(md_files(&dir), before, "{context}: wrote a record file");
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn edit_rejects_control_characters_without_touching_files() {
    let dir = init("edit");
    run_ok(&dir, &["req", "create-group", "--title", "Group"]);
    run_ok(
        &dir,
        &["req", "create", "Requirement", "--group", "REQ-001"],
    );
    run_ok(&dir, &["task", "create", "Task"]);

    let req_path = find_md(&dir, "REQ-001.001.md");
    let task_path = find_md(&dir, "TASK-001.md");

    for title in control_titles() {
        for (kind, id, path) in [
            ("req", "REQ-001.001", &req_path),
            ("task", "TASK-001", &task_path),
        ] {
            let before = fs::read(path).unwrap();
            let out = run(&dir, &[kind, "edit", id, "--title", &title]);
            assert_validation_failure(&out, &format!("{kind} edit {id}"));
            assert_eq!(
                fs::read(path).unwrap(),
                before,
                "{kind} edit {id}: file changed"
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}
