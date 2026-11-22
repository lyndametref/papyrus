//! TASK-113: config-defined document types in `papyrus search`.
//!
//! Scenarios T-113-01 .. T-113-13 from
//! `docs/specifications/search-config-defined-document-types.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NOTE_BLOCK: &str = "  note:\n    prefix: NOTE\n    directory: notes/\n    enabled: true\n    statuses: [active, archived]\n";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn project_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("t113-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn run(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
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

fn search(project: &Path, args: &[&str]) -> Output {
    let mut full: Vec<&str> = vec!["search"];
    full.extend_from_slice(args);
    run(project, &full)
}

fn init(name: &str) -> PathBuf {
    let dir = project_root(name);
    assert_ok(run(&dir, &["init", "Proj", "--no-git"]));
    dir
}

fn config_path(project: &Path) -> PathBuf {
    project.join("papyrus.config.yml")
}

/// Insert a valid `note` type entry immediately under `document_types:`.
fn add_via_config(project: &Path, block: &str) {
    let path = config_path(project);
    let cfg = fs::read_to_string(&path).unwrap();
    let updated = cfg.replacen(
        "document_types:\n",
        &format!("document_types:\n{}", block),
        1,
    );
    assert_ne!(updated, cfg, "document_types key missing");
    fs::write(&path, updated).unwrap();
}

fn add_note(project: &Path) {
    add_via_config(project, NOTE_BLOCK);
}

fn create_note(project: &Path, title: &str, description: &str, extra: &[&str]) -> String {
    let mut args: Vec<&str> = vec!["note", "create", title, "--description", description];
    args.extend_from_slice(extra);
    stdout(&assert_ok(run(project, &args))).trim().to_string()
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn note_record(id: &str, title: &str, status: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: \"{title}\"\nstatus: {status}\n---\n# {id} {title}\n\n## Description\n{body}\n"
    )
}

// T-113-01 -----------------------------------------------------------------
#[test]
fn t113_01_user_added_generic_type_is_searchable_by_title_description_label_reference() {
    let d = init("01");
    add_note(&d);
    let id = create_note(
        &d,
        "Wallet Note",
        "unique-description-marker",
        &["--label", "unique-label-marker", "--reference", "REF-777"],
    );
    assert_eq!(id, "NOTE-001");

    for query in [
        "Wallet Note",
        "unique-description-marker",
        "unique-label-marker",
        "REF-777",
    ] {
        let out = search(&d, &[query]);
        assert_eq!(out.status.code(), Some(0), "query={query}");
        assert!(
            stdout(&out).contains("NOTE-001 doc Wallet Note"),
            "query={query} stdout={}",
            stdout(&out)
        );
        let out = search(&d, &[query, "--type", "doc"]);
        assert!(
            stdout(&out).contains("NOTE-001 doc Wallet Note"),
            "query={query}"
        );
    }
    let _ = fs::remove_dir_all(&d);
}

// T-113-02 -----------------------------------------------------------------
#[test]
fn t113_02_json_shape_and_canonical_path_for_generic_type() {
    let d = init("02");
    add_note(&d);
    create_note(&d, "Wallet Note", "json-marker", &[]);

    let out = assert_ok(search(&d, &["json-marker", "--json"]));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let hits = json.as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["id"], "NOTE-001");
    assert_eq!(hits[0]["type"], "doc");
    assert_eq!(hits[0]["title"], "Wallet Note");
    let path = hits[0]["path"].as_str().unwrap();
    assert!(Path::new(path).is_absolute(), "path not absolute: {path}");
    assert!(path.ends_with("docs/notes/NOTE-001.md"), "path={path}");
    assert_eq!(
        Path::new(path).canonicalize().unwrap(),
        d.canonicalize().unwrap().join("docs/notes/NOTE-001.md")
    );
    let _ = fs::remove_dir_all(&d);
}

