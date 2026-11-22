//! Model-neutral field map for a loaded document record (ADR-039 decision 14,
//! TECH-040 section 3).
//!
//! The map is built once from the owning [`TypeDescriptor`], the record's role,
//! and the parsed [`Document`]. It is the only content projection for a
//! document: the browser list and view renderers never read a typed content
//! field.

use crate::config::{FieldKind, FieldSpec, FieldVerb, Visibility};
use crate::record::document::{self, Document};
use crate::record::view::RecordView;
use crate::registry::{DocumentModel, TypeDescriptor};

/// The role a loaded record plays in its document model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FieldRole {
    /// A `SingleMarkdown` or `SingleHtmlMockup` record.
    #[default]
    Single,
    /// The group record of a `GroupedMarkdown` type.
    Group,
    /// A member record of a `GroupedMarkdown` type.
    Member,
}

/// One resolved field value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    /// A scalar or multi-line text value.
    Text(String),
    /// A list value, in stored order.
    Items(Vec<String>),
    /// No value is stored.
    Absent,
}

impl FieldValue {
    /// The value as display text: text verbatim, items joined with `, `.
    pub fn display(&self) -> Option<String> {
        match self {
            FieldValue::Text(value) => (!value.trim().is_empty()).then(|| value.clone()),
            FieldValue::Items(items) => (!items.is_empty()).then(|| items.join(", ")),
            FieldValue::Absent => None,
        }
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, FieldValue::Absent)
    }
}

/// One descriptor field resolved against a record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedField {
    /// Stable field identity from the descriptor.
    pub key: String,
    /// Declared body heading, with its `## ` prefix, when the field stores one.
    pub heading: Option<String>,
    /// Declared front-matter key, when the field stores one.
    pub frontmatter_key: Option<String>,
    /// Visible label shown in the list and view.
    pub label: String,
    /// Optional help text.
    pub help: Option<String>,
    /// Accepted value form.
    pub kind: FieldKind,
    /// Whether create requires the field.
    pub required: bool,
    /// Whether the edit surface may write the field.
    pub editable: bool,
    /// Resolved value.
    pub value: FieldValue,
}

impl ResolvedField {
    /// Whether the edit surface must flag this field: required and absent.
    pub fn missing_required(&self) -> bool {
        self.required && self.value.is_absent()
    }
}

/// The model-neutral content projection of one document record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentFieldMap {
    pub role: FieldRole,
    /// Resolved fields in descriptor sequence order.
    pub fields: Vec<ResolvedField>,
    /// Fixed metadata block; `SingleHtmlMockup` only.
    pub fixed_metadata: Vec<(String, String)>,
    /// Undeclared stored front-matter keys, read-only.
    pub extra_metadata: Vec<(String, String)>,
    /// Reference visibility resolved for this type and role.
    pub reference_visibility: Visibility,
}

impl Default for DocumentFieldMap {
    fn default() -> Self {
        Self {
            role: FieldRole::Single,
            fields: Vec::new(),
            fixed_metadata: Vec::new(),
            extra_metadata: Vec::new(),
            reference_visibility: Visibility::Visible,
        }
    }
}

impl DocumentFieldMap {
    /// Build the map for a loaded Markdown record.
    pub fn build(
        descriptor: &TypeDescriptor,
        is_group: bool,
        doc: &Document,
        view: Option<&RecordView>,
    ) -> Self {
        let (role, specs, visibility) = role_fields(descriptor, is_group);
        let fields = specs
            .iter()
            .map(|spec| resolve_field(spec, doc, view))
            .collect();
        Self {
            role,
            fields,
            fixed_metadata: Vec::new(),
            extra_metadata: extra_metadata(specs, view),
            reference_visibility: visibility,
        }
    }

    /// Build the map for a `SingleHtmlMockup` record from its metadata comment.
    pub fn for_mockup(descriptor: &TypeDescriptor, fixed_metadata: Vec<(String, String)>) -> Self {
        Self {
            role: FieldRole::Single,
            fields: Vec::new(),
            fixed_metadata,
            extra_metadata: Vec::new(),
            reference_visibility: descriptor
                .reference_visibility
                .unwrap_or(Visibility::Visible),
        }
    }

    /// Resolve one field key (or its front-matter key) to its display value.
    pub fn display_value(&self, key: &str) -> Option<String> {
        if let Some(field) = self
            .fields
            .iter()
            .find(|field| field.key == key || field.frontmatter_key.as_deref() == Some(key))
        {
            return field.value.display();
        }
        self.fixed_metadata
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
    }

    /// Resolve one field by key.
    pub fn field(&self, key: &str) -> Option<&ResolvedField> {
        self.fields.iter().find(|field| field.key == key)
    }
}

