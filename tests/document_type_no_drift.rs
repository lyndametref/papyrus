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
//!
//! Guards owned here after TASK-283:
//!
//! | Guard | Invariant | Failure format |
//! |---|---|---|
//! | D-05 | no document slug literal outside the `SEED_DOCUMENT_TYPES` block | `src/x.rs:12: <text>` |
//! | D-05 (list) | no second document slug list | `src/x.rs:12: <text>` |
//! | D-13 | per-type document modules stay deleted; `ui_mockup` remains | `src/x.rs must be deleted…` |
//! | D-16 | `TypedHandler` is exactly Task, Milestone, Release, Decision | `src/registry.rs:25: TypedHandler …` |
//! | D-17 | no document `TypedHandler` or `RequirementKind` in production | `src/x.rs:12: <text>` |
//! | D-19 | `ArtifactCategory` is exactly Backlog, Decision, Document | `src/registry.rs:25: ArtifactCategory …` |
//! | D-20 | only `document_conflict_page`/`document_not_found_page` remain | `src/x.rs:12: <text>` |

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

/// Line range (1-based, inclusive) of the `SEED_DOCUMENT_TYPES` declaration:
/// the only production place where a document slug literal is allowed.
fn seed_constant_range(content: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.contains("SEED_DOCUMENT_TYPES: ["))?;
    let mut depth = 0i32;
    let mut opened = false;
    for (index, line) in lines.iter().enumerate().skip(start) {
        depth += bracket_delta(line);
        if bracket_delta(line) > 0 {
            opened = true;
        }
        if opened && depth <= 0 {
            return Some((start + 1, index + 1));
        }
    }
    None
}

