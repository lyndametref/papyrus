//! Mechanical no-drift checks for ADR-0013 decision 7 (TASK-97).
//!
//! Each check scans production `.rs` files under `src/` and fails the default
//! `cargo test` run. Every `#[cfg(test)]`-annotated item (`mod`, `fn`, `impl`,
//! macro block, or expression) is excluded by tracking brace depth from the
//! attribute, and scanning resumes after the item closes; production lines after
//! a mid-file test module are therefore scanned (decision D4).
//!
//! Allowlists are the minimum required by the feature specification:
//!
//! | # | Condition | Allowlist |
//! |---|---|---|
//! | 1 | `fs::write` on a record path or `fs::rename` outside the store | `fs::write`: `src/config.rs`, `src/init.rs`, `src/record/`; `fs::rename`: `src/record/`, the `write_config_file` block in `src/config.rs` |
//! | 2 | front-matter delimiter construction (`format!("---`) | `src/record/` |
//! | 3 | ID formatting and prefix-shaped ID parsing | `src/record/identity.rs` |
//! | 4 | storage-ID / translation symbols and offsets | none |
//! | 5 | hardcoded record-directory literals | `src/record/descriptor.rs`, `src/config.rs`, `src/init.rs`, `src/import.rs` |
//! | 6 | Backlog-shaped stem detection or storage prefix construction | `src/record/identity.rs`, `src/record/descriptor.rs` |
//! | 7 | a second record writer (`.tmp` + rename) | `src/record/store.rs`, the `write_config_file` block in `src/config.rs` |
//! | 8 | a second section parser or front-matter delimiter detection | `src/record/`, `src/import.rs` |
//!
//! Every violation reports `file:line: text`.
//!
//! MS-037 release gates owned here after TASK-283:
//!
//! | Guard | Invariant | Failure format |
//! |---|---|---|
//! | no-new-crate | `Cargo.toml` sections and `Cargo.lock` hold only the 4+1 dependency baseline and 36 packages | `Cargo.toml [dependencies] changed: added crate(s) [...]` |
//! | binary budget | `target/release/papyrus` ≤ 12,582,912 bytes (TECH-008 §2) | `target/release/papyrus is N bytes, above the 12582912-byte (12 MiB) budget` |

use std::fs;
use std::path::{Path, PathBuf};

fn src_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(p);
                }
            }
        }
    }
    walk(Path::new("src"), &mut out);
    out.sort();
    out
}

/// Split a source file into production lines, excluding every
/// `#[cfg(test)]`-annotated item and resuming after it closes (D4).
fn production_lines(path: &Path) -> Vec<(usize, String)> {
    let content = fs::read_to_string(path).unwrap_or_default();
    production_lines_in(&content)
}

fn production_lines_in(content: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim_start().starts_with("#[cfg(test)]") {
            index = skip_cfg_test_item(&lines, index + 1);
            continue;
        }
        out.push((index + 1, lines[index].to_string()));
        index += 1;
    }
    out
}

/// Return the index of the first line after the `#[cfg(test)]` item starting at
/// `index`. The item ends at its closing brace, or after a semicolon when the
/// annotated item is a statement.
fn skip_cfg_test_item(lines: &[&str], mut index: usize) -> usize {
    let mut depth = 0i32;
    let mut opened = false;
    while index < lines.len() {
        let line = lines[index];
        let delta = brace_delta(line);
        if delta > 0 {
            opened = true;
        }
        depth += delta;
        index += 1;
        let closed = if opened {
            depth <= 0
        } else {
            line.trim_end().ends_with(';')
        };
        if closed {
            break;
        }
    }
    index
}

