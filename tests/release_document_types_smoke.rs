//! TASK-117: cross-surface and upgrade smoke checks for config-defined
//! document types.
//!
//! The canonical plan is
//! `docs/specifications/config-defined-document-types-verification-and-packaging.md`.
//! CLI rows C-01 .. C-26, the upgrade scenario (section 5), and the browser
//! matrix B-01 .. B-18 are exercised here against the built binary. The
//! existing per-feature suites remain authoritative for their own contracts;
//! this file adds the combined evidence.
//!
//! Every project lives under `./.tmp/task117/` and is removed by [`Scratch`]
//! even when an assertion panics.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Output, Stdio};
use std::time::Duration;

const NOTE_BLOCK: &str = "  note:\n    prefix: NOTE\n    directory: notes/\n    enabled: true\n    statuses: [active, archived, review, approved]\n    default_status: active\n";

const DEFAULT_DOCUMENT_SLUGS: [&str; 11] = [
    "audit",
    "glossary",
    "nfr",
    "req",
    "rule",
    "tech-spec",
    "test-case",
    "test-report",
    "ui-mockup",
    "ui-spec",
    "use-case",
];

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

/// A self-removing scratch directory under the project's `.tmp/task117/`.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".tmp");
        path.push("task117");
        path.push(format!("{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Scratch { path }
    }

    fn project(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn run(project: &Path, args: &[&str]) -> Output {
    let mut full = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn run_with_stdin(project: &Path, args: &[&str], input: &str) -> Output {
    let mut full = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    let mut child = bin()
        .args(&full)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn run_ok(project: &Path, args: &[&str]) -> String {
    let out = run(project, args);
    assert!(
        out.status.success(),
        "command failed: {args:?}\nstderr: {}",
        stderr(&out)
    );
    stdout(&out).trim().to_string()
}

fn init(scratch: &Scratch, name: &str) -> PathBuf {
    let project = scratch.project(name);
    run_ok(&project, &["init", "Proj", "--no-git"]);
    project
}

fn config_path(project: &Path) -> PathBuf {
    project.join("papyrus.config.yml")
}

fn add_note(project: &Path) {
    let path = config_path(project);
    let cfg = fs::read_to_string(&path).unwrap();
    let updated = cfg.replacen(
        "document_types:\n",
        &format!("document_types:\n{NOTE_BLOCK}"),
        1,
    );
    assert_ne!(updated, cfg, "document_types key missing");
    fs::write(&path, updated).unwrap();
}

fn set_note_enabled(project: &Path, enabled: bool) {
    let path = config_path(project);
    let cfg = fs::read_to_string(&path).unwrap();
    let from = if enabled {
        "prefix: NOTE\n    directory: notes/\n    enabled: false"
    } else {
        "prefix: NOTE\n    directory: notes/\n    enabled: true"
    };
    let to = if enabled {
        "prefix: NOTE\n    directory: notes/\n    enabled: true"
    } else {
        "prefix: NOTE\n    directory: notes/\n    enabled: false"
    };
    let updated = cfg.replacen(from, to, 1);
    assert_ne!(updated, cfg, "note enabled state not found");
    fs::write(&path, updated).unwrap();
}

fn create_note(project: &Path, title: &str) -> String {
    run_ok(project, &["note", "create", title])
}

fn active_note_path(project: &Path, id: &str) -> PathBuf {
    project.join("docs/notes").join(format!("{id}.md"))
}

fn archived_note_path(project: &Path, id: &str) -> PathBuf {
    project.join("docs/notes/archived").join(format!("{id}.md"))
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn note_record(id: &str, title: &str, status: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: \"{title}\"\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: {status}\n---\n# {id} {title}\n\n## Description\n{body}\n"
    )
}

fn parse_json(text: &str) -> serde_json::Value {
    serde_json::from_str(text.trim()).unwrap_or_else(|err| panic!("not JSON: {err}\n{text}"))
}

fn hash(path: &Path) -> String {
    Sha256::digest(&fs::read(path).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Minimal SHA-256 so the tests can assert byte identity without a new
/// dependency. Implementation follows FIPS 180-4.
struct Sha256;

impl Sha256 {
    fn digest(data: &[u8]) -> [u8; 32] {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let bit_len = (data.len() as u64) * 8;
        let mut message = data.to_vec();
        message.push(0x80);
        while message.len() % 64 != 56 {
            message.push(0);
        }
        message.extend_from_slice(&bit_len.to_be_bytes());
        for chunk in message.chunks(64) {
            let mut w = [0u32; 64];
            for (i, word) in chunk.chunks(4).take(16).enumerate() {
                w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ (!e & g);
                let temp1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
                *slot = slot.wrapping_add(value);
            }
        }
        let mut out = [0u8; 32];
        for (i, word) in h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

// C-01 .. C-08 --------------------------------------------------------------
#[test]
fn t117_01_generated_help_and_dispatch_matrix() {
    let scratch = Scratch::new("help");
    let project = init(&scratch, "proj");
    add_note(&project);

    let help = run_ok(&project, &["--help"]);
    for slug in DEFAULT_DOCUMENT_SLUGS {
        assert!(help.contains(&format!("  {slug} ")), "help missing {slug}");
    }
    assert!(!help.contains("  deploy "), "deploy must not be seeded");
    assert!(help.contains("  note "), "note declared type must appear");
    assert!(!help.contains("nfreq"), "nfreq alias must not exist");

    let note_help = run_ok(&project, &["note", "--help"]);
    for verb in [
        "create", "list", "view", "edit", "archive", "delete", "status",
    ] {
        assert!(note_help.contains(verb), "note help missing {verb}");
    }

    let req_help = run_ok(&project, &["req", "--help"]);
    assert!(req_help.contains("create-group"));
    assert!(run_ok(&project, &["req", "create", "--help"]).contains("--group"));
    let nfr_help = run_ok(&project, &["nfr", "--help"]);
    assert!(nfr_help.contains("create-group"));
    assert!(run_ok(&project, &["nfr", "create", "--help"]).contains("--group"));

    let mockup_help = run_ok(&project, &["ui-mockup", "--help"]);
    assert!(mockup_help.contains("supersede"));
    assert!(!mockup_help.contains("status"));
    assert!(!run_ok(&project, &["ui-mockup", "create", "--help"]).contains("--status"));

    for (args, label) in [
        (vec!["nfreq", "--help"], "nfreq"),
        (vec!["unknown-slug", "--help"], "unknown slug"),
        (vec!["deploy", "--help"], "undeclared deploy"),
    ] {
        let out = run(&project, &args);
        assert_eq!(out.status.code(), Some(2), "{label} must exit 2");
    }

    set_note_enabled(&project, false);
    for verb in [
        vec!["note", "list"],
        vec!["note", "create", "Hidden"],
        vec!["note", "view", "NOTE-001"],
    ] {
        let out = run(&project, &verb);
        assert_eq!(out.status.code(), Some(2), "disabled {verb:?} must exit 2");
    }
    let missing = run(&project, &["--help"]);
    assert_eq!(missing.status.code(), Some(0));
    assert!(!stdout(&missing).contains("  note "));
    set_note_enabled(&project, true);
    assert!(run_ok(&project, &["--help"]).contains("  note "));
}

// C-09 .. C-19 --------------------------------------------------------------
#[test]
fn t117_02_cli_lifecycle_persistence_and_validation() {
    let scratch = Scratch::new("lifecycle");
    let project = init(&scratch, "proj");
    add_note(&project);

    let id = run_ok(
        &project,
        &[
            "note",
            "create",
            "First note",
            "--description",
            "first body",
            "--label",
            "alpha",
            "--reference",
            "REF-001",
        ],
    );
    assert_eq!(id, "NOTE-001");
    let path = active_note_path(&project, "NOTE-001");
    assert!(path.exists(), "create must write docs/notes/NOTE-001.md");
    let created = fs::read_to_string(&path).unwrap();
    assert!(created.contains("title: \"First note\""));
    assert!(created.contains("first body"));
    assert!(created.contains("- alpha"));

    let list = run_ok(&project, &["note", "list"]);
    assert!(list.starts_with("NOTE-001"));
    let view = run_ok(&project, &["note", "view", "NOTE-001"]);
    assert!(view.contains("NOTE-001"));
    assert!(view.contains("first body"));
    assert!(view.contains("status: active"));

    let edited = run(
        &project,
        &["note", "edit", "NOTE-001", "--title", "Renamed"],
    );
    assert_eq!(edited.status.code(), Some(0));
    let after_edit = fs::read_to_string(&path).unwrap();
    assert!(after_edit.contains("title: Renamed"));
    assert!(
        after_edit.contains("first body"),
        "only supplied field changes"
    );
    assert!(after_edit.contains("REF-001"));

    run_ok(&project, &["note", "status", "NOTE-001", "review"]);
    let reviewed = fs::read_to_string(&path).unwrap();
    assert!(reviewed.contains("status: review"));
    assert!(path.exists(), "status keeps the file in place");

    let before_reject = hash(&path);
    let rejected = run(&project, &["note", "status", "NOTE-001", "archived"]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(stderr(&rejected).contains("archive"));
    assert_eq!(hash(&path), before_reject, "rejected status must not write");

    run_ok(&project, &["note", "archive", "NOTE-001"]);
    let archived = archived_note_path(&project, "NOTE-001");
    assert!(archived.exists(), "archive moves under archived/");
    assert!(!path.exists());
    let archived_body = fs::read_to_string(&archived).unwrap();
    assert!(archived_body.contains("status: archived"));
    assert!(!archived_body.contains("status: review"));
    assert!(!run_ok(&project, &["note", "list"]).contains("NOTE-001"));

    let second = create_note(&project, "Second note");
    assert_eq!(second, "NOTE-002");
    let second_path = active_note_path(&project, "NOTE-002");
    let declined = run_with_stdin(&project, &["note", "delete", "NOTE-002"], "n\n");
    assert_eq!(declined.status.code(), Some(0));
    assert!(stdout(&declined).contains("[y/N]"));
    assert!(second_path.exists(), "decline must not change anything");
    let accepted = run_with_stdin(&project, &["note", "delete", "NOTE-002"], "y\n");
    assert_eq!(accepted.status.code(), Some(0));
    assert!(!second_path.exists());
    assert!(!archived_note_path(&project, "NOTE-002").exists());

    let third = create_note(&project, "Third note");
    assert!(third.starts_with("NOTE-"), "third create returns an ID");
    let forced = run(&project, &["note", "delete", &third, "--force"]);
    assert_eq!(forced.status.code(), Some(0));
    assert!(stdout(&forced).trim().is_empty(), "--force must not prompt");
    assert!(!active_note_path(&project, &third).exists());

    let missing = run(&project, &["note", "view", "NOTE-999"]);
    assert_eq!(missing.status.code(), Some(3));

    let count_before = fs::read_dir(project.join("docs/notes")).unwrap().count();
    for bad in ["", "\n", "\r", "path/separator"] {
        let out = run(&project, &["note", "create", bad]);
        assert_eq!(out.status.code(), Some(4), "invalid title {bad:?}");
        assert!(stderr(&out).contains("cannot be normalized"));
    }
    let count_after = fs::read_dir(project.join("docs/notes")).unwrap().count();
    assert_eq!(count_before, count_after, "invalid titles write nothing");

    if !running_as_root() {
        use std::os::unix::fs::PermissionsExt;
        let dir = project.join("docs/notes");
        let original = fs::metadata(&dir).unwrap().permissions();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
        let out = run(&project, &["note", "create", "Blocked"]);
        fs::set_permissions(&dir, original).unwrap();
        assert_eq!(
            out.status.code(),
            Some(1),
            "unwritable directory is runtime"
        );
        assert!(stderr(&out).contains("Permission denied") || stderr(&out).contains("permission"));
    }
}

fn running_as_root() -> bool {
    let out = Command::new("id").arg("-u").output().unwrap();
    stdout(&out).trim() == "0"
}

// C-20 ---------------------------------------------------------------------
#[test]
fn t117_03_json_is_accepted_on_every_document_verb() {
    let scratch = Scratch::new("json");
    let project = init(&scratch, "proj");
    add_note(&project);

    let create = run(&project, &["note", "create", "Json note", "--json"]);
    assert_eq!(create.status.code(), Some(0));
    assert_eq!(parse_json(&stdout(&create))["id"], "NOTE-001");

    let list = run(&project, &["note", "--json", "list"]);
    assert_eq!(list.status.code(), Some(0));
    let rows = parse_json(&stdout(&list));
    assert_eq!(rows[0]["id"], "NOTE-001");

    let view = run(&project, &["note", "view", "NOTE-001", "--json"]);
    assert_eq!(view.status.code(), Some(0));
    let view_json = parse_json(&stdout(&view));
    assert_eq!(view_json["id"], "NOTE-001");
    assert!(view_json["metadata"]["status"].is_string());

    for args in [
        vec![
            "note",
            "edit",
            "NOTE-001",
            "--title",
            "Json renamed",
            "--json",
        ],
        vec!["note", "status", "NOTE-001", "review", "--json"],
    ] {
        let out = run(&project, &args);
        assert_eq!(out.status.code(), Some(0), "{args:?}");
        assert_eq!(parse_json(&stdout(&out))["status"], "ok");
    }
    let archive = run(&project, &["note", "archive", "NOTE-001", "--json"]);
    assert_eq!(archive.status.code(), Some(0));
    assert_eq!(parse_json(&stdout(&archive))["status"], "ok");

    create_note(&project, "Deleted");
    let delete = run(
        &project,
        &["note", "delete", "NOTE-002", "--force", "--json"],
    );
    assert_eq!(delete.status.code(), Some(0));
    assert_eq!(parse_json(&stdout(&delete))["status"], "ok");

    let error = run(&project, &["note", "view", "NOTE-999", "--json"]);
    assert_eq!(error.status.code(), Some(3));
    let error_json = parse_json(&stderr(&error));
    assert_eq!(error_json["error"]["code"], 3);
    assert!(stdout(&error).trim().is_empty());
}

// C-21 .. C-23 --------------------------------------------------------------
#[test]
fn t117_04_search_front_matter_and_hidden_states() {
    let scratch = Scratch::new("search");
    let project = init(&scratch, "proj");
    add_note(&project);

    run_ok(
        &project,
        &[
            "note",
            "create",
            "Wallet note",
            "--description",
            "body with no special marker",
            "--label",
            "fm-only-label",
        ],
    );
    run_ok(&project, &["note", "create", "Archived note"]);
    run_ok(&project, &["note", "archive", "NOTE-002"]);

    write_file(
        &project.join("docs/notes/deprecated/NOTE-900.md"),
        &note_record(
            "NOTE-900",
            "Deprecated note",
            "deprecated",
            "deprecated-marker-xyz",
        ),
    );

    let before = hash(&config_path(&project));
    let docs_hash = hash(&project.join("docs/notes/NOTE-001.md"));
    let front_matter_only = run_ok(&project, &["search", "fm-only-label"]);
    assert_eq!(front_matter_only, "NOTE-001 doc Wallet note");
    assert_eq!(hash(&config_path(&project)), before);
    assert_eq!(hash(&project.join("docs/notes/NOTE-001.md")), docs_hash);

    let by_title = run_ok(&project, &["search", "Wallet note", "--type", "doc"]);
    assert_eq!(by_title, "NOTE-001 doc Wallet note");
    let json = parse_json(&run_ok(&project, &["search", "Wallet note", "--json"]));
    assert_eq!(json[0]["id"], "NOTE-001");
    assert_eq!(json[0]["type"], "doc");

    let archived = run(&project, &["search", "Archived note"]);
    assert_eq!(archived.status.code(), Some(0));
    assert!(stdout(&archived).trim().is_empty());
    let deprecated = run(&project, &["search", "deprecated-marker-xyz"]);
    assert_eq!(deprecated.status.code(), Some(0));
    assert!(stdout(&deprecated).trim().is_empty());
    assert_eq!(
        run(&project, &["note", "view", "NOTE-900"]).status.code(),
        Some(3)
    );
    assert!(!run_ok(&project, &["note", "list"]).contains("NOTE-900"));

    let restore = run(&project, &["note", "restore", "NOTE-900"]);
    assert_eq!(
        restore.status.code(),
        Some(2),
        "no document restore command"
    );

    fs::rename(
        project.join("docs/notes/deprecated/NOTE-900.md"),
        project.join("docs/notes/NOTE-900.md"),
    )
    .unwrap();
    let recovered = fs::read_to_string(project.join("docs/notes/NOTE-900.md")).unwrap();
    fs::write(
        project.join("docs/notes/NOTE-900.md"),
        recovered.replace("status: deprecated", "status: active"),
    )
    .unwrap();
    let list = run_ok(&project, &["note", "list"]);
    assert!(list.contains("NOTE-900"));
    assert!(run_ok(&project, &["note", "view", "NOTE-900"]).contains("Deprecated note"));

    set_note_enabled(&project, false);
    assert_eq!(run(&project, &["note", "list"]).status.code(), Some(2));
    let hidden_search = run(&project, &["search", "Wallet note", "--json"]);
    assert_eq!(hidden_search.status.code(), Some(0));
    assert!(stdout(&hidden_search).trim().is_empty() || stdout(&hidden_search).trim() == "[]");
}

// C-24 .. C-26 --------------------------------------------------------------
#[test]
fn t117_05_invalid_configuration_fails_closed_and_repairs() {
    let scratch = Scratch::new("config");
    let project = init(&scratch, "proj");
    add_note(&project);

    let path = config_path(&project);
    let original = fs::read_to_string(&path).unwrap();
    let broken = original.replace(
        "statuses: [active, archived, review, approved]",
        "statuses: [active, review, approved]",
    );
    assert_ne!(broken, original);
    fs::write(&path, broken).unwrap();

    let out = run(&project, &["note", "list"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("document_types.note.statuses"));
    let help = run(&project, &["--help"]);
    assert_eq!(help.status.code(), Some(4));
    assert!(stderr(&help).contains("document_types.note.statuses"));

    let get = run(&project, &["config", "get", "document_types.note.statuses"]);
    assert_eq!(get.status.code(), Some(0));
    assert_eq!(stdout(&get).trim(), "[active, review, approved]");
    let list = run(&project, &["config", "list"]);
    assert_eq!(list.status.code(), Some(0));
    assert!(stdout(&list).contains("document_types.note.statuses"));

    let before = hash(&path);
    let invalid = run(
        &project,
        &[
            "config",
            "set",
            "document_types.note.statuses",
            "active, review",
        ],
    );
    assert_eq!(invalid.status.code(), Some(4));
    assert_eq!(hash(&path), before, "invalid merge must not write");

    let repair = run(
        &project,
        &[
            "config",
            "set",
            "document_types.note.statuses",
            "active, archived, review, approved",
        ],
    );
    assert_eq!(repair.status.code(), Some(0));
    assert_eq!(run(&project, &["note", "list"]).status.code(), Some(0));

    // G-1: `config set` cannot bootstrap a type; the error names the missing
    // field and the file is byte-identical.
    let before = hash(&path);
    let bootstrap = run(
        &project,
        &["config", "set", "document_types.newtype.prefix", "NEW"],
    );
    assert_eq!(bootstrap.status.code(), Some(4));
    assert!(stderr(&bootstrap).contains("document_types.newtype.directory"));
    assert_eq!(hash(&path), before);
    let mapping = run(
        &project,
        &["config", "set", "document_types.note", "{prefix: NOTE}"],
    );
    assert_eq!(mapping.status.code(), Some(4));
    assert!(stderr(&mapping).contains("scalar"));
    assert_eq!(hash(&path), before);

    // G-4: `config list` omits `default_status`, but `config get` and
    // `config set` inspect and repair it.
    let broken_default = fs::read_to_string(&path)
        .unwrap()
        .replace("default_status: active", "default_status: archived");
    fs::write(&path, broken_default).unwrap();
    assert_eq!(run(&project, &["note", "list"]).status.code(), Some(4));
    assert_eq!(
        run_ok(
            &project,
            &["config", "get", "document_types.note.default_status"]
        ),
        "archived"
    );
    assert!(
        run_ok(&project, &["config", "list"])
            .contains("document_types.note.default_status: archived"),
        "G-4 resolved: the rejected default_status is printed by `config list`"
    );
    let repair_default = run(
        &project,
        &[
            "config",
            "set",
            "document_types.note.default_status",
            "active",
        ],
    );
    assert_eq!(repair_default.status.code(), Some(0));
    assert_eq!(run(&project, &["note", "list"]).status.code(), Some(0));

    run_ok(&project, &["init", "Proj", "--force", "--no-git"]);
    let reset = fs::read_to_string(&path).unwrap();
    assert!(!reset.contains("  note:"));
    assert_eq!(run(&project, &["req", "list"]).status.code(), Some(0));
    assert_eq!(run(&project, &["note", "list"]).status.code(), Some(2));
}

// Upgrade scenario (spec section 5) -----------------------------------------
#[test]
fn t117_06_upgrade_from_pre_change_configuration() {
    let scratch = Scratch::new("upgrade");
    let project = init(&scratch, "proj");

    let pre_change = "\
project_name: Proj
task_prefix: TASK
milestone_prefix: MS
release_prefix: REL
id_padding: 3
date_format: yyyy-mm-dd
editor: null
backlog_directory: backlog/
decision_directory: decisions/
doc_directory: docs/
statuses: [ToDo, Initiated, Refined]
default_status: ToDo
decision_types:
  adr:
    prefix: ADR
    enabled: true
document_types:
  req:
    prefix: REQ
    directory: requirements/business/
    enabled: true
  deploy:
    prefix: DEP
    directory: deployments/
    enabled: true
browser_host: 127.0.0.1
browser_port: 6420
browser_open: true
";
    fs::write(config_path(&project), pre_change).unwrap();
    write_file(
        &project.join("docs/requirements/business/REQ-001.md"),
        &note_record("REQ-001", "Active requirement", "active", "active body"),
    );
    write_file(
        &project.join("docs/requirements/business/deprecated/REQ-900.md"),
        &note_record(
            "REQ-900",
            "Deprecated requirement",
            "deprecated",
            "deprecated-marker-zzz",
        ),
    );
    write_file(
        &project.join("docs/deployments/DEP-001.md"),
        &note_record("DEP-001", "Release 1", "active", "deployment body"),
    );

    let help = run_ok(&project, &["--help"]);
    assert!(help.contains("  req "));
    assert!(help.contains("  deploy "));
    assert!(!help.contains("nfreq"));
    assert_eq!(run(&project, &["nfreq", "--help"]).status.code(), Some(2));
    assert_eq!(
        run(&project, &["req", "deprecate", "REQ-001"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        run(&project, &["req", "restore", "REQ-001"]).status.code(),
        Some(2)
    );

    let req_list = run_ok(&project, &["req", "list"]);
    assert!(req_list.contains("REQ-001"));
    assert!(!req_list.contains("REQ-900"));
    let search = run(&project, &["search", "deprecated-marker-zzz"]);
    assert_eq!(search.status.code(), Some(0));
    assert!(stdout(&search).trim().is_empty());
    assert!(run_ok(&project, &["deploy", "list"]).contains("DEP-001"));

    // Documented recovery: move into the active directory, set status.
    let deprecated_path = project.join("docs/requirements/business/deprecated/REQ-900.md");
    let active_path = project.join("docs/requirements/business/REQ-900.md");
    fs::rename(&deprecated_path, &active_path).unwrap();
    let recovered = fs::read_to_string(&active_path).unwrap();
    fs::write(
        &active_path,
        recovered.replace("status: deprecated", "status: active"),
    )
    .unwrap();
    let recovered_list = run_ok(&project, &["req", "list"]);
    assert!(recovered_list.contains("REQ-900"));
    assert!(run_ok(&project, &["req", "view", "REQ-900"]).contains("Deprecated requirement"));

    // Break and repair the configuration as the upgrade notes describe.
    let broken = fs::read_to_string(config_path(&project)).unwrap().replace(
        "    directory: requirements/business/\n    enabled: true",
        "    directory: requirements/business/\n    enabled: true\n    statuses: [active, review]",
    );
    fs::write(config_path(&project), broken).unwrap();
    let out = run(&project, &["req", "list"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("document_types.req.statuses"));
    assert_eq!(
        run_ok(&project, &["config", "get", "document_types.req.statuses"]),
        "[active, review]"
    );
    assert_eq!(
        run(
            &project,
            &[
                "config",
                "set",
                "document_types.req.statuses",
                "active, archived, review",
            ],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run(&project, &["req", "list"]).status.code(), Some(0));

    // Removing the pre-change deploy entry makes the command disappear with
    // no static fallback; re-declaring it by editing configuration restores
    // it. The fresh project below asserts the DEP-001 allocation.
    let without_deploy = fs::read_to_string(config_path(&project)).unwrap().replace(
        "  deploy:\n    prefix: DEP\n    directory: deployments/\n    enabled: true\n",
        "",
    );
    fs::write(config_path(&project), without_deploy).unwrap();
    assert_eq!(run(&project, &["deploy", "--help"]).status.code(), Some(2));
}

#[test]
fn t117_07_deploy_command_from_configuration_alone() {
    let scratch = Scratch::new("deploy");
    let project = init(&scratch, "proj");

    assert_eq!(run(&project, &["deploy", "--help"]).status.code(), Some(2));
    assert!(!run_ok(&project, &["--help"]).contains("  deploy "));

    let path = config_path(&project);
    let cfg = fs::read_to_string(&path).unwrap();
    let declared = cfg.replacen(
        "document_types:\n",
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deployments/\n    enabled: true\n",
        1,
    );
    fs::write(&path, declared).unwrap();

    assert!(run_ok(&project, &["--help"]).contains("  deploy "));
    let id = run_ok(&project, &["deploy", "create", "Release 1"]);
    assert_eq!(id, "DEP-001");
    assert!(run_ok(&project, &["deploy", "list"]).contains("DEP-001"));
    assert!(project.join("docs/deployments/DEP-001.md").exists());
}

// Browser matrix B-01 .. B-18 ------------------------------------------------
struct Server {
    child: Child,
    port: u16,
    startup: String,
    _stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start(project: &Path) -> Server {
        for _ in 0..10 {
            let port = free_port();
            let mut child = bin()
                .arg("--dir")
                .arg(project)
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .arg("--no-open")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if !startup.trim().is_empty() {
                return Server {
                    child,
                    port,
                    startup: startup.trim().to_string(),
                    _stdout: reader,
                };
            }
            let _ = child.wait();
        }
        panic!("server did not start");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
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
        .expect("header terminator");
    let head = String::from_utf8_lossy(&bytes[..split]).to_string();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    HttpResponse {
        status,
        headers,
        body: bytes[split + 4..].to_vec(),
    }
}

fn request(port: u16, method: &str, path: &str, body: Option<&str>) -> HttpResponse {
    let mut raw = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        method, path, port
    );
    if let Some(body) = body {
        raw.push_str("Content-Type: application/x-www-form-urlencoded\r\n");
        raw.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    raw.push_str("\r\n");
    if let Some(body) = body {
        raw.push_str(body);
    }
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    parse_response(&bytes)
}

fn get(server: &Server, path: &str) -> HttpResponse {
    request(server.port, "GET", path, None)
}

fn post(server: &Server, path: &str, body: &str) -> HttpResponse {
    request(server.port, "POST", path, Some(body))
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

fn form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", urlencode(key), urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn version_of(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').unwrap();
    rest[..end].to_string()
}

fn enabled_option_values(filter: &str) -> Vec<String> {
    filter
        .split("<option ")
        .skip(1)
        .filter_map(|chunk| {
            let tag = &chunk[..chunk.find('>').expect("option tag")];
            if tag.contains("disabled") {
                return None;
            }
            let (_, rest) = tag.split_once("value=\"")?;
            Some(rest[..rest.find('"').expect("option value")].to_string())
        })
        .collect()
}

fn filter_form(html: &str) -> &str {
    let start = html.find("class=\"filter\"").expect("filter form");
    let end = start + html[start..].find("</form>").expect("filter end");
    &html[start..end]
}

#[test]
fn t117_08_browser_matrix_over_http() {
    let scratch = Scratch::new("browser");
    let project = init(&scratch, "proj");
    add_note(&project);
    run_ok(&project, &["note", "create", "Alpha", "-d", "alpha body"]);
    run_ok(&project, &["note", "create", "Beta", "-d", "beta body"]);
    write_file(
        &project.join("docs/notes/NOTE-003.md"),
        "---\nid: NOTE-003\ntitle: \"Gamma\"\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels:\n  - keep\nauthor_note: preserve-me\nstatus: active\n---\n# NOTE-003 Gamma\n\n## Description\ngamma body\n\n## Custom section\nunknown body\n",
    );
    run_ok(&project, &["note", "status", "NOTE-001", "review"]);
    run_ok(&project, &["note", "status", "NOTE-002", "approved"]);
    write_file(
        &project.join("docs/notes/deprecated/NOTE-900.md"),
        &note_record("NOTE-900", "Deprecated", "deprecated", "deprecated body"),
    );

    let server = Server::start(&project);
    // B-01
    assert_eq!(
        server.startup,
        format!("Serving http://127.0.0.1:{}/", server.port)
    );

    // B-02
    let list = get(&server, "/?type=note");
    assert_eq!(list.status, 200);
    let html = list.text();
    let sidebar_start = html.find(">Documents</p>").expect("Documents group");
    let sidebar_end = html.find(">Configuration</a>").expect("utility");
    assert!(html[sidebar_start..sidebar_end].contains("Notes"));
    assert!(html.contains("data-list-layout=\"shared\""));
    let mut position = 0usize;
    for key in ["id", "title", "status", "created", "updated", "actions"] {
        let marker = format!("data-column=\"{key}\"");
        let found = html[position..]
            .find(&marker)
            .unwrap_or_else(|| panic!("shared column {key}"));
        position += found + marker.len();
    }
    assert!(html.contains("NOTE-001") && html.contains("NOTE-002"));

    // B-03 / B-04
    let detail = get(&server, "/doc/NOTE-001");
    assert_eq!(detail.status, 200);
    assert!(detail.text().contains("review"));
    assert!(!detail.text().contains("name=\"status\""));
    let edit = get(&server, "/doc/NOTE-001/edit");
    assert_eq!(edit.status, 200);
    assert!(!edit.text().contains("name=\"status\""));
    assert!(!edit.text().contains("name=\"status_change\""));
    assert!(edit.text().contains("name=\"labels\""));
    assert!(edit.text().contains("name=\"references\""));

    // B-05: edit round trip preserves unknown section and key.
    let edit_three = get(&server, "/doc/NOTE-003/edit");
    let version = version_of(&edit_three.text());
    let response = post(
        &server,
        "/doc/NOTE-003",
        &form(&[
            ("version", &version),
            ("title", "Gamma Edited"),
            ("description", "gamma body"),
            ("labels", "keep"),
            ("references", ""),
        ]),
    );
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/NOTE-003"));
    let edited = fs::read_to_string(project.join("docs/notes/NOTE-003.md")).unwrap();
    assert!(edited.contains("title: \"Gamma Edited\""));
    assert!(edited.contains("author_note: preserve-me"));
    assert!(edited.contains("## Custom section"));
    assert!(run_ok(&project, &["note", "view", "NOTE-003"]).contains("Gamma Edited"));

    // B-06: identical no-op save is byte-identical and does not refresh.
    let current = get(&server, "/doc/NOTE-003/edit");
    let version = version_of(&current.text());
    let before = hash(&project.join("docs/notes/NOTE-003.md"));
    let noop = post(
        &server,
        "/doc/NOTE-003",
        &form(&[
            ("version", &version),
            ("title", "Gamma Edited"),
            ("description", "gamma body"),
            ("labels", "keep"),
            ("references", ""),
        ]),
    );
    assert_eq!(noop.status, 303);
    assert_eq!(hash(&project.join("docs/notes/NOTE-003.md")), before);

    // B-07 stale version, B-08 archived status rejected.
    let stale = post(
        &server,
        "/doc/NOTE-003",
        &form(&[("version", "1.0.0"), ("title", "Stale")]),
    );
    assert_eq!(stale.status, 409);
    assert_eq!(hash(&project.join("docs/notes/NOTE-003.md")), before);
    let current = get(&server, "/doc/NOTE-003/edit");
    let version = version_of(&current.text());
    let archived = post(
        &server,
        "/doc/NOTE-003",
        &form(&[
            ("version", &version),
            ("status", "archived"),
            ("title", "Gamma Edited"),
        ]),
    );
    assert_eq!(archived.status, 422);
    assert_eq!(hash(&project.join("docs/notes/NOTE-003.md")), before);

    // B-09 / B-10 / B-11 / B-12
    let filter = filter_form(&html);
    assert_eq!(
        enabled_option_values(filter),
        vec!["active", "review", "approved"]
    );
    assert!(!filter.contains("value=\"archived\""));
    assert_eq!(get(&server, "/?type=note&status=review").status, 200);
    let review = get(&server, "/?type=note&status=review").text();
    assert!(review.contains("NOTE-001") && !review.contains("NOTE-002"));
    let approved = get(&server, "/?type=note&status=approved").text();
    assert!(approved.contains("NOTE-002") && !approved.contains("NOTE-001"));
    for query in [
        "?type=note&status=archived",
        "?type=note&status=bogus",
        "?type=note&status=Review",
        "?type=note&status=",
        "?type=note&status=review&status=approved",
    ] {
        let response = get(&server, &format!("/{query}"));
        assert_eq!(response.status, 400, "{query}");
        let body = response.text();
        assert!(body.contains("Bad request"));
        assert!(!body.contains("NOTE-001"), "{query} must not list records");
    }
    let unfiltered = get(&server, "/?type=note").text();
    assert!(unfiltered.contains("NOTE-001") && unfiltered.contains("NOTE-002"));

    // B-13: archived and unknown are the same 404; deprecated is inert.
    run_ok(&project, &["note", "archive", "NOTE-002"]);
    let archived_detail = get(&server, "/doc/NOTE-002");
    let unknown_detail = get(&server, "/doc/NOTE-999");
    assert_eq!(archived_detail.status, 404);
    assert_eq!(unknown_detail.status, 404);
    assert_eq!(
        archived_detail.text().replace("NOTE-002", "NOTE-XXX"),
        unknown_detail.text().replace("NOTE-999", "NOTE-XXX")
    );
    for path in ["/doc/NOTE-002", "/doc/NOTE-002/edit", "/doc/NOTE-900"] {
        assert_eq!(get(&server, path).status, 404, "{path}");
    }
    assert_eq!(
        post(&server, "/doc/NOTE-002", &form(&[("version", "1.0.0")])).status,
        404
    );
    assert!(!get(&server, "/?type=note").text().contains("NOTE-002"));
    assert!(!get(&server, "/api/docs/note").text().contains("NOTE-002"));
    assert!(!get(&server, "/?type=note").text().contains("NOTE-900"));

    // B-15 / B-16 / B-17 / B-18
    let lifecycle = get(&server, "/lifecycle").text();
    assert!(lifecycle.contains("Backlog"));
    assert!(lifecycle.contains("Decisions"));
    assert!(lifecycle.contains("Documents"));
    assert!(lifecycle.contains("Notes"));
    assert_eq!(lifecycle.matches("enabled:").count(), 17);

    let api = get(&server, "/api/docs/note");
    assert_eq!(api.status, 200);
    let rows = parse_json(&api.text());
    let row = &rows[0];
    for field in [
        "id",
        "kind",
        "title",
        "status",
        "labels",
        "references",
        "version",
        "path",
        "metadata",
    ] {
        assert!(row.get(field).is_some(), "API row missing {field}");
    }
    assert_eq!(row["kind"], "note");

    for (path, content_type) in [
        ("/assets/app.css", "text/css"),
        ("/assets/app.js", "javascript"),
    ] {
        let asset = get(&server, path);
        assert_eq!(asset.status, 200, "{path}");
        assert!(!asset.body.is_empty());
        assert!(
            asset
                .header("Content-Type")
                .unwrap_or("")
                .contains(content_type),
            "{path} content type"
        );
        assert_eq!(asset.header("Cache-Control"), Some("no-store"));
    }
    let page = get(&server, "/?type=note");
    assert!(
        page.header("Content-Security-Policy")
            .unwrap_or("")
            .contains("default-src 'none'")
    );
    let body = page.text();
    assert!(
        !body.contains("http://") && !body.contains("https://"),
        "served HTML must not reference external origins"
    );

    // B-14: disabling the type requires a browser restart (startup snapshot).
    drop(server);
    set_note_enabled(&project, false);
    let server = Server::start(&project);
    assert_eq!(get(&server, "/?type=note").status, 404);
    assert_eq!(get(&server, "/doc/NOTE-001").status, 404);
    assert_eq!(get(&server, "/doc/NOTE-001/edit").status, 404);
    assert_eq!(get(&server, "/api/docs/note").status, 404);
    assert!(!get(&server, "/").text().contains(">Notes<"));
    let lifecycle = get(&server, "/lifecycle").text();
    assert!(lifecycle.contains("Notes"));
    assert!(lifecycle.contains("enabled: false"));
}
