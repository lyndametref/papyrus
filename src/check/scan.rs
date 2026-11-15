//! Descriptor-driven, read-only corpus scanners.

use crate::check::findings::{
    DANGLING_MILESTONE, DANGLING_RELEASE, DEPENDENCY_CYCLE, DUPLICATE_ID, FRONTMATTER_CONFLICT,
    Finding, SELF_DEPENDENCY,
};
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::descriptor::RecordDescriptor;
use crate::record::document;
use crate::record::frontmatter::{self, FrontMatter};
use crate::record::identity::{self, ParsedId};
use crate::record::scan::{family_files, project_relative};
use crate::record::{SectionList, SectionValue, retired_sections, section_list, section_scalar};
use crate::registry::{ArtifactCategory, ArtifactRegistry, ContentModel, IdShape};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

/// The ID shape a family claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Plain,
    Dotted,
    Grouped,
}

/// One family as the scanners see it.
#[derive(Clone, Debug)]
pub struct FamilyScan<'a> {
    pub name: String,
    pub descriptor: &'a RecordDescriptor,
    pub prefixes: Vec<String>,
    pub shape: Shape,
    pub extension: &'static str,
}

impl FamilyScan<'_> {
    fn is_document(&self) -> bool {
        self.descriptor.doc_type_key.is_some()
    }
}

/// One scanned family and its canonical members.
pub struct ScannedFamily<'a> {
    pub family: FamilyScan<'a>,
    pub records: Vec<RecordMeta>,
}

/// Metadata read from one scanned record without touching the file.
pub struct RecordMeta {
    pub rel: String,
    pub key: String,
    pub display: String,
    pub number_count: usize,
    pub frontmatter: Option<FrontMatter>,
    pub mismatch: bool,
    pub read_only: bool,
    pub content: String,
}

/// The four static backlog families; a decision is one family over all
/// configured decision-type prefixes.
pub fn static_families(cfg: &Config) -> Vec<FamilyScan<'static>> {
    crate::record::descriptor::descriptors()
        .iter()
        .map(|descriptor| {
            let shape = if descriptor.is_dotted_id() {
                Shape::Dotted
            } else {
                Shape::Plain
            };
            FamilyScan {
                name: descriptor.name.to_string(),
                descriptor,
                prefixes: descriptor.configured_prefixes(cfg),
                shape,
                extension: "md",
            }
        })
        .collect()
}

/// Every configured document type, enabled or disabled.
pub fn document_families<'a>(registry: &'a ArtifactRegistry, cfg: &Config) -> Vec<FamilyScan<'a>> {
    registry
        .descriptors()
        .filter(|descriptor| descriptor.category == ArtifactCategory::Document)
        .filter_map(|descriptor| {
            let record = descriptor.record.as_ref()?;
            let shape = match descriptor.id_shape {
                IdShape::Grouped => Shape::Grouped,
                IdShape::Dotted => Shape::Dotted,
                IdShape::Plain => Shape::Plain,
            };
            let extension = match descriptor.content_model {
                ContentModel::HtmlMockup(_) => "html",
                ContentModel::Generic | ContentModel::Typed(_) => "md",
            };
            Some(FamilyScan {
                name: descriptor.slug.clone(),
                descriptor: record,
                prefixes: record.configured_prefixes(cfg),
                shape,
                extension,
            })
        })
        .collect()
}

/// Read every member of every family exactly once.
pub fn collect<'a>(
    ctx: &Context,
    cfg: &Config,
    families: Vec<FamilyScan<'a>>,
) -> Result<Vec<ScannedFamily<'a>>, AppError> {
    let mut scanned = Vec::with_capacity(families.len());
    for family in families {
        let mut records = Vec::new();
        for path in family_files(ctx, cfg, family.descriptor)? {
            if path.extension().and_then(|value| value.to_str()) != Some(family.extension) {
                continue;
            }
            if let Some(record) = record_meta(cfg, &ctx.project_dir, &family, &path)? {
                records.push(record);
            }
        }
        scanned.push(ScannedFamily { family, records });
    }
    Ok(scanned)
}

