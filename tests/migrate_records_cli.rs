//! Integration test for lazy migration (NFR-04.003 round-trip).
//!
//! Native Papyrus records and legacy Papyrus bodies are read, edited, and
//! listed after lazy on-demand migration. Backlog.md storage-shaped stems are
//! not Papyrus records and stay invisible.

use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("papyrus-migrate-{}", name));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn init(dir: &std::path::Path) {
    let out = bin()
        .args(["--dir", dir.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn task_and_milestone_round_trip_from_native_and_legacy_shapes() {
    let dir = tmpdir("both-shapes");
    let p = dir.to_str().unwrap();
    init(&dir);

    // Native task with complete front matter.
    let native_task = "---\nid: TASK-001\ntitle: Native Task\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nNative body\n<!-- SECTION:DESCRIPTION:END -->\n";
    fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
    fs::write(dir.join("backlog/tasks/TASK-001.md"), native_task).unwrap();

    // Legacy-shaped task (no front matter, Papyrus body).
    let legacy_task = "# TASK-002 Legacy Task\n\n## Status\nIn Progress\n\n## Description\nLegacy body\n\n## Acceptance Criteria\n- [ ] first\n";
    fs::write(dir.join("backlog/tasks/TASK-002.md"), legacy_task).unwrap();

    // Native milestone with complete front matter.
    let native_ms = "---\nid: MS-001\ntitle: Native Milestone\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nNative milestone body\n";
    fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
    fs::write(dir.join("backlog/milestones/MS-001.md"), native_ms).unwrap();

    // Legacy-shaped milestone.
    let legacy_ms = "# MS-002 Legacy Milestone\n\n## Release\nREL-001\n\n## Description\nLegacy milestone body\n";
    fs::write(dir.join("backlog/milestones/MS-002.md"), legacy_ms).unwrap();

    // Create a release so the legacy milestone release reference resolves.
    let out = bin()
        .args(["--dir", p, "release", "create", "R1"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // View every record to trigger lazy migration.
    for id in ["TASK-001", "TASK-002"] {
        let out = bin()
            .args(["--dir", p, "task", "view", id])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "view {} failed: {}",
            id,
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains("## Description"),
            "missing description for {}",
            id
        );
    }

    for id in ["MS-001", "MS-002"] {
        let out = bin()
            .args(["--dir", p, "milestone", "view", id])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "view {} failed: {}",
            id,
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains("## Description"),
            "missing description for {}",
            id
        );
    }

    // Read adoption is front-matter-only: the legacy body is untouched.
    let task_after_view = fs::read_to_string(dir.join("backlog/tasks/TASK-002.md")).unwrap();
    assert!(task_after_view.starts_with("---\n"), "{task_after_view}");
    assert!(
        task_after_view.contains("# TASK-002 Legacy Task"),
        "read adoption must keep the heading: {task_after_view}"
    );
    assert!(
        task_after_view.contains("## Status\nIn Progress"),
        "read adoption must keep legacy sections: {task_after_view}"
    );
    assert!(
        !task_after_view.contains("<!-- SECTION:DESCRIPTION:BEGIN -->")
            && !task_after_view.contains("<!-- AC:BEGIN -->"),
        "read adoption must not convert the body: {task_after_view}"
    );
    let ms_after_view = fs::read_to_string(dir.join("backlog/milestones/MS-002.md")).unwrap();
    assert!(ms_after_view.starts_with("---\n"), "{ms_after_view}");
    assert!(
        ms_after_view.contains("release: REL-001"),
        "{ms_after_view}"
    );
    assert!(
        ms_after_view.contains("# MS-002 Legacy Milestone"),
        "read adoption must keep the heading: {ms_after_view}"
    );
    assert!(
        ms_after_view.contains("## Release\nREL-001"),
        "read adoption must keep legacy sections: {ms_after_view}"
    );

    // Edit round-trip.
    let out = bin()
        .args([
            "--dir",
            p,
            "task",
            "edit",
            "TASK-002",
            "--title",
            "Legacy Task Updated",
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
            "milestone",
            "edit",
            "MS-002",
            "--title",
            "Legacy Milestone Updated",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Re-view and verify updates persisted.
    let out = bin()
        .args(["--dir", p, "task", "view", "TASK-002"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Legacy Task Updated"));

    let out = bin()
        .args(["--dir", p, "milestone", "view", "MS-002"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Legacy Milestone Updated"));

    // First write retires the heading and legacy metadata sections.
    let task_after_edit = fs::read_to_string(dir.join("backlog/tasks/TASK-002.md")).unwrap();
    assert!(!task_after_edit.contains("# TASK-002"), "{task_after_edit}");
    assert!(!task_after_edit.contains("## Status"), "{task_after_edit}");
    assert!(
        task_after_edit.contains("status: \"In Progress\""),
        "derived status must survive the write: {task_after_edit}"
    );
    assert!(
        task_after_edit.contains("## Acceptance Criteria\n- [ ] first"),
        "content section must be preserved: {task_after_edit}"
    );
    let ms_after_edit = fs::read_to_string(dir.join("backlog/milestones/MS-002.md")).unwrap();
    assert!(!ms_after_edit.contains("# MS-002"), "{ms_after_edit}");
    assert!(!ms_after_edit.contains("## Release"), "{ms_after_edit}");
    assert!(
        ms_after_edit.contains("release: REL-001"),
        "front-matter release must survive the write: {ms_after_edit}"
    );

    // Files keep their names and stay in their directories.
    for path in [
        "backlog/tasks/TASK-001.md",
        "backlog/tasks/TASK-002.md",
        "backlog/milestones/MS-001.md",
        "backlog/milestones/MS-002.md",
    ] {
        assert!(dir.join(path).exists(), "missing {}", path);
    }

    // Listing shows every native record.
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(out.status.success());
    let list = String::from_utf8_lossy(&out.stdout);
    assert!(list.contains("TASK-001"), "{list}");
    assert!(list.contains("TASK-002"), "{list}");

    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let list = String::from_utf8_lossy(&out.stdout);
    assert!(list.contains("MS-001"), "{list}");
    assert!(list.contains("MS-002"), "{list}");
}

#[test]
fn backlog_shaped_stems_are_invisible() {
    let dir = tmpdir("backlog-shaped");
    let p = dir.to_str().unwrap();
    init(&dir);

    let storage_task = "---\nid: TASK-1\ntitle: Storage Task\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nStorage body\n<!-- SECTION:DESCRIPTION:END -->\n";
    fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
    let task_path = dir.join("backlog/tasks/task-1 - Storage-Task.md");
    fs::write(&task_path, storage_task).unwrap();

    let storage_ms = "---\nid: m-1\ntitle: Storage Milestone\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nStorage milestone body\n";
    fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
    let ms_path = dir.join("backlog/milestones/m-1 - Storage-Milestone.md");
    fs::write(&ms_path, storage_ms).unwrap();

    for id in ["task-1", "TASK-1"] {
        let out = bin()
            .args(["--dir", p, "task", "view", id])
            .output()
            .unwrap();
        assert!(!out.status.success(), "task {} must not resolve", id);
    }
    for id in ["m-1", "MS-1"] {
        let out = bin()
            .args(["--dir", p, "milestone", "view", id])
            .output()
            .unwrap();
        assert!(!out.status.success(), "milestone {} must not resolve", id);
    }

    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("Storage Task"));

    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("Storage Milestone"));

    // Invisible records are never rewritten.
    assert_eq!(fs::read_to_string(&task_path).unwrap(), storage_task);
    assert_eq!(fs::read_to_string(&ms_path).unwrap(), storage_ms);
}
