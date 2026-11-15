//! Source guard against process-global descriptor caches and leaked slugs.
//!
//! TASK-121 removed `GENERIC_DESC_CACHE` and every `Box::leak` in `src/`.
//! Descriptor identity must come from the `ArtifactRegistry` that owns the
//! configuration, never from process history (ADR-0017 decision 10), so this
//! test scans production lines and fails the default `cargo test` run when a
//! leak, a `'static` interning table, or the deleted generic constructor
//! returns.
//!
//! The scanner mirrors `tests/no_drift.rs`: every `#[cfg(test)]`-annotated
//! item is skipped by tracking brace depth from the attribute, and scanning
//! resumes after it closes.

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

fn scan_files(
    label: &str,
    files: &[(String, String)],
    matches: impl Fn(&str, &str) -> bool,
) -> Result<(), String> {
    let mut violations = Vec::new();
    for (rel, content) in files {
        for (lineno, line) in production_lines_in(content) {
            if matches(rel, &line) {
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

fn source_files() -> Vec<(String, String)> {
    src_files()
        .into_iter()
        .map(|path| {
            let content = fs::read_to_string(&path).unwrap_or_default();
            (relative(&path), content)
        })
        .collect()
}

#[test]
fn no_leaked_slugs_or_descriptors_anywhere_in_src() {
    // `Box::leak` and `mem::forget` hide a lifetime that should be owned; both
    // were used only for the descriptor cache and the clap command name.
    let files = source_files();
    scan_files("leaked descriptor or slug:", &files, |_, line| {
        line.contains("Box::leak")
            || line.contains("mem::forget")
            || line.contains("GENERIC_DESC_CACHE")
            || line.contains("generic_document_descriptor")
    })
    .expect("no production line may leak a descriptor or slug");
}

#[test]
fn no_static_descriptor_cache_in_descriptor_or_tree() {
    // The two files that used to hold process-global state must not regain a
    // `'static` interning table under another name (ADR-0017 decision 10).
    let files: Vec<(String, String)> = source_files()
        .into_iter()
        .filter(|(rel, _)| rel == "src/record/descriptor.rs" || rel == "src/cli/tree.rs")
        .collect();
    assert_eq!(files.len(), 2, "both guarded files must exist");
    scan_files("static descriptor cache:", &files, |_, line| {
        line.contains("OnceLock")
            || line.contains("OnceCell")
            || line.contains("LazyLock")
            || line.contains("lazy_static")
    })
    .expect("descriptor and tree modules may not cache statically");
}

// TASK-121: the scanner itself must report violations, including ones after a
// mid-file test module, so the guard cannot silently pass.
#[test]
fn scanner_reports_violations() {
    let fixture = "\
fn before() {}\n\
\n\
#[cfg(test)]\n\
mod tests {\n\
    fn helper() { let _ = Box::leak(String::new().into_boxed_str()); }\n\
}\n\
\n\
fn after() {\n\
    let _ = Box::leak(String::new().into_boxed_str());\n\
}\n\
";
    let production = production_lines_in(fixture);
    assert!(
        production
            .iter()
            .any(|(_, line)| line.contains("fn before"))
    );
    assert!(
        !production
            .iter()
            .any(|(_, line)| line.contains("fn helper"))
    );
    assert!(production.iter().any(|(_, line)| line.contains("fn after")));

    let files = [("fixture.rs".to_string(), fixture.to_string())];
    let error = scan_files("fixture violation:", &files, |_, line| {
        line.contains("Box::leak")
    })
    .unwrap_err();
    assert!(
        error.contains("fixture.rs:9:"),
        "violation after the test module must report file:line, got: {error}"
    );

    let clean = [("clean.rs".to_string(), "fn ok() {}\n".to_string())];
    scan_files("clean:", &clean, |_, line| line.contains("Box::leak"))
        .expect("a clean file must pass");
}