fn role_fields(
    descriptor: &TypeDescriptor,
    is_group: bool,
) -> (FieldRole, &[FieldSpec], Visibility) {
    match descriptor.document_model {
        DocumentModel::SingleHtmlMockup => (
            FieldRole::Single,
            &[],
            descriptor
                .reference_visibility
                .unwrap_or(Visibility::Visible),
        ),
        DocumentModel::GroupedMarkdown => {
            let (role, fields) = if is_group {
                (FieldRole::Group, descriptor.group.as_ref())
            } else {
                (FieldRole::Member, descriptor.member.as_ref())
            };
            let specs = fields
                .and_then(|role| role.fields.as_deref())
                .unwrap_or(&[]);
            let visibility = fields
                .and_then(|role| role.reference_visibility)
                .or(descriptor.reference_visibility)
                .unwrap_or(Visibility::Visible);
            (role, specs, visibility)
        }
        DocumentModel::SingleMarkdown => (
            FieldRole::Single,
            descriptor.fields.as_slice(),
            descriptor
                .reference_visibility
                .unwrap_or(Visibility::Visible),
        ),
    }
}

fn resolve_field(spec: &FieldSpec, doc: &Document, view: Option<&RecordView>) -> ResolvedField {
    let body = spec
        .heading
        .as_deref()
        .and_then(|heading| document::section_text(doc, document::section_name(heading)));
    let frontmatter = spec
        .frontmatter_key
        .as_deref()
        .and_then(|key| view.and_then(|view| frontmatter_value(view, key, spec.kind)));
    // A field with both storage locations displays the body value. An absent
    // body falls back to the front-matter value so metadata-only records stay
    // readable; the two locations are never concatenated or repaired.
    let value = match spec.kind {
        FieldKind::List => match body {
            Some(text) => items_from_text(&text),
            None => frontmatter.unwrap_or(FieldValue::Absent),
        },
        FieldKind::Text | FieldKind::Multiline => body
            .map(FieldValue::Text)
            .or(frontmatter)
            .filter(|value| !matches!(value, FieldValue::Text(text) if text.trim().is_empty()))
            .unwrap_or(FieldValue::Absent),
    };
    ResolvedField {
        key: spec.key.clone(),
        heading: spec.heading.clone(),
        frontmatter_key: spec.frontmatter_key.clone(),
        label: spec.label.clone().unwrap_or_else(|| human_label(&spec.key)),
        help: spec.help.clone(),
        kind: spec.kind,
        required: spec.required,
        editable: spec.verbs.contains(&FieldVerb::Edit),
        value,
    }
}

fn human_label(key: &str) -> String {
    key.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn frontmatter_value(view: &RecordView, key: &str, kind: FieldKind) -> Option<FieldValue> {
    match kind {
        FieldKind::List => {
            let items = view.list_value(key);
            (!items.is_empty()).then_some(FieldValue::Items(items))
        }
        FieldKind::Text | FieldKind::Multiline => {
            let value = view.value_text(key);
            (!value.trim().is_empty()).then_some(FieldValue::Text(value))
        }
    }
}

fn items_from_text(text: &str) -> FieldValue {
    let mut items = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix('-'))
            .unwrap_or(trimmed)
            .trim();
        if !item.is_empty() && !items.iter().any(|existing| existing == item) {
            items.push(item.to_string());
        }
    }
    if items.is_empty() {
        FieldValue::Absent
    } else {
        FieldValue::Items(items)
    }
}

