//! Byte-for-byte golden snapshots for the document command surface.
//!
//! Captured before the TASK-119 refactor and asserted afterwards. Covers the
//! top-level help, every enabled typed slug, every verb (including
//! `create-group` and `supersede`), a config-defined generic slug, and the
//! exit-2 usage-error cases plus one `--json` application error.
//!
//! Regenerate with `PAPYRUS_UPDATE_CLI_HELP=1 cargo test --test cli_help_golden`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TYPED_SLUGS: [&str; 11] = [
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
];

const GENERIC_SLUG: &str = "note";

fn verbs(slug: &str) -> &'static [&'static str] {
    match slug {
        "req" | "nfr" => &[
            "create",
            "create-group",
            "list",
            "view",
            "edit",
            "archive",
            "delete",
            "status",
        ],
        "ui-mockup" => &[
            "create",
            "list",
            "view",
            "edit",
            "archive",
            "delete",
            "supersede",
        ],
        _ => &[
            "create", "list", "view", "edit", "archive", "delete", "status",
        ],
    }
}

fn cases() -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    out.push(("top".to_string(), vec!["--help".to_string()]));
    out.push(("version".to_string(), vec!["--version".to_string()]));
    out.push((
        "check".to_string(),
        vec!["check".to_string(), "--help".to_string()],
    ));
    for slug in TYPED_SLUGS.iter().chain(std::iter::once(&GENERIC_SLUG)) {
        out.push((
            format!("slug-{slug}"),
            vec![(*slug).to_string(), "--help".to_string()],
        ));
        for verb in verbs(slug) {
            out.push((
                format!("verb-{slug}-{verb}"),
                vec![
                    (*slug).to_string(),
                    (*verb).to_string(),
                    "--help".to_string(),
                ],
            ));
        }
    }
    for slug in ["task", "milestone", "decision"] {
        for verb in ["create", "edit"] {
            out.push((
                format!("verb-{slug}-{verb}"),
                vec![slug.to_string(), verb.to_string(), "--help".to_string()],
            ));
        }
    }
    let error_cases: [(&str, &[&str]); 10] = [
        ("err-unknown-slug", &["nosuch", "--help"]),
        ("err-disabled-slug", &["legacy", "--help"]),
        ("missing-verb-req", &["req"]),
        ("missing-verb-note", &["note"]),
        ("err-unknown-verb", &["req", "nosuch"]),
        ("err-missing-required", &["req", "create"]),
        ("err-malformed", &["req", "list", "--bogus"]),
        ("err-nfreq", &["nfreq"]),
        ("json-app-error", &["--json", "req", "view", "REQ-999"]),
        (
            "json-app-error-generic",
            &["--json", "note", "view", "NOTE-999"],
        ),
    ];
    for (name, args) in error_cases {
        out.push((
            name.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
    }
    out
}

fn fixture_config() -> String {
    fs::read_to_string("tests/fixtures/cli-help/papyrus.config.yml")
        .expect("fixture configuration is readable")
}

fn project_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("cli-help-golden-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp project directory");
    fs::write(dir.join("papyrus.config.yml"), fixture_config()).expect("fixture config written");
    dir
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/cli-help")
}

fn render(stdout: &[u8], stderr: &[u8], code: Option<i32>) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    format!(
        "{{\n  \"exit\": {},\n  \"stdout\": {},\n  \"stderr\": {}\n}}\n",
        code.unwrap_or(-1),
        serde_json::to_string(&stdout).expect("stdout serializes"),
        serde_json::to_string(&stderr).expect("stderr serializes"),
    )
}

#[test]
fn cli_help_matches_golden_snapshots() {
    let project = project_dir();
    let golden = golden_dir();
    if std::env::var_os("PAPYRUS_UPDATE_CLI_HELP").is_some() {
        fs::create_dir_all(&golden).expect("golden directory");
    }
    let mut failures = Vec::new();
    for (name, args) in cases() {
        let output = Command::new(env!("CARGO_BIN_EXE_papyrus"))
            .arg("--dir")
            .arg(&project)
            .args(&args)
            .output()
            .expect("papyrus runs");
        let actual = render(&output.stdout, &output.stderr, output.status.code());
        let path = golden.join(format!("{name}.json"));
        if std::env::var_os("PAPYRUS_UPDATE_CLI_HELP").is_some() {
            fs::write(&path, &actual).expect("snapshot written");
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(expected) => {
                if expected != actual {
                    failures.push(format!(
                        "{name}:\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
                    ));
                }
            }
            Err(_) => failures.push(format!("{name}: missing snapshot {}", path.display())),
        }
    }
    let _ = fs::remove_dir_all(&project);
    assert!(
        failures.is_empty(),
        "document command surface drifted from the pre-refactor golden snapshots:\n\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn golden_snapshot_set_is_complete() {
    let dir = golden_dir();
    if std::env::var_os("PAPYRUS_UPDATE_CLI_HELP").is_some() {
        return;
    }
    let mut missing = Vec::new();
    for (name, _) in cases() {
        let path = dir.join(format!("{name}.json"));
        if !path.exists() {
            missing.push(name);
        }
    }
    assert!(missing.is_empty(), "missing snapshots: {missing:?}");
    let count = fs::read_dir(&dir).expect("golden dir").count();
    assert_eq!(
        count,
        cases().len(),
        "golden directory holds stale snapshots; regenerate intentionally"
    );
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
