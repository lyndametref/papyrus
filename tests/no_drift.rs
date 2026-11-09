//! No-drift checks for ADR-0009 decision 7 (TASK-70).

use std::fs;
use std::path::PathBuf;

fn src_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
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
    walk(std::path::Path::new("src"), &mut out);
    out
}

/// Split a source file into production lines, dropping everything from the
/// first `#[cfg(test)]` attribute to end of file (test modules are excluded
/// per the documented allowlist).
fn production_lines(path: &std::path::Path) -> Vec<(usize, String)> {
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

#[test]
fn no_fs_write_on_record_paths_outside_allowlist() {
    let allowlisted_files = ["config.rs", "init.rs"];
    let mut violations = Vec::new();
    for path in src_files() {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if allowlisted_files.contains(&name) {
            continue;
        }
        for (lineno, line) in production_lines(&path) {
            if line.contains("fs::write") || line.contains("std::fs::write") {
                violations.push(format!("{}:{}: {}", path.display(), lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "fs::write on a record path outside allowlist:\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_format_frontmatter_delimiter_outside_record() {
    let mut violations = Vec::new();
    for path in src_files() {
        // src/record/ is allowed to construct delimiters.
        if path.to_string_lossy().contains("/record/") {
            continue;
        }
        for (lineno, line) in production_lines(&path) {
            if line.contains("format!") && line.contains("\"---") {
                violations.push(format!("{}:{}: {}", path.display(), lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "format!(\"--- delimiter construction outside src/record/:\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_id_formatting_outside_identity() {
    let mut violations = Vec::new();
    for path in src_files() {
        let in_identity = path.to_string_lossy().ends_with("record/identity.rs");
        if in_identity {
            continue;
        }
        for (lineno, line) in production_lines(&path) {
            if !line.contains("format!") {
                continue;
            }
            // ID formatting: a format! combining an ID prefix with zero-padding.
            let is_id_fmt =
                line.contains("\"{}-{:0") || line.contains("\"{}.{:0") || line.contains("{:0width");
            if is_id_fmt {
                violations.push(format!("{}:{}: {}", path.display(), lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "ID formatting outside src/record/identity.rs:\n{}",
        violations.join("\n")
    );
}
