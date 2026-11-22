use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::path::Path;

/// Keys whose value is a YAML sequence of strings (ADR-0024 decision 1).
///
/// The document-statuses rule (`document_types.<slug>.statuses`) is declared by
/// [`is_list_valued_key`] because it depends on the slug. Every other key keeps
/// scalar semantics; this list must never be extended from the shape of a
/// supplied value.
pub const LIST_VALUED_KEYS: &[&str] = &["statuses"];

/// The single list-key classifier (ADR-0024 downstream constraint 1).
///
/// Classification depends only on the key. `document_types.<slug>.statuses` is
/// list-valued for exactly one non-empty slug segment.
pub fn is_list_valued_key(key: &str) -> bool {
    if LIST_VALUED_KEYS.contains(&key) {
        return true;
    }
    let mut parts = key.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("document_types"), Some(slug), Some("statuses"), None) if !slug.is_empty()
    )
}

/// Parse the comma-separated value form of a list-valued key (ADR-0024
/// decision 2).
///
/// Surrounding whitespace is trimmed from each element; order, case, internal
/// whitespace, and duplicates are preserved and no YAML interpretation is
/// applied. An empty value, an element that is empty after trimming, and a
/// value containing a control character are rejected with a message naming the
/// key. Commas are separators only: there is no escape or quote form.
pub fn parse_list_value(key: &str, raw: &str) -> Result<Vec<String>, String> {
    if raw.chars().any(char::is_control) {
        return Err(format!(
            "{}: list value must not contain control characters",
            key
        ));
    }
    if raw.trim().is_empty() {
        return Err(format!("{}: list value must not be empty", key));
    }
    let mut values = Vec::new();
    for element in raw.split(',') {
        let element = element.trim();
        if element.is_empty() {
            return Err(format!("{}: list element must not be empty", key));
        }
        values.push(element.to_string());
    }
    Ok(values)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TypeEntry {
    pub prefix: String,
    pub enabled: bool,
}

impl Default for TypeEntry {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            enabled: true,
        }
    }
}

/// The front-matter and engine-owned keys a document field may never declare
/// as its `key` or `frontmatter_key` (ADR-036 decision 23).
pub const RESERVED_FIELD_KEYS: [&str; 8] = [
    "id",
    "title",
    "status",
    "labels",
    "references",
    "created_date",
    "updated_date",
    "group",
];

/// The entry keys that select behaviour and are not configuration keys
/// (ADR-036 decision 6, REQ-07.004).
pub const RESERVED_ENTRY_KEYS: [&str; 4] = ["model", "title", "content", "id_shape"];

/// Every key a document-type entry may declare (ADR-036 decision 34,
/// TECH-077 section 2).
pub const DOCUMENT_ENTRY_KEYS: [&str; 16] = [
    "prefix",
    "directory",
    "enabled",
    "label",
    "plural",
    "order",
    "summary",
    "type_alias",
    "reference_visibility",
    "statuses",
    "default_status",
    "slots",
    "api_aliases",
    "fields",
    "group",
    "member",
];

/// Every key a field item may declare (ADR-036 decision 11,
/// TECH-077 section 3).
pub const FIELD_SPEC_KEYS: [&str; 9] = [
    "key",
    "heading",
    "frontmatter_key",
    "option",
    "label",
    "kind",
    "required",
    "verbs",
    "help",
];

/// The keys a role block may declare (ADR-036 decision 27).
pub const ROLE_BLOCK_KEYS: [&str; 2] = ["fields", "reference_visibility"];

/// The verbs a field's option is offered on (ADR-036 decision 19).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldVerb {
    Create,
    Edit,
}

impl FieldVerb {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Edit => "edit",
        }
    }
}

/// The accepted value form of a field (ADR-036 decision 17).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    #[default]
    Text,
    Multiline,
    List,
}

impl FieldKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Multiline => "multiline",
            Self::List => "list",
        }
    }
}

fn default_field_verbs() -> Vec<FieldVerb> {
    vec![FieldVerb::Create, FieldVerb::Edit]
}

/// Whether a role's references are rendered (ADR-035, ADR-036 decision 27).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
        }
    }
}

/// One declared document field (ADR-036 decision 11).
///
/// The type has exactly the documented keys: there is no `order` and no
/// `enabled`, and sequence position is the only order. Unknown keys are
/// rejected at deserialization so validation can name their selector path
/// (ADR-036 decision 34).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldSpec {
    /// Stable field identity in field maps, selectors, and form posts.
    pub key: String,
    /// Level-2 Markdown heading, written with its `## ` prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    /// Front-matter key that stores the value; may differ from `key`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontmatter_key: Option<String>,
    /// Long CLI option name without the leading `--`; derived from `key` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    /// Human-facing label for CLI help and the browser form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Value form; `text` when absent.
    #[serde(default)]
    pub kind: FieldKind,
    /// Whether `create` must supply the field.
    #[serde(default)]
    pub required: bool,
    /// Verbs whose option offers the field; defaults to `[create, edit]`.
    #[serde(default = "default_field_verbs")]
    pub verbs: Vec<FieldVerb>,
    /// Option help text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl FieldSpec {
    /// The reserved key this field names as its `key` or `frontmatter_key`.
    pub fn reserved_key(&self) -> Option<&'static str> {
        let frontmatter = self.frontmatter_key.as_deref();
        RESERVED_FIELD_KEYS
            .iter()
            .copied()
            .find(|reserved| self.key == *reserved || frontmatter == Some(*reserved))
    }

    /// Whether the field declares a body heading, a front-matter key, or both.
    pub fn stores_value(&self) -> bool {
        self.heading.is_some() || self.frontmatter_key.is_some()
    }
}

/// One role's field sequence and optional reference visibility (ADR-036
/// decision 27).
///
/// An absent member is `None`; a present empty `fields` collection is
/// `Some(vec![])`, so the two stay distinguishable (ADR-036 decision 32).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct RoleFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_visibility: Option<Visibility>,
}

/// One browser list column declared by a document type (ADR-041).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct DocTypeEntry {
    pub prefix: String,
    pub directory: String,
    pub enabled: bool,
    /// Custom document statuses, in declared order (REQ-07.001).
    ///
    /// `None` means the entry declares no status list; `Some(vec![])` is an
    /// explicit empty declaration and stays distinguishable from an absent one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
    /// Initial status of a new record; falls back to `active` (REQ-07.001).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
    /// Presentation label; filled from the seed when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Presentation plural; filled from the seed when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plural: Option<String>,
    /// Presentation order; filled from the seed when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    /// Presentation summary; filled from the seed when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Alternate type name; filled from the seed when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_alias: Option<String>,
    /// Reference visibility of a single-role type (ADR-035).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_visibility: Option<Visibility>,
    /// Declared browser list columns (ADR-041).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<Slot>>,
    /// Declared API aliases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_aliases: Option<Vec<String>>,
    /// Field sequence of a single-role type (ADR-036 decision 26).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldSpec>>,
    /// Group role of a grouped type (ADR-036 decision 27).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<RoleFields>,
    /// Member role of a grouped type (ADR-036 decision 27).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<RoleFields>,
}

impl Default for DocTypeEntry {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            directory: String::new(),
            enabled: true,
            statuses: None,
            default_status: None,
            label: None,
            plural: None,
            order: None,
            summary: None,
            type_alias: None,
            reference_visibility: None,
            slots: None,
            api_aliases: None,
            fields: None,
            group: None,
            member: None,
        }
    }
}

/// Canonical ordered seeding table for the eleven default document types:
/// `(slug, prefix, directory, has_front_matter_status)`.
///
/// The order is the schema declaration order and is asserted against the
/// interactive `init` artifact list. `statuses` is seeded for every type with
/// front-matter status; `ui-mockup` has none (REQ-05.008).
pub(crate) const DEFAULT_DOCUMENT_TYPES: [(&str, &str, &str, bool); 11] = [
    ("req", "REQ", "requirements/business/", true),
    ("nfr", "NFR", "requirements/non-functional/", true),
    ("glossary", "GLO", "glossary/", true),
    ("rule", "RUL", "business-rules/", true),
    ("ui-spec", "UI", "specifications/ui/", true),
    ("ui-mockup", "MOCK", "specifications/ui/mockups/", false),
    ("audit", "AUD", "audit-records/", true),
    ("tech-spec", "TECH", "technical/", true),
    ("use-case", "UC", "use-cases/", true),
    ("test-case", "TC", "test-cases/", true),
    ("test-report", "TR", "test-reports/", true),
];

fn mapping_get<'a>(mapping: &'a yaml_serde::Mapping, key: &str) -> Option<&'a yaml_serde::Value> {
    mapping.get(yaml_serde::Value::String(key.to_string()))
}

fn mapping_contains(mapping: &yaml_serde::Mapping, key: &str) -> bool {
    mapping.contains_key(yaml_serde::Value::String(key.to_string()))
}

fn value_as_str(value: &yaml_serde::Value) -> Option<&str> {
    match value {
        yaml_serde::Value::String(text) => Some(text),
        _ => None,
    }
}

fn key_name(key: &yaml_serde::Value) -> String {
    match key {
        yaml_serde::Value::String(text) => text.clone(),
        other => render_yaml_flow(other),
    }
}

fn document_model(slug: &str) -> crate::registry::DocumentModel {
    crate::registry::DocumentModel::for_slug(slug)
}

fn field_selector_labels(sequence: &[yaml_serde::Value]) -> Vec<String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in sequence {
        if let yaml_serde::Value::Mapping(mapping) = item
            && let Some(key) = mapping_get(mapping, "key").and_then(value_as_str)
            && !key.is_empty()
        {
            *counts.entry(key.to_string()).or_default() += 1;
        }
    }
    sequence
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let key = match item {
                yaml_serde::Value::Mapping(mapping) => {
                    mapping_get(mapping, "key").and_then(value_as_str)
                }
                _ => None,
            };
            match key {
                Some(key) if counts.get(key).copied().unwrap_or(0) == 1 && !key.is_empty() => {
                    key.to_string()
                }
                _ => index.to_string(),
            }
        })
        .collect()
}

fn push_unknown_keys(
    mapping: &yaml_serde::Mapping,
    allowed: &[&str],
    base: &str,
    reserved_message: &str,
    errors: &mut Vec<String>,
) {
    let mut unknown: Vec<(String, String)> = mapping
        .keys()
        .map(key_name)
        .filter(|name| !allowed.contains(&name.as_str()))
        .map(|name| {
            let message = if RESERVED_ENTRY_KEYS.contains(&name.as_str()) {
                reserved_message.to_string()
            } else {
                "unknown key".to_string()
            };
            (name, message)
        })
        .collect();
    unknown.sort();
    for (name, message) in unknown {
        errors.push(format!("{}.{}: {}", base, name, message));
    }
}

fn validate_enum_key(
    mapping: &yaml_serde::Mapping,
    key: &str,
    allowed: &[&str],
    base: &str,
    errors: &mut Vec<String>,
) {
    let Some(value) = mapping_get(mapping, key) else {
        return;
    };
    let valid = value_as_str(value).is_some_and(|text| allowed.contains(&text));
    if !valid {
        let rendered = match allowed {
            [] => String::new(),
            [only] => (*only).to_string(),
            [first, second] => format!("{} or {}", first, second),
            _ => {
                let (last, rest) = allowed.split_last().expect("three or more values");
                format!("{}, or {}", rest.join(", "), last)
            }
        };
        errors.push(format!("{}.{}: must be {}", base, key, rendered));
    }
}