/// Net brace balance on one line, ignoring braces inside comments, string
/// literals, raw strings, and character literals.
fn brace_delta(line: &str) -> i32 {
    let bytes = line.as_bytes();
    let mut delta = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => break,
            b'/' if bytes.get(i + 1) == Some(&b'*') => match line[i + 2..].find("*/") {
                Some(end) => i += 2 + end + 2,
                None => break,
            },
            b'r' if is_raw_string_start(bytes, i) => i = skip_raw_string(bytes, i),
            b'"' => i = skip_quoted(bytes, i, b'"'),
            b'\'' => i = skip_char(bytes, i),
            b'{' => {
                delta += 1;
                i += 1;
            }
            b'}' => {
                delta -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    delta
}

fn is_raw_string_start(bytes: &[u8], start: usize) -> bool {
    if bytes.get(start) != Some(&b'r') {
        return false;
    }
    let mut i = start + 1;
    while bytes.get(i) == Some(&b'#') {
        i += 1;
    }
    bytes.get(i) == Some(&b'"')
}

fn skip_raw_string(bytes: &[u8], start: usize) -> usize {
    let mut hashes = 0usize;
    let mut i = start + 1;
    while bytes.get(i) == Some(&b'#') {
        hashes += 1;
        i += 1;
    }
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let mut j = i + 1;
            let mut matched = 0usize;
            while matched < hashes && bytes.get(j) == Some(&b'#') {
                matched += 1;
                j += 1;
            }
            if matched == hashes {
                return j;
            }
        }
        i += 1;
    }
    bytes.len()
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_char(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start + 1) == Some(&b'\\') {
        if bytes.get(start + 2) == Some(&b'u') {
            let mut i = start + 3;
            while i < bytes.len() && bytes[i] != b'}' {
                i += 1;
            }
            return (i + 1).min(bytes.len());
        }
        return (start + 3).min(bytes.len());
    }
    if bytes.get(start + 2) == Some(&b'\'') {
        return start + 3;
    }
    // A lifetime such as `'a`, not a character literal.
    start + 1
}

fn relative(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn check(label: &str, allowed: &[&str], matches: impl Fn(&str) -> bool) -> Result<(), String> {
    let files: Vec<(String, String)> = src_files()
        .into_iter()
        .map(|path| {
            let content = fs::read_to_string(&path).unwrap_or_default();
            (relative(&path), content)
        })
        .collect();
    scan_files(label, allowed, &files, matches)
}

fn scan_files(
    label: &str,
    allowed: &[&str],
    files: &[(String, String)],
    matches: impl Fn(&str) -> bool,
) -> Result<(), String> {
    scan_files_scoped(label, allowed, files, matches)
}

/// Line range (1-based, inclusive) of the atomic configuration writer.
///
/// This is the only production block in `src/config.rs` allowed to create a
/// temporary file and rename it; a record write elsewhere in the module is
/// still reported.
fn config_writer_range(content: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.contains("fn write_config_file("))?;
    let mut depth = 0i32;
    let mut opened = false;
    for (index, line) in lines.iter().enumerate().skip(start) {
        let delta = brace_delta(line);
        if delta > 0 {
            opened = true;
        }
        depth += delta;
        if opened && depth <= 0 {
            return Some((start + 1, index + 1));
        }
    }
    None
}

