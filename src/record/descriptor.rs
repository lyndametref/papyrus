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

    /// Configured ID prefixes that belong to this record family.
    ///
    /// Tasks, milestones, and releases have one prefix each. Decisions have
    /// one prefix per enabled decision type, and document records use the
    /// prefix of their document type entry.
    pub fn configured_prefixes(&self, cfg: &Config) -> Vec<String> {
        match self.record_type {
            "task" => vec![cfg.task_prefix.clone()],
            "milestone" => vec![cfg.milestone_prefix.clone()],
            "release" => vec![cfg.release_prefix.clone()],
            "decision" => cfg
                .decision_types
                .values()
                .map(|entry| entry.prefix.clone())
                .collect(),
            _ => self
                .doc_type_key
                .and_then(|key| cfg.document_types.get(key))
                .map(|entry| vec![entry.prefix.clone()])
                .unwrap_or_default(),
        }
    }

    /// Whether the family's IDs carry a dotted `<group>.<sequence>` shape.
    pub fn is_dotted_id(&self) -> bool {
        self.display_id_pattern.contains('.')
    }

    /// Whether a file stem denotes a record of this family.
    ///
    /// Task, milestone, release, and decision families match a configured
    /// prefix with tolerant case, padding, and slug handling, excluding
    /// Backlog-shaped stems. Document families keep their
    /// `display_id_pattern` shape.
    pub fn matches_stem(&self, cfg: &Config, stem: &str) -> bool {
        match self.record_type {
            "task" | "milestone" | "release" | "decision" => self
                .configured_prefixes(cfg)
                .iter()
                .any(|prefix| crate::record::identity::id_number(stem, prefix).is_some()),
            _ => self.configured_prefixes(cfg).iter().any(|prefix| {
                crate::record::identity::parse_id_parts(stem, prefix, self.is_dotted_id()).is_some()
            }),
        }
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
            other => {
                return Err(AppError::Validation(format!(
                    "unknown directory configuration key: {}",
                    other
                )));
            }
        };
        let raw = if let Some(key) = self.doc_type_key {
            let entry = cfg.document_types.get(key).ok_or_else(|| {
                AppError::Validation(format!("unknown document type configuration key: {}", key))
            })?;
            base.join(&entry.directory)
        } else if self.subpath.is_empty() {
            base
        } else {
            base.join(self.subpath)
        };
        Context::ensure_within(project_dir, &raw)
    }

    /// Canonical archive location of a backlog family, if the family has one.
    pub fn resolve_archive_directory(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Option<Result<std::path::PathBuf, AppError>> {
        match self.record_type {
            "task" | "milestone" => {
                let raw = project_dir
                    .join(&cfg.backlog_directory)
                    .join("archive")
                    .join(self.subpath);
                Some(Context::ensure_within(project_dir, &raw))
            }
            _ => None,
        }
    }

    /// Legacy archive alias of a backlog family, read-only.
    pub fn resolve_legacy_archive_directory(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Option<Result<std::path::PathBuf, AppError>> {
        match self.record_type {
            "task" | "milestone" => {
                let raw = project_dir
                    .join(&cfg.backlog_directory)
                    .join(self.subpath)
                    .join("archived");
                Some(Context::ensure_within(project_dir, &raw))
            }
            _ => None,
        }
    }

    /// Deprecated location of a document family, if the family has one.
    pub fn resolve_deprecated_directory(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Option<Result<std::path::PathBuf, AppError>> {
        self.doc_type_key?;
        let active = self.resolve_directory(cfg, project_dir).ok()?;
        let raw = active.join("deprecated");
        Some(Context::ensure_within(project_dir, &raw))
    }

    /// All locations a family may be read from: active, archive, legacy
    /// alias, and deprecated.
    pub fn resolve_read_directories(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Vec<std::path::PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(dir) = self.resolve_directory(cfg, project_dir) {
            dirs.push(dir);
        }
        if let Some(Ok(dir)) = self.resolve_archive_directory(cfg, project_dir) {
            dirs.push(dir);
        }
        if let Some(Ok(dir)) = self.resolve_legacy_archive_directory(cfg, project_dir) {
            dirs.push(dir);
        }
        if let Some(Ok(dir)) = self.resolve_deprecated_directory(cfg, project_dir) {
            dirs.push(dir);
        }
        dirs
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

const TASK_SECTIONS: &[&str] = &["## Description", "## Acceptance Criteria", "## Comments"];
const MILESTONE_SECTIONS: &[&str] = &["## Description"];
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

pub fn descriptor_for_doc_type(key: &str) -> Option<&'static RecordDescriptor> {
    DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.doc_type_key == Some(key))
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
