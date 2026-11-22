//! Duplicate-ID repair planning and application for `check --fix`.
//!
//! Planning is read-only: it orders each duplicate group by the keeper rule,
//! allocates the next free number per family and shape, and lists the inbound
//! references that the repair will leave untouched. Application writes only
//! through [`crate::record::store::reassign_id`].

use crate::check::findings::{DUPLICATE_ID, Finding, REPAIR_SKIPPED};
use crate::check::scan::{RecordMeta, Shape};
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::frontmatter::{self, FrontMatter};
use crate::record::identity;
use crate::record::scan::project_relative;
use serde_json::{Value, json};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Fields whose values may name a record that is about to be reassigned.
const REFERENCE_FIELDS: [&str; 5] = [
    "milestone",
    "release",
    "dependencies",
    "references",
    "documentation",
];

/// One scanned family, borrowed for planning only.
#[derive(Clone, Copy)]
pub struct FamilyView<'a> {
    pub name: &'a str,
    pub shape: Shape,
    pub records: &'a [RecordMeta],
}

/// One affected inbound reference, never rewritten.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    pub family: String,
    pub subject: String,
    pub path: String,
    pub field: String,
    pub value: String,
}

/// One planned or applied reassignment.
#[derive(Clone, Debug)]
pub struct Repair {
    pub family: String,
    pub old_id: String,
    pub old_path: String,
    pub old_abs: PathBuf,
    pub old_key: String,
    pub prefix: String,
    pub old_numbers: Vec<i64>,
    pub version: String,
    pub new_id: String,
    pub new_path: String,
    pub new_abs: PathBuf,
    pub references: Vec<Reference>,
}

/// The whole plan: repairs in keeper order plus every affected reference.
pub struct Plan {
    pub repairs: Vec<Repair>,
    pub references: Vec<Reference>,
}

