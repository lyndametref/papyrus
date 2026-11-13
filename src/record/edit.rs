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

const REQ_KNOWN: &[&str] = &["## User Story", "## Group", "## Description"];
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
const GLOSSARY_KNOWN: &[&str] = &["## Definition"];
const UI_SPEC_KNOWN: &[&str] = &["## View", "## Description"];
const RULE_KNOWN: &[&str] = &[
    "## Description",
    "## Rationale",
    "## Applicability",
    "## Requirements",
];
const AUDIT_KNOWN: &[&str] = &["## Audit Type", "## Content"];

pub fn requirement_ops(
    doc: &Document,
    title: &str,
    story: FieldChange,
    description: FieldChange,
    supports_story: bool,
) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    if supports_story {
        match story {
            FieldChange::Keep => {}
            FieldChange::Clear => set_section(&mut sections, "## User Story", None, REQ_KNOWN),
            FieldChange::Set(value) => {
                set_section(&mut sections, "## User Story", Some(value), REQ_KNOWN);
            }
        }
    }
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, REQ_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), REQ_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
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
                fm_ops.push(PatchOp::SetFrontMatter("assignee".into(), String::new()));
            }
            FieldChange::Set(value) => {
                fm_ops.push(PatchOp::SetFrontMatter("assignee".into(), value.clone()));
            }
            FieldChange::Keep => {}
        }
        fm_ops.push(PatchOp::SetFrontMatter("labels".into(), labels.join("\n")));
        fm_ops.push(PatchOp::SetFrontMatter(
            "dependencies".into(),
            dependencies.join("\n"),
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

pub fn glossary_ops(doc: &Document, title: &str, definition: FieldChange) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    match definition {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Definition", None, GLOSSARY_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Definition", Some(value), GLOSSARY_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
}

pub fn ui_spec_ops(
    doc: &Document,
    title: &str,
    view: FieldChange,
    description: FieldChange,
) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    match view {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## View", None, UI_SPEC_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## View", Some(value), UI_SPEC_KNOWN);
        }
    }
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, UI_SPEC_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), UI_SPEC_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
}

pub fn business_rule_ops(
    doc: &Document,
    title: &str,
    description: FieldChange,
    rationale: FieldChange,
    applicability: FieldChange,
    requirements: Vec<String>,
) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Description", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Description", Some(value), RULE_KNOWN);
        }
    }
    match rationale {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Rationale", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Rationale", Some(value), RULE_KNOWN);
        }
    }
    match applicability {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Applicability", None, RULE_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Applicability", Some(value), RULE_KNOWN);
        }
    }
    if requirements.is_empty() {
        set_section(&mut sections, "## Requirements", None, RULE_KNOWN);
    } else {
        let body = requirements
            .iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n");
        set_section(&mut sections, "## Requirements", Some(body), RULE_KNOWN);
    }
    finish(doc, fm_ops, &sections)
}

pub fn audit_ops(
    doc: &Document,
    title: &str,
    audit_type: &str,
    content: FieldChange,
) -> Vec<PatchOp> {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops = title_ops(doc, title);
    if has_fm {
        fm_ops.push(PatchOp::SetFrontMatter(
            "type".into(),
            audit_type.to_string(),
        ));
    }
    let mut sections = doc.sections.clone();
    set_section(
        &mut sections,
        "## Audit Type",
        Some(audit_type.to_string()),
        AUDIT_KNOWN,
    );
    match content {
        FieldChange::Keep => {}
        FieldChange::Clear => set_section(&mut sections, "## Content", None, AUDIT_KNOWN),
        FieldChange::Set(value) => {
            set_section(&mut sections, "## Content", Some(value), AUDIT_KNOWN);
        }
    }
    finish(doc, fm_ops, &sections)
}

pub fn tech_spec_ops(doc: &Document, title: &str, description: FieldChange) -> Vec<PatchOp> {
    let fm_ops = title_ops(doc, title);
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if !sections.is_empty() {
                    body.push(String::new());
                }
                sections.insert(
                    0,
                    Section {
                        heading: "## Description".to_string(),
                        body,
                    },
                );
            }
        }
    }
    finish(doc, fm_ops, &sections)
}

#[allow(clippy::too_many_arguments)]
pub fn use_case_ops(
    doc: &Document,
    title: &str,
    description: FieldChange,
    applicability: FieldChange,
    labels: Vec<String>,
    references: Vec<String>,
) -> Vec<PatchOp> {
    let has_fm = doc.front_matter.is_some();
    let mut fm_ops = title_ops(doc, title);
    if has_fm {
        fm_ops.push(PatchOp::SetFrontMatter("labels".into(), labels.join("\n")));
        fm_ops.push(PatchOp::SetFrontMatter(
            "references".into(),
            references.join("\n"),
        ));
    }
    let mut sections = doc.sections.clone();
    match description {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            if let Some(index) = sections.iter().position(|s| s.heading == "## Description") {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if !sections.is_empty() {
                    body.push(String::new());
                }
                sections.insert(
                    0,
                    Section {
                        heading: "## Description".to_string(),
                        body,
                    },
                );
            }
        }
    }
    match applicability {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            if let Some(index) = sections
                .iter()
                .position(|s| s.heading == "## Applicability")
            {
                sections.remove(index);
            }
        }
        FieldChange::Set(value) => {
            let known = ["## Description", "## Applicability"];
            if let Some(index) = sections
                .iter()
                .position(|s| s.heading == "## Applicability")
            {
                let trailing = trailing_empty_lines(&sections[index].body);
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                for _ in 0..trailing {
                    body.push(String::new());
                }
                sections[index].body = body;
            } else {
                let order = 1usize;
                let mut position = None;
                for (index, section) in sections.iter().enumerate() {
                    if let Some(section_order) = known.iter().position(|h| *h == section.heading)
                        && section_order < order
                    {
                        position = Some(index + 1);
                    }
                }
                if position.is_none() {
                    for (index, section) in sections.iter().enumerate() {
                        if let Some(section_order) =
                            known.iter().position(|h| *h == section.heading)
                            && section_order > order
                        {
                            position = Some(index);
                            break;
                        }
                    }
                }
                let position = position.unwrap_or(sections.len());
                let mut body: Vec<String> =
                    value.split('\n').map(|line| line.to_string()).collect();
                if position < sections.len() {
                    body.push(String::new());
                }
                sections.insert(
                    position,
                    Section {
                        heading: "## Applicability".to_string(),
                        body,
                    },
                );
            }
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
