//! Record descriptors for managed Markdown record types.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;

use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct RecordDescriptor {
    pub name: &'static str,
    pub record_type: &'static str,
    pub type_name: &'static str,
    pub label: &'static str,
    pub display_id_pattern: &'static str,
    pub directory: &'static str,
    pub subpath: &'static str,
    pub doc_type_key: Option<&'static str>,
    pub frontmatter_keys: &'static [&'static str],
    pub sections: &'static [&'static str],
}

impl RecordDescriptor {
    pub fn has_frontmatter_key(&self, key: &str) -> bool {
        self.frontmatter_keys.contains(&key)
    }

    pub fn resolve_directory(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, AppError> {
        let base = match self.directory {
            "backlog" => project_dir.join(&cfg.backlog_directory),
            "decision" => project_dir.join(&cfg.decision_directory),
            "doc" => project_dir.join(&cfg.doc_directory),
            other => project_dir.join(other),
        };
        let raw = if let Some(key) = self.doc_type_key {
            if let Some(entry) = cfg.document_types.get(key) {
                base.join(&entry.directory)
            } else {
                if self.subpath.is_empty() {
                    base
                } else {
                    base.join(self.subpath)
                }
            }
        } else if self.subpath.is_empty() {
            base
        } else {
            base.join(self.subpath)
        };
        Context::ensure_within(project_dir, &raw)
    }

    // Aliases for callers/tests that prefer other names.
    pub fn resolve_dir(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, AppError> {
        self.resolve_directory(cfg, project_dir)
    }
    pub fn directory_path(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, AppError> {
        self.resolve_directory(cfg, project_dir)
    }
}

impl fmt::Display for RecordDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub const GENERAL_FRONTMATTER_KEYS: &[&str] = &[
    "id",
    "title",
    "created_date",
    "updated_date",
    "labels",
    "references",
];

const TASK_SECTIONS: &[&str] = &[
    "## Description",
    "## Status",
    "## Milestone",
    "## Assignee",
    "## Labels",
    "## Dependencies",
    "## Acceptance Criteria",
    "## Comments",
];
const MILESTONE_SECTIONS: &[&str] = &["## Description", "## Release"];
const RELEASE_SECTIONS: &[&str] = &["## Description"];
const DECISION_SECTIONS: &[&str] = &["## Status", "## Decision", "## Context", "## Consequences"];
const REQUIREMENT_SECTIONS: &[&str] = &["## User Story", "## Group", "## Description"];
const GLOSSARY_SECTIONS: &[&str] = &["## Definition"];
const UI_SPEC_SECTIONS: &[&str] = &["## View", "## Description"];
const TECH_SPEC_SECTIONS: &[&str] = &["## Description"];
const RULE_SECTIONS: &[&str] = &[
    "## Description",
    "## Rationale",
    "## Applicability",
    "## Requirements",
];
const AUDIT_SECTIONS: &[&str] = &["## Audit Type", "## Content"];
const USE_CASE_SECTIONS: &[&str] = &["## Description", "## Applicability"];
const TEST_CASE_SECTIONS: &[&str] = &["## Description", "## Applicability"];
const TEST_REPORT_SECTIONS: &[&str] = &["## Description", "## Applicability"];

pub const DESCRIPTORS: &[RecordDescriptor] = &[
    RecordDescriptor {
        name: "task",
        record_type: "task",
        type_name: "task",
        label: "Task",
        display_id_pattern: "TASK-###",
        directory: "backlog",
        subpath: "tasks",
        doc_type_key: None,
        frontmatter_keys: &[
            "status",
            "assignee",
            "milestone",
            "dependencies",
            "ordinal",
            "documentation",
            "type",
        ],
        sections: TASK_SECTIONS,
    },
    RecordDescriptor {
        name: "milestone",
        record_type: "milestone",
        type_name: "milestone",
        label: "Milestone",
        display_id_pattern: "MS-###",
        directory: "backlog",
        subpath: "milestones",
        doc_type_key: None,
        frontmatter_keys: &["release"],
        sections: MILESTONE_SECTIONS,
    },
    RecordDescriptor {
        name: "release",
        record_type: "release",
        type_name: "release",
        label: "Release",
        display_id_pattern: "REL-###",
        directory: "backlog",
        subpath: "releases",
        doc_type_key: None,
        frontmatter_keys: &[],
        sections: RELEASE_SECTIONS,
    },
    RecordDescriptor {
        name: "decision",
        record_type: "decision",
        type_name: "decision",
        label: "Decision",
        display_id_pattern: "DEC-###",
        directory: "decision",
        subpath: "",
        doc_type_key: None,
        frontmatter_keys: &["superseded_by"],
        sections: DECISION_SECTIONS,
    },
    RecordDescriptor {
        name: "business requirement",
        record_type: "requirement",
        type_name: "requirement",
        label: "Business requirement",
        display_id_pattern: "REQ-###.###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("req"),
        frontmatter_keys: &[],
        sections: REQUIREMENT_SECTIONS,
    },
    RecordDescriptor {
        name: "non-functional requirement",
        record_type: "nfr",
        type_name: "nfr",
        label: "Non-functional requirement",
        display_id_pattern: "NFR-###.###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("nfr"),
        frontmatter_keys: &[],
        sections: REQUIREMENT_SECTIONS,
    },
    RecordDescriptor {
        name: "glossary term",
        record_type: "glossary",
        type_name: "glossary",
        label: "Glossary term",
        display_id_pattern: "GLO-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("glossary"),
        frontmatter_keys: &[],
        sections: GLOSSARY_SECTIONS,
    },
    RecordDescriptor {
        name: "ui specification",
        record_type: "ui-spec",
        type_name: "ui_spec",
        label: "UI specification",
        display_id_pattern: "UI-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("ui-spec"),
        frontmatter_keys: &[],
        sections: UI_SPEC_SECTIONS,
    },
    RecordDescriptor {
        name: "technical specification",
        record_type: "tech-spec",
        type_name: "tech_spec",
        label: "Technical specification",
        display_id_pattern: "TECH-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("tech-spec"),
        frontmatter_keys: &[],
        sections: TECH_SPEC_SECTIONS,
    },
    RecordDescriptor {
        name: "business rule",
        record_type: "rule",
        type_name: "rule",
        label: "Business rule",
        display_id_pattern: "RUL-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("rule"),
        frontmatter_keys: &[],
        sections: RULE_SECTIONS,
    },
    RecordDescriptor {
        name: "audit record",
        record_type: "audit",
        type_name: "audit",
        label: "Audit record",
        display_id_pattern: "AUD-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("audit"),
        frontmatter_keys: &["type"],
        sections: AUDIT_SECTIONS,
    },
    RecordDescriptor {
        name: "use case",
        record_type: "use-case",
        type_name: "use_case",
        label: "Use case",
        display_id_pattern: "UC-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("use-case"),
        frontmatter_keys: &[],
        sections: USE_CASE_SECTIONS,
    },
    RecordDescriptor {
        name: "test case",
        record_type: "test-case",
        type_name: "test_case",
        label: "Test case",
        display_id_pattern: "TC-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("test-case"),
        frontmatter_keys: &[],
        sections: TEST_CASE_SECTIONS,
    },
    RecordDescriptor {
        name: "test report",
        record_type: "test-report",
        type_name: "test_report",
        label: "Test report",
        display_id_pattern: "TR-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some("test-report"),
        frontmatter_keys: &[],
        sections: TEST_REPORT_SECTIONS,
    },
];

pub fn descriptors() -> &'static [RecordDescriptor] {
    DESCRIPTORS
}

pub fn all_descriptors() -> &'static [RecordDescriptor] {
    DESCRIPTORS
}

pub fn list_descriptors() -> &'static [RecordDescriptor] {
    DESCRIPTORS
}

pub fn descriptor_for(name: &str) -> Option<&'static RecordDescriptor> {
    let needle = name.to_lowercase();
    DESCRIPTORS.iter().find(|d| {
        d.name.to_lowercase() == needle
            || d.record_type.to_lowercase() == needle
            || d.type_name.to_lowercase() == needle
            || d.label.to_lowercase() == needle
    })
}

pub fn is_managed_record(name: &str) -> bool {
    descriptor_for(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_count_excludes_ui_mockup() {
        assert_eq!(DESCRIPTORS.len(), 14);
        assert!(descriptor_for("ui mockup").is_none());
    }
}
