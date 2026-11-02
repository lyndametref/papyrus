use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DocTypeEntry {
    pub prefix: String,
    pub directory: String,
    pub enabled: bool,
}

impl Default for DocTypeEntry {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            directory: String::new(),
            enabled: true,
        }
    }
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
        let docs = [
            ("req", "REQ", "requirements/business/"),
            ("nfr", "NFR", "requirements/non-functional/"),
            ("glossary", "GLO", "glossary/"),
            ("rule", "RUL", "business-rules/"),
            ("ui-spec", "UI", "specifications/ui/"),
            ("audit", "AUD", "audit-records/"),
            ("deploy", "DEP", "architecture/deployment/"),
        ];
        for (k, p, d) in docs {
            document_types.insert(
                k.to_string(),
                DocTypeEntry {
                    prefix: p.to_string(),
                    directory: d.to_string(),
                    enabled: true,
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

    pub fn validate(&self) -> Result<(), String> {
        if self.id_padding == 0 {
            return Err("id_padding must be a positive integer".to_string());
        }
        // uniqueness of statuses
        let mut seen = std::collections::HashSet::new();
        for s in &self.statuses {
            if !seen.insert(s.clone()) {
                return Err(format!("duplicate status: {}", s));
            }
        }
        if !self.statuses.contains(&self.default_status) {
            return Err(format!(
                "default_status '{}' is not one of the configured statuses",
                self.default_status
            ));
        }
        if self.statuses.is_empty() {
            return Err("statuses must not be empty".to_string());
        }
        // non-empty prefixes
        if self.task_prefix.is_empty()
            || self.milestone_prefix.is_empty()
            || self.release_prefix.is_empty()
        {
            return Err("ID prefixes must not be empty".to_string());
        }
        Ok(())
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
            self.editor.clone().unwrap_or_default(),
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
        }
        out
    }
}

/// Merge overlay YAML mapping into base recursively.
pub fn merge_yaml(base: &mut yaml_serde::Value, overlay: &yaml_serde::Value) {
    match (base, overlay) {
        (yaml_serde::Value::Mapping(b), yaml_serde::Value::Mapping(o)) => {
            for (k, v) in o {
                if let Some(existing) = b.get_mut(k) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_valid() {
        let c = Config::default();
        assert!(c.validate().is_ok());
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
        assert!(c.validate().is_err());
    }

    #[test]
    fn default_status_membership() {
        let c = Config {
            default_status: "Nope".into(),
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn id_padding_zero_invalid() {
        let c = Config {
            id_padding: 0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
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
}
