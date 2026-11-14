//! TASK-117: mechanical no-drift checks for config-defined document types.
//!
//! Rules D-01 .. D-07 from
//! `docs/specifications/config-defined-document-types-verification-and-packaging.md`
//! section 7. The scanner reads production lines only, following the
//! `#[cfg(test)]` skip pattern established by `tests/no_drift.rs`, and scans
//! only the named user-facing documents. Historical ADRs, historical
//! specifications, `docs/packaging.md` evidence, and `backlog/` records are
//! excluded because they record past behaviour.
//!
//! Every violation reports `file:line: text`.

use std::fs;
use std::path::{Path, PathBuf};

const DOCUMENT_SLUGS: [&str; 11] = [
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

fn src_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }
    }
    walk(Path::new("src"), &mut out);
    out.sort();
    out
}

fn relative(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Split a source file into production lines, excluding every
/// `#[cfg(test)]`-annotated item and resuming after it closes.
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
    start + 1
}

fn scan_src(label: &str, allowed: &[&str], matches: impl Fn(&str) -> bool) -> Result<(), String> {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if allowed.iter().any(|prefix| rel.starts_with(prefix)) {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            if matches(&line) {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("{label}\n{}", violations.join("\n")))
    }
}

fn has_slug_literal(line: &str, slugs: &[&str]) -> bool {
    slugs
        .iter()
        .any(|slug| line.contains(&format!("\"{slug}\"")))
}

fn is_dispatch_line(line: &str, slugs: &[&str]) -> bool {
    has_slug_literal(line, slugs)
        && (line.contains("match")
            || line.contains("if ")
            || line.contains("=>")
            || line.contains("==")
            || line.contains(".eq("))
}

// D-01 ---------------------------------------------------------------------
#[test]
fn d01_no_deprecated_folder_literal_in_src() {
    scan_src("D-01 literal `deprecated/` in src/:", &[], |line| {
        line.contains("deprecated/")
    })
    .unwrap();
}

// D-02 ---------------------------------------------------------------------
#[test]
fn d02_no_nfreq_command_in_src() {
    // The registry's collision-test fixture is inside `#[cfg(test)]`, which
    // `production_lines_in` excludes, so production code has no allowlist.
    scan_src("D-02 `nfreq` command in src/:", &[], |line| {
        line.contains("nfreq")
    })
    .unwrap();
}

// D-03 ---------------------------------------------------------------------
#[test]
fn d03_no_document_deprecate_or_restore_variants() {
    scan_src(
        "D-03 document `deprecate`/`restore` command variants:",
        &["src/registry.rs", "src/task.rs"],
        |line| {
            line.contains("Deprecate {")
                || line.contains("Deprecate,")
                || line.contains("Deprecate =>")
                || line.contains("::Deprecate")
                || line.contains("Restore {")
                || line.contains("Restore,")
                || line.contains("Restore =>")
                || line.contains("::Restore")
        },
    )
    .unwrap();
}

// D-04 ---------------------------------------------------------------------
#[test]
fn d04_no_static_per_type_command_in_cli() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if !rel.starts_with("src/cli/") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            if has_slug_literal(&line, &DOCUMENT_SLUGS) {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-04 static per-type document command/list in src/cli/:\n{}",
        violations.join("\n")
    );
}

// D-05 ---------------------------------------------------------------------
#[test]
fn d05_no_slug_literal_dispatch_outside_type_modules() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        let scanned = rel.starts_with("src/browser/")
            || matches!(
                rel.as_str(),
                "src/cli/dispatch.rs" | "src/cli/tree.rs" | "src/document.rs" | "src/search.rs"
            );
        if !scanned {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            if is_dispatch_line(&line, &DOCUMENT_SLUGS) {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-05 slug-literal behaviour dispatch outside the type module/registry:\n{}",
        violations.join("\n")
    );
}

// D-06 ---------------------------------------------------------------------
fn documentation_files() -> Vec<PathBuf> {
    let mut files = vec![
        PathBuf::from("README.md"),
        PathBuf::from("docs/release-1.0-upgrade-notes.md"),
    ];
    if let Ok(rd) = fs::read_dir("docs/business-rules") {
        let mut rules: Vec<PathBuf> = rd
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
            .collect();
        rules.sort();
        files.extend(rules);
    }
    files
}

fn nfreq_is_removal_wording(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("gone")
        || lower.contains("no longer")
        || lower.contains("before")
        || lower.contains("after")
        || lower.contains("exits `2`")
        || lower.contains("`papyrus nfreq <verb>")
        || line.contains('|')
}

