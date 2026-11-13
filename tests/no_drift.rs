//! Mechanical no-drift checks for ADR-0013 decision 7 (TASK-97).
//!
//! Each check scans production `.rs` files under `src/` and fails the default
//! `cargo test` run. Test modules (`#[cfg(test)]` to end of file) are excluded
//! as documented. Allowlists are the minimum required by the feature
//! specification:
//!
//! | # | Condition | Allowlist |
//! |---|---|---|
//! | 1 | `fs::write` on a record path or `fs::rename` outside the store | `src/config.rs`, `src/init.rs`, `src/record/` |
//! | 2 | front-matter delimiter construction (`format!("---`) | `src/record/` |
//! | 3 | ID formatting and prefix-shaped ID parsing | `src/record/identity.rs` |
//! | 4 | storage-ID / translation symbols and offsets | none |
//! | 5 | hardcoded record-directory literals | `src/record/descriptor.rs`, `src/config.rs`, `src/init.rs`, `src/import.rs` |
//! | 6 | Backlog-shaped stem detection or storage prefix construction | `src/record/identity.rs`, `src/record/descriptor.rs` |
//! | 7 | a second record writer (`.tmp` + rename) | `src/record/store.rs` |
//! | 8 | a second section parser or front-matter delimiter detection | `src/record/`, `src/import.rs` |
//!
//! Every violation reports `file:line: text`.

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

/// Split a source file into production lines, dropping everything from the
/// first `#[cfg(test)]` attribute to end of file (test modules are excluded).
fn production_lines(path: &Path) -> Vec<(usize, String)> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    let mut in_test = false;
    for (i, line) in content.lines().enumerate() {
        if line.contains("#[cfg(test)]") {
            in_test = true;
        }
        if in_test {
            continue;
        }
        out.push((i + 1, line.to_string()));
    }
    out
}

fn relative(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn check(label: &str, allowed: &[&str], matches: impl Fn(&str) -> bool) -> Result<(), String> {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if allowed.iter().any(|prefix| rel.starts_with(prefix)) {
            continue;
        }
        for (lineno, line) in production_lines(&path) {
            if matches(&line) {
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
    // A record file may only be renamed by the shared store.
    check(
        "fs::rename outside src/record/:",
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
        |line| line.contains("format!(\"{}-") || line.contains("strip_prefix(&format!(\"{}-"),
    )
    .unwrap();
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
        "deprecated/",
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
    check(
        "second record writer outside src/record/store.rs:",
        &["src/record/store.rs"],
        |line| line.contains("fn write_atomic") || line.contains(".tmp"),
    )
    .unwrap();
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
