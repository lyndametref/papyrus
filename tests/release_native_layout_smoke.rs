//! TASK-98 release smoke: ADR-0013 stage 8 end-to-end properties and packaging
//! prerequisites.
//!
//! This file adds milestone-level evidence on top of the authoritative
//! per-feature suites (`tests/no_drift.rs`, `tests/config_relocation_cli.rs`,
//! `tests/import_backlog_cli.rs`, `tests/search_cli.rs`, `tests/browser_cli.rs`).
//! It never reads or writes this repository's own `backlog/` store; every
//! fixture is built under `./.tmp/` and removed by [`Scratch`].
//!
//! Covered acceptance criteria: T-98-01 fixture inventory and repository-store
//! protection; T-98-02 adoption and no-op byte identity; T-98-03 no
//! Backlog-shaped write; T-98-04 invisibility before import and readability
//! after; T-98-05 one Papyrus ID on every surface; T-98-06 import fidelity;
//! T-98-07 documented upgrade path; T-98-08 relocation atomicity, conflict,
//! containment, and browser path; T-98-09 front-matter-only search with no
//! write.
//!
//! T-98-10 to T-98-13 (quality gates, package metadata, installed smoke, and the
//! recorded evidence) are exercised by the recorded procedure in
//! `docs/packaging.md`. `t98_03` also asserts that the mechanical suite still
//! carries its eight checks.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// Process and filesystem helpers
// ---------------------------------------------------------------------------

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Project-local scratch directory with automatic cleanup, including on panic.
/// The project convention forbids system temporary directories.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = repo_root()
            .join(".tmp")
            .join(format!("task98-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Scratch(path)
    }
}

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    bin().arg("--dir").arg(dir).args(args).output().unwrap()
}

fn run_ok(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_project(dir: &Path) {
    run_ok(dir, &["init", "Proj", "--no-git"]);
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    walk(dir, &mut out);
    out.sort();
    out
}

/// Deterministic content and mode hash of a tree, including relative paths.
fn hash_tree(dir: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    for file in walk_files(dir) {
        let relative = file.strip_prefix(dir).unwrap_or(&file);
        relative.to_string_lossy().hash(&mut hasher);
        let metadata = fs::metadata(&file).unwrap();
        metadata.permissions().mode().hash(&mut hasher);
        fs::read(&file).unwrap().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap().flatten() {
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    bytes: Vec<u8>,
    mode: u32,
    modified: SystemTime,
}

fn snapshot(path: &Path) -> Snapshot {
    let metadata = fs::metadata(path).unwrap();
    Snapshot {
        bytes: fs::read(path).unwrap(),
        mode: metadata.permissions().mode(),
        modified: metadata.modified().unwrap(),
    }
}

/// Ensure a later rewrite cannot coincidentally keep the previous timestamp.
fn sleep_until_mtime_can_change() {
    std::thread::sleep(Duration::from_millis(1100));
}

/// Assert that adoption prepended a well-formed front-matter block and left
/// every body byte unchanged.
///
/// The adopted bytes must be exactly the block followed by the original bytes,
/// so any dropped, added, reordered, or re-encoded body byte fails here.
fn assert_adoption_only_prepends_front_matter(original: &str, adopted: &str, id: &str) {
    let prefix = adopted.strip_suffix(original).unwrap_or_else(|| {
        panic!("adoption read changed the body bytes of {id}\nadopted: {adopted:?}")
    });
    assert!(
        prefix.starts_with("---\n"),
        "adoption did not prepend a front-matter block to {id}: {prefix:?}"
    );
    assert_eq!(
        prefix.lines().filter(|line| *line == "---").count(),
        2,
        "adopted front-matter block for {id} is malformed: {prefix:?}"
    );
    assert!(
        prefix.contains(&format!("id: {id}")),
        "adopted front-matter block for {id} has no id: {prefix:?}"
    );
    let close = prefix
        .find("\n---\n")
        .unwrap_or_else(|| panic!("front-matter block for {id} is not closed: {prefix:?}"))
        + "\n---\n".len();
    assert!(
        prefix[close..].chars().all(|c| c == '\n'),
        "adoption added body content beyond the block for {id}: {prefix:?}"
    );
}

/// Every `task-<n>` / `m-<n>` form in captured output, ignoring word-internal
/// matches such as `form-1`.
fn backlog_id_forms(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (index, _) in text.char_indices() {
        let previous = text[..index].chars().next_back();
        if previous.is_some_and(|c| c.is_alphanumeric()) {
            continue;
        }
        for prefix in ["task-", "m-"] {
            let Some(rest) = text[index..].strip_prefix(prefix) else {
                continue;
            };
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                out.push(format!("{}{}", prefix, digits));
            }
        }
    }
    out
}

fn assert_no_backlog_ids(surface: &str, text: &str) {
    let forms = backlog_id_forms(text);
    assert!(
        forms.is_empty(),
        "{} exposes Backlog ID form(s) {:?}",
        surface,
        forms
    );
}

fn assert_no_storage_fields(surface: &str, text: &str) {
    for key in ["storage_id", "display_id", "storage_path"] {
        assert!(
            !text.contains(key),
            "{} exposes storage field {}",
            surface,
            key
        );
    }
}

fn repository_backlog_status() -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["status", "--porcelain", "--", "backlog"])
        .output()
        .expect("git status");
    assert!(out.status.success(), "git status failed");
    String::from_utf8_lossy(&out.stdout).to_string()
}

// ---------------------------------------------------------------------------
// Sample Backlog.md project fixture
// ---------------------------------------------------------------------------

const TASK_1: &str = r#"---
id: task-1
title: Foundation Task
status: ToDo
assignee:
  - alice
created_date: '2026-01-01T00:00:00Z'
updated_date: '2026-01-02T00:00:00Z'
labels:
  - alpha
  - fm-only-label
milestone: m-0
dependencies:
  - task-2
references:
  - ref.md
documentation:
  - doc.md
type: task
ordinal: 7
unknown_key: keepme
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Foundation description text.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 first criterion
- [x] #2 second criterion
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] done item
<!-- DOD:END -->

## Implementation Plan
<!-- SECTION:PLAN:BEGIN -->
plan step text
<!-- SECTION:PLAN:END -->

## Implementation Notes
<!-- SECTION:NOTES:BEGIN -->
implementation note text
<!-- SECTION:NOTES:END -->

## Comments
<!-- COMMENTS:BEGIN -->
- **alice**: a comment
<!-- COMMENTS:END -->

## Unknown Section

Unknown section content kept verbatim.

Free text outside any marker block.
"#;

const TASK_2: &str = r#"---
id: task-2
title: Dependent Task
status: In Progress
assignee: []
created_date: '2026-01-03T00:00:00Z'
updated_date: '2026-01-04T00:00:00Z'
milestone: m-999
dependencies: []
labels:
  - beta
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Dependent description.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 only criterion
<!-- AC:END -->
"#;