fn validate_field_sequence(
    slug: &str,
    path: &str,
    value: &yaml_serde::Value,
    errors: &mut Vec<String>,
) {
    let yaml_serde::Value::Sequence(sequence) = value else {
        errors.push(format!(
            "document_types.{}.{}: must be a sequence",
            slug, path
        ));
        return;
    };
    let labels = field_selector_labels(sequence);
    for (index, item) in sequence.iter().enumerate() {
        let selector = format!("document_types.{}.{}[{}]", slug, path, labels[index]);
        let yaml_serde::Value::Mapping(field) = item else {
            errors.push(format!("{}: field must be a mapping", selector));
            continue;
        };
        let mut unknown: Vec<String> = field
            .keys()
            .map(key_name)
            .filter(|name| !FIELD_SPEC_KEYS.contains(&name.as_str()))
            .collect();
        unknown.sort();
        for name in unknown {
            let message = if name == "order" || name == "enabled" {
                "not a valid field key"
            } else {
                "unknown key"
            };
            errors.push(format!("{}.{}: {}", selector, name, message));
        }
        match mapping_get(field, "key").and_then(value_as_str) {
            Some(key) if !key.is_empty() => {}
            _ => errors.push(format!("{}.key: must be a non-empty string", selector)),
        }
        validate_enum_key(
            field,
            "kind",
            &["text", "multiline", "list"],
            &selector,
            errors,
        );
        if let Some(verbs) = mapping_get(field, "verbs") {
            match verbs {
                yaml_serde::Value::Sequence(values) => {
                    for verb in values {
                        let valid = value_as_str(verb)
                            .is_some_and(|text| text == "create" || text == "edit");
                        if !valid {
                            let name = value_as_str(verb).unwrap_or("<non-string>");
                            errors.push(format!(
                                "{}.verbs: invalid verb {}; must be create or edit",
                                selector, name
                            ));
                        }
                    }
                }
                _ => errors.push(format!(
                    "{}.verbs: must be a sequence of create and edit",
                    selector
                )),
            }
        }
        if let Some(required) = mapping_get(field, "required")
            && !matches!(required, yaml_serde::Value::Bool(_))
        {
            errors.push(format!("{}.required: must be a boolean", selector));
        }
    }
}

fn validate_role_value(
    slug: &str,
    role: &str,
    value: &yaml_serde::Value,
    errors: &mut Vec<String>,
) {
    let base = format!("document_types.{}.{}", slug, role);
    let yaml_serde::Value::Mapping(mapping) = value else {
        errors.push(format!("{}: must be a mapping", base));
        return;
    };
    push_unknown_keys(
        mapping,
        &ROLE_BLOCK_KEYS,
        &base,
        "not a configuration key",
        errors,
    );
    validate_enum_key(
        mapping,
        "reference_visibility",
        &["visible", "hidden"],
        &base,
        errors,
    );
    if let Some(fields) = mapping_get(mapping, "fields") {
        validate_field_sequence(slug, &format!("{}.fields", role), fields, errors);
    }
}

/// Validate the declared shape of every `document_types` entry.
///
/// This runs on the raw file before deserialization so an unknown key or an
/// unknown enum value is reported with its selector path instead of a serde
/// message. Diagnostics are aggregated in a deterministic order: slugs in
/// ascending order, entry keys, the role blocks, and finally the field
/// sequence (TECH-077 sections 1 and 8).
pub fn validate_document_types_declarations(root: &yaml_serde::Value) -> Vec<String> {
    let Some(document_types) = mapping_get_declared(root, "document_types") else {
        return Vec::new();
    };
    let yaml_serde::Value::Mapping(entries) = document_types else {
        return vec!["document_types: must be a mapping".to_string()];
    };

    let mut errors = Vec::new();
    let mut slugs: Vec<String> = Vec::new();
    for key in entries.keys() {
        match key {
            yaml_serde::Value::String(slug) => slugs.push(slug.clone()),
            other => errors.push(format!(
                "document_types: keys must be strings, found {}",
                render_yaml_flow(other)
            )),
        }
    }
    slugs.sort();

    for slug in slugs {
        let entry = entries
            .get(yaml_serde::Value::String(slug.clone()))
            .expect("slug came from the mapping keys");
        let base = format!("document_types.{}", slug);
        let yaml_serde::Value::Mapping(mapping) = entry else {
            errors.push(format!("{}: must be a mapping", base));
            continue;
        };
        push_unknown_keys(
            mapping,
            &DOCUMENT_ENTRY_KEYS,
            &base,
            "not a configuration key",
            &mut errors,
        );
        validate_enum_key(
            mapping,
            "reference_visibility",
            &["visible", "hidden"],
            &base,
            &mut errors,
        );

        let grouped = document_model(&slug) == crate::registry::DocumentModel::GroupedMarkdown;
        if grouped {
            for role in ["group", "member"] {
                if let Some(value) = mapping_get(mapping, role) {
                    validate_role_value(&slug, role, value, &mut errors);
                }
            }
            if mapping_contains(mapping, "fields") {
                errors.push(format!("{}.fields: not allowed on a grouped type", base));
            }
        } else {
            for role in ["group", "member"] {
                if mapping_contains(mapping, role) {
                    errors.push(format!(
                        "{}.{}: not allowed on a non-grouped type",
                        base, role
                    ));
                }
            }
            if let Some(fields) = mapping_get(mapping, "fields") {
                validate_field_sequence(&slug, "fields", fields, &mut errors);
            }
        }
    }
    errors
}

fn mapping_get_declared<'a>(
    root: &'a yaml_serde::Value,
    key: &str,
) -> Option<&'a yaml_serde::Value> {
    match root {
        yaml_serde::Value::Mapping(mapping) => mapping_get(mapping, key),
        _ => None,
    }
}

/// Fill one declared entry from its seed entry.
///
/// Scalars fill when absent and override when present; every collection
/// replaces wholesale, including an empty one. `prefix` and `directory` are
/// never filled (ADR-036 decision 32).
fn fill_entry_from_seed(
    seed: &yaml_serde::Value,
    declared: &yaml_serde::Value,
) -> yaml_serde::Value {
    match (seed, declared) {
        (yaml_serde::Value::Mapping(seed_map), yaml_serde::Value::Mapping(declared_map)) => {
            let mut merged = seed_map.clone();
            for (key, value) in declared_map {
                let name = key_name(key);
                if matches!(name.as_str(), "group" | "member")
                    && let Some(seed_role) = mapping_get(seed_map, &name)
                    && mapping_contains(declared_map, &name)
                {
                    let role = fill_entry_from_seed(seed_role, value);
                    merged.insert(key.clone(), role);
                } else {
                    merged.insert(key.clone(), value.clone());
                }
            }
            // `prefix` and `directory` are declared by every entry.
            for required in ["prefix", "directory"] {
                if !mapping_contains(declared_map, required) {
                    merged.remove(yaml_serde::Value::String(required.to_string()));
                }
            }
            yaml_serde::Value::Mapping(merged)
        }
        (_, declared) => declared.clone(),
    }
}

/// Resolve a declared `document_types` mapping against the seed.
///
/// The entry set stays wholesale: only declared slugs survive. Each surviving
/// entry is filled from the seed entry of the same slug, and a user-defined
/// slug is left exactly as declared (its defaults belong to the seed table).
pub fn resolve_document_types(declared: &yaml_serde::Value) -> yaml_serde::Value {
    let yaml_serde::Value::Mapping(entries) = declared else {
        return declared.clone();
    };
    let seed = Config::default();
    let mut resolved = yaml_serde::Mapping::new();
    for (key, declared_entry) in entries {
        let slug = key_name(key);
        let value = match seed.document_types.get(&slug) {
            Some(seed_entry) => match yaml_serde::to_value(seed_entry) {
                Ok(seed_value) => fill_entry_from_seed(&seed_value, declared_entry),
                Err(_) => declared_entry.clone(),
            },
            None => declared_entry.clone(),
        };
        resolved.insert(key.clone(), value);
    }
    yaml_serde::Value::Mapping(resolved)
}

/// One component of a configuration selector path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorSegment {
    pub key: String,
    pub selector: Option<Selector>,
}

/// A keyed or indexed sequence element selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Index(usize),
    Key(String),
}

/// Parse a configuration selector such as
/// `document_types.req.group.fields[story].heading`.
pub fn parse_selector(selector: &str) -> Result<Vec<SelectorSegment>, String> {
    if selector.is_empty() {
        return Err("selector must not be empty".to_string());
    }
    let mut segments = Vec::new();
    for part in selector.split('.') {
        if part.is_empty() {
            return Err("selector contains an empty segment".to_string());
        }
        if let Some(open) = part.find('[') {
            if !part.ends_with(']') {
                return Err("selector has an unterminated bracket".to_string());
            }
            let key = &part[..open];
            let inner = &part[open + 1..part.len() - 1];
            if key.is_empty() || inner.is_empty() || inner.contains('[') || inner.contains(']') {
                return Err("selector has an invalid bracket form".to_string());
            }
            let selector = if inner.chars().all(|c| c.is_ascii_digit()) {
                Selector::Index(
                    inner
                        .parse::<usize>()
                        .map_err(|_| "selector index is out of range".to_string())?,
                )
            } else {
                Selector::Key(inner.to_string())
            };
            segments.push(SelectorSegment {
                key: key.to_string(),
                selector: Some(selector),
            });
        } else {
            segments.push(SelectorSegment {
                key: part.to_string(),
                selector: None,
            });
        }
    }
    Ok(segments)
}

/// Whether a selector path is syntactically valid and sanitized.
pub fn valid_config_selector(selector: &str) -> bool {
    let Ok(segments) = parse_selector(selector) else {
        return false;
    };
    segments.iter().all(|segment| {
        if crate::sanitize::sanitize_path_component(&segment.key).is_none() {
            return false;
        }
        match &segment.selector {
            None => true,
            Some(Selector::Index(_)) => true,
            Some(Selector::Key(key)) => crate::sanitize::sanitize_path_component(key).is_some(),
        }
    })
}

/// Whether a selector addresses a field sequence or a field property.
pub fn selector_targets_field(selector: &str) -> bool {
    parse_selector(selector)
        .map(|segments| segments.iter().any(|segment| segment.key == "fields"))
        .unwrap_or(false)
}

/// Resolve one selector path against an effective configuration value.
pub fn resolve_selector<'a>(
    root: &'a yaml_serde::Value,
    selector: &str,
) -> Result<&'a yaml_serde::Value, String> {
    let segments = parse_selector(selector)?;
    let mut current = root;
    for segment in segments {
        let yaml_serde::Value::Mapping(mapping) = current else {
            return Err("selector did not resolve".to_string());
        };
        current = mapping_get(mapping, &segment.key)
            .ok_or_else(|| "selector did not resolve".to_string())?;
        match segment.selector {
            None => {}
            Some(Selector::Index(index)) => {
                let yaml_serde::Value::Sequence(sequence) = current else {
                    return Err("selector did not resolve".to_string());
                };
                current = sequence
                    .get(index)
                    .ok_or_else(|| "selector index is out of range".to_string())?;
            }
            Some(Selector::Key(key)) => {
                let yaml_serde::Value::Sequence(sequence) = current else {
                    return Err("selector did not resolve".to_string());
                };
                let mut matches = sequence.iter().filter(|item| {
                    matches!(
                        item,
                        yaml_serde::Value::Mapping(mapping)
                            if mapping_get(mapping, "key").and_then(value_as_str) == Some(key.as_str())
                    )
                });
                let first = matches
                    .next()
                    .ok_or_else(|| "selector did not resolve".to_string())?;
                if matches.next().is_some() {
                    return Err("selector is ambiguous".to_string());
                }
                current = first;
            }
        }
    }
    Ok(current)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub project_name: Option<String>,
    pub task_prefix: String,
    pub milestone_prefix: String,
    pub release_prefix: String,
    pub id_padding: u32,
    pub date_format: String,
    pub editor: Option<String>,
    pub backlog_directory: String,
    pub decision_directory: String,
    pub doc_directory: String,
    pub statuses: Vec<String>,
    pub default_status: String,
    pub decision_types: BTreeMap<String, TypeEntry>,
    pub document_types: BTreeMap<String, DocTypeEntry>,
    pub browser_host: String,
    pub browser_port: u16,
    pub browser_open: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut decision_types = BTreeMap::new();
        decision_types.insert(
            "adr".to_string(),
            TypeEntry {
                prefix: "ADR".to_string(),
                enabled: true,
            },
        );
        decision_types.insert(
            "bdr".to_string(),
            TypeEntry {
                prefix: "BDR".to_string(),
                enabled: true,
            },
        );

        let mut document_types = BTreeMap::new();
        // `has_status` marks the front-matter-backed types that declare the
        // seeded status vocabulary. `ui-mockup` has no front-matter status, so
        // a `statuses` key on it is invalid configuration (REQ-05.008).
        for (k, p, d, has_status) in DEFAULT_DOCUMENT_TYPES {
            document_types.insert(
                k.to_string(),
                DocTypeEntry {
                    prefix: p.to_string(),
                    directory: d.to_string(),
                    enabled: true,
                    statuses: if has_status {
                        Some(vec!["active".to_string(), "archived".to_string()])
                    } else {
                        None
                    },
                    default_status: None,
                    ..Default::default()
                },
            );
        }

        Self {
            project_name: None,
            task_prefix: "TASK".to_string(),
            milestone_prefix: "MS".to_string(),
            release_prefix: "REL".to_string(),
            id_padding: 3,
            date_format: "yyyy-mm-dd".to_string(),
            editor: None,
            backlog_directory: "backlog/".to_string(),
            decision_directory: "decisions/".to_string(),
            doc_directory: "docs/".to_string(),
            statuses: vec![
                "ToDo".to_string(),
                "Initiated".to_string(),
                "Refined".to_string(),
                "Implemented".to_string(),
                "Validated".to_string(),
                "Done".to_string(),
            ],
            default_status: "ToDo".to_string(),
            decision_types,
            document_types,
            browser_host: "127.0.0.1".to_string(),
            browser_port: 6420,
            browser_open: true,
        }
    }
}