/// Scan production lines, exempting a file prefix or a single declaration
/// block in `src/config.rs`.
fn scan_files_scoped(
    label: &str,
    allowed: &[&str],
    files: &[(String, String)],
    matches: impl Fn(&str) -> bool,
) -> Result<(), String> {
    let mut violations = Vec::new();
    for (rel, content) in files {
        if allowed.iter().any(|prefix| rel.starts_with(prefix)) {
            continue;
        }
        let block = if rel == "src/config.rs" {
            config_writer_range(content)
        } else {
            None
        };
        for (lineno, line) in production_lines_in(content) {
            let inside_block = block.is_some_and(|(start, end)| lineno >= start && lineno <= end);
            if matches(&line) && !inside_block {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("{}\n{}", label, violations.join("\n")))
    }
}

#[test]
fn no_record_writes_outside_the_store() {
    // `fs::write` is allowed only for non-record files in config/init.
    check(
        "fs::write outside config.rs/init.rs:",
        &["src/config.rs", "src/init.rs", "src/record/"],
        |line| line.contains("fs::write") || line.contains("std::fs::write"),
    )
    .unwrap();
    // A record file may only be renamed by the shared store. The configuration
    // writer owns its own atomic rename because a configuration file is not a
    // record (AUD-151 M3); the exemption is scoped to that one block.
    check(
        "fs::rename outside src/record/ or the configuration writer:",
        &["src/record/"],
        |line| line.contains("fs::rename"),
    )
    .unwrap();
}

#[test]
fn no_frontmatter_delimiter_construction_outside_record() {
    check(
        "format!(\"--- delimiter construction outside src/record/:",
        &["src/record/"],
        |line| line.contains("format!") && line.contains("\"---"),
    )
    .unwrap();
}

/// A `format!("{}-…")` call builds a prefix-shaped ID only when the dash is
/// immediately followed by a placeholder. A dash followed by a literal (for
/// example YAML quoting, `format!("{}- '{}'", …)`) is formatting text, not an
/// ID, and must not trip the heuristic.
fn builds_prefix_shaped_id(line: &str) -> bool {
    let Some(rest) = line.split("format!(\"{}-").nth(1) else {
        return false;
    };
    rest.starts_with("{}") || rest.starts_with("{:") || rest.starts_with('"')
}

#[test]
fn no_id_formatting_or_prefix_parsing_outside_identity() {
    let allowed = &["src/record/identity.rs"];
    check(
        "zero-padded ID formatting outside src/record/identity.rs:",
        allowed,
        |line| {
            line.contains("\"{}-{:0")
                || line.contains("\"{}.{:0")
                || line.contains("{:0width")
                || line.contains(":0>width")
        },
    )
    .unwrap();
    // Forming the `<prefix>-` shape by hand is prefix-shaped ID parsing.
    check(
        "prefix-shaped ID parsing outside src/record/identity.rs:",
        allowed,
        builds_prefix_shaped_id,
    )
    .unwrap();
}

#[test]
fn prefix_shaped_id_detection_ignores_literal_dashes() {
    assert!(builds_prefix_shaped_id(
        "format!(\"{}-{}\", prefix, number)"
    ));
    assert!(builds_prefix_shaped_id(
        "strip_prefix(&format!(\"{}-\", prefix))"
    ));
    assert!(!builds_prefix_shaped_id(
        "return format!(\"{}- '{}'\", indent, val.replace('\\'', \"''\"));"
    ));
    assert!(!builds_prefix_shaped_id("let x = format!(\"{}-text\");"));
}

#[test]
fn no_storage_id_or_offset_symbols() {
    check("storage-ID or storage-offset symbols:", &[], |line| {
        line.contains("to_storage_id")
            || line.contains("to_display_id")
            || line.contains("storage_prefix")
            || line.contains("storage_id")
            || line.contains("storage_offset")
            || line.contains("STORAGE_OFFSET")
            || line.contains("backlog/config.yml")
    })
    .unwrap();
}

#[test]
fn no_hardcoded_record_directories() {
    const DIRS: &[&str] = &[
        "backlog/tasks",
        "backlog/milestones",
        "backlog/releases",
        "archive/tasks",
        "archive/milestones",
        "tasks/archived",
        "milestones/archived",
        "decisions/archived",
        "archived/",
    ];
    check(
        "hardcoded record-directory literals:",
        &[
            "src/record/descriptor.rs",
            "src/config.rs",
            "src/init.rs",
            "src/import.rs",
        ],
        |line| DIRS.iter().any(|dir| line.contains(dir)),
    )
    .unwrap();
}

#[test]
fn no_reimplemented_backlog_stem_detection() {
    check(
        "Backlog-shaped stem detection or storage prefix construction:",
        &["src/record/identity.rs", "src/record/descriptor.rs"],
        |line| {
            line.contains("is_backlog_shaped")
                || line.contains("storage_prefix")
                || line.contains("backlog_shape")
        },
    )
    .unwrap();
}

#[test]
fn no_second_record_writer() {
    // The configuration writer uses a temporary file + rename, but it never
    // writes a record (AUD-151 M3); the exemption is scoped to that one block.
    check(
        "second record writer outside src/record/store.rs or the configuration writer:",
        &["src/record/store.rs"],
        |line| line.contains("fn write_atomic") || line.contains(".tmp"),
    )
    .unwrap();
}

#[test]
fn config_writer_exemption_is_scoped_to_the_atomic_writer_block() {
    let content = fs::read_to_string("src/config.rs").expect("config.rs");
    let (start, end) = config_writer_range(&content).expect("write_config_file block");
    assert!(
        content
            .lines()
            .nth(start - 1)
            .unwrap()
            .contains("fn write_config_file"),
        "the block starts at the writer declaration"
    );
    assert_eq!(
        content.lines().nth(end - 1).unwrap().trim(),
        "}",
        "the block ends at the function close"
    );

    // The real writer body is exempt.
    let writer = [("src/config.rs".to_string(), content.clone())];
    assert!(
        scan_files_scoped(
            "fs::rename in the configuration writer:",
            &["src/record/"],
            &writer,
            |line| line.contains("fs::rename"),
        )
        .is_ok(),
        "the atomic writer keeps its one rename"
    );

    // A record write anywhere else in the same module is reported.
    let planted = [(
        "src/config.rs".to_string(),
        format!(
            "{}\nfn record_write() {{ let _ = std::fs::rename(\"a\", \"b\"); }}\n",
            content
        ),
    )];
    let error = scan_files_scoped(
        "fs::rename outside the configuration writer:",
        &["src/record/"],
        &planted,
        |line| line.contains("fs::rename"),
    )
    .unwrap_err();
    assert!(
        error.contains("record_write"),
        "a record write in config.rs must be caught, got: {error}"
    );
}

#[test]
fn no_second_section_parser() {
    check(
        "second section parser or delimiter detection outside src/record/:",
        &["src/record/", "src/import.rs"],
        |line| {
            line.contains("split(\"## \"")
                || line.contains("split_once(\"## \"")
                || line.contains("strip_prefix(\"## \"")
                || line.contains("splitn(") && line.contains("\"## ")
                || line.contains("starts_with(\"---")
                || line.contains("trim_start_matches(\"---")
                || line.contains("== \"---")
        },
    )
    .unwrap();
}

// MS-037 release gates -------------------------------------------------------
//
// TECH-008 section 2 (12 MiB binary) and ADR-043's dependency baseline: the
// release must not grow a new crate or a new dependency edge, and the release
// binary must stay inside the recorded size budget.

/// Runtime dependencies recorded at MS-037 refinement time.
const BASELINE_DEPENDENCIES: [&str; 4] = ["clap", "serde", "serde_json", "yaml_serde"];
/// Dev dependencies recorded at MS-037 refinement time.
const BASELINE_DEV_DEPENDENCIES: [&str; 1] = ["serde_json"];
/// Build dependencies recorded at MS-037 refinement time.
const BASELINE_BUILD_DEPENDENCIES: [&str; 0] = [];
/// The complete `Cargo.lock` package set recorded at MS-037 refinement time.
const BASELINE_LOCK_PACKAGES: [&str; 36] = [
    "anstream",
    "anstyle",
    "anstyle-parse",
    "anstyle-query",
    "anstyle-wincon",
    "clap",
    "clap_builder",
    "clap_derive",
    "clap_lex",
    "colorchoice",
    "equivalent",
    "foldhash",
    "hashbrown",
    "heck",
    "indexmap",
    "is_terminal_polyfill",
    "itoa",
    "libyaml-rs",
    "memchr",
    "once_cell_polyfill",
    "papyrus",
    "proc-macro2",
    "quote",
    "ryu",
    "serde",
    "serde_core",
    "serde_derive",
    "serde_json",
    "strsim",
    "syn",
    "unicode-ident",
    "utf8parse",
    "windows-link",
    "windows-sys",
    "yaml_serde",
    "zmij",
];
/// TECH-008 section 2: the release binary budget, 12 MiB.
const RELEASE_BINARY_BUDGET_BYTES: u64 = 12 * 1024 * 1024;

/// Top-level dependency keys of one `[section]` in a `Cargo.toml`.
fn toml_section_keys(content: &str, section: &str) -> Vec<String> {
    let target = format!("[{section}]");
    let mut in_section = false;
    let mut keys = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == target;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=')
            && !key.trim().is_empty()
        {
            keys.push(key.trim().to_string());
        }
    }
    keys
}

