use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("papyrus-import-{}", name));
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
        "init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn make_source(d: &std::path::Path) {
    let src = d.join("srcproj");
    fs::create_dir_all(src.join("backlog/tasks")).unwrap();
    fs::create_dir_all(src.join("backlog/archive/tasks")).unwrap();
    fs::create_dir_all(src.join("backlog/milestones")).unwrap();
    fs::create_dir_all(src.join("backlog/archive/milestones")).unwrap();
    fs::create_dir_all(src.join("backlog/docs")).unwrap();
    fs::create_dir_all(src.join("backlog/drafts")).unwrap();
    fs::create_dir_all(src.join("backlog/decisions")).unwrap();
    fs::write(src.join("backlog/config.yml"), "x: 1\n").unwrap();
    fs::create_dir_all(src.join("backlog/.locks")).unwrap();
    fs::write(src.join("backlog/.locks/l.lock"), "lock").unwrap();
    fs::write(src.join("backlog/docs/doc-1.md"), "doc").unwrap();
    fs::write(src.join("backlog/drafts/draft-1.md"), "draft").unwrap();
    fs::write(src.join("backlog/decisions/dec-1.md"), "dec").unwrap();

    let task1 = r#"---
id: task-1
title: Source Task
status: ToDo
assignee:
  - alice
created_date: '2026-01-01T00:00:00Z'
updated_date: '2026-01-02T00:00:00Z'
labels:
  - l1
milestone: m-1
dependencies:
  - task-2
references:
  - ref.md
documentation:
  - doc.md
type: task
ordinal: 5
priority: high
unknown_key: keepme
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Desc text
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 first
- [x] #2 second
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] done item
<!-- DOD:END -->

## Implementation Plan
<!-- SECTION:PLAN:BEGIN -->
plan step
<!-- SECTION:PLAN:END -->

## Implementation Notes
<!-- SECTION:NOTES:BEGIN -->
note text
<!-- SECTION:NOTES:END -->

## Comments
<!-- COMMENTS:BEGIN -->
comment text
<!-- COMMENTS:END -->

## Unknown
free text here
"#;
    fs::write(src.join("backlog/tasks/task-1 - Source Task.md"), task1).unwrap();

    let task2 = r#"---
id: task-2
title: Dep Task
status: In Progress
assignee: []
created_date: '2026-01-03T00:00:00Z'
updated_date: badtimestamp
milestone: m-999
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Dep desc
<!-- SECTION:DESCRIPTION:END -->
"#;
    fs::write(src.join("backlog/tasks/task-2.md"), task2).unwrap();

    // archived task with duplicate source number (task-1 duplicate)
    let arch_task = r#"---
id: task-3
title: Archived Task
status: Done
assignee: []
created_date: '2026-01-04T00:00:00Z'
updated_date: '2026-01-05T00:00:00Z'
---

## Description
archived body
"#;
    fs::write(
        src.join("backlog/archive/tasks/task-3 - Archived Task.md"),
        arch_task,
    )
    .unwrap();

    let ms1 = r#"---
id: m-1
title: Source Milestone
created_date: '2026-01-06T00:00:00Z'
updated_date: '2026-01-07T00:00:00Z'
labels: []
references: []
---

## Description

Milestone body text
"#;
    fs::write(
        src.join("backlog/milestones/m-1 - Source Milestone.md"),
        ms1,
    )
    .unwrap();

    let ms_arch = r#"---
id: m-2
title: Archived Milestone
created_date: '2026-01-08T00:00:00Z'
updated_date: '2026-01-09T00:00:00Z'
labels: []
references: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Archived ms desc
<!-- SECTION:DESCRIPTION:END -->
"#;
    fs::write(
        src.join("backlog/archive/milestones/m-2 - Archived Milestone.md"),
        ms_arch,
    )
    .unwrap();
}