fn parse_value(value: &str, family: &FamilyScan<'_>) -> Option<(ParsedId, String)> {
    for prefix in &family.prefixes {
        let parsed = match family.shape {
            Shape::Grouped => identity::parse_id_parts(value, prefix, true)
                .or_else(|| identity::parse_id_parts(value, prefix, false)),
            Shape::Dotted => identity::parse_id_parts(value, prefix, true),
            Shape::Plain => identity::parse_id_parts(value, prefix, false),
        };
        if let Some(parsed) = parsed {
            return Some((parsed, prefix.clone()));
        }
    }
    None
}

fn record_meta(
    cfg: &Config,
    project_dir: &Path,
    family: &FamilyScan<'_>,
    path: &Path,
) -> Result<Option<RecordMeta>, AppError> {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return Ok(None);
    };
    let Some((stem_parts, stem_prefix)) = parse_value(stem, family) else {
        return Ok(None);
    };

    let content = if family.extension == "md" {
        fs::read_to_string(path).map_err(|error| {
            AppError::Runtime(format!("cannot read record {}: {}", path.display(), error))
        })?
    } else {
        String::new()
    };
    let frontmatter = if family.extension == "md" {
        frontmatter::parse(&content).ok().flatten()
    } else {
        None
    };
    let frontmatter_id = frontmatter
        .as_ref()
        .and_then(|frontmatter| frontmatter::current_value_string(&frontmatter.raw, "id"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let frontmatter_parts = frontmatter_id
        .as_deref()
        .and_then(|value| parse_value(value, family));
    let mismatch = match (&frontmatter_id, &frontmatter_parts) {
        (Some(_), None) => true,
        (Some(_), Some((parts, prefix))) => parts.key(prefix) != stem_parts.key(&stem_prefix),
        (None, _) => false,
    };
    let (key, display, number_count) = match frontmatter_parts {
        Some((parts, prefix)) => (parts.key(&prefix), parts.display, parts.numbers.len()),
        None => (
            stem_parts.key(&stem_prefix),
            stem_parts.display,
            stem_parts.numbers.len(),
        ),
    };
    let read_only = is_read_only(cfg, project_dir, family.descriptor, path);
    Ok(Some(RecordMeta {
        rel: project_relative(project_dir, path),
        key,
        display,
        number_count,
        frontmatter,
        mismatch,
        read_only,
        content,
    }))
}

fn is_read_only(
    cfg: &Config,
    project_dir: &Path,
    descriptor: &RecordDescriptor,
    path: &Path,
) -> bool {
    let legacy = descriptor
        .resolve_legacy_archive_directory(cfg, project_dir)
        .and_then(Result::ok);
    let imported = descriptor
        .resolve_read_import_directory(cfg, project_dir)
        .and_then(Result::ok);
    legacy.is_some_and(|directory| path.starts_with(directory))
        || imported.is_some_and(|directory| path.starts_with(directory))
}

fn family<'a, 'b>(scanned: &'b [ScannedFamily<'a>], name: &str) -> Option<&'b ScannedFamily<'a>> {
    scanned.iter().find(|scanned| scanned.family.name == name)
}

/// One finding per canonical key seen in two or more distinct paths.
pub fn duplicate_ids(scanned: &[ScannedFamily<'_>]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for scanned_family in scanned {
        let mut groups: BTreeMap<&str, Vec<&RecordMeta>> = BTreeMap::new();
        for record in &scanned_family.records {
            groups.entry(record.key.as_str()).or_default().push(record);
        }
        for (_, mut members) in groups {
            if members.len() < 2 {
                continue;
            }
            members.sort_by(|left, right| left.rel.cmp(&right.rel));
            let fixable = members
                .iter()
                .all(|member| !member.mismatch && !member.read_only)
                && !(scanned_family.family.shape == Shape::Grouped && members[0].number_count == 1);
            findings.push(Finding {
                kind: DUPLICATE_ID,
                family: Some(scanned_family.family.name.clone()),
                subject: Some(members[0].display.clone()),
                locations: members.iter().map(|member| member.rel.clone()).collect(),
                detail: format!("{} records share this ID", members.len()),
                fixable,
            });
        }
    }
    findings
}