const TASK_3: &str = r#"---
id: task-3
title: Third Task
status: Implemented
assignee:
  - bob
created_date: '2026-01-05T00:00:00Z'
updated_date: '2026-01-06T00:00:00Z'
labels: []
references:
  - spec.md
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Third description.
<!-- SECTION:DESCRIPTION:END -->
"#;

const TASK_4_ARCHIVED: &str = r#"---
id: task-4
title: Archived Task
status: Done
assignee: []
created_date: '2026-01-07T00:00:00Z'
updated_date: '2026-01-08T00:00:00Z'
---

## Description

Archived body.
"#;

const TASK_5: &str = r#"---
id: task-5
title: Fifth Task
status: ToDo
assignee: []
created_date: '2026-01-09T00:00:00Z'
updated_date: '2026-01-10T00:00:00Z'
---

## Description

Fifth body.
"#;

const MILESTONE_0: &str = r#"---
id: m-0
title: First Milestone
created_date: '2026-01-11T00:00:00Z'
updated_date: '2026-01-12T00:00:00Z'
labels:
  - ms-label
references:
  - ms-ref.md
---

## Description

First milestone body.
"#;

const MILESTONE_1: &str = r#"---
id: m-1
title: Second Milestone
created_date: '2026-01-13T00:00:00Z'
updated_date: '2026-01-14T00:00:00Z'
labels: []
references: []
---

## Description

Second milestone body.
"#;

const MILESTONE_2_ARCHIVED: &str = r#"---
id: m-2
title: Archived Milestone
created_date: '2026-01-15T00:00:00Z'
updated_date: '2026-01-16T00:00:00Z'
labels: []
references: []
---

## Description

Archived milestone body.
"#;

/// Build the isolated sample Backlog.md project at `<root>/source`.
fn build_fixture(root: &Path) -> PathBuf {
    let source = root.join("source");
    let backlog = source.join("backlog");

    write(&backlog.join("tasks/task-1 - Foundation Task.md"), TASK_1);
    write(&backlog.join("tasks/task-2 - Dependent Task.md"), TASK_2);
    write(&backlog.join("tasks/task-3 - Third Task.md"), TASK_3);
    write(
        &backlog.join("archive/tasks/task-4 - Archived Task.md"),
        TASK_4_ARCHIVED,
    );
    write(&backlog.join("tasks/task-5 - Fifth Task.md"), TASK_5);
    write(
        &backlog.join("milestones/m-0 - First Milestone.md"),
        MILESTONE_0,
    );
    write(
        &backlog.join("milestones/m-1 - Second Milestone.md"),
        MILESTONE_1,
    );
    write(
        &backlog.join("archive/milestones/m-2 - Archived Milestone.md"),
        MILESTONE_2_ARCHIVED,
    );
    write(&backlog.join("docs/doc-1.md"), "docs body\n");
    write(&backlog.join("drafts/draft-1.md"), "draft body\n");
    write(&backlog.join("decisions/dec-1.md"), "decision body\n");
    write(&backlog.join("config.yml"), "project_name: Source\n");
    write(&backlog.join(".locks/store.lock"), "lock\n");

    source
}

// ---------------------------------------------------------------------------
// Browser server and HTTP helpers
// ---------------------------------------------------------------------------

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

struct Server {
    child: Child,
    port: u16,
}

impl Server {
    /// Start the browser for `project`; stderr is written under `log_dir` so a
    /// project hash is never disturbed by server diagnostics.
    fn start(project: &Path, log_dir: &Path) -> Server {
        for attempt in 0..10 {
            let port = free_port();
            let stderr_path = log_dir.join(format!("browser-{}-{}.stderr", port, attempt));
            let stderr = fs::File::create(&stderr_path).unwrap();
            let mut child = bin()
                .arg("--dir")
                .arg(project)
                .args(["browser", "--no-open", "--port"])
                .arg(port.to_string())
                .stdout(Stdio::piped())
                .stderr(Stdio::from(stderr))
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if startup.trim().is_empty() {
                let _ = child.wait();
                continue;
            }
            return Server { child, port };
        }
        panic!("browser did not start after retries");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn parse_response(bytes: &[u8]) -> HttpResponse {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response header terminator");
    let head = String::from_utf8_lossy(&bytes[..split]).to_string();
    let body = bytes[split + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line
        .split(' ')
        .nth(1)
        .and_then(|value| value.parse().ok())
        .expect("status code");
    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    HttpResponse {
        status,
        headers,
        body,
    }
}

fn request(
    port: u16,
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: Option<&str>,
) -> HttpResponse {
    let mut raw = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        method, path, port
    )
    .into_bytes();
    if let Some(content_type) = content_type {
        raw.extend_from_slice(format!("Content-Type: {}\r\n", content_type).as_bytes());
    }
    if let Some(body) = body {
        raw.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    raw.extend_from_slice(b"\r\n");
    if let Some(body) = body {
        raw.extend_from_slice(body.as_bytes());
    }

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(&raw).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    parse_response(&bytes)
}

fn get(server: &Server, path: &str) -> HttpResponse {
    request(server.port, "GET", path, None, None)
}

fn post_form(server: &Server, path: &str, body: &str) -> HttpResponse {
    request(
        server.port,
        "POST",
        path,
        Some("application/x-www-form-urlencoded"),
        Some(body),
    )
}

fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{:02X}", other)),
        }
    }
    out
}

fn form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", urlencode(key), urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn extract_version(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').expect("version terminator");
    rest[..end].to_string()
}

fn extract_card_version(html: &str, task_id: &str) -> String {
    let marker = format!("data-task=\"{}\"", task_id);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("card {}", task_id));
    extract_version(&html[start..])
}

fn column_containing<'a>(html: &'a str, status: &str) -> &'a str {
    let marker = format!("data-status=\"{}\"", status);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("column {}", status));
    let end = html[start..]
        .find("</section>")
        .map(|offset| start + offset)
        .unwrap_or(html.len());
    &html[start..end]
}

// ---------------------------------------------------------------------------
// T-98-01 — fixture inventory and repository-store protection
// ---------------------------------------------------------------------------