/// `name = "…"` entries of a `Cargo.lock`.
fn lock_package_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("name = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

fn added<'a>(found: &'a [String], baseline: &[&str]) -> Vec<&'a str> {
    found
        .iter()
        .map(String::as_str)
        .filter(|name| !baseline.contains(name))
        .collect()
}

fn missing<'a>(found: &[String], baseline: &'a [&str]) -> Vec<&'a str> {
    baseline
        .iter()
        .copied()
        .filter(|name| !found.iter().any(|entry| entry == name))
        .collect()
}

/// Guard one `Cargo.toml` section against its recorded baseline.
fn check_dependency_section(
    section: &str,
    found: &[String],
    baseline: &[&str],
) -> Result<(), String> {
    let extra = added(found, baseline);
    let gone = missing(found, baseline);
    if extra.is_empty() && gone.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Cargo.toml [{section}] changed: added crate(s) {extra:?}, removed crate(s) {gone:?}"
        ))
    }
}

/// Guard the full lockfile package set against the recorded baseline.
fn check_lock_packages(found: &[String]) -> Result<(), String> {
    let extra = added(found, &BASELINE_LOCK_PACKAGES);
    let gone = missing(found, &BASELINE_LOCK_PACKAGES);
    if extra.is_empty() && gone.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Cargo.lock changed: added package(s) {extra:?}, removed package(s) {gone:?}"
        ))
    }
}