/// Tasks whose dependency list names their own canonical key.
pub fn self_dependencies(scanned: &[ScannedFamily<'_>]) -> Vec<Finding> {
    let Some(tasks) = family(scanned, "task") else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for record in &tasks.records {
        let Some(frontmatter) = &record.frontmatter else {
            continue;
        };
        let dependencies = frontmatter_list(frontmatter, "dependencies").unwrap_or_default();
        if dependencies.iter().any(|dependency| {
            parse_value(dependency, &tasks.family)
                .is_some_and(|(parts, prefix)| parts.key(&prefix) == record.key)
        }) {
            findings.push(Finding {
                kind: SELF_DEPENDENCY,
                family: Some(tasks.family.name.clone()),
                subject: Some(record.display.clone()),
                locations: vec![record.rel.clone()],
                detail: format!("dependency list contains {}", record.display),
                fixable: false,
            });
        }
    }
    findings
}

/// Cycles in the task dependency graph, one finding per member set.
pub fn dependency_cycles(scanned: &[ScannedFamily<'_>]) -> Vec<Finding> {
    let Some(tasks) = family(scanned, "task") else {
        return Vec::new();
    };
    let mut nodes: BTreeMap<String, &RecordMeta> = BTreeMap::new();
    for record in &tasks.records {
        nodes.entry(record.key.clone()).or_insert(record);
    }
    let keys: Vec<String> = nodes.keys().cloned().collect();
    let index: HashMap<&str, usize> = keys
        .iter()
        .enumerate()
        .map(|(position, key)| (key.as_str(), position))
        .collect();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); keys.len()];
    for (position, key) in keys.iter().enumerate() {
        let Some(record) = nodes.get(key) else {
            continue;
        };
        let Some(frontmatter) = &record.frontmatter else {
            continue;
        };
        let mut neighbours: BTreeSet<usize> = BTreeSet::new();
        for dependency in frontmatter_list(frontmatter, "dependencies").unwrap_or_default() {
            let Some((parts, prefix)) = parse_value(&dependency, &tasks.family) else {
                continue;
            };
            let dependency_key = parts.key(&prefix);
            if dependency_key == *key {
                continue;
            }
            if let Some(&neighbour) = index.get(dependency_key.as_str()) {
                neighbours.insert(neighbour);
            }
        }
        adjacency[position] = neighbours.into_iter().collect();
    }

    let mut colour = vec![0u8; keys.len()];
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut path: Vec<usize> = Vec::new();
    let mut cycles: Vec<Vec<String>> = Vec::new();
    for start in 0..keys.len() {
        if colour[start] != 0 {
            continue;
        }
        colour[start] = 1;
        stack.push((start, 0));
        path.push(start);
        while let Some(&(node, next)) = stack.last() {
            if next < adjacency[node].len() {
                if let Some(frame) = stack.last_mut() {
                    frame.1 += 1;
                }
                let neighbour = adjacency[node][next];
                match colour[neighbour] {
                    0 => {
                        colour[neighbour] = 1;
                        stack.push((neighbour, 0));
                        path.push(neighbour);
                    }
                    1 => {
                        if let Some(position) = path.iter().position(|value| *value == neighbour) {
                            let mut members: Vec<String> = path[position..]
                                .iter()
                                .map(|value| keys[*value].clone())
                                .collect();
                            members.sort();
                            if !cycles.contains(&members) {
                                cycles.push(members);
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                colour[node] = 2;
                stack.pop();
                path.pop();
            }
        }
    }

    let mut findings = Vec::new();
    for members in cycles {
        let mut paths: Vec<String> = members
            .iter()
            .filter_map(|key| nodes.get(key).map(|record| record.rel.clone()))
            .collect();
        paths.sort();
        paths.dedup();
        let subject = nodes.get(&members[0]).map(|record| record.display.clone());
        let detail = members
            .iter()
            .filter_map(|key| nodes.get(key).map(|record| record.display.clone()))
            .collect::<Vec<String>>()
            .join(" -> ");
        findings.push(Finding {
            kind: DEPENDENCY_CYCLE,
            family: Some(tasks.family.name.clone()),
            subject,
            locations: paths,
            detail,
            fixable: false,
        });
    }
    findings
}

/// Dangling task `milestone` and milestone `release` references.
pub fn dangling_references(scanned: &[ScannedFamily<'_>]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let milestone_keys: BTreeSet<String> = family(scanned, "milestone")
        .map(|scanned| {
            scanned
                .records
                .iter()
                .map(|record| record.key.clone())
                .collect()
        })
        .unwrap_or_default();
    let release_keys: BTreeSet<String> = family(scanned, "release")
        .map(|scanned| {
            scanned
                .records
                .iter()
                .map(|record| record.key.clone())
                .collect()
        })
        .unwrap_or_default();

    if let Some(tasks) = family(scanned, "task") {
        for record in &tasks.records {
            let Some(frontmatter) = &record.frontmatter else {
                continue;
            };
            let Some(value) = frontmatter_scalar(frontmatter, "milestone") else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            let resolves = family(scanned, "milestone")
                .and_then(|milestones| parse_value(value, &milestones.family))
                .is_some_and(|(parts, prefix)| milestone_keys.contains(&parts.key(&prefix)));
            if !resolves {
                findings.push(Finding {
                    kind: DANGLING_MILESTONE,
                    family: Some(tasks.family.name.clone()),
                    subject: Some(record.display.clone()),
                    locations: vec![record.rel.clone()],
                    detail: format!("milestone {} does not resolve to a milestone", value),
                    fixable: false,
                });
            }
        }
    }

    if let Some(milestones) = family(scanned, "milestone") {
        for record in &milestones.records {
            let Some(frontmatter) = &record.frontmatter else {
                continue;
            };
            let Some(value) = frontmatter_scalar(frontmatter, "release") else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            let resolves = family(scanned, "release")
                .and_then(|releases| parse_value(value, &releases.family))
                .is_some_and(|(parts, prefix)| release_keys.contains(&parts.key(&prefix)));
            if !resolves {
                findings.push(Finding {
                    kind: DANGLING_RELEASE,
                    family: Some(milestones.family.name.clone()),
                    subject: Some(record.display.clone()),
                    locations: vec![record.rel.clone()],
                    detail: format!("release {} does not resolve to a release", value),
                    fixable: false,
                });
            }
        }
    }

    findings
}

/// Retired metadata sections and document headings that disagree.
pub fn frontmatter_conflicts(scanned: &[ScannedFamily<'_>]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for scanned_family in scanned {
        let descriptor = scanned_family.family.descriptor;
        if descriptor.name == "decision" {
            continue;
        }
        for record in &scanned_family.records {
            let Some(frontmatter) = &record.frontmatter else {
                continue;
            };
            let Ok(document) = document::parse(&record.content) else {
                continue;
            };
            for (section, key, is_list) in retired_sections(descriptor) {
                let heading = format!("## {}", section);
                let occurrences = document
                    .sections
                    .iter()
                    .filter(|section| section.heading == heading)
                    .count();
                if occurrences == 0 {
                    continue;
                }
                if occurrences > 1 {
                    findings.push(conflict(
                        scanned_family,
                        record,
                        format!("section {} occurs more than once", heading),
                    ));
                    continue;
                }
                if *is_list {
                    match section_list(&document, section) {
                        SectionList::Missing => {}
                        SectionList::Invalid => findings.push(conflict(
                            scanned_family,
                            record,
                            format!("section {} contains an empty list item", heading),
                        )),
                        SectionList::Valid(items) => {
                            let Some(frontmatter_items) = frontmatter_list(frontmatter, key) else {
                                continue;
                            };
                            let body_set: BTreeSet<String> = items.iter().cloned().collect();
                            let front_set: BTreeSet<String> =
                                frontmatter_items.iter().cloned().collect();
                            if body_set != front_set {
                                findings.push(conflict(
                                    scanned_family,
                                    record,
                                    format!(
                                        "section {} \"{}\" disagrees with front matter {} \"{}\"",
                                        heading,
                                        body_set.into_iter().collect::<Vec<String>>().join(", "),
                                        key,
                                        front_set.into_iter().collect::<Vec<String>>().join(", ")
                                    ),
                                ));
                            }
                        }
                    }
                } else {
                    match section_scalar(&document, section) {
                        SectionValue::Missing => {}
                        SectionValue::Invalid => findings.push(conflict(
                            scanned_family,
                            record,
                            format!("section {} cannot be reduced to a single value", heading),
                        )),
                        SectionValue::Valid(value) => {
                            let Some(frontmatter_value) = frontmatter_scalar(frontmatter, key)
                            else {
                                continue;
                            };
                            if value.trim() != frontmatter_value.trim() {
                                findings.push(conflict(
                                    scanned_family,
                                    record,
                                    format!(
                                        "section {} \"{}\" disagrees with front matter {} \"{}\"",
                                        heading, value, key, frontmatter_value
                                    ),
                                ));
                            }
                        }
                    }
                }
            }

            if scanned_family.family.is_document()
                && let Some(heading) = document.heading.as_deref()
                && heading.starts_with("# ")
            {
                let (heading_id, heading_title) = document::heading_id_title(&document);
                if let Some(frontmatter_id) = frontmatter_scalar(frontmatter, "id")
                    && heading_id.trim() != frontmatter_id.trim()
                {
                    findings.push(conflict(
                        scanned_family,
                        record,
                        format!(
                            "heading id \"{}\" disagrees with front matter id \"{}\"",
                            heading_id, frontmatter_id
                        ),
                    ));
                }
                if let Some(frontmatter_title) = frontmatter_scalar(frontmatter, "title")
                    && heading_title.trim() != frontmatter_title.trim()
                {
                    findings.push(conflict(
                        scanned_family,
                        record,
                        format!(
                            "heading title \"{}\" disagrees with front matter title \"{}\"",
                            heading_title, frontmatter_title
                        ),
                    ));
                }
            }
        }
    }
    findings
}

fn conflict(scanned: &ScannedFamily<'_>, record: &RecordMeta, detail: String) -> Finding {
    Finding {
        kind: FRONTMATTER_CONFLICT,
        family: Some(scanned.family.name.clone()),
        subject: Some(record.display.clone()),
        locations: vec![record.rel.clone()],
        detail,
        fixable: false,
    }
}

fn frontmatter_scalar(frontmatter: &FrontMatter, key: &str) -> Option<String> {
    let value = frontmatter
        .data
        .get(yaml_serde::Value::String(key.to_string()))?;
    Some(match value {
        yaml_serde::Value::String(value) => value.clone(),
        yaml_serde::Value::Bool(value) => value.to_string(),
        yaml_serde::Value::Number(value) => value.to_string(),
        yaml_serde::Value::Null => String::new(),
        other => yaml_serde::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    })
}

fn frontmatter_list(frontmatter: &FrontMatter, key: &str) -> Option<Vec<String>> {
    let value = frontmatter
        .data
        .get(yaml_serde::Value::String(key.to_string()))?;
    Some(match value {
        yaml_serde::Value::Sequence(items) => items
            .iter()
            .filter_map(|item| match item {
                yaml_serde::Value::String(value) => Some(value.clone()),
                yaml_serde::Value::Bool(value) => Some(value.to_string()),
                yaml_serde::Value::Number(value) => Some(value.to_string()),
                _ => None,
            })
            .collect(),
        yaml_serde::Value::String(value) => value
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect(),
        yaml_serde::Value::Null => Vec::new(),
        other => yaml_serde::to_string(other)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect(),
    })
}