impl Config {
    /// Effective project name, falling back to the directory name.
    pub fn effective_project_name(&self, project_dir: &Path) -> String {
        if let Some(n) = &self.project_name
            && !n.trim().is_empty()
        {
            return n.clone();
        }
        project_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string()
    }

    /// Effective editor command, falling back to `$VISUAL` then `$EDITOR`.
    pub fn effective_editor(&self) -> Option<String> {
        if let Some(e) = &self.editor
            && !e.is_empty()
        {
            return Some(e.clone());
        }
        if let Ok(v) = std::env::var("VISUAL")
            && !v.is_empty()
        {
            return Some(v);
        }
        if let Ok(v) = std::env::var("EDITOR")
            && !v.is_empty()
        {
            return Some(v);
        }
        None
    }

    /// Whether a dotted configuration key addresses a folder.
    pub fn is_folder_key(key: &str) -> bool {
        key == "backlog_directory"
            || key == "decision_directory"
            || key == "doc_directory"
            || (key.starts_with("document_types.")
                && key.ends_with(".directory")
                && key.split('.').count() == 3)
    }

    pub fn validate(&self, project_dir: Option<&Path>) -> Result<(), String> {
        let mut errors: Vec<String> = Vec::new();

        // Top-level fields in struct order.
        if self.task_prefix.is_empty()
            || self.milestone_prefix.is_empty()
            || self.release_prefix.is_empty()
        {
            errors.push("ID prefixes must not be empty".to_string());
        }
        if !(1..=6).contains(&self.id_padding) {
            errors.push("id_padding: must be between 1 and 6".to_string());
        }
        for (label, value) in [
            ("backlog_directory", &self.backlog_directory),
            ("decision_directory", &self.decision_directory),
            ("doc_directory", &self.doc_directory),
        ] {
            if crate::sanitize::sanitize_path_string(value).is_none() {
                errors.push(format!("{}: invalid folder value", label));
            }
        }

        // statuses
        if self.statuses.is_empty() {
            errors.push("statuses must not be empty".to_string());
        }
        let mut seen_statuses: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut dup_statuses: Vec<String> = Vec::new();
        for s in &self.statuses {
            if !seen_statuses.insert(s.clone()) && !dup_statuses.contains(s) {
                dup_statuses.push(s.clone());
            }
        }
        for s in &dup_statuses {
            errors.push(format!("duplicate status: {}", s));
        }
        if !self.statuses.contains(&self.default_status) {
            errors.push(format!(
                "default_status '{}' is not one of the configured statuses",
                self.default_status
            ));
        }

        if let Err(reason) = validate_browser_host(&self.browser_host) {
            errors.push(format!("browser_host: {}", reason));
        }
        if self.browser_port == 0 {
            errors.push("browser_port: must be between 1 and 65535".to_string());
        }

        // document_types entries in BTreeMap order.
        let mut seen_prefixes: Vec<(String, String)> = Vec::new();
        let mut seen_dirs_enabled: Vec<(String, String)> = Vec::new();
        for (slug, entry) in &self.document_types {
            // slug
            if !is_valid_slug(slug) {
                errors.push(format!("document_types.{}: invalid slug", slug));
            }
            // reserved names (case-insensitive)
            for builtin in crate::cli::BUILTIN_COMMANDS {
                if slug.eq_ignore_ascii_case(builtin.name) {
                    errors.push(format!(
                        "document_types.{}: slug conflicts with built-in type {}",
                        slug, builtin.name
                    ));
                    break;
                }
            }

            // prefix
            if entry.prefix.is_empty() {
                errors.push(format!("document_types.{}.prefix: must not be empty", slug));
            } else if !prefix_valid(&entry.prefix) {
                errors.push(format!(
                    "document_types.{}.prefix: must not contain whitespace, control characters, path separators, or '..'",
                    slug
                ));
            }
            // prefix uniqueness (case-insensitive) across all entries
            if !entry.prefix.is_empty() {
                let lower = entry.prefix.to_lowercase();
                if let Some((_, other)) = seen_prefixes.iter().find(|(p, _)| *p == lower) {
                    errors.push(format!(
                        "document_types.{}.prefix: {} is already used by {}",
                        slug, entry.prefix, other
                    ));
                } else {
                    seen_prefixes.push((lower, slug.clone()));
                }
            }

            // directory
            if entry.directory.is_empty() {
                errors.push(format!(
                    "document_types.{}.directory: must not be empty",
                    slug
                ));
            } else if !dir_string_valid(&entry.directory) {
                errors.push(format!(
                    "document_types.{}.directory: value must be relative and stay inside the project directory: {}",
                    slug, entry.directory
                ));
            } else {
                if let Some(project_dir) = project_dir {
                    let full = project_dir.join(&self.doc_directory).join(&entry.directory);
                    if crate::context::Context::ensure_within(project_dir, &full).is_err() {
                        errors.push(format!(
                            "document_types.{}.directory: value must be relative and stay inside the project directory: {}",
                            slug, entry.directory
                        ));
                    }
                }
                // duplicate among enabled entries
                if entry.enabled {
                    if let Some((_, other)) = seen_dirs_enabled
                        .iter()
                        .find(|(d, _)| *d == entry.directory)
                    {
                        errors.push(format!(
                            "document_types.{}.directory: directory is already used by {}",
                            slug, other
                        ));
                    } else {
                        seen_dirs_enabled.push((entry.directory.clone(), slug.clone()));
                    }
                }
            }

            // statuses
            validate_doc_entry_statuses(slug, entry, &mut errors);

            // Role blocks and field sequences (ADR-036 decisions 21 to 27).
            validate_doc_entry_fields(slug, entry, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    /// Flatten effective config into dotted keys in stable order.
    pub fn flattened(&self, project_dir: &Path) -> Vec<(String, String)> {
        let mut out = Vec::new();
        out.push((
            "project_name".to_string(),
            self.effective_project_name(project_dir),
        ));
        out.push(("task_prefix".to_string(), self.task_prefix.clone()));
        out.push((
            "milestone_prefix".to_string(),
            self.milestone_prefix.clone(),
        ));
        out.push(("release_prefix".to_string(), self.release_prefix.clone()));
        out.push(("id_padding".to_string(), self.id_padding.to_string()));
        out.push(("date_format".to_string(), self.date_format.clone()));
        out.push((
            "editor".to_string(),
            self.effective_editor().unwrap_or_default(),
        ));
        out.push((
            "backlog_directory".to_string(),
            self.backlog_directory.clone(),
        ));
        out.push((
            "decision_directory".to_string(),
            self.decision_directory.clone(),
        ));
        out.push(("doc_directory".to_string(), self.doc_directory.clone()));
        // statuses as array
        out.push((
            "statuses".to_string(),
            format!("[{}]", self.statuses.to_vec().join(", ")),
        ));
        out.push(("default_status".to_string(), self.default_status.clone()));
        for (k, v) in &self.decision_types {
            out.push((format!("decision_types.{}.prefix", k), v.prefix.clone()));
            out.push((
                format!("decision_types.{}.enabled", k),
                v.enabled.to_string(),
            ));
        }
        for (k, v) in &self.document_types {
            out.push((format!("document_types.{}.prefix", k), v.prefix.clone()));
            out.push((
                format!("document_types.{}.directory", k),
                v.directory.clone(),
            ));
            out.push((
                format!("document_types.{}.enabled", k),
                v.enabled.to_string(),
            ));
            if let Some(statuses) = &v.statuses {
                out.push((
                    format!("document_types.{}.statuses", k),
                    format!("[{}]", statuses.join(", ")),
                ));
            }
            if let Some(default_status) = &v.default_status {
                out.push((
                    format!("document_types.{}.default_status", k),
                    default_status.clone(),
                ));
            }
            for (label, value) in [
                ("label", v.label.as_ref()),
                ("plural", v.plural.as_ref()),
                ("summary", v.summary.as_ref()),
                ("type_alias", v.type_alias.as_ref()),
            ] {
                if let Some(value) = value {
                    out.push((format!("document_types.{}.{}", k, label), value.clone()));
                }
            }
            if let Some(order) = v.order {
                out.push((format!("document_types.{}.order", k), order.to_string()));
            }
            if let Some(visibility) = v.reference_visibility {
                out.push((
                    format!("document_types.{}.reference_visibility", k),
                    visibility.as_str().to_string(),
                ));
            }
            if let Some(slots) = &v.slots {
                for (index, slot) in slots.iter().enumerate() {
                    out.push((
                        format!("document_types.{}.slots[{}].key", k, index),
                        slot.key.clone(),
                    ));
                    out.push((
                        format!("document_types.{}.slots[{}].label", k, index),
                        slot.label.clone(),
                    ));
                }
            }
            if let Some(aliases) = &v.api_aliases {
                out.push((
                    format!("document_types.{}.api_aliases", k),
                    format!("[{}]", aliases.join(", ")),
                ));
            }
            if let Some(fields) = &v.fields {
                push_field_leaves(&mut out, &format!("document_types.{}.fields", k), fields);
            }
            if let Some(group) = &v.group {
                if let Some(visibility) = group.reference_visibility {
                    out.push((
                        format!("document_types.{}.group.reference_visibility", k),
                        visibility.as_str().to_string(),
                    ));
                }
                if let Some(fields) = &group.fields {
                    push_field_leaves(
                        &mut out,
                        &format!("document_types.{}.group.fields", k),
                        fields,
                    );
                }
            }
            if let Some(member) = &v.member {
                if let Some(visibility) = member.reference_visibility {
                    out.push((
                        format!("document_types.{}.member.reference_visibility", k),
                        visibility.as_str().to_string(),
                    ));
                }
                if let Some(fields) = &member.fields {
                    push_field_leaves(
                        &mut out,
                        &format!("document_types.{}.member.fields", k),
                        fields,
                    );
                }
            }
        }
        out.push(("browser_host".to_string(), self.browser_host.clone()));
        out.push(("browser_port".to_string(), self.browser_port.to_string()));
        out.push(("browser_open".to_string(), self.browser_open.to_string()));
        out
    }

    /// Build the value written by `init`: the full configuration with each
    /// document entry reduced to the keys that are never filled from the seed
    /// (`prefix`, `directory`, plus `enabled` only when disabled)
    /// (REQ-07.002, RE-010 D14).
    pub fn init_value(&self) -> Result<yaml_serde::Value, String> {
        let mut value = yaml_serde::to_value(self).map_err(|error| error.to_string())?;
        let yaml_serde::Value::Mapping(root) = &mut value else {
            return Err("configuration must serialize to a mapping".to_string());
        };
        let mut document_types = yaml_serde::Mapping::new();
        for (slug, entry) in &self.document_types {
            let mut minimal = yaml_serde::Mapping::new();
            minimal.insert(
                yaml_serde::Value::String("prefix".to_string()),
                yaml_serde::Value::String(entry.prefix.clone()),
            );
            minimal.insert(
                yaml_serde::Value::String("directory".to_string()),
                yaml_serde::Value::String(entry.directory.clone()),
            );
            if !entry.enabled {
                minimal.insert(
                    yaml_serde::Value::String("enabled".to_string()),
                    yaml_serde::Value::Bool(false),
                );
            }
            document_types.insert(
                yaml_serde::Value::String(slug.clone()),
                yaml_serde::Value::Mapping(minimal),
            );
        }
        root.insert(
            yaml_serde::Value::String("document_types".to_string()),
            yaml_serde::Value::Mapping(document_types),
        );
        Ok(value)
    }
}

fn is_valid_slug(s: &str) -> bool {
    if s.is_empty() || s.starts_with('-') {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn prefix_valid(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '/' || c == '\\')
    {
        return false;
    }
    !s.contains("..")
}

fn dir_string_valid(s: &str) -> bool {
    if s.is_empty() || s.contains('\\') || s.contains('\0') || s.chars().any(|c| c.is_control()) {
        return false;
    }
    let path = std::path::Path::new(s);
    if path.is_absolute() {
        return false;
    }
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir
            | std::path::Component::Prefix(_)
            | std::path::Component::RootDir => return false,
            std::path::Component::Normal(name) => {
                let name = match name.to_str() {
                    Some(n) => n,
                    None => return false,
                };
                if name.is_empty() || name.contains('\0') {
                    return false;
                }
            }
            std::path::Component::CurDir => {}
        }
    }
    true
}

fn validate_doc_entry_statuses(slug: &str, entry: &DocTypeEntry, errors: &mut Vec<String>) {
    let ui_mockup = slug == "ui-mockup";
    if let Some(statuses) = &entry.statuses {
        if ui_mockup {
            errors.push(format!(
                "document_types.{}.statuses: not allowed for a type without a front-matter status",
                slug
            ));
        } else if statuses.is_empty() {
            errors.push(format!(
                "document_types.{}.statuses: must not be empty",
                slug
            ));
        } else {
            let active_count = statuses.iter().filter(|v| v.as_str() == "active").count();
            if active_count != 1 {
                errors.push(format!(
                    "document_types.{}.statuses: \"active\" must be declared exactly once",
                    slug
                ));
            }
            let archived_count = statuses.iter().filter(|v| v.as_str() == "archived").count();
            if archived_count != 1 {
                errors.push(format!(
                    "document_types.{}.statuses: \"archived\" must be declared exactly once",
                    slug
                ));
            }
            let mut seen: Vec<&str> = Vec::new();
            let mut dup_values: Vec<String> = Vec::new();
            for v in statuses {
                if seen.contains(&v.as_str()) && !dup_values.contains(v) {
                    dup_values.push(v.clone());
                }
                seen.push(v.as_str());
            }
            for v in &dup_values {
                errors.push(format!(
                    "document_types.{}.statuses: duplicate value {}",
                    slug, v
                ));
            }
            let mut invalid_values: Vec<String> = Vec::new();
            for v in statuses {
                if !is_valid_slug(v) && !invalid_values.contains(v) {
                    invalid_values.push(v.clone());
                }
            }
            for v in &invalid_values {
                errors.push(format!(
                    "document_types.{}.statuses: invalid status value {}",
                    slug, v
                ));
            }
        }
    }
    if let Some(default_status) = &entry.default_status {
        if ui_mockup {
            errors.push(format!(
                "document_types.{}.default_status: not allowed for a type without a front-matter status",
                slug
            ));
        } else {
            let declared: Vec<String> = entry
                .statuses
                .clone()
                .unwrap_or_else(|| vec!["active".to_string(), "archived".to_string()]);
            if !declared.contains(default_status) {
                errors.push(format!(
                    "document_types.{}.default_status: {} is not a declared status",
                    slug, default_status
                ));
            }
            if default_status == "archived" {
                errors.push(format!(
                    "document_types.{}.default_status: must not be \"archived\"",
                    slug
                ));
            }
        }
    }
}

fn spec_selector_labels(fields: &[FieldSpec]) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for field in fields {
        if !field.key.is_empty() {
            *counts.entry(field.key.as_str()).or_default() += 1;
        }
    }
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if !field.key.is_empty() && counts.get(field.key.as_str()) == Some(&1) {
                field.key.clone()
            } else {
                index.to_string()
            }
        })
        .collect()
}

