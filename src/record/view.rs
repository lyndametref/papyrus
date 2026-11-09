//! Shared read projection (`RecordView`) for record metadata.
//!
//! A single projection is the only source of record metadata for every CLI
//! and browser surface (planned ticket TASK-67, ADR-0009 decision 5):
//!
//! * `id` is the Papyrus display ID and is the record identity on every
//!   surface;
//! * `metadata` is the record's front-matter keys exactly as stored, in
//!   canonical front-matter key order (general keys first, then
//!   type-specific keys). The Backlog.md storage ID lives only at
//!   `metadata["id"]`;
//! * `version` is the ADR-0006 optimistic-concurrency token; it is not
//!   part of `metadata`.

use crate::error::AppError;
use crate::record::frontmatter;
use serde_json::Value as JsonValue;

pub type MetadataEntry = (String, JsonValue);

/// Shared read projection for one record's metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordView {
    pub id: String,
    pub metadata: Vec<MetadataEntry>,
    pub version: String,
}

impl RecordView {
    pub fn from_content(content: &str, display_id: &str, version: &str) -> Result<Self, AppError> {
        let mut metadata = Vec::new();
        if let Some(fm) = frontmatter::parse(content)? {
            let keys = ordered_keys(&fm.raw);
            let yaml_value = fm.data.clone();
            for key in keys {
                let value = if let yaml_serde::Value::Mapping(m) = &yaml_value {
                    m.get(yaml_serde::Value::String(key.clone()))
                        .cloned()
                        .unwrap_or(yaml_serde::Value::Null)
                } else {
                    yaml_serde::Value::Null
                };
                metadata.push((key, yaml_to_json(&value)));
            }
        }
        Ok(Self {
            id: display_id.to_string(),
            metadata,
            version: version.to_string(),
        })
    }

    /// Lenient projection: falls back to empty metadata when the front
    /// matter cannot be parsed, so CLI list/view rendering never fails
    /// on a record another surface already accepts.
    pub fn lenient(content: &str, display_id: &str) -> Self {
        Self {
            id: display_id.to_string(),
            metadata: Vec::new(),
            version: String::new(),
        }
        .with_metadata_from(content)
    }

    fn with_metadata_from(mut self, content: &str) -> Self {
        if let Ok(parsed) = Self::from_content(content, &self.id, "") {
            self.metadata = parsed.metadata;
        }
        self
    }