#[test]
fn t98_01_fixture_inventory_and_repository_store_untouched() {
    let repo_backlog = repo_root().join("backlog");
    let status_before = repository_backlog_status();
    let hash_before = hash_tree(&repo_backlog);

    let scratch = Scratch::new("fixture");
    let source = build_fixture(&scratch);
    let backlog = source.join("backlog");

    for path in [
        "tasks/task-1 - Foundation Task.md",
        "tasks/task-2 - Dependent Task.md",
        "tasks/task-3 - Third Task.md",
        "tasks/task-5 - Fifth Task.md",
        "archive/tasks/task-4 - Archived Task.md",
        "milestones/m-0 - First Milestone.md",
        "milestones/m-1 - Second Milestone.md",
        "archive/milestones/m-2 - Archived Milestone.md",
        "docs/doc-1.md",
        "drafts/draft-1.md",
        "decisions/dec-1.md",
        "config.yml",
        ".locks/store.lock",
    ] {
        assert!(backlog.join(path).exists(), "fixture missing {}", path);
    }

    assert_eq!(walk_files(&backlog.join("tasks")).len(), 4);
    assert_eq!(walk_files(&backlog.join("archive/tasks")).len(), 1);
    assert_eq!(walk_files(&backlog.join("milestones")).len(), 2);
    assert_eq!(walk_files(&backlog.join("archive/milestones")).len(), 1);

    let task_1 = fs::read_to_string(backlog.join("tasks/task-1 - Foundation Task.md")).unwrap();
    for marker in [
        "<!-- SECTION:DESCRIPTION:BEGIN -->",
        "<!-- AC:BEGIN -->",
        "<!-- DOD:BEGIN -->",
        "<!-- SECTION:PLAN:BEGIN -->",
        "<!-- SECTION:NOTES:BEGIN -->",
        "<!-- COMMENTS:BEGIN -->",
        "## Unknown Section",
        "Free text outside any marker block.",
    ] {
        assert!(task_1.contains(marker), "fixture body marker {}", marker);
    }
    for key in [
        "title:",
        "status:",
        "assignee:",
        "created_date:",
        "updated_date:",
        "labels:",
        "milestone:",
        "dependencies:",
        "references:",
        "documentation:",
        "type:",
        "ordinal:",
        "unknown_key:",
    ] {
        assert!(task_1.contains(key), "fixture front-matter key {}", key);
    }

    drop(scratch);
    assert_eq!(
        hash_tree(&repo_backlog),
        hash_before,
        "backlog/ hash changed"
    );
    assert_eq!(
        repository_backlog_status(),
        status_before,
        "backlog/ git status changed"
    );
}

// ---------------------------------------------------------------------------
// T-98-02 — adoption and no-op writes are byte-identical
// ---------------------------------------------------------------------------

#[test]
fn t98_02_adoption_and_noop_writes_are_byte_identical() {
    let scratch = Scratch::new("idempotent");
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);

    // An already-front-matter task: repeated reads and equal-value patches
    // change no byte, mode, or mtime and do not refresh updated_date.
    let task_id = run_ok(&project, &["task", "create", "Same Title"]);
    let task_path = project.join(format!("backlog/tasks/{}.md", task_id));
    sleep_until_mtime_can_change();
    let before = snapshot(&task_path);
    run_ok(&project, &["task", "view", &task_id]);
    run_ok(&project, &["task", "view", &task_id]);
    run_ok(&project, &["task", "edit", &task_id, "--status", "ToDo"]);
    run_ok(
        &project,
        &["task", "edit", &task_id, "--title", "Same Title"],
    );
    assert_eq!(
        snapshot(&task_path),
        before,
        "no-op task read/edit changed the file"
    );

    // A legacy task adopts front matter on first read. Adoption adds only the
    // front-matter block: every byte of the original body is preserved.
    let legacy_task = project.join("backlog/tasks/TASK-010.md");
    let legacy_task_body =
        "# TASK-010 Legacy Task\n\n## Status\nDone\n\n## Description\nLegacy task body.\n";
    write(&legacy_task, legacy_task_body);
    run_ok(&project, &["task", "view", "TASK-010"]);
    let adopted_text = fs::read_to_string(&legacy_task).unwrap();
    assert_adoption_only_prepends_front_matter(legacy_task_body, &adopted_text, "TASK-010");
    assert!(
        adopted_text.contains("status: Done"),
        "adoption did not move the legacy ## Status value into front matter: {adopted_text}"
    );
    assert!(
        adopted_text.contains("# TASK-010 Legacy Task"),
        "adoption read must keep the legacy heading: {adopted_text}"
    );

    // A repeated adoption read changes no byte, mode, or mtime.
    sleep_until_mtime_can_change();
    let adopted = snapshot(&legacy_task);
    run_ok(&project, &["task", "view", "TASK-010"]);
    assert_eq!(
        snapshot(&legacy_task),
        adopted,
        "second legacy task read changed the file"
    );

    // The first write retires the legacy title heading and the `## Status`
    // metadata section (ADR-0012 decision 4, TASK-89); content sections stay.
    run_ok(&project, &["task", "edit", "TASK-010", "--status", "Done"]);
    let retired_task = fs::read_to_string(&legacy_task).unwrap();
    assert!(
        !retired_task.lines().any(|line| line.starts_with("# ")),
        "first write left a legacy title heading: {retired_task}"
    );
    assert!(
        !retired_task
            .lines()
            .any(|line| line.trim_end() == "## Status"),
        "first write left the legacy ## Status section: {retired_task}"
    );
    assert!(
        retired_task.contains("## Description\nLegacy task body.\n"),
        "first write dropped a retained content section: {retired_task}"
    );
    assert!(
        retired_task.contains("status: Done"),
        "first write lost the front-matter status value: {retired_task}"
    );
    let canonical = snapshot(&legacy_task);
    run_ok(&project, &["task", "edit", "TASK-010", "--status", "Done"]);
    assert_eq!(
        snapshot(&legacy_task),
        canonical,
        "equal-value write after retirement changed the file"
    );
    assert!(legacy_task.exists(), "adoption must not rename the file");

    // A read after retirement and the equal-value write changes no byte,
    // mode, or mtime.
    sleep_until_mtime_can_change();
    run_ok(&project, &["task", "view", "TASK-010"]);
    assert_eq!(
        snapshot(&legacy_task),
        canonical,
        "post-retirement task read changed the file"
    );

    // A legacy milestone behaves the same way: adoption prepends the block
    // only, then the first write retires the heading and `## Release`.
    let legacy_ms = project.join("backlog/milestones/MS-010.md");
    let legacy_ms_body = "# MS-010 Legacy Milestone\n\n## Release\nREL-001\n\n## Description\nLegacy milestone body.\n";
    write(&legacy_ms, legacy_ms_body);
    run_ok(&project, &["milestone", "view", "MS-010"]);
    let adopted_text = fs::read_to_string(&legacy_ms).unwrap();
    assert_adoption_only_prepends_front_matter(legacy_ms_body, &adopted_text, "MS-010");
    assert!(
        adopted_text.contains("release: REL-001"),
        "adoption did not move the legacy ## Release value into front matter: {adopted_text}"
    );
    assert!(
        adopted_text.contains("# MS-010 Legacy Milestone"),
        "adoption read must keep the legacy heading: {adopted_text}"
    );
    sleep_until_mtime_can_change();
    let adopted = snapshot(&legacy_ms);
    run_ok(&project, &["milestone", "view", "MS-010"]);
    assert_eq!(
        snapshot(&legacy_ms),
        adopted,
        "second legacy milestone read changed the file"
    );
    run_ok(
        &project,
        &["milestone", "edit", "MS-010", "--title", "Legacy Milestone"],
    );
    let retired_ms = fs::read_to_string(&legacy_ms).unwrap();
    assert!(
        !retired_ms.lines().any(|line| line.starts_with("# ")),
        "first milestone write left a legacy title heading: {retired_ms}"
    );
    assert!(
        !retired_ms
            .lines()
            .any(|line| line.trim_end() == "## Release"),
        "first milestone write left the legacy ## Release section: {retired_ms}"
    );
    assert!(
        retired_ms.contains("## Description\nLegacy milestone body.\n"),
        "first milestone write dropped a retained content section: {retired_ms}"
    );
    assert!(
        retired_ms.contains("release: REL-001"),
        "first milestone write lost the front-matter release value: {retired_ms}"
    );
    let canonical = snapshot(&legacy_ms);
    run_ok(
        &project,
        &["milestone", "edit", "MS-010", "--title", "Legacy Milestone"],
    );
    assert_eq!(
        snapshot(&legacy_ms),
        canonical,
        "equal-value write after retirement changed the file"
    );

    // A read after retirement and the equal-value write changes no byte,
    // mode, or mtime.
    sleep_until_mtime_can_change();
    run_ok(&project, &["milestone", "view", "MS-010"]);
    assert_eq!(
        snapshot(&legacy_ms),
        canonical,
        "post-retirement milestone read changed the file"
    );

    // A legacy document record type adopts front matter on first read and is
    // then idempotent; adoption again changes only the prepended block.
    let legacy_spec = project.join("docs/technical/TECH-010.md");
    let legacy_spec_body = "# TECH-010 Legacy Spec\n\n## Description\nLegacy specification body.\n";
    write(&legacy_spec, legacy_spec_body);
    run_ok(&project, &["tech-spec", "view", "TECH-010"]);
    let adopted_text = fs::read_to_string(&legacy_spec).unwrap();
    assert_adoption_only_prepends_front_matter(legacy_spec_body, &adopted_text, "TECH-010");
    sleep_until_mtime_can_change();
    let adopted = snapshot(&legacy_spec);
    run_ok(&project, &["tech-spec", "view", "TECH-010"]);
    run_ok(
        &project,
        &["tech-spec", "edit", "TECH-010", "--title", "Legacy Spec"],
    );
    assert_eq!(
        snapshot(&legacy_spec),
        adopted,
        "second legacy document adoption changed the file"
    );

    // A read after the document write changes no byte, mode, or mtime.
    sleep_until_mtime_can_change();
    run_ok(&project, &["tech-spec", "view", "TECH-010"]);
    assert_eq!(
        snapshot(&legacy_spec),
        adopted,
        "post-write legacy document read changed the file"
    );

    // A read that cannot adopt safely reports the error and leaves the file
    // byte-identical; the record is readable once the directory is writable.
    let readonly_task = project.join("backlog/tasks/TASK-020.md");
    write(
        &readonly_task,
        "# TASK-020 Read Only\n\n## Description\nRead only body.\n",
    );
    let tasks_dir = project.join("backlog/tasks");
    let original = fs::metadata(&tasks_dir).unwrap().permissions();
    let original_bytes = fs::read(&readonly_task).unwrap();
    let mut readonly = original.clone();
    readonly.set_mode(0o555);
    fs::set_permissions(&tasks_dir, readonly).unwrap();
    let out = run(&project, &["task", "view", "TASK-020"]);
    fs::set_permissions(&tasks_dir, original).unwrap();
    assert!(
        !out.status.success(),
        "read-only adoption must report an error"
    );
    assert_eq!(
        fs::read(&readonly_task).unwrap(),
        original_bytes,
        "failed adoption changed the file"
    );
    let view = run_ok(&project, &["task", "view", "TASK-020"]);
    assert!(
        view.contains("Read only body."),
        "record not readable after restoring permissions: {view}"
    );
}

