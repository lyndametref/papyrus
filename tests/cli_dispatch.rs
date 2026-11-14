use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(".tmp");
    p.push(format!("cli-dispatch-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn collision_guard_exits_4() {
    let d = tmpdir("collision");
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // inject colliding document type
    let cfg_path = d.join("papyrus.config.yml");
    let mut cfg = fs::read_to_string(&cfg_path).unwrap();
    cfg = cfg.replace(
        "document_types:\n",
        "document_types:\n  help:\n    enabled: true\n    prefix: H\n    directory: h/\n",
    );
    fs::write(&cfg_path, cfg).unwrap();

    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "help"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("document_types.help"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn help_lists_default_slugs() {
    let d = tmpdir("help");
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("req"));
    assert!(s.contains("nfr"));
    let _ = fs::remove_dir_all(&d);
}

use papyrus::cli;
use papyrus::config::{Config, DocTypeEntry};
use papyrus::registry::ArtifactRegistry;
use std::path::Path;

fn tmpdir2(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "papyrus-cli-dispatch-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn init_project(dir: &Path) {
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

#[test]
fn table_driven_builtin_help() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        ("init", &["init", "--help"], &["--no-git", "--force"]),
        (
            "config",
            &["config", "--help"],
            &["list", "get", "set", "unset"],
        ),
        (
            "task",
            &["task", "--help"],
            &[
                "create", "list", "view", "edit", "archive", "restore", "delete",
            ],
        ),
        (
            "milestone",
            &["milestone", "--help"],
            &["create", "list", "view", "edit", "archive", "delete"],
        ),
        (
            "release",
            &["release", "--help"],
            &["create", "list", "view", "edit", "close"],
        ),
        (
            "decision",
            &["decision", "--help"],
            &["create", "list", "view", "edit"],
        ),
        ("search", &["search", "--help"], &["--type", "--limit"]),
        (
            "browser",
            &["browser", "--help"],
            &["--host", "--port", "--no-open", "--debug"],
        ),
        ("import", &["import", "--help"], &["backlog"]),
        (
            "import backlog",
            &["import", "backlog", "--help"],
            &["--from", "--dry-run"],
        ),
        ("help", &["help"], &["Usage:"]),
        ("version", &["--version"], &["papyrus 0.1.0"]),
        ("top help", &["--help"], &["Usage:"]),
    ];

    for (name, args, needles) in cases {
        let out = bin().args(*args).output().unwrap();
        assert!(
            out.status.success(),
            "help failed for {}: {}",
            name,
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        for needle in *needles {
            assert!(
                stdout.contains(needle),
                "{} help missing {:?}:\n{}",
                name,
                needle,
                stdout
            );
        }
    }
}

#[test]
fn generated_help_lists_enabled_slugs_for_default_config() {
    let d = tmpdir2("genhelp");
    // empty directory => no config file => default config
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for slug in [
        "req",
        "nfr",
        "glossary",
        "rule",
        "ui-spec",
        "ui-mockup",
        "audit",
        "tech-spec",
        "use-case",
        "test-case",
        "test-report",
    ] {
        assert!(
            stdout.contains(slug),
            "default help missing slug {}: \n{}",
            slug,
            stdout
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn generated_help_omits_disabled_slug() {
    let d = tmpdir2("disabledhelp");
    init_project(&d);
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "set",
            "document_types.glossary.enabled",
            "false",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("glossary"),
        "disabled slug still listed:\n{}",
        stdout
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn table_driven_exit_two_paths() {
    let d = tmpdir2("exit2");
    init_project(&d);
    // disable one slug for the disabled-slug case
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "set",
            "document_types.glossary.enabled",
            "false",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let dp = d.to_str().unwrap();
    let cases: &[(&str, Vec<String>)] = &[
        (
            "unknown slug",
            vec!["--dir".into(), dp.into(), "definitely-not-a-slug".into()],
        ),
        (
            "disabled slug",
            vec!["--dir".into(), dp.into(), "glossary".into()],
        ),
        (
            "malformed option",
            vec!["--dir".into(), dp.into(), "--definitely-unknown".into()],
        ),
        (
            "unknown subcommand",
            vec![
                "--dir".into(),
                dp.into(),
                "config".into(),
                "definitely-not-a-subcommand".into(),
            ],
        ),
    ];

    for (name, args) in cases {
        let out = bin().args(args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "{} did not exit 2: {:?}\nstderr: {}",
            name,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn decision_archive_absent_and_core_verbs_present() {
    let d = tmpdir2("decisionarchive");
    init_project(&d);
    let dp = d.to_str().unwrap();

    let out = bin()
        .args(["--dir", dp, "decision", "archive", "ADR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "decision archive should exit 2");

    let out = bin()
        .args(["--dir", dp, "decision", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for verb in ["create", "list", "view", "edit"] {
        assert!(
            stdout.contains(verb),
            "decision help missing {}: \n{}",
            verb,
            stdout
        );
    }
    assert!(
        !stdout.contains("archive"),
        "decision help should not list archive:\n{}",
        stdout
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn decision_create_list_view_edit_work() {
    let d = tmpdir2("decisioncore");
    init_project(&d);
    let dp = d.to_str().unwrap();

    let out = bin()
        .args(["--dir", dp, "decision", "create", "Test decision"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args(["--dir", dp, "decision", "list"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args(["--dir", dp, "decision", "view", "ADR-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = bin()
        .args([
            "--dir", dp, "decision", "edit", "ADR-001", "--title", "Renamed",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn document_invocation_parsing_table() {
    use papyrus::cli::parse_document_invocation;
    use papyrus::document::DocumentVerb;

    #[derive(Debug)]
    struct Expected {
        verb: DocumentVerb,
        id: Option<&'static str>,
        title: Option<&'static str>,
        description: Option<&'static str>,
        labels: &'static [&'static str],
        references: &'static [&'static str],
        editor: bool,
        status_value: Option<&'static str>,
        force: bool,
    }

    let mut cfg = Config::default();
    cfg.document_types.insert(
        "note".to_string(),
        DocTypeEntry {
            prefix: "NOTE".to_string(),
            directory: "notes/".to_string(),
            enabled: true,
            statuses: None,
            default_status: None,
        },
    );
    let reg = ArtifactRegistry::build(&cfg).unwrap();
    let cmd = cli::tree(Some(&reg)).unwrap();

    let cases: &[(&str, &[&str], Expected)] = &[
        (
            "create",
            &[
                "note",
                "create",
                "Title",
                "--description",
                "desc",
                "--label",
                "l1",
                "--label",
                "l2",
                "--reference",
                "r1",
            ],
            Expected {
                verb: DocumentVerb::Create,
                id: None,
                title: Some("Title"),
                description: Some("desc"),
                labels: &["l1", "l2"],
                references: &["r1"],
                editor: false,
                status_value: None,
                force: false,
            },
        ),
        (
            "list",
            &["note", "list"],
            Expected {
                verb: DocumentVerb::List,
                id: None,
                title: None,
                description: None,
                labels: &[],
                references: &[],
                editor: false,
                status_value: None,
                force: false,
            },
        ),
        (
            "view",
            &["note", "view", "NOTE-001"],
            Expected {
                verb: DocumentVerb::View,
                id: Some("NOTE-001"),
                title: None,
                description: None,
                labels: &[],
                references: &[],
                editor: false,
                status_value: None,
                force: false,
            },
        ),
        (
            "edit",
            &[
                "note",
                "edit",
                "NOTE-001",
                "--title",
                "New",
                "-d",
                "dd",
                "--editor",
                "--label",
                "x",
                "--reference",
                "y",
            ],
            Expected {
                verb: DocumentVerb::Edit,
                id: Some("NOTE-001"),
                title: Some("New"),
                description: Some("dd"),
                labels: &["x"],
                references: &["y"],
                editor: true,
                status_value: None,
                force: false,
            },
        ),
        (
            "archive",
            &["note", "archive", "NOTE-001"],
            Expected {
                verb: DocumentVerb::Archive,
                id: Some("NOTE-001"),
                title: None,
                description: None,
                labels: &[],
                references: &[],
                editor: false,
                status_value: None,
                force: false,
            },
        ),
        (
            "delete",
            &["note", "delete", "NOTE-001", "--force"],
            Expected {
                verb: DocumentVerb::Delete,
                id: Some("NOTE-001"),
                title: None,
                description: None,
                labels: &[],
                references: &[],
                editor: false,
                status_value: None,
                force: true,
            },
        ),
        (
            "status",
            &["note", "status", "NOTE-001", "archived"],
            Expected {
                verb: DocumentVerb::Status,
                id: Some("NOTE-001"),
                title: None,
                description: None,
                labels: &[],
                references: &[],
                editor: false,
                status_value: Some("archived"),
                force: false,
            },
        ),
    ];

    for (name, args, expected) in cases {
        let full: Vec<&str> = std::iter::once("papyrus")
            .chain(args.iter().copied())
            .collect();
        let matches = cmd
            .clone()
            .try_get_matches_from(full)
            .unwrap_or_else(|e| panic!("parse failed for {}: {}", name, e));
        let (slug, sub) = matches.subcommand().expect("missing slug subcommand");
        assert_eq!(slug, "note", "unexpected slug for {}", name);
        let inv = parse_document_invocation(sub, slug, true)
            .unwrap_or_else(|e| panic!("parse failed for {}: {}", name, e));
        assert_eq!(inv.verb, expected.verb, "verb mismatch for {}", name);
        assert_eq!(inv.id.as_deref(), expected.id, "id mismatch for {}", name);
        assert_eq!(
            inv.title.as_deref(),
            expected.title,
            "title mismatch for {}",
            name
        );
        assert_eq!(
            inv.description.as_deref(),
            expected.description,
            "description mismatch for {}",
            name
        );
        assert_eq!(inv.labels, expected.labels, "labels mismatch for {}", name);
        assert_eq!(
            inv.references, expected.references,
            "references mismatch for {}",
            name
        );
        assert_eq!(inv.editor, expected.editor, "editor mismatch for {}", name);
        assert_eq!(
            inv.status_value.as_deref(),
            expected.status_value,
            "status_value mismatch for {}",
            name
        );
        assert_eq!(inv.force, expected.force, "force mismatch for {}", name);
        assert_eq!(inv.args.get("id"), expected.id, "args id for {}", name);
        assert_eq!(
            inv.args.get_all("label"),
            expected.labels,
            "args labels for {}",
            name
        );
        assert_eq!(
            inv.args.get_all("reference"),
            expected.references,
            "args references for {}",
            name
        );
        assert_eq!(
            inv.args.flag("editor"),
            expected.editor,
            "args editor for {}",
            name
        );
        assert_eq!(inv.slug, "note");
    }
}
