//! Integration test for TASK-69 lazy migration (NFR-04.003 round-trip).
//!
//! Verifies that task and milestone records created in both the legacy
//! Papyrus shape and the Backlog.md storage shape can be viewed, edited,
//! and listed after lazy on-demand migration.

use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-migrate-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
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
fn task_and_milestone_round_trip_from_both_shapes() {
    let dir = tmpdir("both-shapes");
    let p = dir.to_str().unwrap();
    init(&dir);

    // Storage-shaped task (already has complete front matter, Backlog.md body).
    let storage_task = "---\nid: TASK-1\ntitle: Storage Task\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nStorage body\n<!-- SECTION:DESCRIPTION:END -->\n";
    fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
    fs::write(
        dir.join("backlog/tasks/task-1 - Storage-Task.md"),
        storage_task,
    )
    .unwrap();

    // Legacy-shaped task (no front matter, Papyrus body).
    let legacy_task = "# TASK-002 Legacy Task\n\n## Status\nIn Progress\n\n## Description\nLegacy body\n\n## Acceptance Criteria\n- [ ] first\n";
    fs::write(dir.join("backlog/tasks/TASK-002.md"), legacy_task).unwrap();

    // Storage-shaped milestone.
    let storage_ms = "---\nid: m-1\ntitle: Storage Milestone\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nStorage milestone body\n";
    fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
    fs::write(
        dir.join("backlog/milestones/m-1 - Storage-Milestone.md"),
        storage_ms,
    )
    .unwrap();

    // Legacy-shaped milestone.
    let legacy_ms = "# MS-001 Legacy Milestone\n\n## Release\nREL-001\n\n## Description\nLegacy milestone body\n";
    fs::write(dir.join("backlog/milestones/MS-001.md"), legacy_ms).unwrap();

    // Create a release so the legacy milestone release reference resolves.
    let out = bin()
        .args(["--dir", p, "release", "create", "R1"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // View both tasks to trigger lazy migration.
    for id in ["TASK-1", "TASK-002"] {
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

    // View both milestones to trigger lazy migration.
    for id in ["m-1", "MS-001"] {
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
            "MS-001",
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
        .args(["--dir", p, "milestone", "view", "MS-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Legacy Milestone Updated"));

    // Listing still shows all records.
    let out = bin().args(["--dir", p, "task", "list"]).output().unwrap();
    assert!(out.status.success());
    let list = String::from_utf8_lossy(&out.stdout);
    assert!(list.contains("TASK-002"));
    assert!(list.contains("task-1 - Storage-Task"));

    let out = bin()
        .args(["--dir", p, "milestone", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let list = String::from_utf8_lossy(&out.stdout);
    assert!(list.contains("MS-001"));
    assert!(list.contains("m-1 - Storage-Milestone"));
}