// ---------------------------------------------------------------------------
// T-98-03 — no Backlog-shaped write across the exercised lifecycle
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn t98_03_no_backlog_shaped_write_anywhere() {
    let scratch = Scratch::new("no-shape");
    let source = build_fixture(&scratch);
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);

    // create / edit / archive / restore for tasks; close for milestones.
    let task_id = run_ok(&project, &["task", "create", "Lifecycle Task"]);
    run_ok(
        &project,
        &["task", "edit", &task_id, "--status", "Initiated"],
    );
    run_ok(&project, &["task", "archive", &task_id]);
    run_ok(&project, &["task", "restore", &task_id]);
    let ms_id = run_ok(&project, &["milestone", "create", "Lifecycle Milestone"]);
    run_ok(&project, &["milestone", "close", &ms_id]);

    // archive a decision record.
    let decision_id = run_ok(&project, &["decision", "create", "Lifecycle Decision"]);
    run_ok(
        &project,
        &["decision", "edit", &decision_id, "--status", "accepted"],
    );
    run_ok(
        &project,
        &["decision", "edit", &decision_id, "--status", "archived"],
    );

    // adopt a legacy task and milestone, then edit them (retirement on write).
    write(
        &project.join("backlog/tasks/TASK-050.md"),
        "# TASK-050 Legacy Adopt\n\n## Status\nDone\n\n## Description\nAdopted body.\n",
    );
    run_ok(&project, &["task", "view", "TASK-050"]);
    run_ok(
        &project,
        &["task", "edit", "TASK-050", "--title", "Adopted Updated"],
    );
    write(
        &project.join("backlog/milestones/MS-050.md"),
        "# MS-050 Legacy Milestone\n\n## Description\nAdopted milestone.\n",
    );
    run_ok(&project, &["milestone", "view", "MS-050"]);
    run_ok(
        &project,
        &[
            "milestone",
            "edit",
            "MS-050",
            "--title",
            "Adopted Milestone",
        ],
    );

    // import the sample store.
    run_ok(
        &project,
        &[
            "import",
            "backlog",
            "--from",
            source.to_str().unwrap(),
            "--allow-existing",
        ],
    );

    // Every written record path is native.
    let mut checked_tasks = 0;
    let mut checked_milestones = 0;
    for file in walk_files(&project) {
        if file.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let relative = file.strip_prefix(&project).unwrap();
        let stem = file.file_stem().unwrap().to_str().unwrap().to_string();
        assert!(
            !stem.starts_with("task-") && !stem.starts_with("m-"),
            "Backlog-shaped stem written: {}",
            relative.display()
        );

        if relative.starts_with("backlog/tasks") || relative.starts_with("backlog/archive/tasks") {
            assert!(
                stem.len() == 8
                    && stem.starts_with("TASK-")
                    && stem[5..].chars().all(|c| c.is_ascii_digit()),
                "non-native task path: {}",
                relative.display()
            );
            checked_tasks += 1;
        } else if relative.starts_with("backlog/milestones")
            || relative.starts_with("backlog/archive/milestones")
        {
            assert!(
                stem.len() == 6
                    && stem.starts_with("MS-")
                    && stem[3..].chars().all(|c| c.is_ascii_digit()),
                "non-native milestone path: {}",
                relative.display()
            );
            checked_milestones += 1;
        } else if relative.starts_with("decisions/") {
            let rest = stem
                .strip_prefix("ADR-")
                .or_else(|| stem.strip_prefix("BDR-"));
            let Some(rest) = rest else {
                panic!("non-native decision path: {}", relative.display());
            };
            let (number, slug) = rest.split_once('-').expect("decision slug");
            assert!(
                !number.is_empty()
                    && number.chars().all(|c| c.is_ascii_digit())
                    && !slug.is_empty()
                    && slug
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "non-native decision path: {}",
                relative.display()
            );
        }

        // Task and milestone records never regain a legacy metadata section
        // and never carry a storage identity.
        if relative.starts_with("backlog/") {
            let content = fs::read_to_string(&file).unwrap();
            for key in ["storage_id", "display_id", "storage_path"] {
                assert!(
                    !content.contains(key),
                    "{} carries storage field {}",
                    relative.display(),
                    key
                );
            }
            assert!(
                backlog_id_forms(&content).is_empty(),
                "{} carries a storage-shaped ID",
                relative.display()
            );
            if relative.to_string_lossy().contains("tasks/") {
                assert!(
                    !content.contains("\n## Status\n"),
                    "{} regained a legacy ## Status section",
                    relative.display()
                );
            }
        }
    }
    assert!(checked_tasks >= 6, "task paths written: {}", checked_tasks);
    assert!(
        checked_milestones >= 3,
        "milestone paths written: {}",
        checked_milestones
    );

    // The mechanical suite remains part of the same `cargo test` run and still
    // carries its eight checks; its pass result is recorded in the gate table.
    let no_drift = fs::read_to_string(repo_root().join("tests/no_drift.rs")).unwrap();
    assert_eq!(
        no_drift.matches("\nfn no_").count(),
        8,
        "tests/no_drift.rs no longer carries eight mechanical checks"
    );
}

