//! Read-only project diagnostics (`papyrus check`).

mod findings;
mod scan;

use crate::CommandOutput;
use crate::context::Context;
use crate::error::AppError;
use crate::registry::ArtifactRegistry;
use findings::Finding;

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

/// Run the read-only diagnostics and build the report.
pub fn run(ctx: &Context) -> Result<CommandOutput, AppError> {
    let mut findings: Vec<Finding> = Vec::new();
    let mut skipped: Vec<&'static str> = Vec::new();

    let cfg =
        match crate::commands::load_effective_config_tolerant(&ctx.config_path, &ctx.project_dir) {
            Ok(cfg) => cfg,
            Err(error) => {
                findings.push(findings::config_finding(ctx, error.to_string()));
                skipped.extend(ALL_CHECKS);
                return Ok(output(ctx, findings, skipped));
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

    let static_scanned = scan::collect(ctx, &cfg, scan::static_families(&cfg))?;
    findings.extend(scan::duplicate_ids(&static_scanned));
    findings.extend(scan::self_dependencies(&static_scanned));
    findings.extend(scan::dependency_cycles(&static_scanned));
    findings.extend(scan::dangling_references(&static_scanned));
    findings.extend(scan::frontmatter_conflicts(&static_scanned));

    if valid {
        match ArtifactRegistry::build(&cfg) {
            Ok(registry) => {
                let document_scanned =
                    scan::collect(ctx, &cfg, scan::document_families(&registry, &cfg))?;
                findings.extend(scan::duplicate_ids(&document_scanned));
                findings.extend(scan::frontmatter_conflicts(&document_scanned));
            }
            Err(error) => {
                findings.push(findings::config_finding(ctx, error.to_string()));
                skipped.extend(DOCUMENT_CHECKS);
            }
        }
    }

    Ok(output(ctx, findings, skipped))
}

fn output(ctx: &Context, mut findings: Vec<Finding>, skipped: Vec<&'static str>) -> CommandOutput {
    findings::sort(&mut findings);
    let text = findings::render_text(&findings, &skipped);
    let json = if ctx.json {
        Some(findings::to_json(&findings, &skipped))
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
