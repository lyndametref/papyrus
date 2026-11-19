//! TC-040 .. TC-054: multiline standard-input bodies for create and edit.
//!
//! The contract lives in TASK-191; these tests exercise the CLI end to end
//! with piped standard input.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const STDIN_HELP: &str = "Use \"-\" to read multiline text from standard input.";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("multiline-stdin-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn run(project: &Path, args: &[&str]) -> Output {
    bin().arg("--dir").arg(project).args(args).output().unwrap()
}

fn run_ok(project: &Path, args: &[&str]) -> Output {
    let out = run(project, args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).trim().to_string()
}

fn run_stdin(project: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = bin()
        .arg("--dir")
        .arg(project)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input)
        .expect("stdin accepts the fixture bytes");
    child.wait_with_output().unwrap()
}

fn run_stdin_ok(project: &Path, args: &[&str], input: &[u8]) -> Output {
    let out = run_stdin(project, args, input);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn init(name: &str) -> PathBuf {
    let dir = root(name);
    run_ok(&dir, &["init", "Proj", "--no-git"]);
    add_request(&dir);
    dir
}

fn add_request(project: &Path) {
    let path = project.join("papyrus.config.yml");
    let cfg = fs::read_to_string(&path).unwrap();
    let block = "  request:\n    prefix: RQT\n    directory: requests/\n    enabled: true\n";
    let updated = cfg.replacen("document_types:\n", &format!("document_types:\n{block}"), 1);
    assert_ne!(updated, cfg);
    fs::write(&path, updated).unwrap();
}

fn create(project: &Path, args: &[&str]) -> String {
    stdout(&run_ok(project, args))
}

fn view(project: &Path, slug: &str, id: &str) -> String {
    stdout(&run_ok(project, &[slug, "view", id]))
}

fn worker(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
}

#[test]
fn tc_040_document_create_reads_multiline_description_from_stdin() {
    let dir = init("tc040");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["request", "create", "Multiline request", "-d", "-"],
        b"Line one\nLine two\nLine three\n",
    ));
    assert_eq!(id, "RQT-001");
    let content = view(&dir, "request", &id);
    let body = worker(dir.join("docs/requests/RQT-001.md"));
    assert!(
        content.contains("Line one\nLine two\nLine three"),
        "{content}"
    );
    assert!(body.contains("Line one\nLine two\nLine three"), "{body}");
}

#[test]
fn tc_041_document_edit_replaces_description_and_preserves_metadata() {
    let dir = init("tc041");
    let id = create(
        &dir,
        &[
            "request",
            "create",
            "Multiline request",
            "-d",
            "Single line summary",
        ],
    );
    run_stdin_ok(
        &dir,
        &["request", "edit", &id, "-d", "-"],
        b"First paragraph\n\nSecond paragraph\n",
    );
    let content = view(&dir, "request", &id);
    assert!(
        content.contains("First paragraph\n\nSecond paragraph"),
        "{content}"
    );
    assert!(
        content.contains("title: \"Multiline request\""),
        "{content}"
    );
    assert!(!content.contains("Single line summary"), "{content}");
}

#[test]
fn tc_042_decision_create_reads_multiline_decision_from_stdin() {
    let dir = init("tc042");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["decision", "create", "Adopt multiline input", "-d", "-"],
        b"Decision one\nDecision two\nDecision three\n",
    ));
    assert_eq!(id, "ADR-001");
    let content = view(&dir, "decision", &id);
    assert!(
        content.contains("## Decision\nDecision one\nDecision two\nDecision three"),
        "{content}"
    );
}

#[test]
fn tc_043_decision_edit_reads_multiline_context_from_stdin() {
    let dir = init("tc043");
    let id = create(
        &dir,
        &[
            "decision",
            "create",
            "Context edit",
            "-d",
            "Decision text",
            "--context",
            "Initial context",
            "--consequences",
            "Known consequences",
        ],
    );
    run_stdin_ok(
        &dir,
        &["decision", "edit", &id, "--context", "-"],
        b"Context one\nContext two\n",
    );
    let content = view(&dir, "decision", &id);
    assert!(
        content.contains("## Context\nContext one\nContext two"),
        "{content}"
    );
    assert!(content.contains("## Decision\nDecision text"), "{content}");
    assert!(
        content.contains("## Consequences\nKnown consequences"),
        "{content}"
    );
}

#[test]
fn tc_044_decision_edit_reads_multiline_consequences_from_stdin() {
    let dir = init("tc044");
    let id = create(
        &dir,
        &[
            "decision",
            "create",
            "Consequences edit",
            "-d",
            "Decision text",
            "--context",
            "Known context",
            "--consequences",
            "No known consequences",
        ],
    );
    run_stdin_ok(
        &dir,
        &["decision", "edit", &id, "--consequences", "-"],
        b"Consequence one\nConsequence two\n",
    );
    let content = view(&dir, "decision", &id);
    assert!(
        content.contains("## Consequences\nConsequence one\nConsequence two"),
        "{content}"
    );
    assert!(content.contains("## Decision\nDecision text"), "{content}");
    assert!(content.contains("## Context\nKnown context"), "{content}");
}