// ---------------------------------------------------------------------------
// T-98-04 — invisibility before import, readability after, source immutable
// ---------------------------------------------------------------------------

#[test]
fn t98_04_invisible_before_import_readable_after() {
    let scratch = Scratch::new("invisible");
    let source = build_fixture(&scratch);
    let source_hash = hash_tree(&source);

    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);
    // Another family stays visible, proving invisibility is specific to
    // unrecognized Backlog stems and not a general read failure.
    run_ok(
        &project,
        &["req", "create-group", "--title", "Visible Group"],
    );
    run_ok(
        &project,
        &[
            "req",
            "create",
            "Visible Requirement",
            "--group",
            "REQ-001",
            "-d",
            "Visible requirement body.",
        ],
    );
    copy_tree(&source.join("backlog"), &project.join("backlog"));

    // CLI surfaces report no tasks or milestones and no Backlog ID resolves.
    assert_eq!(run_ok(&project, &["task", "list"]), "");
    assert_eq!(run_ok(&project, &["milestone", "list"]), "");
    assert_eq!(run_ok(&project, &["task", "list", "--json"]), "[]");
    assert_eq!(run_ok(&project, &["milestone", "list", "--json"]), "[]");
    assert_eq!(run_ok(&project, &["search", "Foundation"]), "");
    for (kind, id) in [("task", "task-1"), ("milestone", "m-0")] {
        let out = run(&project, &[kind, "view", id]);
        assert_eq!(
            out.status.code(),
            Some(3),
            "{} {} must not resolve",
            kind,
            id
        );
    }

    // Browser surfaces agree, while the requirement family stays visible.
    {
        let server = Server::start(&project, &scratch);
        assert!(
            get(&server, "/?type=task")
                .text()
                .contains("No tasks in this project.")
        );
        assert_eq!(get(&server, "/doc/task-1").status, 404);
        assert_eq!(get(&server, "/doc/m-0").status, 404);
        let tasks: serde_json::Value =
            serde_json::from_str(&get(&server, "/api/tasks").text()).unwrap();
        let milestones: serde_json::Value =
            serde_json::from_str(&get(&server, "/api/milestones").text()).unwrap();
        assert_eq!(tasks.as_array().unwrap().len(), 0);
        assert_eq!(milestones.as_array().unwrap().len(), 0);
        let reqs = get(&server, "/?type=req").text();
        assert!(reqs.contains("Visible Requirement"), "requirements hidden");
        assert!(reqs.contains("REQ-001.001"), "requirement ID missing");
    }

    // Import from the isolated fixture project only.
    let import = run_ok(
        &project,
        &["import", "backlog", "--from", source.to_str().unwrap()],
    );
    assert!(import.contains("Excluded: docs=1 drafts=1 decisions=1 config=1 locks=1"));
    assert_eq!(hash_tree(&source), source_hash, "source store changed");

    // Every in-scope record is readable with its allocated Papyrus ID.
    for id in ["TASK-001", "TASK-002", "TASK-003", "TASK-004", "TASK-005"] {
        let view = run_ok(&project, &["task", "view", id]);
        assert!(!view.is_empty(), "task {} not readable", id);
    }
    for id in ["MS-001", "MS-002", "MS-003"] {
        let view = run_ok(&project, &["milestone", "view", id]);
        assert!(!view.is_empty(), "milestone {} not readable", id);
    }
    assert!(project.join("backlog/archive/tasks/TASK-004.md").exists());
    assert!(
        project
            .join("backlog/archive/milestones/MS-003.md")
            .exists()
    );

    // Excluded families were not imported as records and were not rewritten.
    assert!(project.join("backlog/docs/doc-1.md").exists());
    assert!(project.join("backlog/drafts/draft-1.md").exists());
    assert!(project.join("backlog/decisions/dec-1.md").exists());
    assert_eq!(
        fs::read_to_string(project.join("backlog/config.yml")).unwrap(),
        "project_name: Source\n"
    );
    assert_eq!(
        fs::read_to_string(project.join("backlog/.locks/store.lock")).unwrap(),
        "lock\n"
    );

    // Browser record views now resolve by Papyrus ID.
    {
        let server = Server::start(&project, &scratch);
        assert_eq!(get(&server, "/doc/TASK-001").status, 200);
        assert_eq!(get(&server, "/doc/MS-001").status, 200);
        let tasks: serde_json::Value =
            serde_json::from_str(&get(&server, "/api/tasks").text()).unwrap();
        assert!(
            tasks
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["id"] == "TASK-001")
        );
    }

    assert_eq!(
        hash_tree(&source),
        source_hash,
        "source store changed after CLI and browser checks"
    );
}

