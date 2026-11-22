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
    // Backlog, decision, and project command surfaces (TASK-240, MS-032 AC-5).
    let project_verbs: [(&str, &[&str]); 4] = [
        (
            "task",
            &[
                "create", "list", "view", "edit", "archive", "restore", "delete",
            ],
        ),
        (
            "milestone",
            &[
                "create", "list", "view", "edit", "close", "archive", "delete",
            ],
        ),
        ("release", &["create", "list", "view", "close", "edit"]),
        ("decision", &["create", "list", "view", "edit"]),
    ];
    for (slug, verbs) in project_verbs {
        for verb in verbs {
            out.push((
                format!("verb-{slug}-{verb}"),
                vec![slug.to_string(), (*verb).to_string(), "--help".to_string()],
            ));
        }
    }
    for command in [
        "config",
        "config-get",
        "config-set",
        "config-unset",
        "config-list",
        "help",
        "browser",
        "search",
        "init",
        "import",
    ] {
        let mut args: Vec<String> = command.split('-').map(|part| part.to_string()).collect();
        args.push("--help".to_string());
        out.push((command.to_string(), args));
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

fn project_dir(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("cli-help-golden-{}-{tag}", std::process::id()));
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
    let project = project_dir("snapshots");
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

fn run_case(project: &Path, args: &[String], cwd: &Path, envs: &[(&str, &str)]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_papyrus"));
    command
        .env_clear()
        .arg("--dir")
        .arg(project)
        .args(args)
        .current_dir(cwd);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("papyrus runs");
    render(&output.stdout, &output.stderr, output.status.code())
}

fn contains_bounded(text: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    text.match_indices(needle).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + needle.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn contains_timestamp(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.windows(20).any(|window| {
        window[4] == b'-'
            && window[7] == b'-'
            && window[10] == b'T'
            && window[13] == b':'
            && window[16] == b':'
            && window[..4].iter().all(u8::is_ascii_digit)
            && window[5..7].iter().all(u8::is_ascii_digit)
            && window[8..10].iter().all(u8::is_ascii_digit)
            && window[11..13].iter().all(u8::is_ascii_digit)
            && window[14..16].iter().all(u8::is_ascii_digit)
            && window[17..19].iter().all(u8::is_ascii_digit)
    })
}

fn contains_memory_address(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.windows(6).any(|window| {
        window[0] == b'0'
            && (window[1] == b'x' || window[1] == b'X')
            && window[2..].iter().all(u8::is_ascii_hexdigit)
    })
}

fn assert_no_volatile_tokens(name: &str, rendered: &str, project: &Path) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    assert!(
        !rendered.contains(manifest),
        "{name}: help leaks the manifest path"
    );
    let project_path = project.to_string_lossy();
    assert!(
        !rendered.contains(project_path.as_ref()),
        "{name}: help leaks the project directory"
    );
    let pid = std::process::id().to_string();
    // Short identifiers collide with version digits; a real PID is longer.
    if pid.len() >= 4 {
        assert!(
            !contains_bounded(rendered, &pid),
            "{name}: help leaks the process id {pid}"
        );
    }
    if let Ok(host) = fs::read_to_string("/proc/sys/kernel/hostname") {
        let host = host.trim();
        // The program name and the loopback alias are not host leaks.
        if !host.is_empty() && host != "localhost" && host != "papyrus" {
            assert!(
                !contains_bounded(rendered, host),
                "{name}: help leaks the host name {host}"
            );
        }
    }
    assert!(
        !contains_timestamp(rendered),
        "{name}: help leaks a timestamp"
    );
    assert!(
        !contains_memory_address(rendered),
        "{name}: help leaks a memory address"
    );
}

#[test]
fn help_output_is_deterministic_and_free_of_volatile_tokens() {
    let project = project_dir("determinism");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let home = project.to_string_lossy().into_owned();
    let first_env: [(&str, &str); 5] = [
        ("LC_ALL", "C"),
        ("LANG", "C"),
        ("COLUMNS", "80"),
        ("TZ", "UTC"),
        ("HOME", home.as_str()),
    ];
    let second_env: [(&str, &str); 5] = [
        ("LC_ALL", "C.UTF-8"),
        ("LANG", "C.UTF-8"),
        ("COLUMNS", "200"),
        ("TZ", "Pacific/Kiritimati"),
        ("HOME", home.as_str()),
    ];
    let corpus = cases();
    let selected = [
        "top",
        "verb-req-create",
        "verb-glossary-create",
        "verb-audit-create",
        "verb-ui-mockup-create",
        "verb-req-edit",
        "err-missing-required",
    ];
    for name in selected {
        let (_, args) = corpus
            .iter()
            .find(|(case, _)| case == name)
            .unwrap_or_else(|| panic!("case {name} exists"));
        let first = run_case(&project, args, &manifest, &first_env);
        let repeat = run_case(&project, args, &manifest, &first_env);
        assert_eq!(first, repeat, "{name}: two consecutive runs differ");
        let varied = run_case(&project, args, &project, &second_env);
        assert_eq!(
            first, varied,
            "{name}: help depends on cwd, locale, COLUMNS, or timezone"
        );
        assert_no_volatile_tokens(name, &first, &project);
    }
    let _ = fs::remove_dir_all(&project);
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