fn validate_field_specs(slug: &str, path: &str, fields: &[FieldSpec], errors: &mut Vec<String>) {
    let labels = spec_selector_labels(fields);
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for field in fields {
        if !field.key.is_empty() {
            *counts.entry(field.key.as_str()).or_default() += 1;
        }
    }
    for (index, field) in fields.iter().enumerate() {
        let selector = format!("document_types.{}.{}[{}]", slug, path, labels[index]);
        if RESERVED_FIELD_KEYS.contains(&field.key.as_str()) {
            errors.push(format!(
                "{}.key: reserved field key: {}",
                selector, field.key
            ));
        }
        if let Some(frontmatter_key) = field.frontmatter_key.as_deref()
            && RESERVED_FIELD_KEYS.contains(&frontmatter_key)
        {
            errors.push(format!(
                "{}.frontmatter_key: reserved field key: {}",
                selector, frontmatter_key
            ));
        }
        if field.key.is_empty() {
            errors.push(format!("{}.key: must not be empty", selector));
        }
        if !field.stores_value() {
            errors.push(format!(
                "{}: must declare heading or frontmatter_key",
                selector
            ));
        }
        if !field.key.is_empty() && counts.get(field.key.as_str()).copied().unwrap_or(0) > 1 {
            errors.push(format!(
                "document_types.{}.{}[{}].key: duplicate field key: {}",
                slug, path, index, field.key
            ));
        }
    }
}

fn validate_doc_entry_fields(slug: &str, entry: &DocTypeEntry, errors: &mut Vec<String>) {
    let grouped = document_model(slug) == crate::registry::DocumentModel::GroupedMarkdown;
    if grouped {
        if entry.fields.is_some() {
            errors.push(format!(
                "document_types.{}.fields: not allowed on a grouped type",
                slug
            ));
        }
        for (name, role) in [("group", &entry.group), ("member", &entry.member)] {
            if let Some(role) = role
                && let Some(fields) = &role.fields
            {
                validate_field_specs(slug, &format!("{}.fields", name), fields, errors);
            }
        }
    } else {
        for (name, present) in [
            ("group", entry.group.is_some()),
            ("member", entry.member.is_some()),
        ] {
            if present {
                errors.push(format!(
                    "document_types.{}.{}: not allowed on a non-grouped type",
                    slug, name
                ));
            }
        }
        if let Some(fields) = &entry.fields {
            validate_field_specs(slug, "fields", fields, errors);
        }
    }
}

/// Validate a browser bind host (specification section 1.10).
///
/// A host is an IPv4 literal, an IPv6 literal, or a DNS host name. The value
/// must be at most 255 bytes, must not contain leading or trailing whitespace,
/// must not contain whitespace, control characters, or any of `/ \ @ ? # , %`,
/// and a host name must start with an ASCII letter or digit, may contain only
/// ASCII letters, digits, `.` and `-`, and must not end with `.` or `-`.
pub fn validate_browser_host(host: &str) -> Result<(), String> {
    if host.is_empty() {
        return Err("must not be empty".to_string());
    }
    if host.len() > 255 {
        return Err("must be at most 255 bytes".to_string());
    }
    if host.trim() != host {
        return Err("must not contain leading or trailing whitespace".to_string());
    }
    if host.chars().any(|c| {
        c.is_whitespace() || c.is_control() || matches!(c, '/' | '\\' | '@' | '?' | '#' | ',' | '%')
    }) {
        return Err("contains a forbidden character".to_string());
    }
    if host.parse::<IpAddr>().is_ok() {
        return Ok(());
    }
    let bytes = host.as_bytes();
    if !bytes[0].is_ascii_alphanumeric() {
        return Err("must start with an ASCII letter or digit".to_string());
    }
    if !bytes
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-')
    {
        return Err("is not a valid host name".to_string());
    }
    if host.ends_with('.') || host.ends_with('-') {
        return Err("must not end with '.' or '-'".to_string());
    }
    Ok(())
}

fn push_field_leaves(out: &mut Vec<(String, String)>, base: &str, fields: &[FieldSpec]) {
    let labels = spec_selector_labels(fields);
    for (index, field) in fields.iter().enumerate() {
        let selector = format!("{}[{}]", base, labels[index]);
        out.push((format!("{}.key", selector), field.key.clone()));
        if let Some(heading) = &field.heading {
            out.push((format!("{}.heading", selector), heading.clone()));
        }
        if let Some(frontmatter_key) = &field.frontmatter_key {
            out.push((
                format!("{}.frontmatter_key", selector),
                frontmatter_key.clone(),
            ));
        }
        if let Some(option) = &field.option {
            out.push((format!("{}.option", selector), option.clone()));
        }
        if let Some(label) = &field.label {
            out.push((format!("{}.label", selector), label.clone()));
        }
        out.push((
            format!("{}.kind", selector),
            field.kind.as_str().to_string(),
        ));
        out.push((format!("{}.required", selector), field.required.to_string()));
        let verbs: Vec<&str> = field.verbs.iter().map(|verb| verb.as_str()).collect();
        out.push((
            format!("{}.verbs", selector),
            format!("[{}]", verbs.join(", ")),
        ));
        if let Some(help) = &field.help {
            out.push((format!("{}.help", selector), help.clone()));
        }
    }
}

/// Whether a host is a loopback address for the non-loopback warning.
///
/// Loopback means an IPv4 address in `127.0.0.0/8`, the IPv6 address `::1`, or
/// the literal host name `localhost` (case-insensitive). Other host names are
/// never treated as loopback and are never resolved here.
pub fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => v4.is_loopback(),
        Ok(IpAddr::V6(v6)) => v6.is_loopback(),
        Err(_) => false,
    }
}

/// Merge overlay YAML mapping into base recursively.
pub fn merge_yaml(base: &mut yaml_serde::Value, overlay: &yaml_serde::Value) {
    match (base, overlay) {
        (yaml_serde::Value::Mapping(b), yaml_serde::Value::Mapping(o)) => {
            for (k, v) in o {
                let is_document_types = match k {
                    yaml_serde::Value::String(s) => s == "document_types",
                    _ => false,
                };
                if is_document_types {
                    // Replace wholesale when present, then fill each declared
                    // entry from the seed entry of the same slug (ADR-036
                    // decision 32).
                    b.insert(k.clone(), resolve_document_types(v));
                } else if let Some(existing) = b.get_mut(k) {
                    merge_yaml(existing, v);
                } else {
                    b.insert(k.clone(), v.clone());
                }
            }
        }
        (b, o) => {
            *b = o.clone();
        }
    }
}

/// Get nested value from a YAML mapping using dotted key.
pub fn yaml_get<'a>(root: &'a yaml_serde::Value, key: &str) -> Option<&'a yaml_serde::Value> {
    let mut cur = root;
    for part in key.split('.') {
        match cur {
            yaml_serde::Value::Mapping(m) => {
                let k = yaml_serde::Value::String(part.to_string());
                cur = m.get(&k)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

/// Set nested value in a YAML mapping using dotted key, creating intermediate maps.
pub fn yaml_set(
    root: &mut yaml_serde::Value,
    key: &str,
    value: yaml_serde::Value,
) -> Result<(), String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
        return Err("invalid key".to_string());
    }
    let mut cur = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            match cur {
                yaml_serde::Value::Mapping(m) => {
                    m.insert(yaml_serde::Value::String(part.to_string()), value);
                    return Ok(());
                }
                _ => return Err("invalid structure".to_string()),
            }
        } else {
            match cur {
                yaml_serde::Value::Mapping(m) => {
                    let k = yaml_serde::Value::String(part.to_string());
                    if !m.contains_key(&k) {
                        m.insert(
                            k.clone(),
                            yaml_serde::Value::Mapping(yaml_serde::Mapping::new()),
                        );
                    }
                    cur = m.get_mut(&k).unwrap();
                }
                _ => return Err("invalid structure".to_string()),
            }
        }
    }
    Ok(())
}

