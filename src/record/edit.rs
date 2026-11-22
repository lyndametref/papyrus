//! Type-specific browser patch builders.
//!
//! The browser never assembles a whole record file. Each form edit is turned
//! into one [`PatchOp`] list here; the browser applies that list through
//! [`apply`], which delegates to the single mutation API
//! [`crate::record::patch::apply_patch`].

use crate::error::AppError;
use crate::record::document::{self, Document, Section};
use crate::record::patch::{PatchOp, apply_patch};

/// A change to an editable multi-line field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldChange {
    Keep,
    Clear,
    Set(String),
}

fn title_ops(doc: &Document, title: &str) -> Vec<PatchOp> {
    let (_, current) = document::heading_id_title(doc);
    if title == current {
        return Vec::new();
    }
    let mut ops = Vec::new();
    if doc.front_matter.is_some() {
        ops.push(PatchOp::SetFrontMatter("title".into(), title.to_string()));
    }
    ops.push(PatchOp::SetHeadingTitle(title.to_string()));
    ops
}

fn trailing_empty_lines(body: &[String]) -> usize {
    body.iter().rev().take_while(|line| line.is_empty()).count()
}

fn known_index(heading: &str, known: &[&str]) -> Option<usize> {
    known.iter().position(|h| *h == heading)
}

fn set_section(sections: &mut Vec<Section>, heading: &str, value: Option<String>, known: &[&str]) {
    let existing = sections.iter().position(|s| s.heading == heading);
    match (existing, value) {
        (Some(index), Some(value)) => {
            let trailing = trailing_empty_lines(&sections[index].body);
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            for _ in 0..trailing {
                body.push(String::new());
            }
            sections[index].body = body;
        }
        (Some(index), None) => {
            sections.remove(index);
        }
        (None, Some(value)) => {
            let order = known_index(heading, known).unwrap_or(usize::MAX);
            let mut position = None;
            for (index, section) in sections.iter().enumerate() {
                if let Some(section_order) = known_index(&section.heading, known)
                    && section_order < order
                {
                    position = Some(index + 1);
                }
            }
            if position.is_none() {
                for (index, section) in sections.iter().enumerate() {
                    if let Some(section_order) = known_index(&section.heading, known)
                        && section_order > order
                    {
                        position = Some(index);
                        break;
                    }
                }
            }
            let position = position.unwrap_or(sections.len());
            let mut body: Vec<String> = value.split('\n').map(|line| line.to_string()).collect();
            if position < sections.len() {
                body.push(String::new());
            }
            sections.insert(
                position,
                Section {
                    heading: heading.to_string(),
                    body,
                },
            );
        }
        (None, None) => {}
    }
}

fn section_ops(doc: &Document, final_sections: &[Section]) -> Vec<PatchOp> {
    let orig = &doc.sections;
    let inserted = final_sections
        .iter()
        .any(|s| !orig.iter().any(|o| o.heading == s.heading));
    let mut ops = Vec::new();
    if inserted {
        for o in orig {
            ops.push(PatchOp::ClearSection(o.heading.clone()));
        }
        for s in final_sections {
            ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n")));
        }
        return ops;
    }
    for o in orig {
        if !final_sections.iter().any(|s| s.heading == o.heading) {
            ops.push(PatchOp::ClearSection(o.heading.clone()));
        }
    }
    for s in final_sections {
        match orig.iter().find(|o| o.heading == s.heading) {
            None => ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n"))),
            Some(o) if o.body != s.body => {
                ops.push(PatchOp::SetSection(s.heading.clone(), s.body.join("\n")));
            }
            Some(_) => {}
        }
    }
    ops
}

fn finish(doc: &Document, mut ops: Vec<PatchOp>, final_sections: &[Section]) -> Vec<PatchOp> {
    ops.extend(section_ops(doc, final_sections));
    ops
}

/// Apply one browser patch list and refresh `updated_date` only when the
/// content actually changed. A no-op patch returns the source byte-identical.
pub fn apply(doc: &Document, ops: &[PatchOp]) -> Result<String, AppError> {
    if ops.is_empty() {
        return Ok(doc.source.clone());
    }
    let base = apply_patch(doc, ops)?;
    if base == doc.source || doc.front_matter.is_none() {
        return Ok(base);
    }
    let mut dated = ops.to_vec();
    dated.push(PatchOp::SetFrontMatter(
        "updated_date".into(),
        crate::record::frontmatter::utc_now(),
    ));
    apply_patch(doc, &dated)
}

/// One submitted field value for a document edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentFieldChange {
    /// Leave the stored value unchanged.
    Keep,
    /// Clear an optional field.
    Clear,
    /// Replace a scalar or multi-line value.
    Set(String),
    /// Replace a list value, in submission order.
    List(Vec<String>),
}