// ---------------------------------------------------------------------------
// T-98-05 — one Papyrus ID on every surface; Backlog IDs rejected
// ---------------------------------------------------------------------------

#[test]
fn t98_05_single_id_space_on_every_surface() {
    let scratch = Scratch::new("single-id");
    let source = build_fixture(&scratch);
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);
    run_ok(
        &project,
        &["import", "backlog", "--from", source.to_str().unwrap()],
    );

    // CLI text and JSON.
    let surfaces = [
        ("task list", run_ok(&project, &["task", "list"])),
        ("task view", run_ok(&project, &["task", "view", "TASK-001"])),
        ("milestone list", run_ok(&project, &["milestone", "list"])),
        ("search", run_ok(&project, &["search", "Foundation"])),
        (
            "task list json",
            run_ok(&project, &["task", "list", "--json"]),
        ),
        (
            "task view json",
            run_ok(&project, &["task", "view", "TASK-001", "--json"]),
        ),
        (
            "milestone list json",
            run_ok(&project, &["milestone", "list", "--json"]),
        ),
    ];
    for (surface, text) in &surfaces {
        assert_no_backlog_ids(surface, text);
        assert_no_storage_fields(surface, text);
    }

    let task_json: serde_json::Value =
        serde_json::from_str(&run_ok(&project, &["task", "view", "TASK-001", "--json"])).unwrap();
    assert_eq!(task_json["id"], "TASK-001");
    assert_eq!(task_json["metadata"]["id"], "TASK-001");

    // Browser HTML, JSON API, and rendered links.
    {
        let server = Server::start(&project, &scratch);
        for path in ["/?type=task", "/doc/TASK-001", "/doc/MS-001"] {
            let html = get(&server, path).text();
            assert_no_backlog_ids(path, &html);
            assert_no_storage_fields(path, &html);
        }
        let board = get(&server, "/?type=task").text();
        assert!(board.contains("href=\"/doc/TASK-001\""));

        for path in ["/api/tasks", "/api/milestones", "/api/doc/TASK-001"] {
            let body = get(&server, path).text();
            assert_no_backlog_ids(path, &body);
            assert_no_storage_fields(path, &body);
            let value: serde_json::Value = serde_json::from_str(&body).unwrap();
            let expected = if path == "/api/milestones" {
                "MS-001"
            } else {
                "TASK-001"
            };
            let items = match value {
                serde_json::Value::Array(items) => items,
                other => vec![other],
            };
            assert!(
                items.iter().any(|item| item["id"] == expected),
                "{} missing {}",
                path,
                expected
            );
        }
    }

    // A Backlog ID passed as input is an unknown record.
    assert_eq!(
        run(&project, &["task", "view", "task-1"]).status.code(),
        Some(3)
    );
    assert_eq!(
        run(&project, &["milestone", "view", "m-0"]).status.code(),
        Some(3)
    );
}

// ---------------------------------------------------------------------------
// T-98-06 — import fidelity, destinations, and excluded scope
// ---------------------------------------------------------------------------

#[test]
fn t98_06_import_fidelity() {
    let scratch = Scratch::new("fidelity");
    let source = build_fixture(&scratch);
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);

    let report = run_ok(
        &project,
        &[
            "import",
            "backlog",
            "--from",
            source.to_str().unwrap(),
            "--report",
            "import-report.md",
        ],
    );
    let saved = fs::read_to_string(project.join("import-report.md")).unwrap();
    assert!(report.contains("Mode: import"));
    assert!(saved.contains("Excluded: docs=1 drafts=1 decisions=1 config=1 locks=1"));
    assert!(saved.contains("Planned: tasks=5 milestones=3"));
    assert!(saved.contains("TASK-001 <- task-1 - Foundation Task Foundation Task"));
    assert!(saved.contains("TASK-004 <- task-4 - Archived Task Archived Task [archived]"));
    assert!(saved.contains("MS-001 <- m-0 - First Milestone First Milestone"));
    assert!(saved.contains("MS-003 <- m-2 - Archived Milestone Archived Milestone [archived]"));

    // Consecutive, unique allocation from 1 in source-number order; milestones
    // in the MS- space with no source number preserved.
    for id in ["TASK-001", "TASK-002", "TASK-003", "TASK-005"] {
        assert!(
            project.join(format!("backlog/tasks/{}.md", id)).exists(),
            "missing {}",
            id
        );
    }
    assert!(project.join("backlog/archive/tasks/TASK-004.md").exists());
    for id in ["MS-001", "MS-002"] {
        assert!(
            project
                .join(format!("backlog/milestones/{}.md", id))
                .exists(),
            "missing {}",
            id
        );
    }
    assert!(
        project
            .join("backlog/archive/milestones/MS-003.md")
            .exists()
    );

    // Field mapping.
    let task_1 = fs::read_to_string(project.join("backlog/tasks/TASK-001.md")).unwrap();
    for expected in [
        "title: Foundation Task",
        "status: ToDo",
        "  - alice",
        "created_date: '2026-01-01T00:00:00Z'",
        "updated_date: '2026-01-02T00:00:00Z'",
        "  - alpha",
        "  - fm-only-label",
        "milestone: MS-001",
        "  - TASK-002",
        "  - ref.md",
        "  - doc.md",
        "type: task",
        "ordinal: 7",
        "unknown_key: keepme",
    ] {
        assert!(task_1.contains(expected), "TASK-001 missing {:?}", expected);
    }
    let task_2 = fs::read_to_string(project.join("backlog/tasks/TASK-002.md")).unwrap();
    assert!(
        task_2.contains("status: In Progress"),
        "status not preserved"
    );
    assert!(
        task_2.contains("milestone: null"),
        "dangling milestone not dropped"
    );
    assert!(
        saved.contains("status 'In Progress' does not match a configured status"),
        "missing status warning"
    );
    assert!(
        saved.contains("milestone reference 'm-999' could not be resolved; dropped"),
        "missing milestone warning"
    );

    // Body mapping: ordinals dropped, checkbox states kept, markers removed,
    // unknown section and free text preserved verbatim.
    assert!(task_1.contains("## Description\nFoundation description text."));
    assert!(
        task_1.contains("## Acceptance Criteria\n- [ ] first criterion\n- [x] second criterion")
    );
    assert!(!task_1.contains("#1 "), "AC ordinal was not dropped");
    assert!(!task_1.contains("#2 "), "AC ordinal was not dropped");
    assert!(task_1.contains("## Definition of Done\n- [ ] done item"));
    assert!(task_1.contains("plan step text"));
    assert!(task_1.contains("## Notes\nimplementation note text"));
    assert!(task_1.contains("## Comments\n- **alice**: a comment"));
    assert!(task_1.contains("## Unknown Section"));
    assert!(task_1.contains("Unknown section content kept verbatim."));
    assert!(task_1.contains("Free text outside any marker block."));
    assert!(!task_1.contains("<!-- AC:BEGIN -->"));
    assert!(!task_1.contains("<!-- SECTION:DESCRIPTION:BEGIN -->"));

    // A milestone body keeps a single ## Description and no release value.
    let ms_1 = fs::read_to_string(project.join("backlog/milestones/MS-001.md")).unwrap();
    assert_eq!(ms_1.matches("## Description").count(), 1);
    assert!(ms_1.contains("First milestone body."));
    assert!(!ms_1.contains("<!--"));
    assert!(
        ms_1.contains("release: null"),
        "an imported milestone must not gain a release"
    );

    // Excluded families are counted, not imported.
    assert!(!project.join("backlog/tasks/TASK-006.md").exists());
    assert!(!project.join("backlog/milestones/MS-004.md").exists());
    assert!(!project.join("docs/DOC-001.md").exists());
    assert!(!project.join("drafts/DRAFT-001.md").exists());
}