fn extra_metadata(specs: &[FieldSpec], view: Option<&RecordView>) -> Vec<(String, String)> {
    let Some(view) = view else {
        return Vec::new();
    };
    let declared: Vec<&str> = specs
        .iter()
        .filter_map(|spec| spec.frontmatter_key.as_deref())
        .collect();
    view.metadata
        .iter()
        .filter(|(key, _)| {
            !crate::config::RESERVED_FIELD_KEYS.contains(&key.as_str())
                && !declared.contains(&key.as_str())
        })
        .map(|(key, _)| (key.clone(), view.value_text(key)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::registry::ArtifactRegistry;

    fn descriptor(slug: &str) -> TypeDescriptor {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        registry
            .descriptors()
            .find(|descriptor| descriptor.slug == slug)
            .cloned()
            .unwrap_or_else(|| panic!("descriptor {slug}"))
    }

    fn doc(source: &str) -> Document {
        Document::parse(source).expect("document")
    }

    fn view(source: &str, id: &str) -> RecordView {
        RecordView::from_content(source, id, "1.0.0").expect("record view")
    }

    #[test]
    fn single_role_reads_the_descriptor_field_sequence() {
        let source = "---\nid: GLO-001\ntitle: T\n---\n# GLO-001 T\n\n## Definition\nThe term.\n";
        let map = DocumentFieldMap::build(
            &descriptor("glossary"),
            false,
            &doc(source),
            Some(&view(source, "GLO-001")),
        );
        assert_eq!(map.role, FieldRole::Single);
        assert_eq!(map.fields.len(), 1);
        let field = &map.fields[0];
        assert_eq!(field.key, "definition");
        assert_eq!(field.label, "Definition");
        assert_eq!(field.value, FieldValue::Text("The term.".to_string()));
        assert!(field.required);
        assert!(field.editable);
        assert!(!field.missing_required());
        assert_eq!(map.reference_visibility, Visibility::Visible);
    }

    #[test]
    fn grouped_roles_select_group_and_member_sequences() {
        let source =
            "---\nid: REQ-001\ntitle: Group\n---\n# REQ-001 Group\n\n## Description\nBody.\n";
        let group = DocumentFieldMap::build(
            &descriptor("req"),
            true,
            &doc(source),
            Some(&view(source, "REQ-001")),
        );
        assert_eq!(group.role, FieldRole::Group);
        assert_eq!(
            group
                .fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            ["story", "description"]
        );
        assert_eq!(group.reference_visibility, Visibility::Hidden);

        let member = DocumentFieldMap::build(
            &descriptor("req"),
            false,
            &doc(source),
            Some(&view(source, "REQ-001.001")),
        );
        assert_eq!(member.role, FieldRole::Member);
        assert_eq!(member.fields.len(), 2);
        assert_eq!(member.reference_visibility, Visibility::Hidden);
    }

    #[test]
    fn list_field_reads_items_in_stored_order() {
        let source = "---
id: RUL-001
title: T
---
# RUL-001 T

## Description
D

## Rationale
R

## Applicability
A

## Requirements
- REQ-001.001
- REQ-001.002
";
        let map = DocumentFieldMap::build(
            &descriptor("rule"),
            false,
            &doc(source),
            Some(&view(source, "RUL-001")),
        );
        let field = map.field("requirements").expect("requirements");
        assert_eq!(
            field.value,
            FieldValue::Items(vec!["REQ-001.001".to_string(), "REQ-001.002".to_string()])
        );
        assert_eq!(field.kind, FieldKind::List);
    }

    #[test]
    fn required_absent_value_is_flagged() {
        let source = "---\nid: GLO-001\ntitle: T\n---\n# GLO-001 T\n";
        let map = DocumentFieldMap::build(
            &descriptor("glossary"),
            false,
            &doc(source),
            Some(&view(source, "GLO-001")),
        );
        assert_eq!(map.fields[0].value, FieldValue::Absent);
        assert!(map.fields[0].missing_required());
    }

    #[test]
    fn heading_and_frontmatter_field_displays_the_body_without_merging() {
        let source = "---\nid: AUD-001\ntitle: T\ntype: metadata\n---\n# AUD-001 T\n\n## Audit Type\nbody\n\n## Content\nContent.\n";
        let map = DocumentFieldMap::build(
            &descriptor("audit"),
            false,
            &doc(source),
            Some(&view(source, "AUD-001")),
        );
        let field = map.field("audit_type").expect("audit type");
        assert_eq!(field.value, FieldValue::Text("body".to_string()));
    }

    #[test]
    fn audit_type_reads_the_frontmatter_key_when_the_body_is_absent() {
        let source =
            "---\nid: AUD-001\ntitle: T\ntype: quality\n---\n# AUD-001 T\n\n## Content\nContent.\n";
        let map = DocumentFieldMap::build(
            &descriptor("audit"),
            false,
            &doc(source),
            Some(&view(source, "AUD-001")),
        );
        let field = map.field("audit_type").expect("audit type");
        assert_eq!(field.value, FieldValue::Text("quality".to_string()));
    }

    #[test]
    fn undeclared_frontmatter_keys_are_extra_metadata_and_declared_keys_are_not() {
        let source = "---\nid: AUD-001\ntitle: T\ntype: quality\ncustom: kept\nstatus: active\n---\n# AUD-001 T\n\n## Content\nContent.\n";
        let map = DocumentFieldMap::build(
            &descriptor("audit"),
            false,
            &doc(source),
            Some(&view(source, "AUD-001")),
        );
        assert_eq!(
            map.extra_metadata,
            vec![("custom".to_string(), "kept".to_string())]
        );
        assert!(
            !map.extra_metadata.iter().any(|(key, _)| key == "type"),
            "a declared front-matter key is a field, not extra metadata"
        );
    }

    #[test]
    fn mockup_has_no_fields_and_carries_fixed_metadata() {
        let map = DocumentFieldMap::for_mockup(
            &descriptor("ui-mockup"),
            vec![
                ("status".to_string(), "approved".to_string()),
                ("task".to_string(), "TASK-1".to_string()),
                ("specs".to_string(), "UI-001".to_string()),
                ("refs".to_string(), "docs/x.md".to_string()),
                ("superseded_by".to_string(), "MOCK-002".to_string()),
            ],
        );
        assert!(map.fields.is_empty());
        assert_eq!(map.role, FieldRole::Single);
        assert_eq!(map.fixed_metadata.len(), 5);
        assert_eq!(map.display_value("specs").as_deref(), Some("UI-001"));
        assert_eq!(map.reference_visibility, Visibility::Visible);
    }
}