/// Remove nested key from YAML mapping.
pub fn yaml_unset(root: &mut yaml_serde::Value, key: &str) -> Result<(), String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
        return Err("invalid key".to_string());
    }
    // navigate to parent
    let mut cur = root;
    for part in &parts[..parts.len() - 1] {
        match cur {
            yaml_serde::Value::Mapping(m) => {
                let k = yaml_serde::Value::String(part.to_string());
                cur = match m.get_mut(&k) {
                    Some(v) => v,
                    None => return Ok(()), // already absent
                };
            }
            _ => return Ok(()),
        }
    }
    if let yaml_serde::Value::Mapping(m) = cur {
        m.remove(parts.last().unwrap());
    }
    Ok(())
}

pub fn validate_key_path(key: &str) -> bool {
    !key.is_empty() && !key.split('.').any(|p| p.is_empty())
}

/// Render a configuration value as YAML for persistence.
///
/// Mappings stay in block style. A sequence whose elements are mappings is
/// rendered as a block sequence, while a flat scalar list stays inline as a
/// flow sequence (`[a, b]`). Scalars are quoted by the YAML serializer so the
/// result reparses to the same value. An empty mapping renders as `{}`.
/// The output is round-tripped through the parser before it is returned.
pub fn to_config_yaml(value: &yaml_serde::Value) -> Result<String, String> {
    let mut out = String::new();
    match value {
        yaml_serde::Value::Mapping(mapping) => {
            if mapping.is_empty() {
                out.push_str("{}\n");
            } else {
                render_block_mapping(mapping, 0, &mut out);
            }
        }
        other => {
            out.push_str(&render_yaml_flow(other));
            out.push('\n');
        }
    }
    let round_trip: yaml_serde::Value = yaml_serde::from_str(&out).map_err(|e| e.to_string())?;
    if &round_trip != value {
        return Err("rendered configuration does not round-trip".to_string());
    }
    Ok(out)
}