/// TECH-008 section 2: fail above 12,582,912 bytes.
fn check_binary_budget(size: u64) -> Result<u64, String> {
    if size > RELEASE_BINARY_BUDGET_BYTES {
        Err(format!(
            "target/release/papyrus is {size} bytes, above the {RELEASE_BINARY_BUDGET_BYTES}-byte (12 MiB) budget"
        ))
    } else {
        Ok(size)
    }
}

#[test]
fn no_new_crate_or_dependency_edge() {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    check_dependency_section(
        "dependencies",
        &toml_section_keys(&manifest, "dependencies"),
        &BASELINE_DEPENDENCIES,
    )
    .unwrap();
    check_dependency_section(
        "dev-dependencies",
        &toml_section_keys(&manifest, "dev-dependencies"),
        &BASELINE_DEV_DEPENDENCIES,
    )
    .unwrap();
    check_dependency_section(
        "build-dependencies",
        &toml_section_keys(&manifest, "build-dependencies"),
        &BASELINE_BUILD_DEPENDENCIES,
    )
    .unwrap();

    let lock = fs::read_to_string("Cargo.lock").expect("Cargo.lock");
    check_lock_packages(&lock_package_names(&lock)).unwrap();
}

#[test]
fn no_new_crate_guard_detects_each_planted_addition() {
    let manifest = "[dependencies]\nclap = \"4\"\nserde = \"1\"\nserde_json = \"1\"\nyaml_serde = \"0.10\"\nnewcrate = \"1\"\n";
    let error = check_dependency_section(
        "dependencies",
        &toml_section_keys(manifest, "dependencies"),
        &BASELINE_DEPENDENCIES,
    )
    .unwrap_err();
    assert!(
        error.contains("newcrate"),
        "the added crate must be named: {error}"
    );

    let dev = "[dev-dependencies]\nserde_json = \"1\"\ntokio = \"1\"\n";
    let error = check_dependency_section(
        "dev-dependencies",
        &toml_section_keys(dev, "dev-dependencies"),
        &BASELINE_DEV_DEPENDENCIES,
    )
    .unwrap_err();
    assert!(
        error.contains("tokio"),
        "the added dev crate must be named: {error}"
    );

    let build = "[build-dependencies]\ncc = \"1\"\n";
    let error = check_dependency_section(
        "build-dependencies",
        &toml_section_keys(build, "build-dependencies"),
        &BASELINE_BUILD_DEPENDENCIES,
    )
    .unwrap_err();
    assert!(
        error.contains("cc"),
        "the added build crate must be named: {error}"
    );

    let lock = check_lock_packages(&[
        "papyrus".to_string(),
        "serde".to_string(),
        "planted-crate".to_string(),
    ])
    .unwrap_err();
    assert!(
        lock.contains("planted-crate"),
        "the added lock package must be named: {lock}"
    );
}