fn hash_dir(dir: &std::path::Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut files: Vec<_> = walk_files(dir);
    files.sort();
    for f in files {
        f.hash(&mut hasher);
        if let Ok(b) = fs::read(&f) {
            b.hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn walk_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk_files(&p));
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn help_shows_import_backlog() {
    let out = bin()
        .args(["import", "backlog", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("--from"));
    assert!(text.contains("--source-backlog-dir"));
    assert!(text.contains("--dry-run"));
    assert!(text.contains("--allow-existing"));
    assert!(text.contains("--report"));
}

#[test]
fn invocation_and_basic_import() {
    let d = tmpdir("basic");
    let src = d.join("srcproj");
    // build source
    let _ = fs::remove_dir_all(&src);
    fs::create_dir_all(&d).unwrap();
    // reuse make_source by creating srcproj parent? make_source expects d/srcproj
    make_source(&d);
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // files exist
    assert!(tg.join("backlog/tasks/TASK-001.md").exists());
    assert!(tg.join("backlog/tasks/TASK-002.md").exists());
    assert!(tg.join("backlog/archive/tasks/TASK-003.md").exists()); // archived task
    assert!(tg.join("backlog/milestones/MS-001.md").exists());
    assert!(tg.join("backlog/archive/milestones/MS-002.md").exists());
    assert!(
        tg.join("backlog/archive/tasks/TASK-003.md").exists()
            || tg.join("backlog/tasks/TASK-003.md").exists()
    );
}

#[test]
fn containment_report_relative_path() {
    let d = tmpdir("containment");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    // outside report path fails
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--report",
            "../outside.md",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    // relative inside project succeeds
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--report",
            "report.md",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(tg.join("report.md").exists());
}

#[test]
fn source_scope_exclusions() {
    let d = tmpdir("excl");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Excluded: docs=1 drafts=1 decisions=1 config=1 locks=1"));
}

#[test]
fn id_allocation_from_existing() {
    let d = tmpdir("alloc");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    // create existing task TASK-005
    fs::create_dir_all(tg.join("backlog/tasks")).unwrap();
    let existing = "---\nid: TASK-005\ntitle: Existing\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: ToDo\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\nexisting\n";
    fs::write(tg.join("backlog/tasks/TASK-005.md"), existing).unwrap();
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--allow-existing",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // first imported task should be TASK-006
    assert!(tg.join("backlog/tasks/TASK-006.md").exists());
}

#[test]
fn field_mapping() {
    let d = tmpdir("fields");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--allow-existing",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let p = tg.join("backlog/tasks/TASK-001.md");
    let c = fs::read_to_string(&p).unwrap();
    assert!(c.contains("title: Source Task") || c.contains("title: >-"));
    assert!(c.contains("status: ToDo"));
    assert!(c.contains("- alice"));
    assert!(c.contains("created_date: '2026-01-01T00:00:00Z'"));
    // updated_date bad? task1 valid; task2 approximated. Check task2
    let c2 = fs::read_to_string(tg.join("backlog/tasks/TASK-002.md")).unwrap();
    // In Progress not configured -> preserved verbatim + warning; but status field
    assert!(c2.contains("status: In Progress"));
    assert!(c2.contains("created_date: '2026-01-03T00:00:00Z'"));
    // approximated updated_date should be import time (not badtimestamp)
    assert!(!c2.contains("badtimestamp"));
    // milestone translation: task1 milestone m-1 -> MS-001 (imported milestone number? milestones base 1, so m-1 -> MS-001)
    assert!(c.contains("milestone: MS-001"));
    // dangling milestone m-999 dropped to null
    assert!(c2.contains("milestone: null") || c2.contains("milestone: ~"));
    // dependencies: task1 dep task-2 -> TASK-002
    assert!(c.contains("dependencies:") && c.contains("TASK-002"));
    // references copied
    assert!(c.contains("ref.md") || c.contains("- ref.md"));
    // documentation copied
    assert!(c.contains("doc.md") || c.contains("- doc.md"));
    // type copied
    assert!(c.contains("type: task"));
    // ordinal copied
    assert!(c.contains("ordinal: 5"));
    // unknown key preserved
    assert!(c.contains("unknown_key: keepme"));
    // no release in milestone? check milestone file no release? Actually release: null written? Spec says no milestone release value. We set release null. Check not a source number. Fine.
    let ms = fs::read_to_string(tg.join("backlog/milestones/MS-001.md")).unwrap();
    // release null allowed; ensure no source id
    assert!(!ms.contains("task-1"));
}

#[test]
fn body_mapping() {
    let d = tmpdir("body");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--allow-existing",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let c = fs::read_to_string(tg.join("backlog/tasks/TASK-001.md")).unwrap();
    assert!(c.contains("## Description"));
    assert!(c.contains("Desc text"));
    assert!(c.contains("## Acceptance Criteria"));
    assert!(c.contains("- [ ] first"));
    assert!(c.contains("- [x] second"));
    assert!(!c.contains("#1") && !c.contains("#2"));
    assert!(c.contains("## Definition of Done"));
    assert!(c.contains("## Implementation Notes"));
    assert!(c.contains("plan step"));
    assert!(c.contains("## Notes"));
    assert!(c.contains("note text"));
    assert!(c.contains("## Comments"));
    assert!(c.contains("comment text"));
    assert!(c.contains("## Unknown"));
    assert!(c.contains("free text here"));
    // marker comments removed
    assert!(!c.contains("SECTION:DESCRIPTION:BEGIN"));
    assert!(!c.contains("AC:BEGIN"));
    // milestone single description
    let ms = fs::read_to_string(tg.join("backlog/milestones/MS-001.md")).unwrap();
    assert!(ms.contains("## Description"));
    assert!(ms.contains("Milestone body text"));
}

#[test]
fn collision_and_duplicates() {
    let d = tmpdir("coll");
    make_source(&d);
    let src = d.join("srcproj");
    // create a duplicate source number
    let dup = fs::read_to_string(src.join("backlog/tasks/task-1 - Source Task.md")).unwrap();
    fs::write(src.join("backlog/tasks/task-1 - Duplicate.md"), dup).unwrap();
    let tg = d.join("tg");
    init(&tg);
    // same-title existing target
    fs::create_dir_all(tg.join("backlog/tasks")).unwrap();
    let existing = "---\nid: TASK-009\ntitle: Source Task\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: ToDo\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\nx\n";
    fs::write(tg.join("backlog/tasks/TASK-009.md"), existing).unwrap();

    // existing target refusal without allow-existing
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8_lossy(&out.stderr);
    // either existing family refusal
    assert!(text.contains("already holds") || text.contains("allow-existing"));

    // with allow-existing, same-title warning appears
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--allow-existing",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("possible duplicate"));
    // duplicate source number anomaly reported
    assert!(stdout.contains("duplicate source number"));
    // skipped? archived dup has no title? it has title. no skip
}

#[test]
fn dry_run_no_writes_and_immutable_source() {
    let d = tmpdir("dry");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let before = hash_dir(&src);
    let before_files: Vec<_> = walk_files(&tg.join("backlog/tasks")).into_iter().collect();
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .unwrap();
    // dry run with skipped? no skipped, should succeed
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = hash_dir(&src);
    assert_eq!(before, after);
    // no new task files
    let after_files: Vec<_> = walk_files(&tg.join("backlog/tasks")).into_iter().collect();
    assert_eq!(before_files.len(), after_files.len());
}

#[test]
fn status_mismatch_preserved() {
    let d = tmpdir("status");
    make_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
            "--allow-existing",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("status 'In Progress' does not match")
            || stdout.contains("does not match a configured status")
    );
}

fn make_at_source(d: &std::path::Path) {
    let src = d.join("srcproj");
    fs::create_dir_all(src.join("backlog/tasks")).unwrap();
    fs::create_dir_all(src.join("backlog/milestones")).unwrap();

    let task1 = r#"---
id: task-1
title: At Assignee Task
status: ToDo
assignee:
  - @rust-dev
  - @qa
created_date: '2026-01-01T00:00:00Z'
updated_date: '2026-01-02T00:00:00Z'
labels:
  - l1
  - l2
milestone: m-1
dependencies:
  - task-2
references:
  - ref.md
documentation:
  - doc.md
type: task
ordinal: 7
---
## Description
desc
"#;
    fs::write(
        src.join("backlog/tasks/task-1 - At Assignee Task.md"),
        task1,
    )
    .unwrap();

    let task2 = r#"---
id: task-2
title: Dep Task
status: ToDo
assignee: []
created_date: '2026-01-03T00:00:00Z'
updated_date: '2026-01-03T00:00:00Z'
---
"#;
    fs::write(src.join("backlog/tasks/task-2.md"), task2).unwrap();

    let ms1 = r#"---
id: m-1
title: Source Milestone
created_date: '2026-01-06T00:00:00Z'
updated_date: '2026-01-07T00:00:00Z'
---
"#;
    fs::write(
        src.join("backlog/milestones/m-1 - Source Milestone.md"),
        ms1,
    )
    .unwrap();
}

#[test]
fn import_quoted_assignee_and_list_fields_survive() {
    let d = tmpdir("quoted_assignee");
    make_at_source(&d);
    let src = d.join("srcproj");
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let p = tg.join("backlog/tasks/TASK-001.md");
    let c = fs::read_to_string(&p).unwrap();
    assert!(c.contains("assignee:\n  - '@rust-dev'"), "got: {}", c);
    assert!(c.contains("  - '@qa'"), "got: {}", c);
    assert!(c.contains("labels:\n  - l1"), "got: {}", c);
    assert!(c.contains("  - l2"), "got: {}", c);
    assert!(c.contains("dependencies:\n  - TASK-002"), "got: {}", c);
    assert!(c.contains("documentation:\n  - doc.md"), "got: {}", c);
    assert!(c.contains("references:\n  - ref.md"), "got: {}", c);

    // round-trip via task view
    let out = bin()
        .args(["--dir", tg.to_str().unwrap(), "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = String::from_utf8_lossy(&out.stdout);
    assert!(v.contains("@rust-dev"), "got: {}", v);
    assert!(v.contains("@qa"), "got: {}", v);
    assert!(v.contains("TASK-002"), "got: {}", v);
    assert!(v.contains("doc.md"), "got: {}", v);
}

#[test]
fn import_empty_assignee_list_stays_empty() {
    let d = tmpdir("empty_assignee");
    // minimal source with empty assignee
    let src = d.join("srcproj");
    fs::create_dir_all(src.join("backlog/tasks")).unwrap();
    let task = r#"---
id: task-1
title: Empty Assignee Task
status: ToDo
assignee: []
created_date: '2026-01-01T00:00:00Z'
updated_date: '2026-01-02T00:00:00Z'
---
"#;
    fs::create_dir_all(src.join("backlog/tasks")).unwrap();
    fs::write(src.join("backlog/tasks/task-1.md"), task).unwrap();
    let tg = d.join("tg");
    init(&tg);
    let out = bin()
        .args([
            "--dir",
            tg.to_str().unwrap(),
            "import",
            "backlog",
            "--from",
            src.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let c = fs::read_to_string(tg.join("backlog/tasks/TASK-001.md")).unwrap();
    assert!(c.contains("assignee: []"), "got: {}", c);
    let out = bin()
        .args(["--dir", tg.to_str().unwrap(), "task", "view", "TASK-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("assignee: []"));
}
