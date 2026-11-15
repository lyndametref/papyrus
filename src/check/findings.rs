//! The stable finding model and the two output forms.

use crate::context::Context;
use crate::record::scan::project_relative;
use serde_json::{Value, json};
use std::cmp::Ordering;

pub const CONFIG_INVALID: &str = "config-invalid";
pub const DANGLING_MILESTONE: &str = "dangling-milestone";
pub const DANGLING_RELEASE: &str = "dangling-release";
pub const DEPENDENCY_CYCLE: &str = "dependency-cycle";
pub const DUPLICATE_ID: &str = "duplicate-id";
pub const FRONTMATTER_CONFLICT: &str = "frontmatter-conflict";
pub const SELF_DEPENDENCY: &str = "self-dependency";

/// One detected problem. `kind` is a stable public value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub kind: &'static str,
    pub family: Option<String>,
    pub subject: Option<String>,
    pub locations: Vec<String>,
    pub detail: String,
    pub fixable: bool,
}

/// Sort by `(kind, family, subject, locations)`, absent values first.
pub fn sort(findings: &mut [Finding]) {
    findings.sort_by(compare);
}

fn compare(left: &Finding, right: &Finding) -> Ordering {
    left.kind
        .cmp(right.kind)
        .then_with(|| compare_optional(&left.family, &right.family))
        .then_with(|| compare_optional(&left.subject, &right.subject))
        .then_with(|| left.locations.cmp(&right.locations))
}

fn compare_optional(left: &Option<String>, right: &Option<String>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => left.cmp(right),
    }
}

/// Render the plain report: one line per finding, then one skipped line.
pub fn render_text(findings: &[Finding], skipped: &[&'static str]) -> String {
    let mut lines: Vec<String> = Vec::new();
    if findings.is_empty() {
        lines.push("no findings".to_string());
    }
    for finding in findings {
        let mut line = format!("{}: ", finding.kind);
        if let Some(family) = &finding.family {
            line.push_str(family);
            line.push(' ');
        }
        if let Some(subject) = &finding.subject {
            line.push_str(subject);
            line.push(' ');
        }
        line.push_str("— ");
        line.push_str(&finding.detail);
        line.push_str(" (");
        line.push_str(&finding.locations.join(", "));
        line.push(')');
        lines.push(line);
    }
    if !skipped.is_empty() {
        lines.push(format!("skipped: {}", skipped.join(", ")));
    }
    lines.join("\n")
}

/// Render the machine-readable report. `repaired` stays empty for this run.
pub fn to_json(findings: &[Finding], skipped: &[&'static str]) -> Value {
    let findings: Vec<Value> = findings
        .iter()
        .map(|finding| {
            json!({
                "kind": finding.kind,
                "family": finding.family,
                "subject": finding.subject,
                "locations": finding.locations,
                "detail": finding.detail,
                "fixable": finding.fixable,
            })
        })
        .collect();
    json!({
        "findings": findings,
        "skipped": skipped,
        "repaired": Vec::<Value>::new(),
    })
}

/// One configuration finding; the detail is the diagnostic line verbatim.
pub fn config_finding(ctx: &Context, detail: String) -> Finding {
    Finding {
        kind: CONFIG_INVALID,
        family: None,
        subject: None,
        locations: vec![project_relative(&ctx.project_dir, &ctx.config_path)],
        detail,
        fixable: false,
    }
}