#[test]
fn release_binary_stays_within_the_twelve_mib_budget() {
    let path = Path::new("target/release/papyrus");
    match fs::metadata(path) {
        Ok(metadata) => {
            let size =
                check_binary_budget(metadata.len()).unwrap_or_else(|error| panic!("{error}"));
            eprintln!(
                "target/release/papyrus size: {size} bytes (budget {RELEASE_BINARY_BUDGET_BYTES})"
            );
        }
        Err(_) => eprintln!(
            "release binary guard skipped: target/release/papyrus is absent (run `cargo build --release`)"
        ),
    }
}

#[test]
fn binary_budget_guard_rejects_an_oversized_binary() {
    assert_eq!(
        check_binary_budget(RELEASE_BINARY_BUDGET_BYTES).unwrap(),
        12_582_912
    );
    let error = check_binary_budget(RELEASE_BINARY_BUDGET_BYTES + 1).unwrap_err();
    assert!(error.contains("12 MiB"), "{error}");
    assert!(error.contains("12582913"), "{error}");
}

// TASK-97 D4: a violation after a mid-file test module is reported, and nested
// `#[cfg(test)]` items inside the module do not resume scanning early.
#[test]
fn scanner_reports_violations_after_a_mid_file_test_module() {
    let fixture = "\
fn before() {}\n\
\n\
#[cfg(test)]\n\
mod tests {\n\
    #[cfg(test)]\n\
    mod nested {\n\
        fn inner() { let _ = \"{\"; }\n\
    }\n\
    #[test]\n\
    fn helper() { let _ = r#\"{ unbalanced\"#; }\n\
}\n\
\n\
fn after() {\n\
    let _ = fs::write(\"record.md\", \"x\");\n\
}\n\
";
    let production = production_lines_in(fixture);
    assert!(
        production
            .iter()
            .any(|(_, line)| line.contains("fn before")),
        "production before the test module is scanned"
    );
    assert!(
        !production
            .iter()
            .any(|(_, line)| line.contains("fn helper")),
        "the test module must be excluded"
    );
    assert!(
        production.iter().any(|(_, line)| line.contains("fn after")),
        "scanning must resume after the test module"
    );
    let files = [("fixture.rs".to_string(), fixture.to_string())];
    let error = scan_files("fixture violation:", &[], &files, |line| {
        line.contains("fs::write")
    })
    .unwrap_err();
    assert!(
        error.contains("fixture.rs:14:"),
        "violation after the test module must report file:line, got: {error}"
    );
    assert!(error.contains("fs::write"), "got: {error}");
}

// TASK-97 D4: the src/browser/render.rs production tail after its test module is
// scanned, not truncated at the `#[cfg(test)]` attribute.
#[test]
fn scanner_covers_production_after_the_render_test_module() {
    let source = fs::read_to_string("src/browser/render.rs").expect("render.rs is readable");
    let test_module_line = source
        .lines()
        .position(|line| line.trim_start().starts_with("#[cfg(test)]"))
        .map(|index| index + 1)
        .expect("render.rs has a test module");
    let production = production_lines(Path::new("src/browser/render.rs"));
    assert!(
        production
            .iter()
            .all(|(lineno, _)| *lineno != test_module_line),
        "the test attribute must not be a production line"
    );
    let marker = "pub fn inventory_page";
    let (lineno, _) = production
        .iter()
        .find(|(_, line)| line.contains(marker))
        .unwrap_or_else(|| panic!("{marker} must be scanned"));
    assert!(
        *lineno > test_module_line,
        "{marker} at line {lineno} must be scanned after the test module at {test_module_line}"
    );
}
