//! Project diagnostics (`papyrus check`), read-only by default with an opt-in
//! duplicate-ID repair (`--fix`, `--force`).

mod findings;
mod fix;
mod scan;

use crate::CommandOutput;
use crate::context::Context;
use crate::error::AppError;
use crate::registry::ArtifactRegistry;
use findings::Finding;
use std::io::{BufRead, IsTerminal, Write};

const ALL_CHECKS: [&str; 8] = [
    "duplicate-id",
    "duplicate-id:document-types",
    "self-dependency",
    "dependency-cycle",
    "dangling-milestone",
    "dangling-release",
    "frontmatter-conflict",
    "frontmatter-conflict:document-records",
];

const DOCUMENT_CHECKS: [&str; 2] = [
    "duplicate-id:document-types",
    "frontmatter-conflict:document-records",
];

/// Run the diagnostics on the process stdin/stderr.
pub fn run(ctx: &Context, fix: bool, force: bool) -> Result<CommandOutput, AppError> {
    let stdin = std::io::stdin();
    let stdin_is_terminal = stdin.is_terminal();
    let mut reader = stdin.lock();
    let mut writer = std::io::stderr();
    run_with_io(ctx, fix, force, stdin_is_terminal, &mut reader, &mut writer)
}

/// Run the diagnostics with an injected reader/writer for the repair prompt.
pub fn run_with_io<R: BufRead, W: Write>(
    ctx: &Context,
    fix: bool,
    force: bool,
    stdin_is_terminal: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<CommandOutput, AppError> {
    let mut findings: Vec<Finding> = Vec::new();
    let mut skipped: Vec<&'static str> = Vec::new();

    let cfg =
        match crate::commands::load_effective_config_tolerant(&ctx.config_path, &ctx.project_dir) {
            Ok(cfg) => cfg,
            Err(error) => {
                findings.push(findings::config_finding(ctx, error.to_string()));
                skipped.extend(ALL_CHECKS);
                return Ok(output(ctx, findings, skipped, &[]));
            }
        };

    let mut valid = true;
    if let Err(lines) = cfg.validate(Some(&ctx.project_dir)) {
        valid = false;
        skipped.extend(DOCUMENT_CHECKS);
        for line in lines.lines().filter(|line| !line.trim().is_empty()) {
            findings.push(findings::config_finding(ctx, line.to_string()));
        }
    }

    let static_descriptors = crate::record::descriptor::descriptors();
    let static_scanned =
        scan::collect(ctx, &cfg, scan::static_families(&cfg, &static_descriptors))?;
    findings.extend(scan::duplicate_ids(&static_scanned));
    findings.extend(scan::self_dependencies(&static_scanned));
    findings.extend(scan::dependency_cycles(&static_scanned));
    findings.extend(scan::dangling_references(&static_scanned));
    findings.extend(scan::frontmatter_conflicts(&static_scanned));

    let mut registry: Option<ArtifactRegistry> = None;
    if valid {
        match ArtifactRegistry::build(&cfg) {
            Ok(built) => registry = Some(built),
            Err(error) => {
                findings.push(findings::config_finding(ctx, error.to_string()));
                skipped.extend(DOCUMENT_CHECKS);
            }
        }
    }
    let document_scanned = match registry.as_ref() {
        Some(registry) => {
            let document_scanned =
                scan::collect(ctx, &cfg, scan::document_families(registry, &cfg))?;
            findings.extend(scan::duplicate_ids(&document_scanned));
            findings.extend(scan::frontmatter_conflicts(&document_scanned));
            document_scanned
        }
        None => Vec::new(),
    };

    findings::sort(&mut findings);
    if !fix {
        return Ok(output(ctx, findings, skipped, &[]));
    }

    let mut family_views: Vec<fix::FamilyView<'_>> = Vec::new();
    for scanned in &static_scanned {
        family_views.push(fix::family_view(
            &scanned.family.name,
            scanned.family.shape,
            &scanned.records,
        ));
    }
    for scanned in &document_scanned {
        family_views.push(fix::family_view(
            &scanned.family.name,
            scanned.family.shape,
            &scanned.records,
        ));
    }

    let plan = fix::build_plan(ctx, &cfg, &findings, &family_views);
    if plan.repairs.is_empty() {
        return Ok(output(ctx, findings, skipped, &[]));
    }

    if !force {
        if !stdin_is_terminal {
            writeln!(
                writer,
                "error: --fix requires --force when standard input is not a terminal"
            )
            .map_err(|error| AppError::Runtime(error.to_string()))?;
            writer
                .flush()
                .map_err(|error| AppError::Runtime(error.to_string()))?;
            return Ok(CommandOutput {
                text: String::new(),
                json: None,
                exit_code: 2,
            });
        }
        fix::write_plan(writer, &plan)?;
        if !fix::confirm(reader)? {
            writeln!(writer, "repair cancelled; nothing written")
                .map_err(|error| AppError::Runtime(error.to_string()))?;
            writer
                .flush()
                .map_err(|error| AppError::Runtime(error.to_string()))?;
            return Ok(output(ctx, findings, skipped, &[]));
        }
    }

    let applied = fix::apply(ctx, &plan, &mut findings)?;
    Ok(output(ctx, findings, skipped, &applied))
}

fn output(
    ctx: &Context,
    mut findings: Vec<Finding>,
    skipped: Vec<&'static str>,
    applied: &[fix::Repair],
) -> CommandOutput {
    findings::sort(&mut findings);
    let report_lines = fix::report_lines(applied);
    let text = findings::render_text(&findings, &skipped, &report_lines);
    let json = if ctx.json {
        Some(findings::to_json(
            &findings,
            &skipped,
            fix::repaired_json(applied),
        ))
    } else {
        None
    };
    let exit_code = if findings.is_empty() { 0 } else { 4 };
    CommandOutput {
        text,
        json,
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn write_file(path: &std::path::Path, content: &str) {
        use std::io::Write;
        let mut file = std::fs::File::create(path).expect("create test file");
        file.write_all(content.as_bytes()).expect("write test file");
    }

    fn project(name: &str) -> (Context, PathBuf) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("check-prompt-{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
        write_file(&dir.join("papyrus.config.yml"), "project_name: Test\n");
        let dir = std::fs::canonicalize(&dir).unwrap();
        let ctx = Context {
            project_dir: dir.clone(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        };
        let content = "---\nid: TASK-001\ntitle: T\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: ToDo\n---\n\n## Description\n\nBody\n";
        write_file(&dir.join("backlog/tasks/TASK-001.md"), content);
        write_file(&dir.join("backlog/tasks/TASK-001-copy.md"), content);
        (ctx, dir)
    }

    #[test]
    fn a_confirmed_prompt_applies_the_plan_and_keeps_stdout_for_the_report() {
        let (ctx, dir) = project("confirmed");
        let mut reader = Cursor::new(b"yes\n".to_vec());
        let mut writer = Vec::new();
        let output = run_with_io(&ctx, true, false, true, &mut reader, &mut writer).unwrap();

        assert_eq!(output.exit_code, 4);
        assert!(
            output.text.contains("repaired: task TASK-001 -> TASK-002"),
            "{}",
            output.text
        );
        assert!(
            !output.text.contains("fix plan:"),
            "stdout carries only the report"
        );
        let prompt = String::from_utf8(writer).unwrap();
        assert!(prompt.contains("fix plan: 1 reassignment"), "{}", prompt);
        assert!(prompt.contains("apply repairs? [y/N]"), "{}", prompt);
        assert!(dir.join("backlog/tasks/TASK-002.md").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_cancelled_prompt_writes_nothing_and_keeps_the_report() {
        let (ctx, dir) = project("cancelled");
        let before = std::fs::read(dir.join("backlog/tasks/TASK-001.md")).unwrap();
        let mut reader = Cursor::new(b"n\n".to_vec());
        let mut writer = Vec::new();
        let output = run_with_io(&ctx, true, false, true, &mut reader, &mut writer).unwrap();

        assert_eq!(output.exit_code, 4);
        assert!(output.text.contains("duplicate-id: task TASK-001"));
        assert!(!output.text.contains("repaired:"));
        let prompt = String::from_utf8(writer).unwrap();
        assert!(
            prompt.contains("repair cancelled; nothing written"),
            "{}",
            prompt
        );
        assert_eq!(
            std::fs::read(dir.join("backlog/tasks/TASK-001.md")).unwrap(),
            before
        );
        assert!(!dir.join("backlog/tasks/TASK-002.md").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_non_terminal_plan_is_refused_before_the_prompt() {
        let (ctx, dir) = project("non-terminal");
        let mut reader = Cursor::new(b"y\n".to_vec());
        let mut writer = Vec::new();
        let output = run_with_io(&ctx, true, false, false, &mut reader, &mut writer).unwrap();

        assert_eq!(output.exit_code, 2);
        assert!(output.text.is_empty());
        assert!(output.json.is_none());
        let stderr = String::from_utf8(writer).unwrap();
        assert!(
            stderr.contains("--fix requires --force when standard input is not a terminal"),
            "{}",
            stderr
        );
        assert!(dir.join("backlog/tasks/TASK-001.md").exists());
        assert!(!dir.join("backlog/tasks/TASK-002.md").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_clean_corpus_never_prompts_even_without_a_terminal() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("check-prompt-clean");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_file(&dir.join("papyrus.config.yml"), "project_name: Test\n");
        let dir = std::fs::canonicalize(&dir).unwrap();
        let ctx = Context {
            project_dir: dir.clone(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        };
        let mut reader = Cursor::new(Vec::<u8>::new());
        let mut writer = Vec::new();
        let output = run_with_io(&ctx, true, false, false, &mut reader, &mut writer).unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.text, "no findings");
        assert!(writer.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