/// Build the one patch for a Markdown document edit from the record's field
/// map and the submitted values (TECH-040 section 7).
///
/// The field-to-`PatchOp` mapping is the shared one in [`crate::document`];
/// this helper only pairs a resolved field with its submitted change and adds
/// the general title, labels, and references operations.
pub fn document_edit_ops(
    doc: &Document,
    title: &str,
    fields: &[(
        &crate::record::field_map::ResolvedField,
        DocumentFieldChange,
    )],
    labels: Option<&[String]>,
    references: Option<&[String]>,
) -> Vec<PatchOp> {
    let mut ops = title_ops(doc, title);
    if let Some(front_matter) = &doc.front_matter {
        if let Some(labels) = labels
            && (!labels.is_empty()
                || crate::record::frontmatter::contains_key(&front_matter.raw, "labels"))
        {
            ops.push(PatchOp::SetFrontMatterValue(
                "labels".into(),
                crate::record::frontmatter::FrontMatterValue::Sequence(labels.to_vec()),
            ));
        }
        if let Some(references) = references
            && (!references.is_empty()
                || crate::record::frontmatter::contains_key(&front_matter.raw, "references"))
        {
            ops.push(PatchOp::SetFrontMatterValue(
                "references".into(),
                crate::record::frontmatter::FrontMatterValue::Sequence(references.to_vec()),
            ));
        }
    }
    for (field, change) in fields {
        let heading = field.heading.as_deref();
        let frontmatter_key = field.frontmatter_key.as_deref();
        match change {
            DocumentFieldChange::Keep => {}
            DocumentFieldChange::Clear => {
                crate::document::clear_field_ops(&mut ops, heading, frontmatter_key);
            }
            DocumentFieldChange::Set(value) => {
                crate::document::set_scalar_ops(&mut ops, heading, frontmatter_key, value);
            }
            DocumentFieldChange::List(values) => {
                crate::document::list_field_ops(&mut ops, heading, frontmatter_key, values);
            }
        }
    }
    ops
}

const TASK_KNOWN: &[&str] = &[
    "## Description",
    "## Status",
    "## Milestone",
    "## Assignee",
    "## Labels",
    "## Dependencies",
    "## Acceptance Criteria",
    "## Comments",
];
const MILESTONE_KNOWN: &[&str] = &["## Description", "## Release"];
const RELEASE_KNOWN: &[&str] = &["## Description"];
const DECISION_KNOWN: &[&str] = &["## Status", "## Decision", "## Context", "## Consequences"];
/// Patch list for one status-only task change from the backlog board.
///
/// The board never edits any other field. A record with front matter keeps its
/// `status` key; a legacy record without front matter uses its `## Status`
/// section. The returned list always contains exactly one operation.
pub fn task_status_ops(doc: &Document, status: &str) -> Vec<PatchOp> {
    if doc.front_matter.is_some() {
        vec![PatchOp::SetFrontMatter("status".into(), status.to_string())]
    } else {
        vec![PatchOp::SetSection("## Status".into(), status.to_string())]
    }
}

#[allow(clippy::too_many_arguments)]
pub fn task_ops(
    doc: &Document,
    title: &str,
    description: FieldChange,
    status: &str,
    milestone: FieldChange,
    assignee: FieldChange,
    labels: Vec<String>,
    dependencies: Vec<String>,
    acceptance_criteria: Vec<String>,
) -> Vec<PatchOp> {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops = title_ops(doc, title);
    if has_fm {
        fm_ops.push(PatchOp::SetFrontMatter("status".into(), status.to_string()));
        match &milestone {
            FieldChange::Clear => fm_ops.push(PatchOp::SetFrontMatterValue(
                "milestone".into(),
                crate::record::frontmatter::FrontMatterValue::Null,
            )),
            FieldChange::Set(value) => {
                fm_ops.push(PatchOp::SetFrontMatter("milestone".into(), value.clone()));
            }
            FieldChange::Keep => {}
        }
        match &assignee {
            FieldChange::Clear => {
                fm_ops.push(PatchOp::SetFrontMatterValue(
                    "assignee".into(),
                    crate::record::frontmatter::FrontMatterValue::Sequence(Vec::new()),
                ));
            }
            FieldChange::Set(value) => {
                fm_ops.push(PatchOp::SetFrontMatterValue(
                    "assignee".into(),
                    crate::record::frontmatter::FrontMatterValue::Sequence(vec![value.clone()]),
                ));
            }
            FieldChange::Keep => {}
        }
        fm_ops.push(PatchOp::SetFrontMatterValue(
            "labels".into(),
            crate::record::frontmatter::FrontMatterValue::Sequence(labels.clone()),
        ));
        fm_ops.push(PatchOp::SetFrontMatterValue(
            "dependencies".into(),
            crate::record::frontmatter::FrontMatterValue::Sequence(dependencies.clone()),
        ));
    }
    let mut sections = doc.sections.clone();
    set_section(
        &mut sections,
        "## Status",
        Some(status.to_string()),
        TASK_KNOWN,
    );
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), TASK_KNOWN);
        }
    }
    match milestone {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Milestone", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Milestone", Some(value), TASK_KNOWN);
        }
    }
    match assignee {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Assignee", None, TASK_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Assignee", Some(value), TASK_KNOWN);
        }
    }
    if labels.is_empty() {
        set_section(&mut sections, "## Labels", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Labels",
            Some(render_list_items(&labels, "-")),
            TASK_KNOWN,
        );
    }
    if dependencies.is_empty() {
        set_section(&mut sections, "## Dependencies", None, TASK_KNOWN);
    } else {
        set_section(
            &mut sections,
            "## Dependencies",
            Some(render_list_items(&dependencies, "-")),
            TASK_KNOWN,
        );
    }
    if acceptance_criteria.is_empty() {
        set_section(&mut sections, "## Acceptance Criteria", None, TASK_KNOWN);
    } else {
        let body = acceptance_criteria
            .iter()
            .map(|v| format!("- [ ] {}", v))
            .collect::<Vec<_>>()
            .join("\n");
        set_section(
            &mut sections,
            "## Acceptance Criteria",
            Some(body),
            TASK_KNOWN,
        );
    }
    finish(doc, fm_ops, &sections)
}