    /// The `metadata` JSON object, keys in canonical front-matter order.
    /// `serde_json` is built with `preserve_order`, so the insertion
    /// order of [`Self::metadata`] is the serialized order.
    pub fn metadata_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();
        for (k, v) in &self.metadata {
            map.insert(k.clone(), v.clone());
        }
        JsonValue::Object(map)
    }

    pub fn metadata_value(&self, key: &str) -> Option<&JsonValue> {
        self.metadata.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Title from `metadata["title"]`, if present.
    pub fn title(&self) -> Option<String> {
        self.str_value("title")
    }

    /// A metadata value rendered as a plain string, if it is a string.
    pub fn str_value(&self, key: &str) -> Option<String> {
        match self.metadata_value(key) {
            Some(JsonValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// A metadata value rendered as a list of strings, if it is an array.
    pub fn list_value(&self, key: &str) -> Vec<String> {
        match self.metadata_value(key) {
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// One CLI list line rendered from this projection. The line starts
    /// with the display ID and title, then shows the general metadata
    /// fields (`created_date`, `updated_date`, `labels`, `references`)
    /// and every remaining key (the type-specific keys) in canonical
    /// order.
    pub fn list_line(&self, fallback_title: &str) -> String {
        let title = self.title().unwrap_or_else(|| fallback_title.to_string());
        if self.metadata.is_empty() {
            // Records without front matter (for example HTML mockups and
            // unmigrated legacy files) keep the plain identity line.
            return format!("{} {}", self.id, title);
        }
        let mut parts: Vec<String> = Vec::new();
        let shown = ["created_date", "updated_date", "labels", "references"];
        for key in shown {
            parts.push(format!("{}={}", key, self.value_text(key)));
        }
        for (key, _) in &self.metadata {
            if key == "id" || key == "title" || shown.contains(&key.as_str()) {
                continue;
            }
            parts.push(format!("{}={}", key, self.value_text(key)));
        }
        format!("{} {} | {}", self.id, title, parts.join(" "))
    }

    /// A metadata value rendered for plain-text and HTML surfaces:
    /// strings verbatim, lists comma-joined, missing or null values as
    /// an empty string, other scalars in JSON text form.
    pub fn value_text(&self, key: &str) -> String {
        match self.metadata_value(key) {
            None | Some(JsonValue::Null) => String::new(),
            Some(JsonValue::String(s)) => s.clone(),
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(","),
            Some(other) => other.to_string(),
        }
    }
}

fn ordered_keys(raw: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for line in raw.lines() {
        if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
            continue;
        }
        if let Some((k, _)) = line.split_once(':') {
            let k = k.trim().to_string();
            if !k.is_empty() && !keys.contains(&k) {
                keys.push(k);
            }
        }
    }
    keys
}

fn yaml_to_json(value: &yaml_serde::Value) -> JsonValue {
    match value {
        yaml_serde::Value::Null => JsonValue::Null,
        yaml_serde::Value::Bool(b) => JsonValue::Bool(*b),
        yaml_serde::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                JsonValue::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null)
            } else {
                JsonValue::Null
            }
        }
        yaml_serde::Value::String(s) => JsonValue::String(s.clone()),
        yaml_serde::Value::Sequence(seq) => {
            JsonValue::Array(seq.iter().map(yaml_to_json).collect())
        }
        yaml_serde::Value::Mapping(m) => {
            let mut map = serde_json::Map::new();
            for (k, v) in m {
                let key = match k {
                    yaml_serde::Value::String(s) => s.clone(),
                    other => yaml_serde::to_string(other)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                };
                map.insert(key, yaml_to_json(v));
            }
            JsonValue::Object(map)
        }
        yaml_serde::Value::Tagged(t) => yaml_to_json(&t.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"---
id: task-1
title: Alpha task
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T15:06:00Z'
labels:
  - bug
  - ui
references: []
status: open
milestone: m-2
---

# TASK-001 Alpha task

## Description
Desc
"#;

    #[test]
    fn metadata_keys_keep_canonical_front_matter_order() {
        let view = RecordView::from_content(SAMPLE, "TASK-001", "1").unwrap();
        let keys: Vec<&str> = view.metadata.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            [
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references",
                "status",
                "milestone"
            ]
        );
        // Serialized order matches the stored order (preserve_order).
        let json = view.metadata_json().to_string();
        let mut last = 0usize;
        for key in [
            "\"id\"",
            "\"title\"",
            "\"created_date\"",
            "\"status\"",
            "\"milestone\"",
        ] {
            let at = json.find(key).expect("key present");
            assert!(at > last, "keys out of order in {json}");
            last = at;
        }
    }

    #[test]
    fn identity_is_display_id_and_storage_id_stays_in_metadata() {
        let view = RecordView::from_content(SAMPLE, "TASK-001", "1").unwrap();
        assert_eq!(view.id, "TASK-001");
        assert_eq!(
            view.str_value("id").as_deref(),
            Some("task-1"),
            "storage ID belongs inside metadata only"
        );
    }

    #[test]
    fn list_line_shows_general_and_type_specific_keys() {
        let view = RecordView::from_content(SAMPLE, "TASK-001", "1").unwrap();
        let line = view.list_line("fallback");
        assert!(line.starts_with("TASK-001 Alpha task"), "got: {line}");
        assert!(
            line.contains("created_date=2026-11-08T15:06:00Z"),
            "got: {line}"
        );
        assert!(
            line.contains("updated_date=2026-11-08T15:06:00Z"),
            "got: {line}"
        );
        assert!(line.contains("labels=bug,ui"), "got: {line}");
        assert!(line.contains("references="), "got: {line}");
        assert!(line.contains("status=open"), "got: {line}");
        assert!(line.contains("milestone=m-2"), "got: {line}");
    }

    #[test]
    fn missing_front_matter_projects_empty_metadata() {
        let view = RecordView::from_content("# TASK-001 Legacy\n", "TASK-001", "1").unwrap();
        assert!(view.metadata.is_empty());
        assert_eq!(view.metadata_json().to_string(), "{}");
        let line = view.list_line("Legacy title");
        assert_eq!(line, "TASK-001 Legacy title");
    }

    #[test]
    fn list_value_reads_string_arrays() {
        let view = RecordView::from_content(SAMPLE, "TASK-001", "1").unwrap();
        assert_eq!(
            view.list_value("labels"),
            vec!["bug".to_string(), "ui".to_string()]
        );
        assert!(view.list_value("references").is_empty());
    }
}