/// Render a value inline: sequences as `[a, b]`, mappings as `{k: v}`, scalars
/// as their YAML scalar form.
pub fn render_yaml_flow(value: &yaml_serde::Value) -> String {
    match value {
        yaml_serde::Value::Sequence(sequence) => {
            let elements: Vec<String> = sequence.iter().map(render_yaml_flow).collect();
            format!("[{}]", elements.join(", "))
        }
        yaml_serde::Value::Mapping(mapping) => {
            let entries: Vec<String> = mapping
                .iter()
                .map(|(key, value)| {
                    format!("{}: {}", render_yaml_flow(key), render_yaml_flow(value))
                })
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        other => render_yaml_scalar(other),
    }
}

fn is_block_sequence(sequence: &[yaml_serde::Value]) -> bool {
    !sequence.is_empty()
        && sequence
            .iter()
            .any(|item| matches!(item, yaml_serde::Value::Mapping(inner) if !inner.is_empty()))
}

fn render_block_mapping(mapping: &yaml_serde::Mapping, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    for (key, value) in mapping {
        let key = render_yaml_scalar(key);
        match value {
            yaml_serde::Value::Mapping(inner) if !inner.is_empty() => {
                out.push_str(&format!("{}{}:\n", pad, key));
                render_block_mapping(inner, indent + 2, out);
            }
            yaml_serde::Value::Sequence(sequence) if is_block_sequence(sequence) => {
                out.push_str(&format!("{}{}:\n", pad, key));
                render_block_sequence(sequence, indent + 2, out);
            }
            other => {
                out.push_str(&format!("{}{}: {}\n", pad, key, render_yaml_flow(other)));
            }
        }
    }
}

fn render_block_sequence(sequence: &[yaml_serde::Value], indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    for item in sequence {
        match item {
            yaml_serde::Value::Mapping(mapping) if !mapping.is_empty() => {
                render_block_sequence_mapping(mapping, indent, out);
            }
            yaml_serde::Value::Sequence(inner) if is_block_sequence(inner) => {
                out.push_str(&format!("{}-\n", pad));
                render_block_sequence(inner, indent + 2, out);
            }
            other => {
                out.push_str(&format!("{}- {}\n", pad, render_yaml_flow(other)));
            }
        }
    }
}

fn render_block_sequence_mapping(mapping: &yaml_serde::Mapping, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    for (index, (key, value)) in mapping.iter().enumerate() {
        let marker = if index == 0 { "- " } else { "  " };
        let prefix = format!("{}{}", pad, marker);
        let key = render_yaml_scalar(key);
        match value {
            yaml_serde::Value::Mapping(inner) if !inner.is_empty() => {
                out.push_str(&format!("{}{}:\n", prefix, key));
                render_block_mapping(inner, indent + 4, out);
            }
            yaml_serde::Value::Sequence(sequence) if is_block_sequence(sequence) => {
                out.push_str(&format!("{}{}:\n", prefix, key));
                render_block_sequence(sequence, indent + 4, out);
            }
            other => {
                out.push_str(&format!("{}{}: {}\n", prefix, key, render_yaml_flow(other)));
            }
        }
    }
}

fn render_yaml_scalar(value: &yaml_serde::Value) -> String {
    match value {
        yaml_serde::Value::Null => "null".to_string(),
        yaml_serde::Value::Bool(flag) => flag.to_string(),
        yaml_serde::Value::Number(number) => number.to_string(),
        yaml_serde::Value::String(text) => {
            if text.starts_with('#') {
                // Body headings keep a stable double-quoted form so a
                // round-trip reproduces `"## User Story"` (ADR-0024).
                serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
            } else if text.chars().any(char::is_control) {
                // JSON string escaping is valid YAML double-quoted syntax and
                // keeps a single line; a literal block would break the layout.
                serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
            } else {
                yaml_serde::to_string(&yaml_serde::Value::String(text.clone()))
                    .map(|rendered| rendered.trim_end_matches('\n').to_string())
                    .unwrap_or_else(|_| serde_json::to_string(text).unwrap_or_default())
            }
        }
        other => render_yaml_flow(other),
    }
}

pub fn write_config_file(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_valid() {
        let c = Config::default();
        assert!(c.validate(None).is_ok());
        assert_eq!(
            c.statuses,
            vec![
                "ToDo",
                "Initiated",
                "Refined",
                "Implemented",
                "Validated",
                "Done"
            ]
        );
        assert_eq!(c.default_status, "ToDo");
        assert_eq!(c.id_padding, 3);
    }

    #[test]
    fn duplicate_status_invalid() {
        let c = Config {
            statuses: vec!["A".into(), "A".into()],
            ..Config::default()
        };
        assert!(c.validate(None).is_err());
    }

    #[test]
    fn default_status_membership() {
        let c = Config {
            default_status: "Nope".into(),
            ..Config::default()
        };
        assert!(c.validate(None).is_err());
    }

    #[test]
    fn id_padding_zero_invalid() {
        let c = Config {
            id_padding: 0,
            ..Config::default()
        };
        let err = c.validate(None).unwrap_err();
        assert_eq!(err, "id_padding: must be between 1 and 6");
    }

    #[test]
    fn id_padding_seven_invalid() {
        let c = Config {
            id_padding: 7,
            ..Config::default()
        };
        let err = c.validate(None).unwrap_err();
        assert_eq!(err, "id_padding: must be between 1 and 6");
    }

    #[test]
    fn id_padding_one_valid() {
        let c = Config {
            id_padding: 1,
            ..Config::default()
        };
        assert!(c.validate(None).is_ok());
    }

    #[test]
    fn id_padding_six_valid() {
        let c = Config {
            id_padding: 6,
            ..Config::default()
        };
        assert!(c.validate(None).is_ok());
    }

    #[test]
    fn browser_defaults() {
        let c = Config::default();
        assert_eq!(c.browser_host, "127.0.0.1");
        assert_eq!(c.browser_port, 6420);
        assert!(c.browser_open);
        assert!(c.validate(None).is_ok());
    }

    #[test]
    fn browser_port_zero_is_invalid() {
        let c = Config {
            browser_port: 0,
            ..Config::default()
        };
        assert!(c.validate(None).is_err());
    }

    #[test]
    fn browser_host_validation() {
        for host in [
            "127.0.0.1",
            "::1",
            "localhost",
            "example.test",
            "0.0.0.0",
            "host-1.example",
        ] {
            assert!(
                validate_browser_host(host).is_ok(),
                "host should be valid: {}",
                host
            );
        }
        for host in [
            "",
            " bad",
            "bad host",
            "bad/host",
            "bad\\host",
            "bad@host",
            "bad?host",
            "bad#host",
            "bad,host",
            "bad%host",
            "host.",
            "host-",
            "-host",
            "_host",
        ] {
            assert!(
                validate_browser_host(host).is_err(),
                "host should be invalid: {}",
                host
            );
        }
    }

    #[test]
    fn browser_validation_messages_name_the_key() {
        let c = Config {
            browser_port: 0,
            ..Config::default()
        };
        let error = c.validate(None).unwrap_err();
        assert!(error.starts_with("browser_port:"), "{}", error);

        let c = Config {
            browser_host: "bad/host".to_string(),
            ..Config::default()
        };
        let error = c.validate(None).unwrap_err();
        assert!(error.starts_with("browser_host:"), "{}", error);
    }

    #[test]
    fn flattened_browser_keys_follow_document_types() {
        let c = Config::default();
        let keys: Vec<String> = c
            .flattened(Path::new("/project"))
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        let tail: Vec<&str> = keys
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(String::as_str)
            .collect();
        assert_eq!(tail, vec!["browser_host", "browser_port", "browser_open"]);
    }

    #[test]
    fn browser_keys_serialize_to_json() {
        let c = Config::default();
        let value = serde_json::to_value(&c).unwrap();
        assert_eq!(value["browser_host"], "127.0.0.1");
        assert_eq!(value["browser_port"], 6420);
        assert_eq!(value["browser_open"], true);
    }

    #[test]
    fn yaml_merge_and_set() {
        let mut base = yaml_serde::to_value(Config::default()).unwrap();
        // top-level set via yaml_set
        let mut root = yaml_serde::Value::Mapping(yaml_serde::Mapping::new());
        yaml_set(
            &mut root,
            "task_prefix",
            yaml_serde::Value::String("PAP".into()),
        )
        .unwrap();
        merge_yaml(&mut base, &root);
        let cfg: Config = yaml_serde::from_value(base).unwrap();
        assert_eq!(cfg.task_prefix, "PAP");
    }

    #[test]
    fn doc_type_entry_without_status_fields_deserializes() {
        let entry: DocTypeEntry = yaml_serde::from_str(
            "prefix: DEP\ndirectory: architecture/deployment/\nenabled: true\n",
        )
        .unwrap();
        assert_eq!(entry.prefix, "DEP");
        assert_eq!(entry.directory, "architecture/deployment/");
        assert!(entry.enabled);
        assert_eq!(entry.statuses, None);
        assert_eq!(entry.default_status, None);
    }

    #[test]
    fn doc_type_entry_deserializes_statuses_and_default_status() {
        let entry: DocTypeEntry = yaml_serde::from_str(
            "prefix: REQ\ndirectory: requirements/business/\nenabled: true\nstatuses:\n  - active\n  - archived\n  - review\ndefault_status: review\n",
        )
        .unwrap();
        assert_eq!(
            entry.statuses,
            Some(vec![
                "active".to_string(),
                "archived".to_string(),
                "review".to_string()
            ])
        );
        assert_eq!(entry.default_status, Some("review".to_string()));
    }

    #[test]
    fn status_declaration_order_is_preserved() {
        let entry: DocTypeEntry =
            yaml_serde::from_str("statuses:\n  - active\n  - review\n  - archived\n  - draft\n")
                .unwrap();
        assert_eq!(
            entry.statuses,
            Some(vec![
                "active".to_string(),
                "review".to_string(),
                "archived".to_string(),
                "draft".to_string()
            ])
        );
    }

    #[test]
    fn empty_status_list_deserializes_as_some_empty() {
        let entry: DocTypeEntry = yaml_serde::from_str("statuses: []\n").unwrap();
        assert_eq!(entry.statuses, Some(Vec::new()));
    }

    #[test]
    fn absent_status_fields_are_omitted_on_serialization() {
        let entry = DocTypeEntry {
            prefix: "DEP".to_string(),
            directory: "architecture/deployment/".to_string(),
            enabled: true,
            statuses: None,
            default_status: None,
            ..Default::default()
        };
        let value = serde_json::to_value(&entry).unwrap();
        let object = value.as_object().unwrap();
        assert!(!object.contains_key("statuses"));
        assert!(!object.contains_key("default_status"));

        let yaml = yaml_serde::to_value(&entry).unwrap();
        let yaml_serde::Value::Mapping(mapping) = yaml else {
            panic!("expected a mapping");
        };
        assert!(!mapping.contains_key(yaml_serde::Value::String("statuses".to_string())));
        assert!(!mapping.contains_key(yaml_serde::Value::String("default_status".to_string())));
    }

    #[test]
    fn existing_configuration_files_still_load() {
        let cfg: Config = yaml_serde::from_str(
            "document_types:\n  deploy:\n    prefix: DEP\n    directory: architecture/deployment/\n    enabled: false\n",
        )
        .unwrap();
        let deploy = cfg.document_types.get("deploy").expect("deploy entry");
        assert_eq!(deploy.prefix, "DEP");
        assert_eq!(deploy.directory, "architecture/deployment/");
        assert!(!deploy.enabled);
        assert_eq!(deploy.statuses, None);
        assert_eq!(deploy.default_status, None);

        // The top-level task `statuses` keys are unaffected; only the document
        // entry must stay free of the new status keys.
        let value = serde_json::to_value(&cfg).unwrap();
        let deploy = value["document_types"]["deploy"].as_object().unwrap();
        assert!(!deploy.contains_key("statuses"));
        assert!(!deploy.contains_key("default_status"));
    }

    use crate::commands::{
        config_set, config_unset, load_effective_config, load_effective_config_tolerant,
        load_effective_value,
    };
    use crate::context::Context;
    use crate::error::AppError;
    use crate::init::{InitOptions, run_init_with_io};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Temporary project directory that removes itself when the test ends.
    struct TempProject {
        path: PathBuf,
    }

    impl TempProject {
        fn new(tag: &str) -> Self {
            let base = PathBuf::from(".tmp").join("config-unit-tests");
            std::fs::create_dir_all(&base).expect("create temp base");
            let unique = format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                TEMP_DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
            );
            let path = base.join(unique);
            std::fs::create_dir_all(&path).expect("create temp project");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn config_path(&self) -> PathBuf {
            self.path.join("papyrus.config.yml")
        }

        fn write_config(&self, content: &str) {
            std::fs::write(self.config_path(), content).expect("write config file");
        }

        fn read_config(&self) -> String {
            std::fs::read_to_string(self.config_path()).expect("read config file")
        }

        fn context(&self) -> Context {
            Context {
                project_dir: self.path.clone(),
                config_path: self.config_path(),
                json: false,
                no_color: true,
            }
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
            // Succeeds only when the last test using the base directory is done.
            let _ = std::fs::remove_dir(PathBuf::from(".tmp").join("config-unit-tests"));
        }
    }

    fn entry(prefix: &str, directory: &str) -> DocTypeEntry {
        DocTypeEntry {
            prefix: prefix.to_string(),
            directory: directory.to_string(),
            enabled: true,
            statuses: None,
            default_status: None,
            ..Default::default()
        }
    }

    fn config_with_single_type(slug: &str, entry: DocTypeEntry) -> Config {
        let mut config = Config {
            document_types: BTreeMap::new(),
            ..Config::default()
        };
        config.document_types.insert(slug.to_string(), entry);
        config
    }

    fn validate_single(slug: &str, entry: DocTypeEntry) -> Result<(), String> {
        config_with_single_type(slug, entry).validate(None)
    }

    #[test]
    fn slug_grammar_rejects_invalid_document_type_keys() {
        for slug in [
            "",
            "bad slug",
            "Bad.Slug",
            "-leading-dash",
            "bad/slug",
            "bad\\slug",
            "bad..slug",
            "bad\tslug",
            "bad\u{1}slug",
        ] {
            let err = validate_single(slug, entry("NOTE", "notes/")).unwrap_err();
            assert!(err.contains("invalid slug"), "slug {:?} => {}", slug, err);
        }
        assert!(validate_single("my-type_1", entry("NOTE", "notes/")).is_ok());
    }

    #[test]
    fn reserved_slug_conflicts_are_case_insensitive() {
        for (slug, builtin) in [
            ("Config", "config"),
            ("SEARCH", "search"),
            ("task", "task"),
            ("check", "check"),
            ("Check", "check"),
            ("CHECK", "check"),
        ] {
            let err = validate_single(slug, entry("NOTE", "notes/")).unwrap_err();
            assert_eq!(
                err,
                format!(
                    "document_types.{}: slug conflicts with built-in type {}",
                    slug, builtin
                ),
                "slug {:?} reports the configured spelling once",
                slug
            );
        }
    }

    #[test]
    fn every_builtin_command_name_is_reserved() {
        let builtins: Vec<&str> = crate::cli::BUILTIN_COMMANDS
            .iter()
            .map(|builtin| builtin.name)
            .collect();
        assert!(
            builtins.contains(&"check"),
            "check is a built-in command name: {:?}",
            builtins
        );
        for builtin in builtins {
            let err = validate_single(builtin, entry("NOTE", "notes/")).unwrap_err();
            assert_eq!(
                err,
                format!(
                    "document_types.{}: slug conflicts with built-in type {}",
                    builtin, builtin
                )
            );
        }
    }

    #[test]
    fn prefix_grammar_rejects_invalid_values() {
        for prefix in [
            "",
            "bad prefix",
            "bad\tprefix",
            "bad\u{1}prefix",
            "bad/prefix",
            "bad\\prefix",
            "bad..prefix",
        ] {
            let err = validate_single("note", entry(prefix, "notes/")).unwrap_err();
            assert!(
                err.contains("document_types.note.prefix"),
                "prefix {:?} => {}",
                prefix,
                err
            );
        }
    }

    #[test]
    fn prefix_uniqueness_is_case_insensitive_and_includes_disabled_entries() {
        let mut config = Config {
            document_types: BTreeMap::new(),
            ..Config::default()
        };
        config
            .document_types
            .insert("aaa".to_string(), entry("REQ", "a/"));
        config.document_types.insert(
            "bbb".to_string(),
            DocTypeEntry {
                enabled: false,
                ..entry("req", "b/")
            },
        );

        let err = config.validate(None).unwrap_err();
        assert!(
            err.contains("document_types.bbb.prefix: req is already used by aaa"),
            "{}",
            err
        );
    }

    #[test]
    fn directory_grammar_rejects_invalid_values() {
        for directory in [
            "",
            "/absolute/path",
            "../escape",
            "a/../b",
            "bad\\path",
            "bad\u{1}path",
        ] {
            let err = validate_single("note", entry("NOTE", directory)).unwrap_err();
            assert!(
                err.contains("document_types.note.directory"),
                "directory {:?} => {}",
                directory,
                err
            );
        }
    }

    #[test]
    fn duplicate_directories_rejected_only_among_enabled_entries() {
        let mut config = Config {
            document_types: BTreeMap::new(),
            ..Config::default()
        };
        config
            .document_types
            .insert("aaa".to_string(), entry("AAA", "shared/"));
        config.document_types.insert(
            "bbb".to_string(),
            DocTypeEntry {
                enabled: false,
                ..entry("BBB", "shared/")
            },
        );
        assert!(config.validate(None).is_ok());

        config.document_types.get_mut("bbb").unwrap().enabled = true;
        let err = config.validate(None).unwrap_err();
        assert!(
            err.contains("document_types.bbb.directory: directory is already used by aaa"),
            "{}",
            err
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_directory_escape_is_rejected_with_project_dir() {
        let project = TempProject::new("symlink");
        let outside = TempProject::new("symlink-outside");
        std::fs::create_dir_all(project.path().join("docs")).expect("create docs directory");
        let outside_target = std::fs::canonicalize(outside.path()).expect("canonicalize outside");
        std::os::unix::fs::symlink(outside_target, project.path().join("docs").join("escape"))
            .expect("create symlink");

        let config = config_with_single_type("note", entry("NOTE", "escape/"));
        assert!(config.validate(None).is_ok());

        let err = config.validate(Some(project.path())).unwrap_err();
        assert!(
            err.contains(
                "document_types.note.directory: value must be relative and stay inside the project directory"
            ),
            "{}",
            err
        );
    }

    #[test]
    fn status_lists_reject_empty_duplicate_missing_and_invalid_values() {
        let cases: &[(&str, &[&str], &str)] = &[
            ("empty", &[], "statuses: must not be empty"),
            (
                "duplicate",
                &["active", "archived", "active"],
                "duplicate value active",
            ),
            (
                "missing-active",
                &["draft", "archived"],
                "\"active\" must be declared exactly once",
            ),
            (
                "missing-archived",
                &["active", "draft"],
                "\"archived\" must be declared exactly once",
            ),
            (
                "invalid-value",
                &["active", "archived", "bad status"],
                "invalid status value bad status",
            ),
        ];
        for (label, statuses, expected) in cases {
            let mut doc = entry("NOTE", "notes/");
            doc.statuses = Some(statuses.iter().map(|value| value.to_string()).collect());
            let err = validate_single("note", doc).unwrap_err();
            assert!(err.contains(expected), "{}: {}", label, err);
        }
    }

    #[test]
    fn ui_mockup_rejects_status_declarations() {
        let mut config = Config::default();
        let mockup = config.document_types.get_mut("ui-mockup").unwrap();
        mockup.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        let err = config.validate(None).unwrap_err();
        assert!(
            err.contains(
                "document_types.ui-mockup.statuses: not allowed for a type without a front-matter status"
            ),
            "{}",
            err
        );

        let mut config = Config::default();
        config
            .document_types
            .get_mut("ui-mockup")
            .unwrap()
            .default_status = Some("active".to_string());
        let err = config.validate(None).unwrap_err();
        assert!(
            err.contains(
                "document_types.ui-mockup.default_status: not allowed for a type without a front-matter status"
            ),
            "{}",
            err
        );
    }

    #[test]
    fn default_status_must_be_declared_and_not_archived() {
        let mut doc = entry("NOTE", "notes/");
        doc.statuses = Some(vec!["active".to_string(), "archived".to_string()]);
        doc.default_status = Some("draft".to_string());
        let err = validate_single("note", doc).unwrap_err();
        assert!(
            err.contains("document_types.note.default_status: draft is not a declared status"),
            "{}",
            err
        );

        let doc = DocTypeEntry {
            default_status: Some("archived".to_string()),
            ..entry("NOTE", "notes/")
        };
        let err = validate_single("note", doc).unwrap_err();
        assert!(
            err.contains("document_types.note.default_status: must not be \"archived\""),
            "{}",
            err
        );
    }

    #[test]
    fn browser_diagnostics_follow_struct_field_order() {
        let config = Config {
            browser_host: "bad/host".to_string(),
            browser_port: 0,
            ..Config::default()
        };
        let err = config.validate(None).unwrap_err();
        assert_eq!(
            err.lines().collect::<Vec<_>>(),
            vec![
                "browser_host: contains a forbidden character",
                "browser_port: must be between 1 and 65535",
            ]
        );
    }

    #[test]
    fn duplicate_yaml_keys_are_rejected_by_load_effective_value() {
        let project = TempProject::new("duplicate-key");
        project.write_config("task_prefix: A\ntask_prefix: B\n");

        let err = load_effective_value(&project.config_path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "{:?}", err);
        assert_eq!(err.code(), 4);
        assert!(err.message().contains("duplicate entry"), "{}", err);
    }

    #[test]
    fn document_types_overlay_replaces_defaults_wholesale() {
        let project = TempProject::new("wholesale");
        project.write_config("document_types:\n  note:\n    prefix: NOTE\n    directory: notes/\n");

        let value = load_effective_value(&project.config_path()).unwrap();
        let document_types = yaml_get(&value, "document_types").unwrap();
        let yaml_serde::Value::Mapping(mapping) = document_types else {
            panic!("document_types must be a mapping");
        };
        let keys: Vec<String> = mapping
            .keys()
            .filter_map(|key| match key {
                yaml_serde::Value::String(value) => Some(value.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(keys, vec!["note".to_string()]);
        assert!(yaml_get(&value, "document_types.deploy").is_none());
    }

    #[test]
    fn absent_document_types_key_yields_the_same_default_set() {
        let project = TempProject::new("default-map");
        project.write_config("project_name: Test\n");
        let cfg = load_effective_config(&project.config_path(), project.path()).unwrap();
        assert_eq!(cfg.document_types, Config::default().document_types);
        assert_eq!(
            cfg.document_types["req"].statuses,
            Some(vec!["active".to_string(), "archived".to_string()])
        );
    }

    #[test]
    fn strict_load_validates_while_tolerant_load_does_not() {
        let project = TempProject::new("strict-tolerant");
        project.write_config(
            "document_types:\n  \"bad slug\":\n    prefix: BAD\n    directory: bad/\n",
        );

        let tolerant = load_effective_config_tolerant(&project.config_path(), project.path());
        assert!(tolerant.is_ok(), "{:?}", tolerant.err());

        let err = load_effective_config(&project.config_path(), project.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "{:?}", err);
        assert!(err.message().contains("invalid slug"), "{}", err);
    }

    #[test]
    fn config_set_materializes_only_the_target_entry_and_refuses_an_invalid_result() {
        let project = TempProject::new("config-set");
        let ctx = project.context();

        config_set(&ctx, "document_types.glossary.enabled", "false").unwrap();
        let value: yaml_serde::Value =
            yaml_serde::from_str(&project.read_config()).expect("parse written config");
        let document_types = yaml_get(&value, "document_types").unwrap();
        let yaml_serde::Value::Mapping(mapping) = document_types else {
            panic!("document_types must be a mapping");
        };
        assert_eq!(mapping.len(), 1, "only the target entry is written");
        assert!(yaml_get(&value, "document_types.glossary").is_some());
        assert!(yaml_get(&value, "document_types.req").is_none());
        assert_eq!(
            yaml_get(&value, "document_types.glossary.prefix").and_then(|v| match v {
                yaml_serde::Value::String(text) => Some(text.as_str()),
                _ => None,
            }),
            Some("GLO")
        );
        assert_eq!(
            yaml_get(&value, "document_types.glossary.directory").and_then(|v| match v {
                yaml_serde::Value::String(text) => Some(text.as_str()),
                _ => None,
            }),
            Some("glossary/")
        );
        assert!(matches!(
            yaml_get(&value, "document_types.glossary.enabled"),
            Some(yaml_serde::Value::Bool(false))
        ));

        let before = project.read_config();
        let err = config_set(&ctx, "document_types.glossary.statuses", "active,review")
            .err()
            .expect("config set must refuse an invalid result");
        assert!(matches!(err, AppError::Validation(_)), "{:?}", err);
        assert!(
            err.message().contains("document_types.glossary.statuses"),
            "{}",
            err
        );
        assert_eq!(project.read_config(), before);
    }

    #[test]
    fn config_unset_refuses_an_invalid_result() {
        let project = TempProject::new("config-unset");
        project.write_config("document_types:\n  note:\n    prefix: NOTE\n    directory: notes/\n");
        let ctx = project.context();
        let before = project.read_config();

        let err = config_unset(&ctx, "document_types.note.directory")
            .err()
            .expect("config unset must refuse an invalid result");
        assert!(matches!(err, AppError::Validation(_)), "{:?}", err);
        assert!(
            err.message()
                .contains("document_types.note.directory: must not be empty"),
            "{}",
            err
        );
        assert_eq!(project.read_config(), before);
    }

    #[test]
    fn init_force_repairs_an_invalid_configuration() {
        let project = TempProject::new("init-force");
        project.write_config(
            "document_types:\n  \"bad slug\":\n    prefix: BAD\n    directory: bad/\n",
        );

        let opts = InitOptions {
            name: "repair".to_string(),
            no_git: true,
            force: true,
        };
        let mut reader = std::io::empty();
        let mut writer = std::io::sink();
        run_init_with_io(
            project.path().to_path_buf(),
            opts,
            false,
            &mut reader,
            &mut writer,
        )
        .unwrap();

        let config = load_effective_config(&project.config_path(), project.path()).unwrap();
        assert_eq!(config.project_name.as_deref(), Some("repair"));
        assert!(!config.document_types.contains_key("deploy"));
        assert!(!config.document_types.contains_key("bad slug"));
        for slug in [
            "req",
            "nfr",
            "glossary",
            "rule",
            "ui-spec",
            "ui-mockup",
            "audit",
            "tech-spec",
            "use-case",
            "test-case",
            "test-report",
        ] {
            assert!(config.document_types.contains_key(slug), "missing {}", slug);
        }
        assert!(config.validate(Some(project.path())).is_ok());
    }

    #[test]
    fn list_key_classification_covers_declared_shapes_only() {
        for key in [
            "statuses",
            "document_types.audit.statuses",
            "document_types.a.statuses",
        ] {
            assert!(
                is_list_valued_key(key),
                "key should be list-valued: {}",
                key
            );
        }
        for key in [
            "default_status",
            "document_types.audit.default_status",
            "document_types.audit.prefix",
            "document_types.audit.directory",
            "document_types.audit.enabled",
            "id_padding",
            "browser_port",
            "browser_host",
            "document_types.a.b.statuses",
            "document_types..statuses",
            "document_types.statuses",
            "statuses.x",
            "Statuses",
        ] {
            assert!(!is_list_valued_key(key), "key should be scalar: {}", key);
        }

        // Every sequence-typed field in the configuration model is covered.
        let value = yaml_serde::to_value(Config::default()).unwrap();
        assert!(matches!(
            yaml_get(&value, "statuses"),
            Some(yaml_serde::Value::Sequence(_))
        ));
        assert!(is_list_valued_key("statuses"));
        assert_eq!(LIST_VALUED_KEYS, &["statuses"]);
    }

    #[test]
    fn parse_list_value_trims_and_preserves_order_and_duplicates() {
        assert_eq!(
            parse_list_value("statuses", " a , b ").unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            parse_list_value("statuses", "ToDo").unwrap(),
            vec!["ToDo".to_string()]
        );
        assert_eq!(
            parse_list_value("statuses", "B, a, B").unwrap(),
            vec!["B".to_string(), "a".to_string(), "B".to_string()]
        );
        assert_eq!(
            parse_list_value("statuses", "in review, done").unwrap(),
            vec!["in review".to_string(), "done".to_string()]
        );
        // Elements are strings, never YAML values.
        assert_eq!(
            parse_list_value("statuses", "true,1,[a]").unwrap(),
            vec!["true".to_string(), "1".to_string(), "[a]".to_string()]
        );
    }

    #[test]
    fn parse_list_value_rejects_empty_elements_and_control_characters() {
        for raw in ["", "   ", ",", "a,,b", "a,", ",a", " , "] {
            let err = parse_list_value("statuses", raw).unwrap_err();
            assert!(err.starts_with("statuses:"), "{:?} => {}", raw, err);
            assert!(err.contains("must not be empty"), "{:?} => {}", raw, err);
        }
        for raw in ["a\nb", "a\tb", "a\u{1}b", "\u{7}"] {
            let err = parse_list_value("statuses", raw).unwrap_err();
            assert!(err.starts_with("statuses:"), "{:?} => {}", raw, err);
            assert!(
                err.contains("must not contain control characters"),
                "{:?} => {}",
                raw,
                err
            );
        }
    }

    #[test]
    fn to_config_yaml_renders_sequences_in_flow_form() {
        let value: yaml_serde::Value = yaml_serde::from_str(
            "project_name: Acme\nstatuses:\n  - ToDo\n  - Initiated\n  - Done\n",
        )
        .unwrap();
        let rendered = to_config_yaml(&value).unwrap();
        assert!(
            rendered.contains("statuses: [ToDo, Initiated, Done]\n"),
            "{}",
            rendered
        );
        assert!(!rendered.contains("- ToDo"), "{}", rendered);
        let reparsed: yaml_serde::Value = yaml_serde::from_str(&rendered).unwrap();
        assert_eq!(reparsed, value);
    }

    #[test]
    fn to_config_yaml_keeps_nested_mappings_in_block_style() {
        let value: yaml_serde::Value = yaml_serde::from_str(
            "document_types:\n  audit:\n    prefix: AUD\n    statuses:\n      - active\n      - archived\n",
        )
        .unwrap();
        let rendered = to_config_yaml(&value).unwrap();
        assert!(
            rendered.contains(
                "document_types:\n  audit:\n    prefix: AUD\n    statuses: [active, archived]\n"
            ),
            "{}",
            rendered
        );
    }

    #[test]
    fn to_config_yaml_quotes_ambiguous_and_multiline_scalars() {
        let mut mapping = yaml_serde::Mapping::new();
        mapping.insert(
            yaml_serde::Value::String("flag".to_string()),
            yaml_serde::Value::String("true".to_string()),
        );
        mapping.insert(
            yaml_serde::Value::String("number".to_string()),
            yaml_serde::Value::String("1".to_string()),
        );
        mapping.insert(
            yaml_serde::Value::String("multiline".to_string()),
            yaml_serde::Value::String("a\nb".to_string()),
        );
        let value = yaml_serde::Value::Mapping(mapping);
        let rendered = to_config_yaml(&value).unwrap();
        assert!(rendered.contains("flag: 'true'"), "{}", rendered);
        assert!(rendered.contains("number: '1'"), "{}", rendered);
        let reparsed: yaml_serde::Value = yaml_serde::from_str(&rendered).unwrap();
        assert_eq!(reparsed, value);
    }

    #[test]
    fn flattened_renders_document_status_lists_in_flow_form() {
        let config = Config::default();
        let flattened: BTreeMap<String, String> = config
            .flattened(Path::new("/project"))
            .into_iter()
            .collect();
        assert_eq!(
            flattened
                .get("document_types.audit.statuses")
                .map(String::as_str),
            Some("[active, archived]")
        );
        assert_eq!(
            flattened
                .get("document_types.req.statuses")
                .map(String::as_str),
            Some("[active, archived]")
        );
        assert!(
            !flattened.contains_key("document_types.ui-mockup.statuses"),
            "ui-mockup declares no status list"
        );
        assert_eq!(
            flattened.get("statuses").map(String::as_str),
            Some("[ToDo, Initiated, Refined, Implemented, Validated, Done]")
        );
    }

    #[test]
    fn flattened_renders_document_default_status_after_statuses() {
        let mut config = Config::default();
        let audit = config.document_types.get_mut("audit").unwrap();
        audit.statuses = Some(vec![
            "active".to_string(),
            "archived".to_string(),
            "open".to_string(),
        ]);
        audit.default_status = Some("open".to_string());
        config
            .document_types
            .get_mut("ui-mockup")
            .unwrap()
            .default_status = Some("draft".to_string());

        let flattened = config.flattened(Path::new("/project"));
        let keys: Vec<&str> = flattened.iter().map(|(key, _)| key.as_str()).collect();
        let index = |key: &str| {
            keys.iter()
                .position(|candidate| *candidate == key)
                .unwrap_or_else(|| panic!("missing key {key}: {keys:?}"))
        };

        let audit_default = index("document_types.audit.default_status");
        assert_eq!(flattened[audit_default].1, "open");
        assert_eq!(
            audit_default,
            index("document_types.audit.statuses") + 1,
            "default_status follows the same slug's statuses"
        );

        let mockup_default = index("document_types.ui-mockup.default_status");
        assert_eq!(flattened[mockup_default].1, "draft");
        assert_eq!(
            mockup_default,
            index("document_types.ui-mockup.enabled") + 1,
            "default_status follows enabled when no status list is declared"
        );

        assert!(
            !keys.contains(&"document_types.req.default_status"),
            "an entry without default_status gets no key: {:?}",
            keys
        );
        assert!(
            !flattened
                .iter()
                .any(|(key, value)| key.ends_with(".default_status") && value.is_empty()),
            "no empty default_status values"
        );
    }

    #[test]
    fn field_spec_declares_the_documented_keys_and_defaults() {
        let field: FieldSpec = yaml_serde::from_str(
            "key: story\nheading: '## User Story'\nfrontmatter_key: story_text\noption: story\nlabel: Story\nkind: multiline\nrequired: true\nhelp: The user story.\n",
        )
        .unwrap();
        assert_eq!(field.key, "story");
        assert_eq!(field.heading.as_deref(), Some("## User Story"));
        assert_eq!(field.frontmatter_key.as_deref(), Some("story_text"));
        assert_eq!(field.option.as_deref(), Some("story"));
        assert_eq!(field.label.as_deref(), Some("Story"));
        assert_eq!(field.kind, FieldKind::Multiline);
        assert!(field.required);
        assert_eq!(field.verbs, vec![FieldVerb::Create, FieldVerb::Edit]);
        assert_eq!(field.help.as_deref(), Some("The user story."));
        assert!(field.stores_value());

        let minimal: FieldSpec = yaml_serde::from_str("key: description\n").unwrap();
        assert_eq!(minimal.kind, FieldKind::Text);
        assert!(!minimal.required);
        assert_eq!(minimal.verbs, vec![FieldVerb::Create, FieldVerb::Edit]);
        assert!(!minimal.stores_value(), "neither location is declared");
    }

    #[test]
    fn field_spec_rejects_order_enabled_and_unknown_keys() {
        for extra in ["order: 10", "enabled: true", "short: d", "template: x"] {
            let yaml = format!("key: description\nheading: '## Description'\n{extra}\n");
            assert!(
                yaml_serde::from_str::<FieldSpec>(&yaml).is_err(),
                "field key must be rejected: {extra}"
            );
        }
        assert_eq!(
            RESERVED_ENTRY_KEYS,
            ["model", "title", "content", "id_shape"]
        );
    }

    #[test]
    fn field_kind_and_verbs_reject_unknown_values() {
        for kind in ["text", "multiline", "list"] {
            let yaml = format!("key: description\nheading: '## Description'\nkind: {kind}\n");
            assert!(
                yaml_serde::from_str::<FieldSpec>(&yaml).is_ok(),
                "kind must be accepted: {kind}"
            );
        }
        assert!(
            yaml_serde::from_str::<FieldSpec>(
                "key: description\nheading: '## Description'\nkind: paragraph\n"
            )
            .is_err()
        );

        let yaml = "key: description\nheading: '## Description'\nverbs: [create]\n";
        assert_eq!(
            yaml_serde::from_str::<FieldSpec>(yaml).unwrap().verbs,
            vec![FieldVerb::Create]
        );
        assert!(
            yaml_serde::from_str::<FieldSpec>(
                "key: description\nheading: '## Description'\nverbs: [delete]\n"
            )
            .is_err()
        );
    }

    #[test]
    fn reference_visibility_accepts_only_visible_and_hidden() {
        for (raw, expected) in [
            ("visible", Visibility::Visible),
            ("hidden", Visibility::Hidden),
        ] {
            let yaml = format!("prefix: DEP\ndirectory: deploy/\nreference_visibility: {raw}\n");
            assert_eq!(
                yaml_serde::from_str::<DocTypeEntry>(&yaml)
                    .unwrap()
                    .reference_visibility,
                Some(expected)
            );
        }
        assert!(
            yaml_serde::from_str::<DocTypeEntry>(
                "prefix: DEP\ndirectory: deploy/\nreference_visibility: sometimes\n"
            )
            .is_err()
        );
    }

    #[test]
    fn reserved_field_keys_cover_every_engine_owned_key() {
        assert_eq!(
            RESERVED_FIELD_KEYS,
            [
                "id",
                "title",
                "status",
                "labels",
                "references",
                "created_date",
                "updated_date",
                "group",
            ]
        );
        for reserved in RESERVED_FIELD_KEYS {
            let as_key: FieldSpec =
                yaml_serde::from_str(&format!("key: {reserved}\nheading: '## X'\n")).unwrap();
            assert_eq!(as_key.reserved_key(), Some(reserved), "{reserved} as key");

            let as_frontmatter: FieldSpec =
                yaml_serde::from_str(&format!("key: custom\nfrontmatter_key: {reserved}\n"))
                    .unwrap();
            assert_eq!(
                as_frontmatter.reserved_key(),
                Some(reserved),
                "{reserved} as frontmatter_key"
            );
        }
        let ok: FieldSpec = yaml_serde::from_str("key: story\nheading: '## User Story'\n").unwrap();
        assert_eq!(ok.reserved_key(), None);
    }

    #[test]
    fn grouped_role_blocks_declare_fields_and_reference_visibility() {
        let entry: DocTypeEntry = yaml_serde::from_str(
            "prefix: REQ\ndirectory: requirements/business/\ngroup:\n  reference_visibility: hidden\n  fields:\n    - key: story\n      heading: '## User Story'\nmember:\n  fields: []\n  reference_visibility: visible\n",
        )
        .unwrap();
        let group = entry.group.expect("group block");
        assert_eq!(group.reference_visibility, Some(Visibility::Hidden));
        assert_eq!(group.fields.as_ref().unwrap().len(), 1);
        let member = entry.member.expect("member block");
        assert_eq!(member.reference_visibility, Some(Visibility::Visible));
        assert_eq!(member.fields, Some(Vec::new()), "fields: [] stays present");
        assert_eq!(entry.fields, None, "a grouped type has no top-level fields");
    }

    #[test]
    fn absent_collections_are_none_and_empty_ones_are_some() {
        let absent: DocTypeEntry =
            yaml_serde::from_str("prefix: DEP\ndirectory: deploy/\n").unwrap();
        assert_eq!(absent.fields, None);
        assert_eq!(absent.group, None);
        assert_eq!(absent.member, None);
        assert_eq!(absent.slots, None);
        assert_eq!(absent.api_aliases, None);
        assert_eq!(absent.reference_visibility, None);
        assert_eq!(absent.label, None);
        assert_eq!(absent.plural, None);
        assert_eq!(absent.order, None);
        assert_eq!(absent.summary, None);
        assert_eq!(absent.type_alias, None);

        let empty: DocTypeEntry = yaml_serde::from_str(
            "prefix: DEP\ndirectory: deploy/\nfields: []\nslots: []\napi_aliases: []\n",
        )
        .unwrap();
        assert_eq!(empty.fields, Some(Vec::new()));
        assert_eq!(empty.slots, Some(Vec::new()));
        assert_eq!(empty.api_aliases, Some(Vec::new()));
    }

    #[test]
    fn doc_type_entry_deserializes_presentation_and_slot_declarations() {
        let entry: DocTypeEntry = yaml_serde::from_str(
            "prefix: DEP\ndirectory: deploy/\nlabel: Deployment\nplural: Deployments\norder: 120\nsummary: Deployment records.\ntype_alias: deployment\nreference_visibility: visible\nslots:\n  - key: environment\n    label: Environment\napi_aliases: [/api/deployments]\nfields:\n  - key: description\n    heading: '## Description'\nstatuses: [active, archived]\ndefault_status: active\n",
        )
        .unwrap();
        assert_eq!(entry.label.as_deref(), Some("Deployment"));
        assert_eq!(entry.plural.as_deref(), Some("Deployments"));
        assert_eq!(entry.order, Some(120));
        assert_eq!(entry.summary.as_deref(), Some("Deployment records."));
        assert_eq!(entry.type_alias.as_deref(), Some("deployment"));
        assert_eq!(entry.reference_visibility, Some(Visibility::Visible));
        assert_eq!(
            entry.slots,
            Some(vec![Slot {
                key: "environment".to_string(),
                label: "Environment".to_string(),
            }])
        );
        assert_eq!(
            entry.api_aliases,
            Some(vec!["/api/deployments".to_string()])
        );
        assert_eq!(entry.fields.as_ref().unwrap().len(), 1);
        assert_eq!(
            entry.statuses,
            Some(vec!["active".to_string(), "archived".to_string()])
        );
        assert_eq!(entry.default_status.as_deref(), Some("active"));
    }

    #[test]
    fn doc_type_entry_rejects_behaviour_keys() {
        for key in RESERVED_ENTRY_KEYS {
            let yaml = format!("prefix: DEP\ndirectory: deploy/\n{key}: value\n");
            assert!(
                yaml_serde::from_str::<DocTypeEntry>(&yaml).is_err(),
                "entry key must be rejected: {key}"
            );
        }
        assert!(
            yaml_serde::from_str::<DocTypeEntry>("prefix: DEP\ndirectory: deploy/\ntemplate: x\n")
                .is_err()
        );
    }
    #[test]
    fn to_config_yaml_renders_mapping_sequences_as_blocks_and_flat_lists_inline() {
        let value: yaml_serde::Value = yaml_serde::from_str(
            "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields:\n      - key: description\n        heading: '## User Story'\n        verbs: [create]\n",
        )
        .unwrap();
        let rendered = to_config_yaml(&value).unwrap();
        assert!(
            rendered.contains(
                "    fields:\n      - key: description\n        heading: \"## User Story\"\n        verbs: [create]\n"
            ),
            "{rendered}"
        );
        assert!(!rendered.contains("[{key:"), "{rendered}");
        let reparsed: yaml_serde::Value = yaml_serde::from_str(&rendered).unwrap();
        assert_eq!(reparsed, value);
    }

    #[test]
    fn init_value_is_minimal_and_marks_only_disabled_entries() {
        let mut config = Config::default();
        config.document_types.get_mut("req").unwrap().enabled = false;
        let value = config.init_value().unwrap();
        let rendered = to_config_yaml(&value).unwrap();
        assert!(
            rendered.contains(
                "  req:\n    prefix: REQ\n    directory: requirements/business/\n    enabled: false\n"
            ),
            "{rendered}"
        );
        assert!(
            rendered
                .contains("  nfr:\n    prefix: NFR\n    directory: requirements/non-functional/\n"),
            "{rendered}"
        );
        assert!(
            !rendered.contains("statuses: [active, archived]"),
            "{rendered}"
        );
        assert!(!rendered.contains("    fields:"), "{rendered}");
        let reparsed: Config = yaml_serde::from_value(value).unwrap();
        assert!(!reparsed.document_types["req"].enabled);
        assert_eq!(reparsed.document_types["nfr"].statuses, None);
    }

    #[test]
    fn validate_document_types_declarations_names_unknown_keys_and_enums() {
        let value: yaml_serde::Value = yaml_serde::from_str(
            "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    model: SingleMarkdown\n    fields:\n      - key: description\n        heading: '## Description'\n        kind: paragraph\n        order: 1\n",
        )
        .unwrap();
        let errors = validate_document_types_declarations(&value);
        assert_eq!(
            errors,
            vec![
                "document_types.deploy.model: not a configuration key",
                "document_types.deploy.fields[description].order: not a valid field key",
                "document_types.deploy.fields[description].kind: must be text, multiline, or list",
            ]
        );
    }

    #[test]
    fn resolve_selector_reads_keyed_and_indexed_field_forms() {
        let value: yaml_serde::Value = yaml_serde::from_str(
            "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields:\n      - key: story\n        heading: '## User Story'\n      - key: description\n        heading: '## Description'\n",
        )
        .unwrap();
        let keyed =
            resolve_selector(&value, "document_types.deploy.fields[story].heading").unwrap();
        let indexed = resolve_selector(&value, "document_types.deploy.fields[0].heading").unwrap();
        assert_eq!(keyed, indexed);
        assert!(resolve_selector(&value, "document_types.deploy.fields[9].heading").is_err());
        assert!(resolve_selector(&value, "document_types.deploy.fields[unknown].heading").is_err());
    }
}
