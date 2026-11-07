//! Integration tests for `papyrus search` (TASK-5).
//!
//! Each test maps to a check in
//! `docs/specifications/search-requirements.md` (T-01 ... T-17).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-search-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn init(d: &Path) {
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(out.status.success(), "init failed: {}", stderr(&out));
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn assert_ok(out: Output) -> Output {
    assert!(
        out.status.success(),
        "command failed: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    out
}

/// Run a papyrus command with `--dir` for the given project.
fn run(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

/// Run `papyrus search` with the given arguments.
fn search(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["search"];
    full.extend_from_slice(args);
    run(project, &full)
}

fn create_group(project: &Path, kind: &str, title: &str) -> String {
    let out = assert_ok(run(project, &[kind, "create-group", "--title", title]));
    stdout(&out).trim().to_string()
}

fn create_req(project: &Path, kind: &str, title: &str, group: &str, extra: &[&str]) -> String {
    let mut args: Vec<&str> = vec![kind, "create", title, "--group", group];
    args.extend_from_slice(extra);
    let out = assert_ok(run(project, &args));
    stdout(&out).trim().to_string()
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

/// Standard fixture: one business group + member and one NFR group + member.
fn setup_standard(d: &Path) {
    init(d);
    let group = create_group(d, "req", "Payments Group");
    assert_eq!(group, "REQ-001");
    let req = create_req(
        d,
        "req",
        "Capture payment",
        "REQ-001",
        &["--story", "as a cashier", "-d", "The quick brown fox jumps"],
    );
    assert_eq!(req, "REQ-001.001");

    let nfr_group = create_group(d, "nfreq", "Performance Group");
    assert_eq!(nfr_group, "NFR-001");
    let nfr = create_req(
        d,
        "nfreq",
        "Latency budget",
        "NFR-001",
        &["-d", "response time"],
    );
    assert_eq!(nfr, "NFR-001.001");
}

fn snapshot(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                let rel = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                out.push(rel);
                if path.is_dir() {
                    walk(&path, base, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// T-01: command shape and documented type vocabulary
// ---------------------------------------------------------------------------

#[test]
fn t01_help_shows_documented_shape() {
    let out = bin().args(["search", "--help"]).output().unwrap();
    assert_ok(out.clone());
    let s = stdout(&out);
    for needle in [
        "search",
        "<query>",
        "--type",
        "--limit",
        "task",
        "milestone",
        "release",
        "decision",
        "doc",
    ] {
        assert!(s.contains(needle), "help missing {needle:?}: {s}");
    }
}

#[test]
fn t01_each_valid_type_is_accepted() {
    let d = tmpdir("t01-types");
    init(&d);
    for ty in ["task", "milestone", "release", "decision", "doc"] {
        let out = search(&d, &["anything", "--type", ty]);
        assert_eq!(out.status.code(), Some(0), "type={ty}");
        assert_eq!(stdout(&out), "", "type={ty}");
    }
    // Repeating a type is allowed and has no extra effect.
    let out = search(&d, &["anything", "--type", "doc", "--type", "doc"]);
    assert_eq!(out.status.code(), Some(0));
}

// ---------------------------------------------------------------------------
// T-02: default search across business and non-functional requirements
// ---------------------------------------------------------------------------

#[test]
fn t02_default_search_covers_req_nfr_and_groups() {
    let d = tmpdir("t02");
    setup_standard(&d);

    let out = search(&d, &["capture"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    let out = search(&d, &["latency"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "NFR-001.001 doc Latency budget");

    // Group documents are records in their own right.
    let out = search(&d, &["Payments"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out).trim(),
        "REQ-001 doc Payments Group\nREQ-001.001 doc Capture payment"
    );
}

// ---------------------------------------------------------------------------
// T-03: case-insensitive matching
// ---------------------------------------------------------------------------

#[test]
fn t03_matching_is_case_insensitive() {
    let d = tmpdir("t03");
    setup_standard(&d);
    let lower = search(&d, &["capture payment"]);
    let upper = search(&d, &["CAPTURE PAYMENT"]);
    let mixed = search(&d, &["CaPtUrE pAyMeNt"]);
    assert_eq!(stdout(&lower), stdout(&upper));
    assert_eq!(stdout(&lower), stdout(&mixed));
    assert!(stdout(&lower).contains("REQ-001.001"));
}

// ---------------------------------------------------------------------------
// T-04: quoted phrase is one contiguous substring; metacharacters are literal
// ---------------------------------------------------------------------------

#[test]
fn t04_quoted_phrase_is_contiguous() {
    let d = tmpdir("t04");
    setup_standard(&d);

    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    // Words that are not contiguous do not match the phrase.
    let out = search(&d, &["quick fox"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    // Internal whitespace is preserved exactly.
    let out = search(&d, &["quick  brown"]);
    assert_eq!(stdout(&out), "");
}

#[test]
fn t04_regex_metacharacters_are_literal() {
    let d = tmpdir("t04-meta");
    setup_standard(&d);

    let out = search(&d, &["REQ-001.001"]);
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    for query in [".*", "REQ-001X001", "["] {
        let out = search(&d, &[query]);
        assert_eq!(out.status.code(), Some(0), "query={query}");
        assert_eq!(stdout(&out), "", "query={query}");
    }
}

// ---------------------------------------------------------------------------
// T-05: ID, title, user story, description, and editor-added body are searchable
// ---------------------------------------------------------------------------

#[test]
fn t05_all_fields_are_searchable() {
    let d = tmpdir("t05");
    setup_standard(&d);
    let member = d.join("docs/requirements/business/REQ-001.001.md");

    for query in ["REQ-001.001", "Capture payment", "cashier", "brown fox"] {
        let out = search(&d, &[query]);
        assert_eq!(out.status.code(), Some(0), "query={query}");
        assert!(
            stdout(&out).contains("REQ-001.001"),
            "query={query} output={}",
            stdout(&out)
        );
    }

    // A section added after creation (as an editor would) is searchable.
    let mut content = read_file(&member);
    content.push_str("\n## Notes\neditor only secret\n");
    fs::write(&member, content).unwrap();

    let out = search(&d, &["editor only"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");
}

#[test]
fn t05_non_utf8_record_is_skipped() {
    let d = tmpdir("t05-nonutf8");
    setup_standard(&d);
    fs::write(
        d.join("docs/requirements/business/BROKEN.md"),
        [0xffu8, 0xfe, 0x00],
    )
    .unwrap();
    // Search still succeeds and finds valid records.
    let out = search(&d, &["capture"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");
    let out = search(&d, &["broken"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

// ---------------------------------------------------------------------------
// T-06: type filters, including unimplemented types
// ---------------------------------------------------------------------------

#[test]
fn t06_type_filters() {
    let d = tmpdir("t06");
    setup_standard(&d);

    let out = search(&d, &["capture", "--type", "doc"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    for ty in ["task", "milestone", "release", "decision"] {
        let out = search(&d, &["capture", "--type", ty]);
        assert_eq!(out.status.code(), Some(0), "type={ty}");
        assert_eq!(stdout(&out), "", "type={ty}");
    }

    // Combined with doc, unimplemented types contribute nothing.
    let out = search(&d, &["capture", "--type", "task", "--type", "doc"]);
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    // Only unimplemented types -> empty success.
    let out = search(&d, &["capture", "--type", "task", "--type", "task"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

// ---------------------------------------------------------------------------
// T-07: limit applied after ordering
// ---------------------------------------------------------------------------

#[test]
fn t07_limit_caps_after_ordering() {
    let d = tmpdir("t07");
    init(&d);
    for title in ["Limit One", "Limit Two", "Limit Three"] {
        create_group(&d, "req", title);
    }

    let all = search(&d, &["limit"]);
    assert_eq!(
        stdout(&all).trim(),
        "REQ-001 doc Limit One\nREQ-002 doc Limit Two\nREQ-003 doc Limit Three"
    );

    let out = search(&d, &["limit", "--limit", "2"]);
    assert_eq!(
        stdout(&out).trim(),
        "REQ-001 doc Limit One\nREQ-002 doc Limit Two"
    );

    let out = search(&d, &["limit", "--limit", "3"]);
    assert_eq!(stdout(&out), stdout(&all));

    let out = search(&d, &["limit", "--limit", "5"]);
    assert_eq!(stdout(&out), stdout(&all));
}

// ---------------------------------------------------------------------------
// T-08: results ordered by record ID ascending
// ---------------------------------------------------------------------------

#[test]
fn t08_results_ordered_by_id() {
    let d = tmpdir("t08");
    setup_standard(&d);
    let out = search(&d, &["Group"]);
    let ids: Vec<String> = stdout(&out)
        .lines()
        .map(|line| line.split(' ').next().unwrap_or("").to_string())
        .collect();
    assert_eq!(
        ids,
        vec!["NFR-001", "NFR-001.001", "REQ-001", "REQ-001.001"]
    );
}

// ---------------------------------------------------------------------------
// T-09: deprecated records hidden, restored records visible
// ---------------------------------------------------------------------------

#[test]
fn t09_deprecate_and_restore_visibility() {
    let d = tmpdir("t09");
    setup_standard(&d);

    assert_ok(run(&d, &["req", "deprecate", "REQ-001.001"]));
    let out = search(&d, &["capture"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    assert_ok(run(&d, &["req", "restore", "REQ-001.001"]));
    let out = search(&d, &["capture"]);
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");

    // Deprecating a group does not cascade to its active members.
    assert_ok(run(&d, &["req", "deprecate", "REQ-001"]));
    let out = search(&d, &["capture"]);
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");
    let out = search(&d, &["Payments Group"]);
    assert_eq!(stdout(&out).trim(), "REQ-001.001 doc Capture payment");
}

// ---------------------------------------------------------------------------
// T-10: disabled document types are excluded
// ---------------------------------------------------------------------------

#[test]
fn t10_disabled_document_types_are_excluded() {
    let d = tmpdir("t10");
    setup_standard(&d);

    assert_ok(run(
        &d,
        &["config", "set", "document_types.req.enabled", "false"],
    ));
    let out = search(&d, &["capture"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    let out = search(&d, &["latency"]);
    assert_eq!(stdout(&out).trim(), "NFR-001.001 doc Latency budget");

    assert_ok(run(
        &d,
        &["config", "set", "document_types.nfr.enabled", "false"],
    ));
    let out = search(&d, &["latency"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    let out = search(&d, &["Group"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

// ---------------------------------------------------------------------------
// T-11: configured doc_directory, directories, and prefixes are honored
// ---------------------------------------------------------------------------

#[test]
fn t11_custom_document_directory_and_prefix() {
    let d = tmpdir("t11");
    init(&d);
    write_file(
        &d.join("papyrus.config.yml"),
        "doc_directory: knowledge/\ndocument_types:\n  req:\n    prefix: BRQ\n    directory: biz/\n    enabled: true\n  nfr:\n    prefix: BNF\n    directory: nonfunc/\n    enabled: true\n",
    );
    fs::create_dir_all(d.join("knowledge/biz/deprecated")).unwrap();
    fs::create_dir_all(d.join("knowledge/nonfunc/deprecated")).unwrap();

    let group = create_group(&d, "req", "Custom Group");
    assert_eq!(group, "BRQ-001");

    let out = search(&d, &["custom"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "BRQ-001 doc Custom Group");

    let out = search(&d, &["custom", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let path = value[0]["path"].as_str().unwrap();
    assert!(path.ends_with("knowledge/biz/BRQ-001.md"), "path={path}");
    let canonical = fs::canonicalize(&d).unwrap();
    assert!(Path::new(path).starts_with(&canonical), "path={path}");
}

// ---------------------------------------------------------------------------
// T-12: exact text output
// ---------------------------------------------------------------------------

#[test]
fn t12_exact_text_output() {
    let d = tmpdir("t12");
    setup_standard(&d);

    let out = search(&d, &["capture"]);
    assert_eq!(stdout(&out), "REQ-001.001 doc Capture payment\n");

    let out = search(&d, &["no-such-text-anywhere"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

// ---------------------------------------------------------------------------
// T-13: JSON output and error shape
// ---------------------------------------------------------------------------

#[test]
fn t13_json_output() {
    let d = tmpdir("t13");
    setup_standard(&d);

    let out = search(&d, &["group", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let array = value.as_array().unwrap();
    let ids: Vec<&str> = array
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec!["NFR-001", "NFR-001.001", "REQ-001", "REQ-001.001"]
    );
    for item in array {
        assert_eq!(item["type"], "doc");
        assert!(item["title"].is_string());
        let path = item["path"].as_str().unwrap();
        assert!(Path::new(path).is_absolute(), "path={path}");
        assert!(Path::new(path).exists(), "path={path}");
    }

    let out = search(&d, &["no-such-text-anywhere", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "[]");

    // Errors use the existing JSON error object on stderr.
    let out = search(&d, &["", "--json"]);
    assert_eq!(out.status.code(), Some(4));
    let value: serde_json::Value = serde_json::from_str(&stderr(&out)).unwrap();
    assert_eq!(value["error"]["code"], 4);
    assert!(value["error"]["message"].is_string());
}

// ---------------------------------------------------------------------------
// T-14: usage errors exit 2
// ---------------------------------------------------------------------------

#[test]
fn t14_usage_errors_exit_2() {
    let d = tmpdir("t14");
    init(&d);

    let cases: &[&[&str]] = &[
        &[],                             // missing query
        &["one", "two"],                 // unquoted multi-word query
        &["-draft"],                     // option-like query without --
        &["text", "--type", "bogus"],    // unknown type
        &["text", "--type", "DOC"],      // uppercase type
        &["text", "--type", "task,doc"], // comma-joined type
        &["text", "--type"],             // missing type value
        &["text", "--limit", "abc"],     // non-integer limit
        &["text", "--limit", "0"],       // zero limit
        &["text", "--limit", "-1"],      // negative limit
    ];
    for case in cases {
        let out = search(&d, case);
        assert_eq!(
            out.status.code(),
            Some(2),
            "case={case:?} stderr={}",
            stderr(&out)
        );
    }

    // A query beginning with '-' is accepted after `--`.
    let out = search(&d, &["--", "-draft"]);
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
}

// ---------------------------------------------------------------------------
// T-15: validation errors exit 4
// ---------------------------------------------------------------------------

#[test]
fn t15_validation_errors_exit_4() {
    let d = tmpdir("t15");
    init(&d);

    for query in ["", "   ", "\t", "bad\rx", "bad\nx"] {
        let out = search(&d, &[query]);
        assert_eq!(out.status.code(), Some(4), "query={query:?}");
        assert_eq!(stdout(&out), "", "query={query:?}");
    }

    // A query containing `/` or `\` is a literal search character, not a path.
    let out = search(&d, &["a/b"]);
    assert_eq!(out.status.code(), Some(0));
    let out = search(&d, &["a\\b"]);
    assert_eq!(out.status.code(), Some(0));
}

// ---------------------------------------------------------------------------
// T-16: symlink escape exits 1 without exposing outside content
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn t16a_symlinked_directory_escape_exits_one() {
    use std::os::unix::fs::symlink;

    let root = tmpdir("t16a");
    let proj = root.join("proj");
    fs::create_dir_all(&proj).unwrap();
    init(&proj);
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    write_file(&outside.join("secret.md"), "# SECRET SECRETWORD\n");

    let active = proj.join("docs/requirements/business");
    fs::remove_dir_all(&active).unwrap();
    symlink(&outside, &active).unwrap();

    let out = search(&proj, &["secretword"]);
    assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(!stderr(&out).contains("SECRETWORD"));
}

#[cfg(unix)]
#[test]
fn t16b_symlinked_file_escape_exits_one() {
    use std::os::unix::fs::symlink;

    let root = tmpdir("t16b");
    let proj = root.join("proj");
    fs::create_dir_all(&proj).unwrap();
    setup_standard(&proj);
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    let outside_file = outside.join("secret.md");
    write_file(&outside_file, "# SECRET SECRETWORD\n");

    let target = proj.join("docs/requirements/business/REQ-001.001.md");
    fs::remove_file(&target).unwrap();
    symlink(&outside_file, &target).unwrap();

    let out = search(&proj, &["secretword"]);
    assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(!stderr(&out).contains("SECRETWORD"));
}

// ---------------------------------------------------------------------------
// T-17: search creates no files or directories
// ---------------------------------------------------------------------------

#[test]
fn t17_search_creates_no_files_or_directories() {
    // Project with no configuration and no document directories.
    let d = tmpdir("t17-uninitialized");
    let before = snapshot(&d);
    let out = search(&d, &["anything"]);
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert_eq!(before, snapshot(&d));

    // Initialized project whose configured type directory is missing.
    let d2 = tmpdir("t17-missing-dir");
    init(&d2);
    fs::remove_dir_all(d2.join("docs/requirements/business")).unwrap();
    let before = snapshot(&d2);
    let out = search(&d2, &["anything"]);
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert_eq!(before, snapshot(&d2));
}

// ---------------------------------------------------------------------------
// Business rules search (TASK-44)
// ---------------------------------------------------------------------------

fn create_rule(project: &Path, title: &str, description: &str) -> String {
    let out = assert_ok(run(
        project,
        &[
            "rule",
            "create",
            title,
            "--description",
            description,
            "--rationale",
            "rationale",
            "--applicability",
            "applies",
        ],
    ));
    stdout(&out).trim().to_string()
}

#[test]
fn br_search_default_and_doc_include_rul() {
    let d = tmpdir("br-default");
    init(&d);
    let id = create_rule(&d, "Rule Title", "quick brown fox");
    assert_eq!(id, "RUL-001");

    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("RUL-001 doc Rule Title"));

    let out = search(&d, &["quick brown", "--type", "doc"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("RUL-001 doc Rule Title"));

    for ty in ["task", "milestone", "release", "decision"] {
        let out = search(&d, &["quick brown", "--type", ty]);
        assert_eq!(out.status.code(), Some(0), "type={ty}");
        assert_eq!(stdout(&out), "", "type={ty}");
    }
}

#[test]
fn br_search_quoted_phrase_and_limit() {
    let d = tmpdir("br-phrase");
    init(&d);
    create_rule(&d, "First Rule", "quick brown fox");
    create_rule(&d, "Second Rule", "quick brown fox");

    // contiguous phrase matches
    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).lines().count(), 2);

    // non-contiguous words do not match as a phrase
    let out = search(&d, &["quick fox"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    // --limit caps combined result set
    let out = search(&d, &["quick brown", "--limit", "1"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).lines().count(), 1);
}

#[test]
fn br_search_br_files_are_ignored() {
    let d = tmpdir("br-ignore");
    init(&d);
    // A canonical BR-* document sharing the directory must never be a record.
    write_file(
        &d.join("docs/business-rules/BR-999.md"),
        "# BR-999 canonical document with searchable phrase\n",
    );
    // Also a non <prefix>-<digits> file.
    write_file(
        &d.join("docs/business-rules/notes.md"),
        "# notes with searchable phrase\n",
    );
    create_rule(&d, "Rule Title", "searchable phrase");

    let out = search(&d, &["searchable phrase"]);
    assert_eq!(out.status.code(), Some(0));
    let s = stdout(&out);
    assert!(s.contains("RUL-001"), "output={s}");
    assert!(!s.contains("BR-999"), "output={s}");
    assert!(!s.contains("notes"), "output={s}");
}

#[test]
fn br_search_deprecated_excluded_restored_visible() {
    let d = tmpdir("br-depr");
    init(&d);
    create_rule(&d, "Rule Title", "quick brown fox");

    assert_ok(run(&d, &["rule", "deprecate", "RUL-001"]));
    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    assert_ok(run(&d, &["rule", "restore", "RUL-001"]));
    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("RUL-001 doc Rule Title"));
}

#[test]
fn br_search_disabled_type_contributes_nothing() {
    let d = tmpdir("br-disabled");
    init(&d);
    create_rule(&d, "Rule Title", "quick brown fox");

    assert_ok(run(
        &d,
        &["config", "set", "document_types.rule.enabled", "false"],
    ));
    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    // Re-enable restores visibility.
    assert_ok(run(
        &d,
        &["config", "set", "document_types.rule.enabled", "true"],
    ));
    let out = search(&d, &["quick brown"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("RUL-001 doc Rule Title"));
}

#[test]
fn br_search_missing_dir_empty_success() {
    let d = tmpdir("br-missing");
    init(&d);
    fs::remove_dir_all(d.join("docs/business-rules")).unwrap();
    let out = search(&d, &["anything"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

#[test]
fn br_search_creates_no_files_or_directories() {
    let d = tmpdir("br-nowrite");
    init(&d);
    fs::remove_dir_all(d.join("docs/business-rules")).unwrap();
    let before = snapshot(&d);
    let out = search(&d, &["anything"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    assert_eq!(before, snapshot(&d));
}

#[cfg(unix)]
#[test]
fn br_search_symlink_dir_escape_exits_one() {
    use std::os::unix::fs::symlink;

    let root = tmpdir("br-symdir");
    let proj = root.join("proj");
    fs::create_dir_all(&proj).unwrap();
    init(&proj);
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    write_file(&outside.join("secret.md"), "# SECRET SECRETWORD\n");

    let active = proj.join("docs/business-rules");
    fs::remove_dir_all(&active).unwrap();
    symlink(&outside, &active).unwrap();

    let out = search(&proj, &["secretword"]);
    assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(!stderr(&out).contains("SECRETWORD"));
}

#[cfg(unix)]
#[test]
fn br_search_symlink_file_escape_exits_one() {
    use std::os::unix::fs::symlink;

    let root = tmpdir("br-symfile");
    let proj = root.join("proj");
    fs::create_dir_all(&proj).unwrap();
    init(&proj);
    create_rule(&proj, "Rule Title", "placeholder");
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    let outside_file = outside.join("secret.md");
    write_file(&outside_file, "# SECRET SECRETWORD\n");

    let target = proj.join("docs/business-rules/RUL-001.md");
    fs::remove_file(&target).unwrap();
    symlink(&outside_file, &target).unwrap();

    let out = search(&proj, &["secretword"]);
    assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(!stderr(&out).contains("SECRETWORD"));
}
