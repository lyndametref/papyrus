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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
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
}

impl Default for DocTypeEntry {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            directory: String::new(),
            enabled: true,
            statuses: None,
            default_status: None,
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
        let docs = [
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
        for (k, p, d, has_status) in docs {
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
        }
        out.push(("browser_host".to_string(), self.browser_host.clone()));
        out.push(("browser_port".to_string(), self.browser_port.to_string()));
        out.push(("browser_open".to_string(), self.browser_open.to_string()));
        out
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
                    // Replace wholesale when present.
                    b.insert(k.clone(), v.clone());
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
/// Mappings stay in block style, sequences are rendered inline as flow
/// sequences (`[a, b]`), and scalars are quoted by the YAML serializer so the
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

fn render_block_mapping(mapping: &yaml_serde::Mapping, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    for (key, value) in mapping {
        let key = render_yaml_scalar(key);
        match value {
            yaml_serde::Value::Mapping(inner) if !inner.is_empty() => {
                out.push_str(&format!("{}{}:\n", pad, key));
                render_block_mapping(inner, indent + 2, out);
            }
            other => {
                out.push_str(&format!("{}{}: {}\n", pad, key, render_yaml_flow(other)));
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
            if text.chars().any(char::is_control) {
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
        for (slug, builtin) in [("Config", "config"), ("SEARCH", "search"), ("task", "task")] {
            let err = validate_single(slug, entry("NOTE", "notes/")).unwrap_err();
            assert!(
                err.contains(&format!("slug conflicts with built-in type {}", builtin)),
                "slug {:?} => {}",
                slug,
                err
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
    fn config_set_materializes_defaults_and_refuses_an_invalid_result() {
        let project = TempProject::new("config-set");
        let ctx = project.context();

        config_set(&ctx, "document_types.glossary.enabled", "false").unwrap();
        let value: yaml_serde::Value =
            yaml_serde::from_str(&project.read_config()).expect("parse written config");
        let document_types = yaml_get(&value, "document_types").unwrap();
        let yaml_serde::Value::Mapping(mapping) = document_types else {
            panic!("document_types must be a mapping");
        };
        assert_eq!(mapping.len(), Config::default().document_types.len());
        assert!(yaml_get(&value, "document_types.req").is_some());
        assert!(yaml_get(&value, "document_types.deploy").is_none());
        assert!(matches!(
            yaml_get(&value, "document_types.glossary.enabled"),
            Some(yaml_serde::Value::Bool(false))
        ));

        let before = project.read_config();
        let err = config_set(&ctx, "document_types.glossary.prefix", "REQ")
            .err()
            .expect("config set must refuse an invalid result");
        assert!(matches!(err, AppError::Validation(_)), "{:?}", err);
        assert!(err.message().contains("already used"), "{}", err);
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
}