/// Net square-bracket balance on one line, ignoring comments, strings, raw
/// strings, and character literals, so a slug list can be delimited exactly.
fn bracket_delta(line: &str) -> i32 {
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
            b'[' => {
                delta += 1;
                i += 1;
            }
            b']' => {
                delta -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    delta
}

/// Every production document slug literal outside the `SEED_DOCUMENT_TYPES`
/// block, as `file:line: text`.
fn slug_literal_violations(files: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (rel, content) in files {
        let allowed = if rel == "src/config.rs" {
            seed_constant_range(content)
        } else {
            None
        };
        for (lineno, line) in production_lines_in(content) {
            if !has_slug_literal(&line, &DOCUMENT_SLUGS) {
                continue;
            }
            if let Some((start, end)) = allowed
                && lineno >= start
                && lineno <= end
            {
                continue;
            }
            violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
        }
    }
    violations
}

/// A second slug list: one production line naming two or more distinct
/// document slugs. The seed declaration is written one slug per entry line and
/// is the only allowed definition.
fn second_slug_list_violations(files: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (rel, content) in files {
        let allowed = if rel == "src/config.rs" {
            seed_constant_range(content)
        } else {
            None
        };
        for (lineno, line) in production_lines_in(content) {
            if let Some((start, end)) = allowed
                && lineno >= start
                && lineno <= end
            {
                continue;
            }
            let distinct = DOCUMENT_SLUGS
                .iter()
                .filter(|slug| line.contains(&format!("\"{slug}\"")))
                .count();
            if distinct >= 2 {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    violations
}

/// The source files the drift guards scan.
fn source_files() -> Vec<(String, String)> {
    src_files()
        .into_iter()
        .map(|path| {
            let content = fs::read_to_string(&path).unwrap_or_default();
            (relative(&path), content)
        })
        .collect()
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
// The slug guard owns the invariant across all of `src/`: a document slug
// literal selects behaviour only in the seed table. Allowlist: the
// `SEED_DOCUMENT_TYPES` declaration block in `src/config.rs` and nothing else.
// Tests and `docs/` are exempt by construction (only `src/*.rs` production
// lines are scanned).
#[test]
fn d05_no_slug_literal_outside_the_seed_table() {
    let violations = slug_literal_violations(&source_files());
    assert!(
        violations.is_empty(),
        "D-05 any document slug literal outside SEED_DOCUMENT_TYPES in src/:\n{}",
        violations.join("\n")
    );
}

#[test]
fn d05_scanner_flags_a_planted_slug_match_in_a_new_module() {
    let planted = [(
        "src/new_module.rs".to_string(),
        "pub fn pick(slug: &str) -> u8 {\n    match slug {\n        \"req\" => 1,\n        _ => 0,\n    }\n}\n"
            .to_string(),
    )];
    let violations = slug_literal_violations(&planted);
    assert!(
        violations
            .iter()
            .any(|line| line.starts_with("src/new_module.rs:3: ")),
        "a planted slug match must report file:line, got: {violations:?}"
    );
    assert!(
        violations.iter().any(|line| line.contains("\"req\"")),
        "the planted line text must be reported, got: {violations:?}"
    );

    // The seed block itself is the allowlist, and a literal outside it in the
    // config module is still flagged.
    let config = "const SEED_DOCUMENT_TYPES: [&str; 1] = [\n    \"req\",\n];\nfn other() { let _ = \"nfr\"; }\n";
    let violations = slug_literal_violations(&[("src/config.rs".to_string(), config.to_string())]);
    assert_eq!(violations.len(), 1, "got: {violations:?}");
    assert!(
        violations[0].starts_with("src/config.rs:4: "),
        "{violations:?}"
    );
}

#[test]
fn d05_seed_constant_range_covers_the_seed_declaration() {
    let content = fs::read_to_string("src/config.rs").expect("config.rs");
    let (start, end) = seed_constant_range(&content).expect("seed declaration");
    assert!(
        content
            .lines()
            .nth(start - 1)
            .unwrap()
            .contains("SEED_DOCUMENT_TYPES")
    );
    assert!(
        end > start + 11,
        "the seed block must span the seed entries"
    );
    assert!(
        content.lines().nth(end - 1).unwrap().trim() == "];",
        "the seed block ends at the array close"
    );
}

// D-05 (second list) -------------------------------------------------------
#[test]
fn d05_no_second_document_slug_list_in_src() {
    let violations = second_slug_list_violations(&source_files());
    assert!(
        violations.is_empty(),
        "a second document slug list in src/:\n{}",
        violations.join("\n")
    );
    let planted = [(
        "src/list.rs".to_string(),
        "const DOCS: [&str; 2] = [\"req\", \"nfr\"];\n".to_string(),
    )];
    assert!(
        !second_slug_list_violations(&planted).is_empty(),
        "a planted second slug list must be detected"
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

fn table_command_slugs(content: &str) -> Vec<(usize, Vec<String>)> {
    let mut tables = Vec::new();
    let mut current: Option<(usize, Vec<String>, bool)> = None;
    for (index, line) in content.lines().enumerate() {
        if !line.trim_start().starts_with('|') {
            if let Some((start, slugs, migration)) = current.take()
                && !migration
            {
                tables.push((start, slugs));
            }
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let migration_row = cells.iter().any(|cell| cell.eq_ignore_ascii_case("before"))
            && cells.iter().any(|cell| cell.eq_ignore_ascii_case("after"));
        if current.is_none() {
            current = Some((index + 1, Vec::new(), migration_row));
        }
        let (_, slugs, migration) = current.as_mut().expect("table is present");
        if migration_row {
            *migration = true;
        }
        for slug in DOCUMENT_SLUGS {
            if line.contains(&format!("papyrus {slug} ")) && !slugs.iter().any(|s| s == slug) {
                slugs.push(slug.to_string());
            }
        }
    }
    if let Some((start, slugs, migration)) = current
        && !migration
    {
        tables.push((start, slugs));
    }
    tables
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
    for (line, slugs) in table_command_slugs(content) {
        if slugs.len() >= 3 {
            return Some(format!(
                "table static document-command inventory at line {line}: {}",
                slugs.join(", ")
            ));
        }
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
    // A migration table is not a command inventory: its Before/After columns
    // show one mapping per row, not the command surface.
    let migration = "| Before | After |\n|---|---|\n| `papyrus req archive <id>` | `papyrus req status <id> active` |\n| `papyrus nfr list` | `papyrus nfr view <id>` |\n| `papyrus glossary list` | `papyrus rule list` |\n";
    assert!(
        static_inventory_violation(migration).is_none(),
        "a migration table must not count as a static inventory"
    );
    let inventory = "| Command | Meaning |\n|---|---|\n| `papyrus req list` | list |\n| `papyrus nfr list` | list |\n| `papyrus glossary list` | list |\n";
    assert!(
        static_inventory_violation(inventory).is_some(),
        "a real command inventory must still be rejected"
    );
}

// TASK-119 guards ----------------------------------------------------------

/// The only typed document module left after TASK-259: the HTML mockup keeps
/// its own metadata writer until MS-035.
const TYPED_DOCUMENT_MODULES: [&str; 1] = ["src/ui_mockup.rs"];

/// The nine per-type CLI modules deleted by TASK-259.
const REMOVED_DOCUMENT_MODULES: [&str; 9] = [
    "src/requirement.rs",
    "src/glossary.rs",
    "src/rule.rs",
    "src/ui_spec.rs",
    "src/audit.rs",
    "src/tech_spec.rs",
    "src/use_case.rs",
    "src/test_case.rs",
    "src/test_report.rs",
];

/// The transitional document command tables deleted by TASK-259. The HTML
/// mockup keeps its model-owned verb table until MS-035.
const REMOVED_DOCUMENT_COMMAND_BUILDERS: [&str; 20] = [
    "fn document_schema(",
    "fn requirement_verbs(",
    "fn glossary_verbs(",
    "fn rule_verbs(",
    "fn ui_spec_verbs(",
    "fn audit_verbs(",
    "fn tech_spec_verbs(",
    "fn use_case_verbs(",
    "fn test_case_verbs(",
    "fn test_report_verbs(",
    "fn requirement_business_schema(",
    "fn requirement_non_functional_schema(",
    "fn glossary_schema(",
    "fn rule_schema(",
    "fn ui_spec_schema(",
    "fn audit_schema(",
    "fn tech_spec_schema(",
    "fn use_case_schema(",
    "fn test_case_schema(",
    "fn test_report_schema(",
];

/// The eight document ops builders deleted by MS-035 TASK-269.
const DEFERRED_DOCUMENT_OPS_BUILDERS: [&str; 8] = [
    "generic_document_ops",
    "requirement_ops",
    "glossary_ops",
    "ui_spec_ops",
    "business_rule_ops",
    "audit_ops",
    "tech_spec_ops",
    "use_case_ops",
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
        production.contains("fn create_document(")
            && production.contains("fn edit_document(")
            && production.contains("fn view_document("),
        "document::run must use the field-driven document engine"
    );
    assert!(
        !production.contains("fn dispatch_typed("),
        "the per-type dispatcher must be gone"
    );
    assert!(
        !production.contains("fn dispatch_typed_module"),
        "the second dispatcher must not return"
    );
}

// D-12 ---------------------------------------------------------------------
#[test]
fn d12_document_records_have_one_source() {
    // The parallel static document table is gone: no production call site may
    // mention the removed lookup.
    scan_src("D-12 `descriptor_for_doc_type` in src/:", &[], |line| {
        line.contains("descriptor_for_doc_type")
    })
    .unwrap();

    // Only the four backlog families stay static, and none describes a
    // document type; document records are projected from the artifact registry.
    let records = papyrus::record::descriptor::descriptors();
    assert_eq!(
        records.len(),
        4,
        "static record table must hold backlog only"
    );
    assert!(
        records
            .iter()
            .all(|descriptor| descriptor.doc_type_key.is_none()),
        "a static record must not describe a document type"
    );
    for (name, record_type) in [
        ("task", "task"),
        ("milestone", "milestone"),
        ("release", "release"),
        ("decision", "decision"),
    ] {
        assert!(
            records
                .iter()
                .any(|descriptor| descriptor.name == name && descriptor.record_type == record_type),
            "static descriptor missing: {name}"
        );
    }
}

// D-13 ---------------------------------------------------------------------
#[test]
fn d13_typed_document_modules_are_deleted_and_ui_mockup_remains() {
    for rel in REMOVED_DOCUMENT_MODULES {
        assert!(
            !Path::new(rel).exists(),
            "{rel} must be deleted by TASK-259"
        );
    }
    assert!(
        Path::new("src/ui_mockup.rs").exists(),
        "ui_mockup must remain as the SingleHtmlMockup surface"
    );
}

// D-14 ---------------------------------------------------------------------
#[test]
fn d14_transitional_document_command_tables_are_gone() {
    let registry = fs::read_to_string("src/registry.rs").expect("registry.rs");
    let production: String = production_lines_in(&registry)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    for builder in REMOVED_DOCUMENT_COMMAND_BUILDERS {
        assert!(
            !production.contains(builder),
            "transitional document command table must be deleted: {builder}"
        );
    }
}

// D-15 ---------------------------------------------------------------------
#[test]
fn d15_deferred_document_ops_builders_are_absent() {
    let edit = fs::read_to_string("src/record/edit.rs").expect("edit.rs");
    let production: String = production_lines_in(&edit)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    for builder in DEFERRED_DOCUMENT_OPS_BUILDERS {
        assert!(
            !production.contains(&format!("fn {builder}(")),
            "document ops builder must be deleted after MS-035 TASK-269: {builder}"
        );
    }
    for builder in [
        "task_status_ops",
        "task_ops",
        "milestone_ops",
        "release_ops",
        "decision_ops",
    ] {
        assert!(
            production.contains(&format!("fn {builder}(")),
            "backlog/decision builder must remain: {builder}"
        );
    }
}

// D-16 ---------------------------------------------------------------------
/// The variants of the enum introduced by `header`, with the declaration line.
fn enum_variants(production: &[(usize, String)], header: &str) -> (usize, Vec<String>) {
    let (enum_line, _) = production
        .iter()
        .find(|(_, line)| line.contains(header))
        .unwrap_or_else(|| panic!("missing enum: {header}"));
    let mut variants = Vec::new();
    for (_, line) in production.iter().filter(|(lineno, _)| lineno > enum_line) {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("///") {
            variants.push(trimmed.to_string());
        }
    }
    (*enum_line, variants)
}

#[test]
fn d16_only_backlog_and_decision_typed_handlers_exist() {
    let rel = "src/registry.rs";
    let registry = fs::read_to_string(rel).expect("registry.rs");
    let (enum_line, variants) =
        enum_variants(&production_lines_in(&registry), "pub enum TypedHandler");
    assert_eq!(
        variants,
        vec![
            "Task,".to_string(),
            "Milestone,".to_string(),
            "Release,".to_string(),
            "Decision,".to_string(),
        ],
        "{rel}:{enum_line}: TypedHandler must have exactly the four explicit \
         backlog/decision variants, found {variants:?}"
    );
}

// D-19 ---------------------------------------------------------------------
#[test]
fn d19_artifact_category_has_exactly_three_variants() {
    let rel = "src/registry.rs";
    let registry = fs::read_to_string(rel).expect("registry.rs");
    let (enum_line, variants) =
        enum_variants(&production_lines_in(&registry), "pub enum ArtifactCategory");
    assert_eq!(
        variants,
        vec![
            "Backlog,".to_string(),
            "Decision,".to_string(),
            "Document,".to_string(),
        ],
        "{rel}:{enum_line}: ArtifactCategory must be exactly Backlog, Decision, \
         Document (three handlers), found {variants:?}"
    );
}

#[test]
fn d16_and_d19_variant_scanners_detect_planted_variants() {
    let planted = vec![
        (7, "pub enum TypedHandler {".to_string()),
        (8, "    Task,".to_string()),
        (9, "    Document,".to_string()),
        (10, "}".to_string()),
    ];
    let (enum_line, variants) = enum_variants(&planted, "pub enum TypedHandler");
    assert_eq!(enum_line, 7);
    assert_eq!(variants, vec!["Task,".to_string(), "Document,".to_string()]);
    assert!(variants.contains(&"Document,".to_string()));
}

// D-20 ---------------------------------------------------------------------
/// The only conflict/not-found renderers allowed anywhere in `src/`. The
/// backlog/decision conflict page is one bespoke page; documents share one
/// conflict page and one not-found page.
const ALLOWED_RENDERERS: [&str; 3] = [
    "backlog_decision_conflict_page",
    "document_conflict_page",
    "document_not_found_page",
];

fn renderer_tokens(line: &str) -> Vec<String> {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| token.contains("conflict_page") || token.contains("not_found_page"))
        .map(str::to_string)
        .collect()
}

fn renderer_violations(files: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (rel, content) in files {
        for (lineno, line) in production_lines_in(content) {
            for token in renderer_tokens(&line) {
                if !ALLOWED_RENDERERS.contains(&token.as_str()) {
                    violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
                    break;
                }
            }
        }
    }
    violations
}

#[test]
fn d20_only_the_shared_document_renderers_exist() {
    let files = source_files();
    let violations = renderer_violations(&files);
    assert!(
        violations.is_empty(),
        "D-20 per-type conflict/not-found renderer beyond \
         document_conflict_page and document_not_found_page:\n{}",
        violations.join("\n")
    );

    // The shared renderers really exist and keep their call sites.
    let browser = fs::read_to_string("src/browser/mod.rs").expect("browser/mod.rs");
    assert!(browser.contains("render::document_conflict_page"));
    assert!(browser.contains("render::document_not_found_page"));

    // Planted definitions and call sites are detected with file:line.
    for planted in [
        "pub fn conflict_page_audit_record() {}",
        "pub fn tech_spec_not_found_page() {}",
        "let page = render::test_case_not_found_page(id);",
        "let page = conflict_page(id);",
    ] {
        let files = [("src/browser/render.rs".to_string(), format!("{planted}\n"))];
        let violations = renderer_violations(&files);
        assert!(
            violations
                .iter()
                .any(|line| line.starts_with("src/browser/render.rs:1: ")),
            "the planted renderer {planted:?} must be rejected, got: {violations:?}"
        );
    }

    let allowed = "let page = render::document_not_found_page(slug);\n";
    let files = [("src/browser/render.rs".to_string(), allowed.to_string())];
    assert!(
        renderer_violations(&files).is_empty(),
        "the shared document renderer is allowed"
    );
}

// D-17 ---------------------------------------------------------------------
#[test]
fn d17_no_document_typed_handler_or_requirement_kind_in_production() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            if [
                "TypedHandler::Requirement",
                "TypedHandler::Glossary",
                "TypedHandler::UiSpec",
                "TypedHandler::Rule",
                "TypedHandler::UiMockup",
                "TypedHandler::Audit",
                "TypedHandler::TechSpec",
                "TypedHandler::UseCase",
                "TypedHandler::TestCase",
                "TypedHandler::TestReport",
                "RequirementKind",
            ]
            .iter()
            .any(|needle| line.contains(needle))
            {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-17 document TypedHandler/RequirementKind in production code:\n{}",
        violations.join("\n")
    );
}

/// Whether `line` reads `record.<field>` (not e.g. `meta_record.specs`).
fn field_read(line: &str, field: &str) -> bool {
    let needle = format!("record.{field}");
    let bytes = line.as_bytes();
    let mut from = 0;
    while let Some(found) = line[from..].find(&needle) {
        let at = from + found;
        let before = at
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied();
        let boundary = before.is_none_or(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_'));
        if boundary {
            return true;
        }
        from = at + needle.len();
    }
    false
}

// D-18 ---------------------------------------------------------------------
#[test]
fn d18_document_renderers_do_not_read_typed_record_fields() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if !(rel.starts_with("src/browser/") || rel == "src/document.rs") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in production_lines_in(&content) {
            if [
                "story",
                "rationale",
                "applicability",
                "requirements",
                "audit_type",
                "definition",
                "view",
                "specs",
                "mockup_task",
            ]
            .iter()
            .any(|field| field_read(&line, field))
            {
                violations.push(format!("{}:{}: {}", rel, lineno, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-18 typed document field read in a document renderer:\n{}",
        violations.join("\n")
    );
}

// D-18 (description arm) ---------------------------------------------------
#[test]
fn d18_document_arms_do_not_read_record_description() {
    let mut violations = Vec::new();
    for path in src_files() {
        let rel = relative(&path);
        if !(rel.starts_with("src/browser/") || rel == "src/document.rs") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !production_line(&lines, index) || !field_read(line, "description") {
                continue;
            }
            let mut context = "";
            for previous in lines[..=index].iter().rev() {
                let trimmed = previous.trim();
                if trimmed.starts_with("fn ")
                    || trimmed.starts_with("Some(TypedHandler::")
                    || trimmed.starts_with("None => {")
                {
                    context = trimmed;
                    break;
                }
            }
            let backlog_or_decision = ["Task", "Milestone", "Release", "Decision"]
                .iter()
                .any(|name| context.contains(name));
            if !backlog_or_decision {
                violations.push(format!("{}:{}: {}", rel, index + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "D-18 document arm reads record.description:\n{}",
        violations.join("\n")
    );
}

/// Whether one source line belongs to production code, following the same
/// `#[cfg(test)]` skip rule as the other scanners.
fn production_line(lines: &[&str], index: usize) -> bool {
    let mut cursor = 0usize;
    while cursor < lines.len() {
        if lines[cursor].trim_start().starts_with("#[cfg(test)]") {
            let end = skip_cfg_test_item(lines, cursor + 1);
            if index >= cursor && index < end {
                return false;
            }
            cursor = end;
        } else {
            cursor += 1;
        }
    }
    true
}

// D-05 (TASK-268) ----------------------------------------------------------
#[test]
fn d05_presentation_table_holds_no_document_entry() {
    for slug in DOCUMENT_SLUGS {
        assert!(
            papyrus::browser::routes::PRESENTATION
                .iter()
                .all(|entry| entry.slug != slug),
            "PRESENTATION still holds a document entry: {slug}"
        );
    }
    for slug in DOCUMENT_SLUGS {
        assert!(
            papyrus::config::seed_document_type(slug).is_some(),
            "{slug} is a seed type"
        );
    }
}
