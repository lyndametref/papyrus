//! Record descriptors for managed Markdown record types.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    /// Canonical archived location of a family, if the family has one.
    ///
    /// Document descriptors resolve to `<active dir>/archived`. Task and
    /// milestone descriptors resolve to their archive location so downstream
    /// has one method for both active and archived paths. Decision and other
    /// families have no archived location.
    pub fn resolve_archived_directory(
        &self,
        cfg: &crate::config::Config,
        project_dir: &std::path::Path,
    ) -> Option<Result<std::path::PathBuf, AppError>> {
        match self.record_type {
            "task" | "milestone" => self.resolve_archive_directory(cfg, project_dir),
            _ => {
                if self.doc_type_key.is_some() {
                    let active = self.resolve_directory(cfg, project_dir).ok()?;
                    let raw = active.join("archived");
                    Some(Context::ensure_within(project_dir, &raw))
                } else {
                    None
                }
            }
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

    /// Declared status vocabulary for this record family (main + custom).
    pub fn declared_statuses(&self, cfg: &crate::config::Config) -> Vec<String> {
        match self.record_type {
            "task" => cfg.statuses.clone(),
            "milestone" => vec!["open".to_string(), "closed".to_string()],
            "release" => vec!["planned".to_string(), "released".to_string()],
            _ if self.doc_type_key.is_some() => self
                .doc_type_key
                .and_then(|k| cfg.document_types.get(k))
                .and_then(|e| e.statuses.clone())
                .unwrap_or_else(|| vec!["active".to_string(), "archived".to_string()]),
            _ => Vec::new(),
        }
    }

    /// All locations a family may be read from: active, archived, and the
    /// task/milestone legacy alias. No deprecated path is included.
    pub fn resolve_read_directories(
        &self,
        cfg: &Config,
        project_dir: &std::path::Path,
    ) -> Vec<std::path::PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(dir) = self.resolve_directory(cfg, project_dir) {
            dirs.push(dir);
        }
        if let Some(Ok(dir)) = self.resolve_archived_directory(cfg, project_dir)
            && !dirs.contains(&dir)
        {
            dirs.push(dir);
        }
        if let Some(Ok(dir)) = self.resolve_legacy_archive_directory(cfg, project_dir)
            && !dirs.contains(&dir)
        {
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
        frontmatter_keys: &["status", "release"],
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
        frontmatter_keys: &["status"],
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

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

static GENERIC_DESC_CACHE: OnceLock<Mutex<BTreeMap<String, &'static RecordDescriptor>>> =
    OnceLock::new();

/// Build (and cache) a runtime [`RecordDescriptor`] for a config-defined
/// (generic) document type. The slug is leaked to satisfy the `'static`
/// lifetime required by the record layer; the real directory, prefix, and
/// status vocabulary are resolved through `doc_type_key`.
pub fn generic_document_descriptor(slug: &str) -> &'static RecordDescriptor {
    let cache = GENERIC_DESC_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut map = cache.lock().expect("generic descriptor cache poisoned");
    if let Some(existing) = map.get(slug) {
        return existing;
    }
    let slug_static: &'static str = Box::leak(slug.to_string().into_boxed_str());
    let descriptor = Box::leak(Box::new(RecordDescriptor {
        name: slug_static,
        record_type: "document",
        type_name: "document",
        label: slug_static,
        display_id_pattern: "GEN-###",
        directory: "doc",
        subpath: "",
        doc_type_key: Some(slug_static),
        frontmatter_keys: &["status"],
        sections: &["## Description"],
    }));
    map.insert(slug.to_string(), descriptor);
    descriptor
}

pub fn is_managed_record(name: &str) -> bool {
    descriptor_for(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_four_backlog_families_are_static() {
        assert_eq!(DESCRIPTORS.len(), 4);
        assert!(
            DESCRIPTORS
                .iter()
                .all(|descriptor| descriptor.doc_type_key.is_none()),
            "document records must come from the artifact registry"
        );
        assert!(descriptor_for("ui mockup").is_none());
    }
}