fn deprecated_is_inert_wording(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("inert")
        || lower.contains("not read")
        || lower.contains("no longer")
        || lower.contains("not listed")
        || lower.contains("move each")
        || lower.contains("move the file")
        || lower.contains("manual move")
        || lower.contains("legacy")
        || lower.contains("recover")
        || lower.contains("follow")
        || lower.contains("applies to")
        || lower.contains("used")
}

/// Distinct document slugs invoked as `papyrus <slug>` in a fenced block.
fn fenced_command_slugs(content: &str) -> Vec<(usize, Vec<String>)> {
    let mut blocks = Vec::new();
    let mut current: Option<(usize, Vec<String>)> = None;
    for (index, line) in content.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            if let Some(block) = current.take() {
                blocks.push(block);
            } else {
                current = Some((index + 1, Vec::new()));
            }
            continue;
        }
        if let Some((_, slugs)) = current.as_mut() {
            for slug in DOCUMENT_SLUGS {
                if line.contains(&format!("papyrus {slug} ")) && !slugs.iter().any(|s| s == slug) {
                    slugs.push(slug.to_string());
                }
            }
        }
    }
    blocks
}

fn table_command_slugs(content: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    for line in content.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        for slug in DOCUMENT_SLUGS {
            if line.contains(&format!("papyrus {slug} ")) && !slugs.iter().any(|s| s == slug) {
                slugs.push(slug.to_string());
            }
        }
    }
    slugs
}

fn static_inventory_violation(content: &str) -> Option<String> {
    for (line, slugs) in fenced_command_slugs(content) {
        if slugs.len() >= 3 {
            return Some(format!(
                "fenced static document-command inventory at line {line}: {}",
                slugs.join(", ")
            ));
        }
    }
    let table = table_command_slugs(content);
    if table.len() >= 3 {
        return Some(format!(
            "table static document-command inventory: {}",
            table.join(", ")
        ));
    }
    None
}