// T-113-03 -----------------------------------------------------------------
#[test]
fn t113_03_non_document_type_filters_never_return_generic_records() {
    let d = init("03");
    add_note(&d);
    create_note(&d, "Wallet Note", "filter-marker", &[]);

    for ty in ["task", "milestone", "release", "decision"] {
        let out = search(&d, &["filter-marker", "--type", ty]);
        assert_eq!(out.status.code(), Some(0), "type={ty}");
        assert!(!stdout(&out).contains("NOTE-001"), "type={ty}");
    }
    let _ = fs::remove_dir_all(&d);
}

// T-113-04 -----------------------------------------------------------------
#[test]
fn t113_04_disabled_type_contributes_nothing_and_reenable_restores_it() {
    let d = init("04");
    add_note(&d);
    create_note(&d, "Wallet Note", "toggle-marker", &[]);

    assert!(stdout(&search(&d, &["toggle-marker"])).contains("NOTE-001"));

    assert_ok(run(
        &d,
        &["config", "set", "document_types.note.enabled", "false"],
    ));
    let out = search(&d, &["toggle-marker"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(!stdout(&out).contains("NOTE-001"));

    assert_ok(run(
        &d,
        &["config", "set", "document_types.note.enabled", "true"],
    ));
    assert!(stdout(&search(&d, &["toggle-marker"])).contains("NOTE-001"));
    let _ = fs::remove_dir_all(&d);
}

// T-113-05 -----------------------------------------------------------------
#[test]
fn t113_05_archived_records_and_archived_subdirectory_are_not_scanned() {
    let d = init("05");
    add_note(&d);
    create_note(&d, "Wallet Note", "archive-marker", &[]);
    assert_ok(run(&d, &["note", "archive", "NOTE-001"]));

    let out = search(&d, &["archive-marker"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");

    write_file(
        &d.join("docs/notes/archived/NOTE-998.md"),
        &note_record("NOTE-998", "Hand Archived", "archived", "archive-marker"),
    );
    let out = search(&d, &["archive-marker"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    let _ = fs::remove_dir_all(&d);
}

// T-113-06 -----------------------------------------------------------------
#[test]
fn t113_06_archived_status_in_active_directory_is_relocated_and_hidden() {
    let d = init("06");
    add_note(&d);
    create_note(&d, "Wallet Note", "relocate-marker", &[]);
    let active = d.join("docs/notes/NOTE-001.md");
    let content = fs::read_to_string(&active)
        .unwrap()
        .replace("status: active", "status: archived");
    fs::write(&active, content).unwrap();

    let out = search(&d, &["relocate-marker"]);
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(!active.exists(), "active file should be relocated");
    assert!(d.join("docs/notes/archived/NOTE-001.md").exists());
    assert!(
        stderr(&out).contains("relocated"),
        "stderr={}",
        stderr(&out)
    );
    let _ = fs::remove_dir_all(&d);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let blocked = init("06-blocked");
        add_note(&blocked);
        create_note(&blocked, "Wallet Note", "blocked-marker", &[]);
        let archived = blocked.join("docs/notes/archived");
        fs::create_dir_all(&archived).unwrap();
        fs::set_permissions(&archived, fs::Permissions::from_mode(0o555)).unwrap();
        let active = blocked.join("docs/notes/NOTE-001.md");
        let content = fs::read_to_string(&active)
            .unwrap()
            .replace("status: active", "status: archived");
        fs::write(&active, content).unwrap();

        let out = search(&blocked, &["blocked-marker"]);
        assert_eq!(out.status.code(), Some(4), "stderr={}", stderr(&out));
        assert_eq!(stdout(&out), "");
        assert!(active.exists(), "source must stay intact");
        assert!(
            stderr(&out).contains("NOTE-001.md"),
            "stderr={}",
            stderr(&out)
        );

        fs::set_permissions(&archived, fs::Permissions::from_mode(0o755)).unwrap();
        let _ = fs::remove_dir_all(&blocked);
    }
}

// T-113-07 -----------------------------------------------------------------
#[test]
fn t113_07_legacy_inactive_subdirectory_is_never_scanned() {
    let d = init("07");
    add_note(&d);
    write_file(
        &d.join("docs/notes/deprecated/NOTE-900.md"),
        &note_record("NOTE-900", "Legacy Note", "active", "legacy-marker"),
    );

    let out = search(&d, &["legacy-marker"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    let _ = fs::remove_dir_all(&d);
}

// T-113-08 -----------------------------------------------------------------
#[test]
fn t113_08_custom_status_record_stays_searchable() {
    let d = init("08");
    add_note(&d);
    create_note(&d, "Wallet Note", "review-marker", &[]);
    assert_ok(run(
        &d,
        &[
            "config",
            "set",
            "document_types.note.statuses",
            "active,archived,review",
        ],
    ));
    assert_ok(run(&d, &["note", "status", "NOTE-001", "review"]));

    let out = search(&d, &["review-marker"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("NOTE-001 doc Wallet Note"));
    let _ = fs::remove_dir_all(&d);
}

// T-113-09 -----------------------------------------------------------------
#[test]
fn t113_09_configuration_changes_follow_new_prefix_and_directory() {
    let d = init("09");
    add_note(&d);
    create_note(&d, "Original Note", "original-marker", &[]);

    assert_ok(run(
        &d,
        &["config", "set", "document_types.note.prefix", "NTE"],
    ));
    assert_ok(run(
        &d,
        &["config", "set", "document_types.note.directory", "memos/"],
    ));
    let id = create_note(&d, "Renamed Note", "renamed-marker", &[]);
    assert_eq!(id, "NTE-001");

    let out = search(&d, &["renamed-marker"]);
    assert_eq!(
        stdout(&out).trim(),
        "NTE-001 doc Renamed Note",
        "stderr={}",
        stderr(&out)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&assert_ok(search(
        &d,
        &["renamed-marker", "--json"],
    ))))
    .unwrap();
    assert!(
        json[0]["path"]
            .as_str()
            .unwrap()
            .ends_with("docs/memos/NTE-001.md"),
        "json={}",
        json
    );
    assert!(!stdout(&search(&d, &["original-marker"])).contains("NOTE-001"));
    let _ = fs::remove_dir_all(&d);
}

// T-113-10 -----------------------------------------------------------------
#[test]
fn t113_10_builtin_req_mockup_and_rule_titles_are_unchanged() {
    let d = init("10");
    assert_ok(run(
        &d,
        &["req", "create-group", "--title", "Payments Group"],
    ));
    assert_ok(run(
        &d,
        &[
            "req",
            "create",
            "Capture payment",
            "--group",
            "REQ-001",
            "--description",
            "cashier flow",
        ],
    ));
    assert_ok(run(
        &d,
        &[
            "ui-mockup",
            "create",
            "Mockup Title Marker",
            "--task",
            "TASK-001",
            "--spec",
            "UI-001",
        ],
    ));
    assert_ok(run(
        &d,
        &[
            "rule",
            "create",
            "Rule Title Marker",
            "--description",
            "rule body",
            "--rationale",
            "r",
            "--applicability",
            "a",
        ],
    ));

    let group = stdout(&search(&d, &["Payments"]));
    assert!(
        group.contains("REQ-001 doc Payments Group"),
        "stdout={group}"
    );
    let member = stdout(&search(&d, &["Capture payment"]));
    assert!(
        member.contains("REQ-001.001 doc Capture payment"),
        "stdout={member}"
    );
    assert!(
        stdout(&search(&d, &["Mockup Title Marker"])).contains("MOCK-001 doc Mockup Title Marker")
    );
    assert!(stdout(&search(&d, &["Rule Title Marker"])).contains("RUL-001 doc Rule Title Marker"));
    let _ = fs::remove_dir_all(&d);
}

// T-113-11 -----------------------------------------------------------------
#[test]
fn t113_11_ordering_limit_and_repeated_type_filter() {
    let d = init("11");
    add_note(&d);
    create_note(&d, "Order Marker One", "order-marker", &[]);
    create_note(&d, "Order Marker Two", "order-marker", &[]);
    assert_ok(run(
        &d,
        &[
            "task",
            "create",
            "Order Marker Task",
            "--description",
            "order-marker",
        ],
    ));

    let out = assert_ok(search(&d, &["order-marker"]));
    let text = stdout(&out);
    let ids: Vec<&str> = text
        .lines()
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(ids, vec!["NOTE-001", "NOTE-002", "TASK-001"]);

    let limited = assert_ok(search(&d, &["order-marker", "--limit", "2"]));
    assert_eq!(stdout(&limited).lines().count(), 2);
    assert!(stdout(&limited).contains("NOTE-001"));
    assert!(stdout(&limited).contains("NOTE-002"));
    assert!(!stdout(&limited).contains("TASK-001"));

    let repeated = assert_ok(search(
        &d,
        &["order-marker", "--type", "doc", "--type", "doc"],
    ));
    assert_eq!(stdout(&repeated).lines().count(), 2);
    let _ = fs::remove_dir_all(&d);
}

// T-113-12 -----------------------------------------------------------------
#[test]
fn t113_12_missing_unreadable_and_symlink_escape() {
    let missing = init("12-missing");
    add_note(&missing);
    let out = search(&missing, &["anything"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    let _ = fs::remove_dir_all(&missing);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let unreadable = init("12-unreadable");
        add_note(&unreadable);
        create_note(&unreadable, "Wallet Note", "locked-marker", &[]);
        let notes = unreadable.join("docs/notes");
        fs::set_permissions(&notes, fs::Permissions::from_mode(0o000)).unwrap();
        let out = search(&unreadable, &["locked-marker"]);
        assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
        assert_eq!(stdout(&out), "");
        fs::set_permissions(&notes, fs::Permissions::from_mode(0o755)).unwrap();
        let _ = fs::remove_dir_all(&unreadable);

        let escape = init("12-symlink");
        add_note(&escape);
        create_note(&escape, "Wallet Note", "escape-marker", &[]);
        let outside_dir = project_root("12-outside");
        let outside = outside_dir.join("outside.md");
        write_file(&outside, "# SECRET SECRETWORD\n");
        let record = escape.join("docs/notes/NOTE-001.md");
        fs::remove_file(&record).unwrap();
        std::os::unix::fs::symlink(&outside, &record).unwrap();
        let out = search(&escape, &["secretword"]);
        assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
        assert_eq!(stdout(&out), "");
        assert!(!stderr(&out).contains("SECRETWORD"));
        let _ = fs::remove_dir_all(&escape);
        let _ = fs::remove_dir_all(&outside_dir);
    }
}

// T-113-13 -----------------------------------------------------------------
#[test]
fn t113_13_search_module_has_no_static_document_key_list_or_slug_arm() {
    let source = include_str!("../src/search.rs");
    // Only the production module matters: its inline test module may name
    // descriptors without introducing a search-time slug arm.
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);
    let source = production;
    let forbidden_key_list = ["DOC", "_KEYS"].concat();
    let forbidden_collector = ["collect", "_type"].concat();
    assert!(!source.contains(&forbidden_key_list));
    assert!(!source.contains(&forbidden_collector));
    for slug in [
        ["re", "q"].concat(),
        ["nf", "r"].concat(),
        ["gloss", "ary"].concat(),
        ["ui-", "spec"].concat(),
        ["ui-", "mockup"].concat(),
        ["te", "st-report"].concat(),
    ] {
        let literal = format!("\"{}\"", slug);
        assert!(!source.contains(&literal), "slug arm remains: {}", slug);
    }
}