pub fn milestone_ops(
    doc: &Document,
    title: &str,
    description: FieldChange,
    release: FieldChange,
) -> Vec<PatchOp> {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops = title_ops(doc, title);
    if has_fm {
        match &release {
            FieldChange::Clear => fm_ops.push(PatchOp::SetFrontMatterValue(
                "release".into(),
                crate::record::frontmatter::FrontMatterValue::Null,
            )),
            FieldChange::Set(value) => {
                fm_ops.push(PatchOp::SetFrontMatter("release".into(), value.clone()));
            }
            FieldChange::Keep => {}
        }
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => {
            set_section(
                &mut sections,
                "## Description",
                Some(value),
                MILESTONE_KNOWN,
            );
        }
    }
    match release {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Release", None, MILESTONE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Release", Some(value), MILESTONE_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
}

pub fn release_ops(doc: &Document, title: &str, description: FieldChange) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, RELEASE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), RELEASE_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
}

pub fn decision_ops(
    doc: &Document,
    title: &str,
    status: &str,
    decision: FieldChange,
    context: FieldChange,
    consequences: FieldChange,
) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    set_section(
        &mut sections,
        "## Status",
        Some(status.to_string()),
        DECISION_KNOWN,
    );
    match decision {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Decision", None, DECISION_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Decision", Some(value), DECISION_KNOWN);
        }
    }
    match context {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Context", None, DECISION_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Context", Some(value), DECISION_KNOWN);
        }
    }
    match consequences {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Consequences", None, DECISION_KNOWN),
        FieldChange::Set(value) => {
            set_section(
                &mut sections,
                "## Consequences",
                Some(value),
                DECISION_KNOWN,
            );
        }
    }
    finish(doc, fm_ops, &sections)
}
fn render_list_items(items: &[String], prefix: &str) -> String {
    items
        .iter()
        .map(|v| format!("{} {}", prefix, v))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRONT_MATTER_TASK: &str = "---\nid: TASK-001\ntitle: One\ntype: task\nstatus: ToDo\nupdated_date: 2020-01-01\n---\n# TASK-001 One\n\n## Description\n\nBody\n";

    const SECTION_TASK: &str = "# TASK-001 One\n\n## Description\n\nBody\n\n## Status\n\nToDo\n";

    #[test]
    fn task_status_ops_uses_front_matter_when_present() {
        let doc = Document::parse(FRONT_MATTER_TASK).unwrap();
        let ops = task_status_ops(&doc, "Refined");
        assert_eq!(ops.len(), 1);
        assert!(matches!(
            &ops[0],
            PatchOp::SetFrontMatter(key, value) if key == "status" && value == "Refined"
        ));
        let output = apply(&doc, &ops).unwrap();
        assert!(output.contains("status: Refined"));
        assert!(!output.contains("\n## Status\n"));
    }

    #[test]
    fn task_status_ops_uses_status_section_without_front_matter() {
        let doc = Document::parse(SECTION_TASK).unwrap();
        let ops = task_status_ops(&doc, "Done");
        assert_eq!(ops.len(), 1);
        assert!(matches!(
            &ops[0],
            PatchOp::SetSection(heading, value) if heading == "## Status" && value == "Done"
        ));
        let output = apply(&doc, &ops).unwrap();
        assert!(output.contains("## Status"));
        assert!(output.contains("Done"));
        assert!(!output.contains("ToDo"));
        assert!(output.find("## Description").unwrap() < output.find("## Status").unwrap());
    }

    #[test]
    fn task_status_ops_changes_only_status_and_updated_date() {
        let doc = Document::parse(FRONT_MATTER_TASK).unwrap();
        let output = apply(&doc, &task_status_ops(&doc, "Implemented")).unwrap();
        let before: Vec<&str> = FRONT_MATTER_TASK.lines().collect();
        let after: Vec<&str> = output.lines().collect();
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(after.iter()) {
            if *old != *new {
                assert!(
                    old.starts_with("status:") || old.starts_with("updated_date:"),
                    "unexpected changed line: {old:?} -> {new:?}"
                );
            }
        }
    }

    #[test]
    fn task_status_ops_same_value_is_byte_identical() {
        let doc = Document::parse(FRONT_MATTER_TASK).unwrap();
        let output = apply(&doc, &task_status_ops(&doc, "ToDo")).unwrap();
        assert_eq!(output, FRONT_MATTER_TASK);
    }
}
