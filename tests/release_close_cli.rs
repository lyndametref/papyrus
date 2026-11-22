//! TASK-118: `papyrus release close` end-to-end.
//!
//! Covers the only `planned -> released` transition: success output, content
//! preservation, timestamp handling, rejection of non-planned statuses,
//! unknown IDs, JSON output, absent-status compatibility, and the `status`
//! seeded by `release create`.
//!
//! Every project lives under `./.tmp/` and is removed by [`Scratch`].

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".tmp");
        path.push(format!("release-close-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Scratch { path }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init(scratch: &Scratch) -> PathBuf {
    let project = scratch.path.join("project");
    fs::create_dir_all(&project).unwrap();
    run_ok(&project, &["init", "Proj", "--no-git"]);
    project
}

fn release_path(project: &Path, id: &str) -> PathBuf {
    project
        .join("backlog")
        .join("releases")
        .join(format!("{id}.md"))
}

fn create_release(project: &Path) -> String {
    run_ok(
        project,
        &[
            "release",
            "create",
            "Release One",
            "--description",
            "First body",
            "--label",
            "alpha",
            "--label",
            "beta",
            "--reference",
            "REQ-001",
        ],
    )
}

fn read_release(project: &Path, id: &str) -> String {
    fs::read_to_string(release_path(project, id)).unwrap()
}

/// Read a scalar front-matter value without quotes.
fn frontmatter_value(content: &str, key: &str) -> String {
    let end = content
        .find("\n---")
        .unwrap_or_else(|| panic!("missing front-matter terminator:\n{content}"));
    content[..end]
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name == key).then(|| {
                value
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string()
            })
        })
        .unwrap_or_else(|| panic!("missing front-matter key {key}:\n{content}"))
}

#[test]
fn create_seeds_planned_status() {
    let scratch = Scratch::new("create-status");
    let project = init(&scratch);
    let id = create_release(&project);
    assert_eq!(id, "REL-001");

    let content = read_release(&project, &id);
    assert_eq!(frontmatter_value(&content, "status"), "planned");
}

#[test]
fn close_transitions_planned_to_released_and_preserves_content() {
    let scratch = Scratch::new("success");
    let project = init(&scratch);
    let id = create_release(&project);

    let before = read_release(&project, &id);
    assert_eq!(frontmatter_value(&before, "status"), "planned");
    let created_before = frontmatter_value(&before, "created_date");
    let updated_before = frontmatter_value(&before, "updated_date");

    // The clock has one-second resolution; ensure the refresh is observable.
    std::thread::sleep(Duration::from_millis(1100));

    let out = run(&project, &["release", "close", &id]);
    assert!(
        out.status.success(),
        "close failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), id);

    let after = read_release(&project, &id);
    assert_eq!(frontmatter_value(&after, "status"), "released");
    assert_eq!(frontmatter_value(&after, "created_date"), created_before);
    assert_ne!(
        frontmatter_value(&after, "updated_date"),
        updated_before,
        "close must refresh updated_date"
    );

    // Title, description, labels, references, and body survive untouched.
    assert_eq!(frontmatter_value(&after, "title"), "Release One");
    assert!(after.contains("labels:\n  - alpha\n  - beta\n"));
    assert!(after.contains("references:\n  - REQ-001\n"));
    assert!(after.contains("# REL-001 Release One"));
    assert!(after.contains("## Description\nFirst body"));

    // Only status and updated_date may differ from the pre-close file.
    let expected = before
        .replace("status: planned", "status: released")
        .replace(
            &format!("updated_date: '{updated_before}'"),
            &format!(
                "updated_date: '{}'",
                frontmatter_value(&after, "updated_date")
            ),
        );
    assert_eq!(after, expected);

    // `view` sees the stored transition (DoD #2).
    let viewed = run_ok(&project, &["release", "view", &id]);
    assert_eq!(frontmatter_value(&viewed, "status"), "released");
}

#[test]
fn closing_again_exits_4_and_leaves_the_file_byte_identical() {
    let scratch = Scratch::new("repeat");
    let project = init(&scratch);
    let id = create_release(&project);
    run_ok(&project, &["release", "close", &id]);
    let released = fs::read(release_path(&project, &id)).unwrap();

    let out = run(&project, &["release", "close", &id]);
    assert_eq!(out.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("released"),
        "rejection must name the current status: {stderr}"
    );
    assert_eq!(
        fs::read(release_path(&project, &id)).unwrap(),
        released,
        "a rejected close must not write"
    );
}

#[test]
fn unknown_release_exits_3() {
    let scratch = Scratch::new("unknown");
    let project = init(&scratch);

    let out = run(&project, &["release", "close", "REL-999"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("REL-999"));
}

#[test]
fn json_emits_one_object_with_id_and_released_status() {
    let scratch = Scratch::new("json");
    let project = init(&scratch);
    let id = create_release(&project);

    let out = run(&project, &["--json", "release", "close", &id]);
    assert!(
        out.status.success(),
        "close failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().count(), 1, "exactly one output line");
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(value["id"], id);
    assert_eq!(value["status"], "released");
    assert_eq!(
        value.as_object().map(|object| object.len()),
        Some(2),
        "JSON must carry only id and status"
    );
}

#[test]
fn absent_status_is_treated_as_planned() {
    let scratch = Scratch::new("absent-status");
    let project = init(&scratch);
    let id = create_release(&project);

    let path = release_path(&project, &id);
    let content = fs::read_to_string(&path).unwrap();
    fs::write(&path, content.replace("status: planned\n", "")).unwrap();

    let out = run(&project, &["release", "close", &id]);
    assert!(
        out.status.success(),
        "close failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        frontmatter_value(&read_release(&project, &id), "status"),
        "released"
    );
}

#[test]
fn empty_status_is_treated_as_planned() {
    let scratch = Scratch::new("empty-status");
    let project = init(&scratch);
    let id = create_release(&project);

    let path = release_path(&project, &id);
    let content = fs::read_to_string(&path).unwrap();
    fs::write(&path, content.replace("status: planned", "status:")).unwrap();

    let out = run(&project, &["release", "close", &id]);
    assert!(
        out.status.success(),
        "close failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        frontmatter_value(&read_release(&project, &id), "status"),
        "released"
    );
}

#[test]
fn any_other_status_is_refused_without_a_write() {
    let scratch = Scratch::new("other-status");
    let project = init(&scratch);
    let id = create_release(&project);

    let path = release_path(&project, &id);
    let content = fs::read_to_string(&path).unwrap();
    fs::write(&path, content.replace("status: planned", "status: draft")).unwrap();
    let before = fs::read(&path).unwrap();

    let out = run(&project, &["release", "close", &id]);
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("draft"));
    assert_eq!(fs::read(&path).unwrap(), before);
}