#[test]
fn tc_045_task_create_reads_multiline_description_from_stdin() {
    let dir = init("tc045");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["task", "create", "Multiline task", "-d", "-"],
        b"Task line one\nTask line two\nTask line three\n",
    ));
    assert_eq!(id, "TASK-001");
    let content = view(&dir, "task", &id);
    assert!(
        content.contains("Task line one\nTask line two\nTask line three"),
        "{content}"
    );
}

#[test]
fn tc_046_task_edit_reads_multiline_description_and_keeps_status() {
    let dir = init("tc046");
    let id = create(
        &dir,
        &["task", "create", "Task", "-d", "Original description"],
    );
    run_stdin_ok(
        &dir,
        &["task", "edit", &id, "-d", "-"],
        b"Edited line one\nEdited line two\n",
    );
    let content = view(&dir, "task", &id);
    assert!(
        content.contains("Edited line one\nEdited line two"),
        "{content}"
    );
    assert!(content.contains("status: ToDo"), "{content}");
    assert!(!content.contains("Original description"), "{content}");
}

#[test]
fn tc_047_milestone_create_reads_multiline_description_from_stdin() {
    let dir = init("tc047");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["milestone", "create", "Multiline milestone", "-d", "-"],
        b"Milestone line one\nMilestone line two\nMilestone line three\n",
    ));
    assert_eq!(id, "MS-001");
    let content = view(&dir, "milestone", &id);
    assert!(
        content.contains("Milestone line one\nMilestone line two\nMilestone line three"),
        "{content}"
    );
}

#[test]
fn tc_048_milestone_edit_reads_multiline_description_and_keeps_status() {
    let dir = init("tc048");
    let id = create(
        &dir,
        &[
            "milestone",
            "create",
            "Milestone",
            "-d",
            "Original milestone description",
        ],
    );
    run_stdin_ok(
        &dir,
        &["milestone", "edit", &id, "-d", "-"],
        b"Edited milestone one\nEdited milestone two\n",
    );
    let content = view(&dir, "milestone", &id);
    assert!(
        content.contains("Edited milestone one\nEdited milestone two"),
        "{content}"
    );
    assert!(content.contains("status: open"), "{content}");
    assert!(
        !content.contains("Original milestone description"),
        "{content}"
    );
}

#[test]
fn tc_049_titles_with_embedded_newlines_are_rejected() {
    let dir = init("tc049");
    let request = run(&dir, &["request", "create", "Bad\nTitle"]);
    assert_eq!(request.status.code(), Some(4));
    let task = run(&dir, &["task", "create", "Bad\nTitle"]);
    assert_eq!(task.status.code(), Some(4));
    assert_eq!(stdout(&run_ok(&dir, &["request", "list"])), "");
    assert_eq!(stdout(&run_ok(&dir, &["task", "list"])), "");
}

#[test]
fn tc_050_identifiers_with_embedded_newlines_are_rejected() {
    let dir = init("tc050");
    let task_id = create(&dir, &["task", "create", "Task"]);
    let request_id = create(&dir, &["request", "create", "Request"]);
    let task_before = worker(dir.join(format!("backlog/tasks/{task_id}.md")));
    let request_before = worker(dir.join(format!("docs/requests/{request_id}.md")));

    let edit = run(&dir, &["task", "edit", &format!("{task_id}\nX"), "-d", "x"]);
    assert_eq!(edit.status.code(), Some(4));
    let view = run(&dir, &["request", "view", &format!("{request_id}\nX")]);
    assert_eq!(view.status.code(), Some(4));

    assert_eq!(
        worker(dir.join(format!("backlog/tasks/{task_id}.md"))),
        task_before
    );
    assert_eq!(
        worker(dir.join(format!("docs/requests/{request_id}.md"))),
        request_before
    );
}

#[test]
fn tc_051_single_line_metadata_rejects_embedded_newlines() {
    let dir = init("tc051");
    let id = create(&dir, &["request", "create", "Labeled"]);
    let out = run(&dir, &["request", "edit", &id, "--label", "bad\nlabel"]);
    assert_eq!(out.status.code(), Some(4));
    let content = view(&dir, "request", &id);
    assert!(content.contains("labels: []"), "{content}");
}