// ---------------------------------------------------------------------------
// T-98-07 — documented upgrade path
// ---------------------------------------------------------------------------

#[test]
fn t98_07_documented_upgrade_path() {
    let scratch = Scratch::new("upgrade");
    let source = build_fixture(&scratch);
    let source_hash = hash_tree(&source);
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);
    copy_tree(&source.join("backlog"), &project.join("backlog"));

    // No automatic import: the copied store is invisible until the explicit
    // command runs.
    assert_eq!(run_ok(&project, &["task", "list"]), "");

    // Dry run writes nothing anywhere.
    let before_dry_run = hash_tree(&project);
    let plan = run_ok(
        &project,
        &[
            "import",
            "backlog",
            "--from",
            source.to_str().unwrap(),
            "--dry-run",
        ],
    );
    assert!(plan.contains("Mode: dry-run (no writes)"));
    assert_eq!(
        hash_tree(&project),
        before_dry_run,
        "dry run wrote to the project"
    );
    assert_eq!(
        hash_tree(&source),
        source_hash,
        "dry run wrote to the source"
    );

    // Real import keeps the plan/report.
    let out = run_ok(
        &project,
        &[
            "import",
            "backlog",
            "--from",
            source.to_str().unwrap(),
            "--report",
            "import-report.md",
        ],
    );
    assert!(out.contains("Planned: tasks=5 milestones=3"));
    let report = fs::read_to_string(project.join("import-report.md")).unwrap();
    assert!(report.contains("Mode: import"));
    assert!(report.contains("TASK-001 <- task-1"));
    assert_eq!(
        hash_tree(&source),
        source_hash,
        "import wrote to the source"
    );
    assert!(project.join("backlog/tasks/TASK-001.md").exists());
    assert!(project.join("backlog/milestones/MS-001.md").exists());

    // Reallocation: source `task-N` becomes `TASK-0NN` and `m-0` becomes
    // `MS-001`; the old Backlog IDs never resolve.
    let view = run_ok(&project, &["task", "view", "TASK-001"]);
    assert!(view.contains("Foundation Task"));
    assert_eq!(
        run(&project, &["task", "view", "task-1"]).status.code(),
        Some(3)
    );
    assert_eq!(
        run(&project, &["milestone", "view", "m-0"]).status.code(),
        Some(3)
    );

    // Normal CLI and browser use after the import.
    assert!(run_ok(&project, &["task", "list"]).contains("TASK-001"));
    {
        let server = Server::start(&project, &scratch);
        assert_eq!(get(&server, "/doc/TASK-001").status, 200);
        assert_eq!(get(&server, "/").status, 200);

        // The per-milestone backlog board round-trips a status change to the
        // native record (REQ-06.013).
        let record_path = project.join("backlog/tasks/TASK-001.md");
        let board = get(&server, "/?type=task").text();
        let version = extract_card_version(&board, "TASK-001");
        let body = form_body(&[
            ("version", &version),
            ("status_change", "1"),
            ("status", "Initiated"),
        ]);
        let response = post_form(&server, "/doc/TASK-001", &body);
        assert_eq!(response.status, 303);
        assert!(
            fs::read_to_string(&record_path)
                .unwrap()
                .contains("status: Initiated"),
            "board status change was not persisted"
        );
        let reloaded = get(&server, "/?type=task").text();
        assert!(
            column_containing(&reloaded, "Initiated").contains("TASK-001"),
            "card did not move to the Initiated column"
        );
    }

    // A second run is refused without --allow-existing.
    let refused = run(
        &project,
        &["import", "backlog", "--from", source.to_str().unwrap()],
    );
    assert_eq!(refused.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("use --allow-existing"),
        "unexpected refusal: {stderr}"
    );
    assert!(!project.join("backlog/tasks/TASK-006.md").exists());

    // With --allow-existing the import duplicates with new IDs; existing
    // records are not overwritten, merged, or renumbered.
    let old_task_1 = fs::read_to_string(project.join("backlog/tasks/TASK-001.md")).unwrap();
    run_ok(
        &project,
        &[
            "import",
            "backlog",
            "--from",
            source.to_str().unwrap(),
            "--allow-existing",
        ],
    );
    for id in ["TASK-006", "TASK-007", "TASK-008", "TASK-010"] {
        assert!(
            project.join(format!("backlog/tasks/{}.md", id)).exists(),
            "duplicate import missing {}",
            id
        );
    }
    for id in ["MS-004", "MS-005"] {
        assert!(
            project
                .join(format!("backlog/milestones/{}.md", id))
                .exists(),
            "duplicate import missing {}",
            id
        );
    }
    assert_eq!(
        fs::read_to_string(project.join("backlog/tasks/TASK-001.md")).unwrap(),
        old_task_1,
        "duplicate import overwrote an existing record"
    );
}

// ---------------------------------------------------------------------------
// T-98-08 — relocation atomicity, conflict, containment, browser path
// ---------------------------------------------------------------------------