/// Build the plan from the scan's findings. Only `duplicate-id` findings with
/// `fixable == true` are considered; a group is planned whole or not at all.
pub fn build_plan(
    ctx: &Context,
    cfg: &Config,
    findings: &[Finding],
    families: &[FamilyView<'_>],
) -> Plan {
    let mut allocator = Allocator::new(families);
    let mut repairs: Vec<Repair> = Vec::new();
    for finding in findings {
        if finding.kind != DUPLICATE_ID || !finding.fixable {
            continue;
        }
        let Some(name) = finding.family.as_deref() else {
            continue;
        };
        let Some(family) = families.iter().find(|family| family.name == name) else {
            continue;
        };
        let mut members: Vec<&RecordMeta> = family
            .records
            .iter()
            .filter(|record| finding.locations.contains(&record.rel))
            .collect();
        if members.len() < 2 {
            continue;
        }
        members.sort_by(|left, right| keeper_order(left, right));

        for member in members.iter().skip(1) {
            let group = if family.shape == Shape::Plain {
                None
            } else {
                match member.numbers.first() {
                    Some(first) => Some(*first),
                    None => continue,
                }
            };
            let number = allocator.next(family.name, &member.prefix, group);
            let padding = cfg.id_padding as usize;
            let new_id = match group {
                Some(first) => {
                    let group_id = identity::format_id(&member.prefix, first, padding);
                    identity::format_seq_id(&group_id, number, padding)
                }
                None => identity::format_id(&member.prefix, number, padding),
            };
            let new_abs = reassigned_path(&member.path, &new_id);
            repairs.push(Repair {
                family: family.name.to_string(),
                old_id: member.display.clone(),
                old_path: member.rel.clone(),
                old_abs: member.path.clone(),
                old_key: member.key.clone(),
                prefix: member.prefix.clone(),
                old_numbers: member.numbers.clone(),
                version: member.version.clone(),
                new_id,
                new_path: project_relative(&ctx.project_dir, &new_abs),
                new_abs,
                references: Vec::new(),
            });
        }
    }

    for repair in &mut repairs {
        repair.references = references_for(families, repair);
    }
    let mut references: Vec<Reference> = repairs
        .iter()
        .flat_map(|repair| repair.references.iter().cloned())
        .collect();
    references.sort_by(|left, right| reference_key(left).cmp(&reference_key(right)));
    references.dedup();

    Plan {
        repairs,
        references,
    }
}

/// Order a duplicate group: earliest valid `created_date`, then path. A
/// missing or invalid date sorts after every valid one.
fn keeper_order(left: &RecordMeta, right: &RecordMeta) -> Ordering {
    match (created_seconds(left), created_seconds(right)) {
        (Some(left_seconds), Some(right_seconds)) => left_seconds
            .cmp(&right_seconds)
            .then_with(|| left.rel.cmp(&right.rel)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.rel.cmp(&right.rel),
    }
}

fn created_seconds(record: &RecordMeta) -> Option<i64> {
    let frontmatter = record.frontmatter.as_ref()?;
    let value = super::scan::frontmatter_scalar(frontmatter, "created_date")?;
    frontmatter::timestamp_seconds(&value)
}

/// Observed numbers per family, configured prefix, and optional group.
struct Allocator {
    observed: BTreeMap<(String, String, Option<i64>), Vec<i64>>,
}

impl Allocator {
    fn new(families: &[FamilyView<'_>]) -> Self {
        let mut observed: BTreeMap<(String, String, Option<i64>), Vec<i64>> = BTreeMap::new();
        for family in families {
            for record in family.records {
                let (group, number) = match (family.shape, record.numbers.as_slice()) {
                    (Shape::Plain, [number, ..]) => (None, *number),
                    (_, [group, number, ..]) => (Some(*group), *number),
                    // A group-level record is not part of a member space.
                    _ => continue,
                };
                observed
                    .entry(allocation_key(family.name, &record.prefix, group))
                    .or_default()
                    .push(number);
            }
        }
        Self { observed }
    }

    fn next(&mut self, family: &str, prefix: &str, group: Option<i64>) -> i64 {
        let key = (family.to_string(), prefix.to_ascii_lowercase(), group);
        let observed = self.observed.entry(key).or_default();
        let number = identity::allocate_next(observed);
        observed.push(number);
        number
    }
}

fn allocation_key(family: &str, prefix: &str, group: Option<i64>) -> (String, String, Option<i64>) {
    (family.to_string(), prefix.to_ascii_lowercase(), group)
}

/// `<same directory>/<new id>.<same extension>`; a cosmetic slug is dropped.
fn reassigned_path(path: &Path, new_id: &str) -> PathBuf {
    let extension = path.extension().and_then(|value| value.to_str());
    let file_name = match extension {
        Some(extension) if !extension.is_empty() => format!("{}.{}", new_id, extension),
        _ => new_id.to_string(),
    };
    path.with_file_name(file_name)
}

fn references_for(families: &[FamilyView<'_>], repair: &Repair) -> Vec<Reference> {
    let dotted = repair.old_numbers.len() >= 2;
    let mut references: Vec<Reference> = Vec::new();
    for family in families {
        for record in family.records {
            if record.path == repair.old_abs {
                continue;
            }
            let Some(frontmatter) = record.frontmatter.as_ref() else {
                continue;
            };
            for field in REFERENCE_FIELDS {
                for value in reference_values(frontmatter, field) {
                    let Some(parts) =
                        identity::parse_id_parts(value.trim(), &repair.prefix, dotted)
                    else {
                        continue;
                    };
                    if parts.key(&repair.prefix) != repair.old_key {
                        continue;
                    }
                    references.push(Reference {
                        family: family.name.to_string(),
                        subject: record.display.clone(),
                        path: record.rel.clone(),
                        field: field.to_string(),
                        value,
                    });
                }
            }
        }
    }
    references.sort_by(|left, right| reference_key(left).cmp(&reference_key(right)));
    references.dedup();
    references
}

fn reference_values(frontmatter: &FrontMatter, field: &str) -> Vec<String> {
    let Some(value) = frontmatter
        .data
        .get(yaml_serde::Value::String(field.to_string()))
    else {
        return Vec::new();
    };
    let scalar = |value: &yaml_serde::Value| match value {
        yaml_serde::Value::String(value) => Some(value.clone()),
        yaml_serde::Value::Bool(value) => Some(value.to_string()),
        yaml_serde::Value::Number(value) => Some(value.to_string()),
        _ => None,
    };
    match value {
        yaml_serde::Value::Sequence(items) => items.iter().filter_map(scalar).collect(),
        yaml_serde::Value::Null => Vec::new(),
        other => scalar(other)
            .filter(|value| !value.trim().is_empty())
            .into_iter()
            .collect(),
    }
}

fn reference_key(reference: &Reference) -> (&str, &str, &str, &str, &str) {
    (
        &reference.family,
        &reference.subject,
        &reference.path,
        &reference.field,
        &reference.value,
    )
}

/// The plan block and prompt, written to stderr before reading input.
pub fn plan_lines(plan: &Plan) -> Vec<String> {
    let count = plan.repairs.len();
    let noun = if count == 1 {
        "reassignment"
    } else {
        "reassignments"
    };
    let mut lines = vec![format!("fix plan: {} {}", count, noun)];
    for repair in &plan.repairs {
        lines.push(format!(
            "reassign: {} {} {} -> {}",
            repair.family, repair.old_id, repair.old_path, repair.new_id
        ));
    }
    for reference in &plan.references {
        lines.push(reference_line(reference));
    }
    lines.push("apply repairs? [y/N]".to_string());
    lines
}

pub fn write_plan<W: Write>(writer: &mut W, plan: &Plan) -> Result<(), AppError> {
    for line in plan_lines(plan) {
        writeln!(writer, "{}", line).map_err(|error| AppError::Runtime(error.to_string()))?;
    }
    writer
        .flush()
        .map_err(|error| AppError::Runtime(error.to_string()))
}

/// Read one line and confirm only `y` or `yes` (trimmed, ASCII-lowercased).
pub fn confirm<R: BufRead>(reader: &mut R) -> Result<bool, AppError> {
    let mut answer = String::new();
    reader
        .read_line(&mut answer)
        .map_err(|error| AppError::Runtime(error.to_string()))?;
    Ok(is_confirmation(&answer))
}

fn is_confirmation(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Apply the plan through the single write path, re-checking each file's
/// version token immediately before the write. Skipped repairs become
/// `repair-skipped` findings; applied repairs are returned in plan order.
pub fn apply(
    ctx: &Context,
    cfg: &crate::config::Config,
    registry: &crate::registry::ArtifactRegistry,
    plan: &Plan,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Repair>, AppError> {
    let mut applied = Vec::new();
    for repair in &plan.repairs {
        let metadata = fs::metadata(&repair.old_abs).map_err(|error| {
            AppError::Runtime(format!(
                "cannot read record {}: {}",
                repair.old_abs.display(),
                error
            ))
        })?;
        if crate::record::store::version_token(&metadata) != repair.version {
            findings.push(skipped_finding(
                repair,
                "record changed since the scan; repair skipped".to_string(),
            ));
            continue;
        }
        if repair.new_abs.exists() {
            findings.push(skipped_finding(
                repair,
                format!(
                    "target path {} already exists; repair skipped",
                    repair.new_path
                ),
            ));
            continue;
        }
        let actual =
            crate::record::store::reassign_id(ctx, cfg, registry, &repair.old_abs, &repair.new_id)?;
        let mut applied_repair = repair.clone();
        applied_repair.new_abs = actual.clone();
        applied_repair.new_path = project_relative(&ctx.project_dir, &actual);
        applied.push(applied_repair);
    }
    Ok(applied)
}

fn skipped_finding(repair: &Repair, detail: String) -> Finding {
    Finding {
        kind: REPAIR_SKIPPED,
        family: Some(repair.family.clone()),
        subject: Some(repair.old_id.clone()),
        locations: vec![repair.old_path.clone()],
        detail,
        fixable: false,
    }
}

/// Text lines for the report: repaired lines in plan order, then reference
/// lines of applied repairs in reference order.
pub fn report_lines(applied: &[Repair]) -> Vec<String> {
    let mut lines: Vec<String> = applied
        .iter()
        .map(|repair| {
            format!(
                "repaired: {} {} -> {} ({} -> {})",
                repair.family, repair.old_id, repair.new_id, repair.old_path, repair.new_path
            )
        })
        .collect();
    let mut references: Vec<&Reference> = applied
        .iter()
        .flat_map(|repair| repair.references.iter())
        .collect();
    references.sort_by(|left, right| reference_key(left).cmp(&reference_key(right)));
    references.dedup_by(|left, right| reference_key(left) == reference_key(right));
    for reference in references {
        lines.push(reference_line(reference));
    }
    lines
}

fn reference_line(reference: &Reference) -> String {
    format!(
        "reference: {} {} ({}) {} = \"{}\"",
        reference.family, reference.subject, reference.path, reference.field, reference.value
    )
}

/// The JSON `repaired` entries, in plan order and the documented key order.
pub fn repaired_json(applied: &[Repair]) -> Vec<Value> {
    applied
        .iter()
        .map(|repair| {
            let references: Vec<Value> = repair
                .references
                .iter()
                .map(|reference| {
                    json!({
                        "family": reference.family,
                        "subject": reference.subject,
                        "path": reference.path,
                        "field": reference.field,
                        "value": reference.value,
                    })
                })
                .collect();
            json!({
                "kind": DUPLICATE_ID,
                "family": repair.family,
                "id": repair.old_id,
                "path": repair.old_path,
                "new_id": repair.new_id,
                "new_path": repair.new_path,
                "references": references,
            })
        })
        .collect()
}

/// Build the planning view for one scanned family.
pub fn family_view<'a>(name: &'a str, shape: Shape, records: &'a [RecordMeta]) -> FamilyView<'a> {
    FamilyView {
        name,
        shape,
        records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::frontmatter;
    use std::path::PathBuf;

    fn record(rel: &str, id: &str, created: Option<&str>, numbers: Vec<i64>) -> RecordMeta {
        let created_line = match created {
            Some(value) => format!("created_date: '{}'\n", value),
            None => String::new(),
        };
        let content = format!(
            "---\nid: {}\ntitle: T\n{}---\n\n## Description\n\nBody\n",
            id, created_line
        );
        let frontmatter = frontmatter::parse(&content).expect("frontmatter");
        RecordMeta {
            rel: rel.to_string(),
            path: PathBuf::from(rel),
            key: id.to_string(),
            display: id.to_string(),
            number_count: numbers.len(),
            numbers,
            prefix: "TASK".to_string(),
            version: "0.0.0".to_string(),
            frontmatter,
            mismatch: false,
            read_only: false,
            content,
        }
    }

    #[test]
    fn keeper_prefers_earliest_valid_date_then_path() {
        let records = [
            record(
                "backlog/tasks/TASK-001-copy.md",
                "TASK-001",
                Some("2026-02-01T00:00:00Z"),
                vec![1],
            ),
            record(
                "backlog/tasks/TASK-001.md",
                "TASK-001",
                Some("2026-01-01T00:00:00Z"),
                vec![1],
            ),
            record("backlog/tasks/TASK-001-late.md", "TASK-001", None, vec![1]),
        ];
        let mut members: Vec<&RecordMeta> = records.iter().collect();
        members.sort_by(|left, right| keeper_order(left, right));
        assert_eq!(members[0].rel, "backlog/tasks/TASK-001.md");
        assert_eq!(members[1].rel, "backlog/tasks/TASK-001-copy.md");
        assert_eq!(
            members[2].rel, "backlog/tasks/TASK-001-late.md",
            "a missing date sorts last"
        );
    }

    #[test]
    fn keeper_compares_offsets_as_instants_and_ties_on_path() {
        let records = [
            record(
                "b.md",
                "TASK-001",
                Some("2026-01-01T12:00:00+02:00"),
                vec![1],
            ),
            record("a.md", "TASK-001", Some("2026-01-01T10:00:00Z"), vec![1]),
            record("c.md", "TASK-001", Some("not a date"), vec![1]),
            record("d.md", "TASK-001", None, vec![1]),
        ];
        let mut members: Vec<&RecordMeta> = records.iter().collect();
        members.sort_by(|left, right| keeper_order(left, right));
        assert_eq!(members[0].rel, "a.md", "equal instants tie-break on path");
        assert_eq!(members[1].rel, "b.md");
        assert_eq!(members[2].rel, "c.md", "an invalid date sorts last");
        assert_eq!(members[3].rel, "d.md");
    }

    #[test]
    fn plain_and_member_allocations_advance_sequentially_in_one_run() {
        let records = [
            record("backlog/tasks/TASK-001.md", "TASK-001", None, vec![1]),
            record("backlog/tasks/TASK-001-copy.md", "TASK-001", None, vec![1]),
            record(
                "docs/requirements/business/REQ-001.001.md",
                "REQ-001.001",
                None,
                vec![1, 1],
            ),
            record(
                "docs/requirements/business/REQ-001.001-copy.md",
                "REQ-001.001",
                None,
                vec![1, 1],
            ),
        ];
        let task_records = records[..2].to_vec();
        let mut member_records = records[2..].to_vec();
        for record in &mut member_records {
            record.prefix = "REQ".to_string();
        }
        let task_view = family_view("task", Shape::Plain, &task_records);
        let member_view = family_view("req", Shape::Grouped, &member_records);
        let mut allocator = Allocator::new(&[task_view, member_view]);

        assert_eq!(allocator.next("task", "TASK", None), 2);
        assert_eq!(allocator.next("task", "TASK", None), 3);
        assert_eq!(allocator.next("req", "REQ", Some(1)), 2);
        assert_eq!(allocator.next("req", "REQ", Some(1)), 3);

        let padding = 3usize;
        let group = identity::format_id("REQ", 1, padding);
        assert_eq!(identity::format_seq_id(&group, 2, padding), "REQ-001.002");
    }

    #[test]
    fn confirmation_accepts_only_y_and_yes() {
        for answer in ["y", "Y", "yes", "YES", " yes \n"] {
            assert!(is_confirmation(answer), "{:?} confirms", answer);
        }
        for answer in ["n", "no", "", "\n", "yeah", "yep"] {
            assert!(!is_confirmation(answer), "{:?} cancels", answer);
        }
    }

    #[test]
    fn confirm_treats_end_of_input_as_cancel() {
        let mut empty = std::io::Cursor::new(Vec::<u8>::new());
        assert!(!confirm(&mut empty).unwrap());
        let mut yes = std::io::Cursor::new(b"yes\n".to_vec());
        assert!(confirm(&mut yes).unwrap());
    }

    fn write_file(path: &Path, content: &str) {
        use std::io::Write;
        let mut file = std::fs::File::create(path).expect("create test file");
        file.write_all(content.as_bytes()).expect("write test file");
    }

    fn tmp_project(name: &str) -> (Context, PathBuf) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("check-fix-unit-{}", name));
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
        (ctx, dir)
    }

    fn scanned_task(ctx: &Context, name: &str) -> RecordMeta {
        let path = ctx.project_dir.join("backlog/tasks").join(name);
        let content = "---\nid: TASK-001\ntitle: T\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\n---\n\n## Description\n\nBody\n";
        write_file(&path, content);
        let frontmatter = frontmatter::parse(content).unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        RecordMeta {
            rel: project_relative(&ctx.project_dir, &path),
            path,
            key: "TASK-001".to_string(),
            display: "TASK-001".to_string(),
            number_count: 1,
            numbers: vec![1],
            prefix: "TASK".to_string(),
            version: crate::record::store::version_token(&metadata),
            frontmatter,
            mismatch: false,
            read_only: false,
            content: content.to_string(),
        }
    }

    fn duplicate_finding(records: &[RecordMeta]) -> Finding {
        Finding {
            kind: DUPLICATE_ID,
            family: Some("task".to_string()),
            subject: Some("TASK-001".to_string()),
            locations: records.iter().map(|record| record.rel.clone()).collect(),
            detail: "2 records share this ID".to_string(),
            fixable: true,
        }
    }

    #[test]
    fn a_changed_file_is_skipped_and_reported() {
        let (ctx, _dir) = tmp_project("changed");
        let records = vec![
            scanned_task(&ctx, "TASK-001.md"),
            scanned_task(&ctx, "TASK-001-copy.md"),
        ];
        let finding = duplicate_finding(&records);
        let view = family_view("task", Shape::Plain, &records);
        let mut plan = build_plan(&ctx, &Config::default(), &[finding], &[view]);
        assert_eq!(plan.repairs.len(), 1);
        plan.repairs[0].version = "0.0.0".to_string();

        let mut findings = Vec::new();
        let applied = apply(
            &ctx,
            &Config::default(),
            &crate::registry::ArtifactRegistry::build(&Config::default()).unwrap(),
            &plan,
            &mut findings,
        )
        .unwrap();
        assert!(applied.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, REPAIR_SKIPPED);
        assert_eq!(
            findings[0].detail,
            "record changed since the scan; repair skipped"
        );
        assert!(records[0].path.exists());
        assert!(records[1].path.exists());
        let _ = std::fs::remove_dir_all(&ctx.project_dir);
    }

    #[test]
    fn an_appearing_target_path_is_skipped_and_reported() {
        let (ctx, _dir) = tmp_project("target");
        let records = vec![
            scanned_task(&ctx, "TASK-001.md"),
            scanned_task(&ctx, "TASK-001-copy.md"),
        ];
        let finding = duplicate_finding(&records);
        let view = family_view("task", Shape::Plain, &records);
        let plan = build_plan(&ctx, &Config::default(), &[finding], &[view]);
        let repair = &plan.repairs[0];
        write_file(&repair.new_abs, "occupied");

        let mut findings = Vec::new();
        let applied = apply(
            &ctx,
            &Config::default(),
            &crate::registry::ArtifactRegistry::build(&Config::default()).unwrap(),
            &plan,
            &mut findings,
        )
        .unwrap();
        assert!(applied.is_empty());
        assert_eq!(findings[0].kind, REPAIR_SKIPPED);
        assert_eq!(
            findings[0].detail,
            format!(
                "target path {} already exists; repair skipped",
                repair.new_path
            )
        );
        assert_eq!(
            std::fs::read_to_string(&repair.new_abs).unwrap(),
            "occupied"
        );
        assert!(records[0].path.exists());
        let _ = std::fs::remove_dir_all(&ctx.project_dir);
    }
}