#[test]
fn d06_no_stale_nfreq_or_static_command_block_in_user_docs() {
    let mut violations = Vec::new();
    for path in documentation_files() {
        let rel = relative(&path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in content.lines().enumerate() {
            if line.contains("papyrus nfreq") && !nfreq_is_removal_wording(line) {
                violations.push(format!("{}:{}: {}", rel, lineno + 1, line.trim()));
            }
            if line.contains("deprecated/") && !deprecated_is_inert_wording(line) {
                violations.push(format!("{}:{}: {}", rel, lineno + 1, line.trim()));
            }
        }
        if let Some(message) = static_inventory_violation(&content) {
            violations.push(format!("{rel}: {message}"));
        }
    }
    assert!(
        violations.is_empty(),
        "D-06 stale document-type wording in user-facing documentation:\n{}",
        violations.join("\n")
    );
}

// D-07 ---------------------------------------------------------------------
#[test]
fn d07_no_static_inventory_in_config_type_specifications() {
    let specs = [
        "docs/specifications/search-config-defined-document-types.md",
        "docs/specifications/browser-config-defined-document-types.md",
    ];
    let mut violations = Vec::new();
    for rel in specs {
        let content = fs::read_to_string(rel).unwrap_or_default();
        if let Some(message) = static_inventory_violation(&content) {
            violations.push(format!("{rel}: {message}"));
        }
    }
    assert!(
        violations.is_empty(),
        "D-07 static document-type inventory in a config-type specification:\n{}",
        violations.join("\n")
    );
}

// Scanner self-tests --------------------------------------------------------
#[test]
fn scanner_flags_planted_violations_and_accepts_allowlisted_lines() {
    // D-01
    assert!(
        production_lines_in("let path = \"docs/deprecated/\";")
            .iter()
            .any(|(_, line)| line.contains("deprecated/"))
    );
    // D-02
    assert!(
        production_lines_in("Command::new(\"nfreq\")")
            .iter()
            .any(|(_, line)| line.contains("nfreq"))
    );
    // D-03
    assert!(
        production_lines_in("enum DocumentVerb { Restore { id: String } }")
            .iter()
            .any(|(_, line)| line.contains("Restore {"))
    );
    // D-05
    assert!(is_dispatch_line(
        "match slug { \"req\" => run(), _ => {} }",
        &DOCUMENT_SLUGS
    ));
    assert!(!is_dispatch_line("\"req\",", &DOCUMENT_SLUGS));
    // The `#[cfg(test)]` fixture is excluded.
    let fixture =
        "#[cfg(test)]\nmod tests {\n    fn collision() { let _ = \"nfreq\"; }\n}\nfn after() {}\n";
    assert!(
        !production_lines_in(fixture)
            .iter()
            .any(|(_, line)| line.contains("nfreq"))
    );
    // D-06 allowlisted migration wording.
    assert!(nfreq_is_removal_wording(
        "You used `papyrus nfreq ...`. The alias is gone; use `papyrus nfr ...`."
    ));
    assert!(!nfreq_is_removal_wording("papyrus nfreq list"));
    assert!(deprecated_is_inert_wording(
        "A `deprecated/` folder is inert: Papyrus does not read it."
    ));
    assert!(!deprecated_is_inert_wording(
        "Read every `deprecated/` file."
    ));
    // D-06 / D-07 static inventory.
    let planted = "```sh\npapyrus req list\npapyrus nfr list\npapyrus audit list\n```\n";
    assert!(static_inventory_violation(planted).is_some());
    let allowed = "```sh\npapyrus req archive REQ-003\n```\n";
    assert!(static_inventory_violation(allowed).is_none());
}

// TASK-119 guards ----------------------------------------------------------

const TYPED_DOCUMENT_MODULES: [&str; 10] = [
    "src/requirement.rs",
    "src/glossary.rs",
    "src/rule.rs",
    "src/ui_spec.rs",
    "src/audit.rs",
    "src/tech_spec.rs",
    "src/use_case.rs",
    "src/test_case.rs",
    "src/test_report.rs",
    "src/ui_mockup.rs",
];

const REMOVED_DOCUMENT_ENUMS: [&str; 10] = [
    "ReqCmd",
    "GlossaryCmd",
    "RuleCmd",
    "UiSpecCmd",
    "AuditCmd",
    "TechSpecCmd",
    "UseCaseCmd",
    "TestCaseCmd",
    "TestReportCmd",
    "UiMockupCmd",
];

// D-08 ---------------------------------------------------------------------
#[test]
fn d08_no_per_type_document_clap_enum_returns() {
    scan_src("D-08 per-type document clap enum in src/:", &[], |line| {
        REMOVED_DOCUMENT_ENUMS
            .iter()
            .any(|name| line.contains(name))
    })
    .unwrap();
}

// D-09 ---------------------------------------------------------------------
#[test]
fn d09_no_document_dispatch_plumbing_in_cli() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if !rel.starts_with("src/cli/") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            let lowered = line.trim_start();
            if lowered.contains("TypedHandler")
                || lowered.contains("dispatch_typed_module")
                || (lowered.contains("from_arg_matches")
                    && !lowered.contains("BuiltinCmd::from_arg_matches"))
                || (lowered.contains("augment_subcommands")
                    && !lowered.contains("BuiltinCmd::augment_subcommands"))
            {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-09 document dispatch plumbing in src/cli/:\n{}",
        violations.join("\n")
    );
}

// D-10 ---------------------------------------------------------------------
#[test]
fn d10_no_module_local_document_directory_helpers() {
    let mut violations = Vec::new();
    for rel in TYPED_DOCUMENT_MODULES {
        let content = fs::read_to_string(rel).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            let trimmed = line.trim();
            if trimmed.contains("cfg.doc_directory")
                || trimmed.contains("load_effective_config")
                || trimmed.starts_with("fn load_config(")
                || trimmed.starts_with("pub fn load_config(")
                || trimmed.starts_with("fn type_entry(")
                || trimmed.starts_with("pub fn type_entry(")
                || trimmed.starts_with("fn active_dir(")
                || trimmed.starts_with("pub fn active_dir(")
                || trimmed.starts_with("fn archived_dir(")
                || trimmed.starts_with("pub fn archived_dir(")
            {
                violations.push(format!("{}:{}: {}", rel, lineno, trimmed));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-10 module-local document directory/config helpers:\n{}",
        violations.join("\n")
    );
}

// D-11 ---------------------------------------------------------------------
#[test]
fn d11_document_tree_is_descriptor_driven() {
    let tree = fs::read_to_string("src/cli/tree.rs").unwrap_or_default();
    let production: String = production_lines_in(&tree)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        production.contains("ordered_verb_specs"),
        "document_command must build from descriptor verb metadata"
    );
    assert!(
        !production.contains("ContentModel::"),
        "document_command must not branch on the content model"
    );

    let document = fs::read_to_string("src/document.rs").unwrap_or_default();
    let production: String = production_lines_in(&document)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        production.contains("fn dispatch_typed("),
        "document::run must delegate typed content verbs through one handler"
    );
    assert!(
        !production.contains("fn dispatch_typed_module"),
        "the second dispatcher must not return"
    );
}