#[test]
fn tc_052_literal_value_does_not_read_standard_input() {
    let dir = init("tc052");
    // Invalid UTF-8 on stdin proves the literal path never reads it.
    let out = run_stdin(
        &dir,
        &[
            "request",
            "create",
            "Single line request",
            "-d",
            "single line description",
        ],
        b"\xff\xfe",
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let id = stdout(&out);
    let content = view(&dir, "request", &id);
    assert!(content.contains("single line description"), "{content}");
}

#[cfg(unix)]
#[test]
fn tc_053_editor_path_still_writes_multiline_description() {
    use std::os::unix::fs::PermissionsExt;

    let dir = init("tc053");
    let id = create(
        &dir,
        &["request", "create", "Editor request", "-d", "Single line"],
    );
    let script = dir.join("editor.sh");
    fs::write(
        &script,
        "#!/bin/sh\nsed -i '/^## Description$/,$c\\## Description\\n\\nEditor line one\\nEditor line two\\nEditor line three' \"$1\"\n",
    )
    .unwrap();
    let mut perms = fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script, perms).unwrap();

    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .env("EDITOR", script.to_str().unwrap())
        .args(["request", "edit", &id, "--editor"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    let content = view(&dir, "request", &id);
    assert!(
        content.contains("Editor line one\nEditor line two\nEditor line three"),
        "{content}"
    );
}

#[test]
fn tc_054_help_documents_the_stdin_path() {
    let dir = init("tc054");
    let cases: &[(&[&str], &[&str])] = &[
        (&["request", "create", "--help"], &["--description"]),
        (&["request", "edit", "--help"], &["--description"]),
        (&["req", "create", "--help"], &["--description"]),
        (&["req", "edit", "--help"], &["--description"]),
        (&["task", "create", "--help"], &["--description"]),
        (&["task", "edit", "--help"], &["--description"]),
        (&["milestone", "create", "--help"], &["--description"]),
        (&["milestone", "edit", "--help"], &["--description"]),
        (
            &["decision", "create", "--help"],
            &["--decision", "--context", "--consequences"],
        ),
        (
            &["decision", "edit", "--help"],
            &["--decision", "--context", "--consequences"],
        ),
    ];
    for (args, fields) in cases {
        let out = run_ok(&dir, args);
        let help = String::from_utf8_lossy(&out.stdout);
        assert!(help.contains(STDIN_HELP), "{args:?}: {help}");
        for field in *fields {
            assert!(help.contains(field), "{args:?} missing {field}: {help}");
        }
    }
}

#[test]
fn empty_and_whitespace_only_stdin_exit_4_without_allocating_an_id() {
    let dir = init("empty");
    for input in [b"".as_slice(), b"  \n\t\n ".as_slice()] {
        let out = run_stdin(&dir, &["request", "create", "Empty", "-d", "-"], input);
        assert_eq!(out.status.code(), Some(4));
    }
    let id = stdout(&run_stdin_ok(
        &dir,
        &["request", "create", "After failures", "-d", "-"],
        b"line\n",
    ));
    assert_eq!(id, "RQT-001", "failed input must not allocate an ID");
}

#[test]
fn invalid_utf8_stdin_exits_4() {
    let dir = init("utf8");
    let out = run_stdin(
        &dir,
        &["task", "create", "Bad bytes", "-d", "-"],
        b"ok\n\xff\n",
    );
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(stdout(&run_ok(&dir, &["task", "list"])), "");
}

#[test]
fn multiple_sentinels_exit_4_before_reading_or_writing() {
    let dir = init("multi");
    let out = run_stdin(
        &dir,
        &[
            "decision",
            "create",
            "Conflict",
            "-d",
            "-",
            "--context",
            "-",
        ],
        b"never read\n",
    );
    assert_eq!(out.status.code(), Some(4));
    let message = stderr(&out);
    assert!(message.contains("--decision"), "{message}");
    assert!(message.contains("--context"), "{message}");
    assert_eq!(stdout(&run_ok(&dir, &["decision", "list"])), "");
}

#[test]
fn crlf_input_is_normalized_and_internal_blank_lines_are_preserved() {
    let dir = init("crlf");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["task", "create", "CRLF task", "-d", "-"],
        b"one\r\ntwo\r\n\r\nthree\r\n",
    ));
    let raw = worker(dir.join(format!("backlog/tasks/{id}.md")));
    assert!(!raw.contains('\r'), "{raw}");
    assert!(raw.contains("one\ntwo\n\nthree"), "{raw}");
}

#[test]
fn typed_document_types_accept_stdin_description_on_create_and_edit() {
    let dir = init("typed");
    let id = stdout(&run_stdin_ok(
        &dir,
        &["tech-spec", "create", "Typed spec", "--description", "-"],
        b"Spec line one\nSpec line two\n",
    ));
    let content = view(&dir, "tech-spec", &id);
    assert!(
        content.contains("Spec line one\nSpec line two"),
        "{content}"
    );

    run_stdin_ok(
        &dir,
        &["tech-spec", "edit", &id, "--description", "-"],
        b"Edited spec one\n\nEdited spec two\n",
    );
    let content = view(&dir, "tech-spec", &id);
    assert!(
        content.contains("Edited spec one\n\nEdited spec two"),
        "{content}"
    );
}