#[test]
fn t98_08_relocation_atomicity_and_browser_path() {
    let scratch = Scratch::new("relocation");

    // Success through the CLI, including unmanaged files and subfolders.
    let cli = scratch.join("cli");
    fs::create_dir_all(&cli).unwrap();
    init_project(&cli);
    write(&cli.join("backlog/extra.txt"), "unmanaged");
    write(&cli.join("backlog/sub/nested.txt"), "nested");
    run_ok(
        &cli,
        &["config", "set", "backlog_directory", "native/store"],
    );
    assert!(cli.join("native/store/extra.txt").exists());
    assert!(cli.join("native/store/sub/nested.txt").exists());
    assert!(!cli.join("backlog").exists(), "empty source folder remains");
    let config = fs::read_to_string(cli.join("papyrus.config.yml")).unwrap();
    assert!(config.contains("backlog_directory: native/store"));

    // The browser configuration view reaches the same path.
    let browser = scratch.join("browser");
    fs::create_dir_all(&browser).unwrap();
    init_project(&browser);
    write(&browser.join("backlog/extra.txt"), "unmanaged");
    {
        let server = Server::start(&browser, &scratch);
        let body = form_body(&[("key", "backlog_directory"), ("value", "moved/here")]);
        let response = post_form(&server, "/config", &body);
        assert_eq!(response.status, 303);
        assert_eq!(response.header("Location"), Some("/config"));
    }
    assert!(browser.join("moved/here/extra.txt").exists());
    let config = fs::read_to_string(browser.join("papyrus.config.yml")).unwrap();
    assert!(config.contains("backlog_directory: moved/here"));

    // Destination conflict: exit 4, move nothing, configuration unchanged.
    let conflict = scratch.join("conflict");
    fs::create_dir_all(&conflict).unwrap();
    init_project(&conflict);
    write(&conflict.join("backlog/extra.txt"), "source");
    write(&conflict.join("destination/extra.txt"), "destination");
    let config_path = conflict.join("papyrus.config.yml");
    let before = fs::read(&config_path).unwrap();
    let out = run(
        &conflict,
        &["config", "set", "backlog_directory", "destination"],
    );
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(
        fs::read_to_string(conflict.join("backlog/extra.txt")).unwrap(),
        "source"
    );
    assert_eq!(
        fs::read_to_string(conflict.join("destination/extra.txt")).unwrap(),
        "destination"
    );
    assert_eq!(fs::read(&config_path).unwrap(), before);

    // Unwritable destination: exit 4, move nothing, configuration unchanged.
    let unwritable = scratch.join("unwritable");
    fs::create_dir_all(&unwritable).unwrap();
    init_project(&unwritable);
    write(&unwritable.join("backlog/extra.txt"), "source");
    let locked = unwritable.join("locked");
    fs::create_dir_all(&locked).unwrap();
    let config_path = unwritable.join("papyrus.config.yml");
    let before = fs::read(&config_path).unwrap();
    let original = fs::metadata(&locked).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_mode(0o555);
    fs::set_permissions(&locked, readonly).unwrap();
    let out = run(
        &unwritable,
        &["config", "set", "backlog_directory", "locked"],
    );
    fs::set_permissions(&locked, original).unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(
        unwritable.join("backlog/extra.txt").exists(),
        "conflict moved the source"
    );
    assert_eq!(
        fs::read_to_string(unwritable.join("backlog/extra.txt")).unwrap(),
        "source"
    );
    assert_eq!(fs::read(&config_path).unwrap(), before);

    // Out-of-project destination: exit 4, nothing created outside, mode
    // unchanged.
    let escape = scratch.join("escape");
    fs::create_dir_all(&escape).unwrap();
    init_project(&escape);
    write(&escape.join("backlog/extra.txt"), "source");
    let config_path = escape.join("papyrus.config.yml");
    let before = fs::read(&config_path).unwrap();
    let outside = scratch.join("escaped-target");
    let out = run(
        &escape,
        &[
            "config",
            "set",
            "backlog_directory",
            outside.to_str().unwrap(),
        ],
    );
    assert_eq!(out.status.code(), Some(4));
    assert!(!outside.exists(), "out-of-project destination was created");
    assert!(escape.join("backlog/extra.txt").exists());
    assert_eq!(fs::read(&config_path).unwrap(), before);
}

// ---------------------------------------------------------------------------
// T-98-09 — search over whole persisted text, including front matter
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn t98_09_search_indexes_front_matter_without_writes() {
    let scratch = Scratch::new("search");
    let source = build_fixture(&scratch);
    let project = scratch.join("project");
    fs::create_dir_all(&project).unwrap();
    init_project(&project);
    run_ok(
        &project,
        &["import", "backlog", "--from", source.to_str().unwrap()],
    );

    // Deprecated records disappear from search results and restored records
    // reappear. Create and settle the record before the write-free checkpoint.
    let spec = run_ok(
        &project,
        &[
            "tech-spec",
            "create",
            "Visible Spec",
            "--description",
            "Searchable specification body.",
        ],
    );
    assert!(
        run_ok(&project, &["search", "Visible Spec"]).contains(&spec),
        "active specification not searchable"
    );
    run_ok(&project, &["tech-spec", "archive", &spec]);
    assert_eq!(run_ok(&project, &["search", "Visible Spec"]), "");
    run_ok(&project, &["tech-spec", "restore", &spec]);
    assert!(
        run_ok(&project, &["search", "Visible Spec"]).contains(&spec),
        "restored specification not searchable"
    );

    // `fm-only-label` occurs only in TASK-001's front-matter labels.
    let before = hash_tree(&project);
    let hit = run_ok(&project, &["search", "fm-only-label"]);
    assert!(
        hit.contains("TASK-001"),
        "front-matter-only query missed: {hit}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&run_ok(&project, &["search", "fm-only-label", "--json"])).unwrap();
    assert!(
        json.as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "TASK-001" && item["type"] == "task")
    );

    // Quoted phrase, type filter, limit, empty result, and archived/archived
    // visibility.
    assert!(
        run_ok(&project, &["search", "Foundation Task", "--type", "task"]).contains("TASK-001")
    );
    assert_eq!(
        run_ok(
            &project,
            &["search", "Foundation Task", "--type", "milestone"]
        ),
        ""
    );
    assert!(run_ok(&project, &["search", "Task", "--limit", "1"]).contains("TASK-001"));
    assert_eq!(run_ok(&project, &["search", "zzz-no-such-value"]), "");
    assert_eq!(run_ok(&project, &["search", "Archived Task"]), "");

    // The browser record view and JSON API expose the matching metadata; the
    // browser has no search endpoint (search is the documented CLI surface).
    {
        let server = Server::start(&project, &scratch);
        let record = get(&server, "/doc/TASK-001").text();
        assert!(
            record.contains("fm-only-label"),
            "browser record view missing label"
        );
        let api = get(&server, "/api/doc/TASK-001").text();
        assert!(api.contains("fm-only-label"));
    }

    assert_eq!(hash_tree(&project), before, "search wrote to the project");
}
