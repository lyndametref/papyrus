//! TASK-137: read-only `papyrus check` diagnostics.
//!
//! Every finding kind is built from a real writer command, then the defect is
//! introduced. The suite also pins the output shapes, ordering, exit codes,
//! skipped checks, `deprecated/` exclusion, and the read-only guarantee.

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn base_dir() -> PathBuf {
    PathBuf::from(".tmp").join("check-cli")
}

struct Project {
    path: PathBuf,
}

impl Project {
    fn new(name: &str) -> Self {
        let path = base_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create project directory");
        Self { path }
    }

    fn initialized(name: &str) -> Self {
        let project = Self::new(name);
        project.ok(&["init", "Proj", "--no-git"]);
        project
    }

    fn run(&self, args: &[&str]) -> Output {
        bin()
            .arg("--dir")
            .arg(&self.path)
            .args(args)
            .output()
            .expect("run papyrus")
    }

    fn run_json(&self, args: &[&str]) -> Output {
        let mut all = vec!["--json"];
        all.extend_from_slice(args);
        self.run(&all)
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "command {:?} failed (exit {:?}): {}",
            args,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).expect("read project file")
    }

    fn write(&self, relative: &str, content: &str) {
        fs::write(self.path(relative), content).expect("write project file");
    }

    fn replace(&self, relative: &str, from: &str, to: &str) {
        let content = self.read(relative);
        assert!(
            content.contains(from),
            "{} must contain {:?}",
            relative,
            from
        );
        self.write(relative, &content.replacen(from, to, 1));
    }

    fn append(&self, relative: &str, extra: &str) {
        let content = self.read(relative);
        self.write(relative, &format!("{}{}", content, extra));
    }

    fn copy(&self, from: &str, to: &str) {
        fs::copy(self.path(from), self.path(to)).expect("copy record");
    }

    fn json(&self, args: &[&str]) -> (Value, Option<i32>) {
        let out = self.run_json(args);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let value = serde_json::from_str(stdout.trim()).expect("check emits JSON");
        (value, out.status.code())
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn finding(value: &Value, index: usize) -> &Value {
    &value["findings"][index]
}

fn kinds(value: &Value) -> Vec<&str> {
    value["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .map(|finding| finding["kind"].as_str().expect("kind string"))
        .collect()
}

#[test]
fn clean_project_reports_no_findings_and_exits_zero() {
    let project = Project::initialized("clean");
    let out = project.run(&["check"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout_of(&out).trim(), "no findings");
    assert_eq!(stderr_of(&out), "");

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        value,
        json!({"findings": [], "skipped": [], "repaired": []})
    );
}

#[test]
fn duplicate_task_id_is_detected_with_sorted_locations_and_fixability() {
    let project = Project::initialized("duplicate-task");
    project.ok(&["task", "create", "First task"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["duplicate-id"]);
    assert_eq!(
        finding(&value, 0),
        &json!({
            "kind": "duplicate-id",
            "family": "task",
            "subject": "TASK-001",
            "locations": [
                "backlog/tasks/TASK-001-copy.md",
                "backlog/tasks/TASK-001.md"
            ],
            "detail": "2 records share this ID",
            "fixable": true
        })
    );

    let text = stdout_of(&project.run(&["check"]));
    assert!(
        text.contains(
            "duplicate-id: task TASK-001 — 2 records share this ID (backlog/tasks/TASK-001-copy.md, backlog/tasks/TASK-001.md)"
        ),
        "unexpected text report: {}",
        text
    );
}

#[test]
fn duplicate_ids_stay_per_family_and_group_space() {
    let project = Project::initialized("duplicate-families");
    project.ok(&["req", "create-group", "--title", "Group"]);
    project.ok(&["req", "create", "Member", "--group", "REQ-001"]);
    project.copy(
        "docs/requirements/business/REQ-001.md",
        "docs/requirements/business/REQ-001-copy.md",
    );
    project.copy(
        "docs/requirements/business/REQ-001.001.md",
        "docs/requirements/business/REQ-001.001-copy.md",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["duplicate-id", "duplicate-id"]);

    let group = finding(&value, 0);
    let member = finding(&value, 1);
    assert_eq!(group["family"], "req");
    assert_eq!(group["subject"], "REQ-001");
    assert_eq!(
        group["fixable"], false,
        "a grouped-family group ID is never repairable"
    );
    assert_eq!(member["family"], "req");
    assert_eq!(member["subject"], "REQ-001.001");
    assert_eq!(
        member["fixable"], true,
        "member-level duplicates are repairable"
    );
}

#[test]
fn duplicate_ids_cover_decisions_and_configured_document_types() {
    let project = Project::initialized("duplicate-other-families");
    project.ok(&["decision", "create", "A decision", "-t", "adr"]);
    project.copy(
        "decisions/ADR-001-a-decision.md",
        "decisions/ADR-001-copy.md",
    );
    project.ok(&["tech-spec", "create", "Original", "--description", "Body"]);
    project.copy(
        "docs/technical/TECH-001.md",
        "docs/technical/TECH-001-copy.md",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    let families: Vec<&str> = value["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|finding| finding["family"].as_str().expect("family"))
        .collect();
    assert!(families.contains(&"decision"), "findings: {:?}", value);
    assert!(families.contains(&"tech-spec"), "findings: {:?}", value);
    assert_eq!(families.len(), 2, "one finding per duplicated key");
}

#[test]
fn self_dependency_is_detected() {
    let project = Project::initialized("self-dependency");
    project.ok(&["task", "create", "Self"]);
    project.replace(
        "backlog/tasks/TASK-001.md",
        "dependencies: []",
        "dependencies:\n  - TASK-001",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["self-dependency"]);
    assert_eq!(
        finding(&value, 0),
        &json!({
            "kind": "self-dependency",
            "family": "task",
            "subject": "TASK-001",
            "locations": ["backlog/tasks/TASK-001.md"],
            "detail": "dependency list contains TASK-001",
            "fixable": false
        })
    );
}

#[test]
fn dependency_cycle_is_reported_once_in_canonical_order() {
    let project = Project::initialized("dependency-cycle");
    project.ok(&["task", "create", "One"]);
    project.ok(&["task", "create", "Two"]);
    project.replace(
        "backlog/tasks/TASK-001.md",
        "dependencies: []",
        "dependencies:\n  - TASK-002",
    );
    project.replace(
        "backlog/tasks/TASK-002.md",
        "dependencies: []",
        "dependencies:\n  - TASK-001",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["dependency-cycle"]);
    assert_eq!(
        finding(&value, 0),
        &json!({
            "kind": "dependency-cycle",
            "family": "task",
            "subject": "TASK-001",
            "locations": [
                "backlog/tasks/TASK-001.md",
                "backlog/tasks/TASK-002.md"
            ],
            "detail": "TASK-001 -> TASK-002",
            "fixable": false
        })
    );
}

#[test]
fn dangling_milestone_and_release_references_are_detected() {
    let project = Project::initialized("dangling");
    project.ok(&["task", "create", "Task"]);
    project.replace(
        "backlog/tasks/TASK-001.md",
        "milestone: null",
        "milestone: MS-999",
    );
    project.ok(&["milestone", "create", "Milestone"]);
    project.replace(
        "backlog/milestones/MS-001.md",
        "release: null",
        "release: REL-999",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(
        kinds(&value),
        vec!["dangling-milestone", "dangling-release"]
    );
    assert_eq!(
        finding(&value, 0),
        &json!({
            "kind": "dangling-milestone",
            "family": "task",
            "subject": "TASK-001",
            "locations": ["backlog/tasks/TASK-001.md"],
            "detail": "milestone MS-999 does not resolve to a milestone",
            "fixable": false
        })
    );
    assert_eq!(
        finding(&value, 1),
        &json!({
            "kind": "dangling-release",
            "family": "milestone",
            "subject": "MS-001",
            "locations": ["backlog/milestones/MS-001.md"],
            "detail": "release REL-999 does not resolve to a release",
            "fixable": false
        })
    );
}

#[test]
fn retired_sections_agree_silently_and_disagree_loudly() {
    let project = Project::initialized("retired-sections");
    project.ok(&["task", "create", "Task"]);

    project.append("backlog/tasks/TASK-001.md", "\n## Status\nToDo\n");
    let clean = project.run(&["check"]);
    assert_eq!(clean.status.code(), Some(0), "{}", stdout_of(&clean));
    assert_eq!(stdout_of(&clean).trim(), "no findings");

    project.ok(&["task", "create", "Second"]);
    project.append(
        "backlog/tasks/TASK-002.md",
        "\n## Labels\n- urgent\n\n## Status\nDone\n",
    );
    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(
        kinds(&value),
        vec!["frontmatter-conflict", "frontmatter-conflict"]
    );
    let details: Vec<&str> = value["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|finding| finding["detail"].as_str().expect("detail"))
        .collect();
    assert!(
        details.contains(&"section ## Labels \"urgent\" disagrees with front matter labels \"\""),
        "details: {:?}",
        details
    );
    assert!(
        details.contains(&"section ## Status \"Done\" disagrees with front matter status \"ToDo\""),
        "details: {:?}",
        details
    );
}

#[test]
fn unreadable_retained_section_is_a_conflict() {
    let project = Project::initialized("unreadable-section");
    project.ok(&["task", "create", "Task"]);
    project.append(
        "backlog/tasks/TASK-001.md",
        "\n## Status\nDone\nImplemented\n",
    );

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["frontmatter-conflict"]);
    assert!(
        finding(&value, 0)["detail"]
            .as_str()
            .expect("detail")
            .contains("cannot be reduced to a single value"),
        "got: {}",
        finding(&value, 0)["detail"]
    );
}

#[test]
fn document_heading_conflict_is_detected_but_decisions_are_excluded() {
    let project = Project::initialized("heading-conflict");
    project.ok(&["tech-spec", "create", "Original", "--description", "Body"]);
    project.replace(
        "docs/technical/TECH-001.md",
        "# TECH-001 Original",
        "# TECH-001 Changed",
    );
    project.ok(&["decision", "create", "A decision", "-t", "adr"]);

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["frontmatter-conflict"]);
    assert_eq!(
        finding(&value, 0),
        &json!({
            "kind": "frontmatter-conflict",
            "family": "tech-spec",
            "subject": "TECH-001",
            "locations": ["docs/technical/TECH-001.md"],
            "detail": "heading title \"Changed\" disagrees with front matter title \"Original\"",
            "fixable": false
        })
    );
}

#[test]
fn invalid_configuration_reports_findings_and_runs_independent_checks() {
    let project = Project::initialized("invalid-config");
    project.ok(&["task", "create", "First"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    project.write("papyrus.config.yml", "statuses: []\n");

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert!(kinds(&value).contains(&"config-invalid"));
    assert!(kinds(&value).contains(&"duplicate-id"));
    assert_eq!(
        value["skipped"],
        json!([
            "duplicate-id:document-types",
            "frontmatter-conflict:document-records"
        ])
    );

    let text = stdout_of(&project.run(&["check"]));
    assert!(text.contains("config-invalid:"), "{}", text);
    assert!(
        text.contains(
            "skipped: duplicate-id:document-types, frontmatter-conflict:document-records"
        ),
        "{}",
        text
    );
}

#[test]
fn malformed_configuration_names_every_corpus_check_as_skipped() {
    let project = Project::initialized("malformed-config");
    project.write("papyrus.config.yml", "statuses: [\n");

    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["config-invalid"]);
    let skipped = value["skipped"].as_array().expect("skipped array");
    assert_eq!(skipped.len(), 8, "every corpus check is named: {}", value);
    for name in [
        "duplicate-id",
        "duplicate-id:document-types",
        "self-dependency",
        "dependency-cycle",
        "dangling-milestone",
        "dangling-release",
        "frontmatter-conflict",
        "frontmatter-conflict:document-records",
    ] {
        assert!(skipped.contains(&json!(name)), "missing {}", name);
    }
}

#[test]
fn deprecated_folder_and_unknown_dependency_produce_no_finding() {
    let project = Project::initialized("exclusions");
    project.ok(&["tech-spec", "create", "Original", "--description", "Body"]);
    fs::create_dir_all(project.path("docs/technical/deprecated")).expect("deprecated directory");
    project.copy(
        "docs/technical/TECH-001.md",
        "docs/technical/deprecated/TECH-001.md",
    );
    project.ok(&["task", "create", "Task"]);
    project.replace(
        "backlog/tasks/TASK-001.md",
        "dependencies: []",
        "dependencies:\n  - TASK-999",
    );

    let out = project.run(&["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout_of(&out));
    assert_eq!(stdout_of(&out).trim(), "no findings");
}

#[test]
fn findings_are_deterministic_and_match_the_goldens() {
    let project = Project::initialized("ordering");
    project.ok(&["task", "create", "One"]);
    project.ok(&["task", "create", "Two"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    project.replace(
        "backlog/tasks/TASK-001.md",
        "dependencies: []",
        "dependencies:\n  - TASK-001",
    );
    project.replace(
        "backlog/tasks/TASK-002.md",
        "dependencies: []",
        "dependencies:\n  - TASK-999",
    );
    project.replace(
        "backlog/tasks/TASK-002.md",
        "milestone: null",
        "milestone: MS-999",
    );
    project.ok(&["milestone", "create", "Milestone"]);
    project.replace(
        "backlog/milestones/MS-001.md",
        "release: null",
        "release: REL-999",
    );
    project.copy(
        "backlog/milestones/MS-001.md",
        "backlog/milestones/MS-001-copy.md",
    );
    project.ok(&["tech-spec", "create", "Original", "--description", "Body"]);
    project.replace(
        "docs/technical/TECH-001.md",
        "# TECH-001 Original",
        "# TECH-001 Changed",
    );

    let first = project.run(&["check"]);
    let second = project.run(&["check"]);
    assert_eq!(first.status.code(), Some(4));
    assert_eq!(
        stdout_of(&first),
        stdout_of(&second),
        "text is deterministic"
    );
    let first_json = project.run_json(&["check"]);
    let second_json = project.run_json(&["check"]);
    assert_eq!(stdout_of(&first_json), stdout_of(&second_json));
    assert_eq!(first_json.status.code(), Some(4));

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/check");
    assert_eq!(
        stdout_of(&first),
        fs::read_to_string(golden.join("report.txt")).expect("text golden")
    );
    assert_eq!(
        stdout_of(&first_json),
        fs::read_to_string(golden.join("report.json")).expect("json golden")
    );
}

#[test]
fn read_only_run_preserves_every_scanned_file() {
    let project = Project::initialized("read-only");
    project.ok(&["task", "create", "Active"]);
    project.ok(&["task", "create", "Archived"]);
    project.ok(&["task", "archive", "TASK-002"]);
    project.ok(&["task", "create", "Legacy"]);
    fs::create_dir_all(project.path("backlog/tasks/archived")).expect("legacy directory");
    fs::rename(
        project.path("backlog/tasks/TASK-003.md"),
        project.path("backlog/tasks/archived/TASK-003.md"),
    )
    .expect("place a record in the read-only legacy directory");
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );

    let before = snapshot(&project.path);
    let out = project.run(&["check"]);
    assert_eq!(out.status.code(), Some(4), "{}", stdout_of(&out));
    let after = snapshot(&project.path);
    assert_eq!(before, after, "check must not change bytes or mtimes");
    assert!(
        project.path("backlog/archive/tasks/TASK-002.md").exists(),
        "archived records stay in place"
    );
    assert!(
        project.path("backlog/tasks/archived/TASK-003.md").exists(),
        "legacy records stay in place"
    );
}

#[test]
fn missing_configuration_is_reported_as_a_finding() {
    let project = Project::new("missing-config");
    let (value, code) = project.json(&["check"]);
    assert_eq!(code, Some(4));
    assert_eq!(kinds(&value), vec!["config-invalid"]);
    assert_eq!(
        finding(&value, 0)["locations"],
        json!(["papyrus.config.yml"])
    );
    assert_eq!(
        value["skipped"].as_array().expect("skipped").len(),
        8,
        "no configuration value means every corpus check is skipped"
    );

    let help = project.run(&["check", "--help"]);
    assert_eq!(help.status.code(), Some(0), "{}", stderr_of(&help));
    assert!(stdout_of(&help).contains("Usage:"));
}

#[test]
fn usage_and_io_failures_keep_their_exit_codes() {
    let project = Project::initialized("exit-codes");
    let usage = project.run(&["check", "--bogus"]);
    assert_eq!(usage.status.code(), Some(2));

    fs::remove_dir_all(project.path("backlog/tasks")).expect("remove task directory");
    fs::write(project.path("backlog/tasks"), "not a directory").expect("blocking file");
    let io = project.run(&["check"]);
    assert_eq!(io.status.code(), Some(1));
    assert!(
        stderr_of(&io).contains("cannot read directory"),
        "{}",
        stderr_of(&io)
    );
    assert_eq!(stdout_of(&io), "", "I/O failures use the error path");
}

fn set_created_date(project: &Project, relative: &str, value: &str) {
    let content = project.read(relative);
    let mut lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.starts_with("created_date:") {
                format!("created_date: '{}'", value)
            } else {
                line.to_string()
            }
        })
        .collect();
    lines.push(String::new());
    project.write(relative, &lines.join("\n"));
}

fn updated_date_of(project: &Project, relative: &str) -> String {
    let content = project.read(relative);
    content
        .lines()
        .find_map(|line| line.strip_prefix("updated_date:"))
        .map(|value| value.trim().trim_matches('\'').to_string())
        .expect("updated_date")
}

#[test]
fn fix_keeps_the_earliest_created_date_over_path_order() {
    let project = Project::initialized("fix-keeper-date");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    // The copy sorts first by path, but the original is the earliest record.
    set_created_date(
        &project,
        "backlog/tasks/TASK-001.md",
        "2026-01-01T00:00:00Z",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001-copy.md",
        "2026-02-01T00:00:00Z",
    );

    let out = project.run(&["check", "--fix", "--force"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr_of(&out));
    assert!(
        stdout_of(&out).contains(
            "repaired: task TASK-001 -> TASK-002 (backlog/tasks/TASK-001-copy.md -> backlog/tasks/TASK-002.md)"
        ),
        "{}",
        stdout_of(&out)
    );
    assert!(
        project.path("backlog/tasks/TASK-001.md").exists(),
        "keeper stays"
    );
    assert!(!project.path("backlog/tasks/TASK-001-copy.md").exists());
    assert!(project.path("backlog/tasks/TASK-002.md").exists());
    assert!(
        project
            .read("backlog/tasks/TASK-002.md")
            .contains("id: TASK-002")
    );
}

#[test]
fn fix_tie_breaks_on_the_lowest_project_relative_path() {
    let project = Project::initialized("fix-keeper-path");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001.md",
        "2026-01-01T00:00:00Z",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001-copy.md",
        "2026-01-01T00:00:00Z",
    );

    let out = project.run(&["check", "--fix", "--force"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr_of(&out));
    assert!(
        stdout_of(&out).contains(
            "repaired: task TASK-001 -> TASK-002 (backlog/tasks/TASK-001.md -> backlog/tasks/TASK-002.md)"
        ),
        "{}",
        stdout_of(&out)
    );
    assert!(
        project.path("backlog/tasks/TASK-001-copy.md").exists(),
        "copy is the keeper"
    );
    assert!(!project.path("backlog/tasks/TASK-001.md").exists());
}

#[test]
fn fix_allocates_two_group_members_sequentially_in_one_run() {
    let project = Project::initialized("fix-allocation");
    project.ok(&["task", "create", "Original"]);
    project.copy("backlog/tasks/TASK-001.md", "backlog/tasks/TASK-001-b.md");
    project.copy("backlog/tasks/TASK-001.md", "backlog/tasks/TASK-001-a.md");
    for relative in [
        "backlog/tasks/TASK-001-a.md",
        "backlog/tasks/TASK-001-b.md",
        "backlog/tasks/TASK-001.md",
    ] {
        set_created_date(&project, relative, "2026-01-01T00:00:00Z");
    }

    let out = project.run(&["check", "--fix", "--force"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(
        text.contains(
            "repaired: task TASK-001 -> TASK-002 (backlog/tasks/TASK-001-b.md -> backlog/tasks/TASK-002.md)"
        ),
        "{}",
        text
    );
    assert!(
        text.contains(
            "repaired: task TASK-001 -> TASK-003 (backlog/tasks/TASK-001.md -> backlog/tasks/TASK-003.md)"
        ),
        "{}",
        text
    );
    assert!(project.path("backlog/tasks/TASK-001-a.md").exists());
    assert!(project.path("backlog/tasks/TASK-002.md").exists());
    assert!(project.path("backlog/tasks/TASK-003.md").exists());
}

#[test]
fn fix_refuses_the_three_refusal_classes_and_writes_nothing() {
    let project = Project::initialized("fix-refusals");
    // 1. front-matter id and stem disagree.
    project.ok(&["task", "create", "Mismatch"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-002-copy.md",
    );
    // 2. a member in the read-only legacy directory.
    project.ok(&["task", "create", "Legacy"]);
    fs::create_dir_all(project.path("backlog/tasks/archived")).expect("legacy directory");
    project.copy(
        "backlog/tasks/TASK-003.md",
        "backlog/tasks/archived/TASK-003.md",
    );
    // 3. a grouped-family group ID.
    project.ok(&["req", "create-group", "--title", "Group"]);
    project.copy(
        "docs/requirements/business/REQ-001.md",
        "docs/requirements/business/REQ-001-copy.md",
    );

    let before = snapshot(&project.path);
    let (value, code) = project.json(&["check", "--fix", "--force"]);
    let after = snapshot(&project.path);
    assert_eq!(code, Some(4), "refusals still report findings");
    assert_eq!(value["repaired"], json!([]));
    assert_eq!(
        findings_of(&value, "duplicate-id").len(),
        3,
        "each refusal stays reported: {}",
        value
    );
    for finding in findings_of(&value, "duplicate-id") {
        assert_eq!(finding["fixable"], false, "{}", finding);
    }
    assert_eq!(before, after, "a refused plan writes nothing");
}

#[test]
fn fix_lists_references_and_never_rewrites_the_referring_file() {
    let project = Project::initialized("fix-references");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001.md",
        "2026-01-01T00:00:00Z",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001-copy.md",
        "2026-01-01T00:00:00Z",
    );
    project.ok(&["task", "create", "Referring"]);
    project.replace(
        "backlog/tasks/TASK-002.md",
        "milestone: null",
        "milestone: TASK-001",
    );
    project.replace(
        "backlog/tasks/TASK-002.md",
        "dependencies: []",
        "dependencies:\n  - TASK-001",
    );
    let referring = project.read("backlog/tasks/TASK-002.md");

    let out = project.run(&["check", "--fix", "--force"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(
        text.contains(
            "repaired: task TASK-001 -> TASK-003 (backlog/tasks/TASK-001.md -> backlog/tasks/TASK-003.md)"
        ),
        "{}",
        text
    );
    let dependencies = text
        .find(r#"reference: task TASK-002 (backlog/tasks/TASK-002.md) dependencies = "TASK-001""#);
    let milestone =
        text.find(r#"reference: task TASK-002 (backlog/tasks/TASK-002.md) milestone = "TASK-001""#);
    assert!(dependencies.is_some() && milestone.is_some(), "{}", text);
    assert!(
        dependencies < milestone,
        "references keep their sorted order: {}",
        text
    );
    assert_eq!(
        project.read("backlog/tasks/TASK-002.md"),
        referring,
        "the referring file is byte-identical"
    );
}

#[test]
fn fix_exits_four_after_repair_and_a_second_run_is_clean_and_writes_nothing() {
    let project = Project::initialized("fix-idempotent");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );

    let first = project.run(&["check", "--fix", "--force"]);
    assert_eq!(first.status.code(), Some(4));
    let after_first = snapshot(&project.path);
    let second = project.run(&["check", "--fix", "--force"]);
    assert_eq!(second.status.code(), Some(0), "{}", stdout_of(&second));
    assert_eq!(stdout_of(&second).trim(), "no findings");
    assert_eq!(
        snapshot(&project.path),
        after_first,
        "the second run writes nothing"
    );
}

#[test]
fn fix_without_force_on_piped_stdin_exits_two_and_writes_nothing() {
    let project = Project::initialized("fix-non-terminal");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    let before = snapshot(&project.path);

    let out = project.run(&["check", "--fix"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout_of(&out), "");
    assert!(
        stderr_of(&out).contains("--fix requires --force when standard input is not a terminal"),
        "{}",
        stderr_of(&out)
    );
    assert_eq!(snapshot(&project.path), before);
}

#[test]
fn force_without_fix_is_a_clap_usage_error_before_any_scan() {
    let project = Project::initialized("fix-force-usage");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    let before = snapshot(&project.path);

    let out = project.run(&["check", "--force"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout_of(&out), "");
    assert!(!stderr_of(&out).is_empty(), "clap names the missing --fix");
    assert_eq!(snapshot(&project.path), before);
}

#[test]
fn fix_with_nothing_to_repair_never_prompts_and_exits_zero() {
    let project = Project::initialized("fix-clean");
    let out = project.run(&["check", "--fix"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr_of(&out));
    assert_eq!(stdout_of(&out).trim(), "no findings");
    assert_eq!(stderr_of(&out), "");
}

#[test]
fn fix_repairs_a_crash_state_left_by_the_two_write_steps() {
    let project = Project::initialized("fix-crash");
    project.ok(&["task", "create", "Original"]);
    project.copy(
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-001-copy.md",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001.md",
        "2026-01-01T00:00:00Z",
    );
    set_created_date(
        &project,
        "backlog/tasks/TASK-001-copy.md",
        "2026-01-01T00:00:00Z",
    );
    let original = project.read("backlog/tasks/TASK-001.md");

    assert_eq!(
        project.run(&["check", "--fix", "--force"]).status.code(),
        Some(4)
    );
    assert!(project.path("backlog/tasks/TASK-002.md").exists());
    // Crash state: the new file exists and the old file reappears.
    project.write("backlog/tasks/TASK-001.md", &original);

    let out = project.run(&["check", "--fix", "--force"]);
    assert_eq!(out.status.code(), Some(4), "{}", stdout_of(&out));
    assert!(!project.path("backlog/tasks/TASK-001.md").exists());
    assert!(project.path("backlog/tasks/TASK-002.md").exists());
    assert!(project.path("backlog/tasks/TASK-003.md").exists());
    let clean = project.run(&["check"]);
    assert_eq!(clean.status.code(), Some(0), "{}", stdout_of(&clean));
}

#[test]
fn fix_force_text_and_json_reports_match_the_goldens() {
    let text_project = Project::initialized("fix-golden-text");
    let json_project = Project::initialized("fix-golden-json");
    for project in [&text_project, &json_project] {
        project.ok(&["task", "create", "Alpha"]);
        project.copy(
            "backlog/tasks/TASK-001.md",
            "backlog/tasks/TASK-001-copy.md",
        );
        set_created_date(project, "backlog/tasks/TASK-001.md", "2026-01-01T00:00:00Z");
        set_created_date(
            project,
            "backlog/tasks/TASK-001-copy.md",
            "2026-01-01T00:00:00Z",
        );
        project.ok(&["task", "create", "Beta"]);
        project.replace(
            "backlog/tasks/TASK-002.md",
            "milestone: null",
            "milestone: TASK-001",
        );
    }

    let text = text_project.run(&["check", "--fix", "--force"]);
    assert_eq!(text.status.code(), Some(4), "{}", stderr_of(&text));
    let json = json_project.run_json(&["check", "--fix", "--force"]);
    assert_eq!(json.status.code(), Some(4), "{}", stderr_of(&json));

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/check");
    assert_eq!(
        stdout_of(&text),
        fs::read_to_string(golden.join("fix-report.txt")).expect("fix text golden")
    );
    assert_eq!(
        stdout_of(&json),
        fs::read_to_string(golden.join("fix-report.json")).expect("fix json golden")
    );

    for project in [&text_project, &json_project] {
        let updated = updated_date_of(project, "backlog/tasks/TASK-003.md");
        assert!(
            papyrus::record::frontmatter::timestamp_seconds(&updated).is_some(),
            "updated_date keeps the ISO-8601 UTC shape: {}",
            updated
        );
    }
}

fn findings_of<'a>(value: &'a Value, kind: &str) -> Vec<&'a Value> {
    value["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .filter(|finding| finding["kind"] == kind)
        .collect()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

fn walk(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
    let entries = fs::read_dir(directory).expect("read snapshot directory");
    for entry in entries {
        let entry = entry.expect("snapshot entry");
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, files);
        } else if path.is_file() {
            let bytes = fs::read(&path).expect("snapshot bytes");
            let modified = fs::metadata(&path)
                .expect("snapshot metadata")
                .modified()
                .expect("snapshot mtime");
            let relative = path
                .strip_prefix(root)
                .expect("path under project")
                .to_path_buf();
            files.insert(relative, (bytes, modified));
        }
    }
}
