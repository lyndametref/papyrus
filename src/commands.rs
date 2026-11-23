use crate::CommandOutput;
use crate::config::{
    Config, is_list_valued_key, merge_yaml, parse_list_value, parse_selector, render_yaml_flow,
    resolve_selector, selector_targets_field, to_config_yaml, valid_config_selector,
    validate_browser_host, validate_document_types_declarations, yaml_get, yaml_set, yaml_unset,
};
use crate::context::Context;
use crate::error::AppError;
use std::path::Path;

fn yaml_to_json(v: &yaml_serde::Value) -> serde_json::Value {
    match v {
        yaml_serde::Value::Null => serde_json::Value::Null,
        yaml_serde::Value::Bool(b) => serde_json::Value::Bool(*b),
        yaml_serde::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::json!(i)
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::String(n.to_string())
            }
        }
        yaml_serde::Value::String(s) => serde_json::Value::String(s.clone()),
        yaml_serde::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json).collect())
        }
        yaml_serde::Value::Mapping(m) => {
            let mut obj = serde_json::Map::new();
            for (k, val) in m {
                let key = match k {
                    yaml_serde::Value::String(s) => s.clone(),
                    other => yaml_serde::to_string(other)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                };
                obj.insert(key, yaml_to_json(val));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

fn yaml_scalar_to_string(v: &yaml_serde::Value) -> String {
    match v {
        yaml_serde::Value::Null => "null".to_string(),
        yaml_serde::Value::Bool(b) => b.to_string(),
        yaml_serde::Value::Number(n) => n.to_string(),
        yaml_serde::Value::String(s) => s.clone(),
        other => yaml_serde::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

fn read_raw_config(config_path: &Path) -> Result<yaml_serde::Value, AppError> {
    if !config_path.exists() {
        return Err(AppError::Runtime(format!(
            "configuration file not found: {}; run `papyrus init` to create it",
            config_path.display()
        )));
    }
    let raw = std::fs::read_to_string(config_path).map_err(|e| AppError::Runtime(e.to_string()))?;
    crate::config::validate_yaml_input(&raw, crate::config::MAX_CONFIG_YAML_BYTES, "configuration")
        .map_err(AppError::Validation)?;
    yaml_serde::from_str(&raw).map_err(|e| AppError::Validation(e.to_string()))
}

fn effective_value_from_raw(overlay: &yaml_serde::Value) -> Result<yaml_serde::Value, AppError> {
    let default_cfg = Config::default();
    let mut base =
        yaml_serde::to_value(&default_cfg).map_err(|e| AppError::Runtime(e.to_string()))?;
    merge_yaml(&mut base, overlay);
    Ok(base)
}

/// Validate a raw file value as a candidate configuration.
///
/// One deterministic pass: the raw declaration validator reports unknown keys,
/// unknown enum values, entry and field type errors, field grammar, reserved
/// and duplicate field keys, and slot rules with their selector paths; the
/// browser keys are checked next; only a shape-valid file is deserialized and
/// checked by [`Config::validate`] for the remaining entry invariants. Every
/// collected diagnostic is returned together (TECH-077 section 8).
fn validate_candidate(
    file_val: &yaml_serde::Value,
    project_dir: &Path,
) -> Result<Config, AppError> {
    let mut errors = validate_document_types_declarations(file_val);
    let val = effective_value_from_raw(file_val)?;
    if let Err(reason) = validate_browser_yaml(&val) {
        errors.push(reason);
    }
    if !errors.is_empty() {
        return Err(AppError::Validation(errors.join("\n")));
    }
    let cfg: Config =
        yaml_serde::from_value(val).map_err(|e| AppError::Validation(e.to_string()))?;
    cfg.validate(Some(project_dir))
        .map_err(AppError::Validation)?;
    Ok(cfg)
}

pub fn load_effective_value(config_path: &Path) -> Result<yaml_serde::Value, AppError> {
    let overlay = read_raw_config(config_path)?;
    effective_value_from_raw(&overlay)
}

/// Validate the browser keys in the merged YAML before deserialization.
///
/// `browser_port` is a `u16` and `browser_open` is a `bool`, so an invalid
/// value in the configuration file would otherwise fail with a serde message
/// that does not name the offending key. Validating the merged YAML first
/// keeps the error stable and names the key (specification section 8.2).
fn validate_browser_yaml(val: &yaml_serde::Value) -> Result<(), String> {
    if let Some(port) = yaml_get(val, "browser_port") {
        let in_range = match port {
            yaml_serde::Value::Number(number) => number
                .as_u64()
                .map(|port| (1..=65535).contains(&port))
                .unwrap_or(false),
            _ => false,
        };
        if !in_range {
            return Err("browser_port: must be between 1 and 65535".to_string());
        }
    }
    if let Some(open) = yaml_get(val, "browser_open")
        && !matches!(open, yaml_serde::Value::Bool(_))
    {
        return Err("browser_open: must be a boolean".to_string());
    }
    if let Some(host) = yaml_get(val, "browser_host") {
        match host {
            yaml_serde::Value::String(host) => {
                validate_browser_host(host).map_err(|reason| format!("browser_host: {}", reason))?
            }
            _ => return Err("browser_host: must be a string".to_string()),
        }
    }
    Ok(())
}

pub fn load_effective_config(config_path: &Path, project_dir: &Path) -> Result<Config, AppError> {
    let raw = read_raw_config(config_path)?;
    validate_candidate(&raw, project_dir)
}

pub fn load_effective_config_tolerant(
    config_path: &Path,
    project_dir: &Path,
) -> Result<Config, AppError> {
    let val = load_effective_value(config_path)?;
    validate_browser_yaml(&val).map_err(AppError::Validation)?;
    let cfg: Config =
        yaml_serde::from_value(val).map_err(|e| AppError::Validation(e.to_string()))?;
    let _ = project_dir;
    Ok(cfg)
}

pub fn config_list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_effective_config_tolerant(&ctx.config_path, &ctx.project_dir)?;
    // resolve project_name into a cloned config for json
    let mut cfg_json = cfg.clone();
    if cfg_json.project_name.is_none() {
        cfg_json.project_name = Some(cfg_json.effective_project_name(&ctx.project_dir));
    }
    cfg_json.editor = cfg_json.effective_editor();
    let text_lines: Vec<String> = cfg
        .flattened(&ctx.project_dir)
        .into_iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect();
    let text = text_lines.join("\n");
    let json = if ctx.json {
        Some(serde_json::to_value(&cfg_json).map_err(|e| AppError::Runtime(e.to_string()))?)
    } else {
        None
    };
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

pub fn config_get(ctx: &Context, key: &str) -> Result<CommandOutput, AppError> {
    if !valid_config_selector(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    let val = load_effective_value(&ctx.config_path)?;
    validate_browser_yaml(&val).map_err(AppError::Validation)?;

    // Effective defaults for project_name and editor.
    if key == "project_name" {
        let found = yaml_get(&val, key);
        let is_empty = match found {
            None => true,
            Some(yaml_serde::Value::Null) => true,
            Some(yaml_serde::Value::String(s)) => s.trim().is_empty(),
            _ => false,
        };
        if is_empty {
            let name = effective_project_name_from_dir(&ctx.project_dir);
            let json = if ctx.json {
                Some(serde_json::Value::String(name.clone()))
            } else {
                None
            };
            return Ok(CommandOutput {
                text: name,
                json,
                ..Default::default()
            });
        }
    }
    if key == "editor" {
        let found = yaml_get(&val, key);
        let from_config = match found {
            Some(yaml_serde::Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };
        if from_config.is_none() {
            let effective = std::env::var("VISUAL")
                .ok()
                .filter(|v| !v.is_empty())
                .or_else(|| std::env::var("EDITOR").ok().filter(|v| !v.is_empty()));
            let text = effective.clone().unwrap_or_else(|| "null".to_string());
            let json = if ctx.json {
                Some(match effective {
                    Some(v) => serde_json::Value::String(v),
                    None => serde_json::Value::Null,
                })
            } else {
                None
            };
            return Ok(CommandOutput {
                text,
                json,
                ..Default::default()
            });
        }
    }

    let found = match resolve_selector(&val, key) {
        Ok(found) => found,
        Err(reason) if key.starts_with("document_types.") => {
            return Err(AppError::Validation(format!("{}: {}", key, reason)));
        }
        Err(_) => {
            return Err(AppError::NotFound(format!("key not found: {}", key)));
        }
    };
    // A sequence is printed in flow form, including a block-style list read
    // from a hand-edited file and a field's verbs list.
    let text = if matches!(found, yaml_serde::Value::Sequence(_)) {
        render_yaml_flow(found)
    } else {
        yaml_scalar_to_string(found)
    };
    let json = if ctx.json {
        Some(yaml_to_json(found))
    } else {
        None
    };
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

#[cfg(test)]
thread_local! {
    static FAIL_NEXT_CONFIG_WRITE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Arm one injected configuration-write failure for the current test thread.
#[cfg(test)]
pub(crate) fn fail_next_config_write() {
    FAIL_NEXT_CONFIG_WRITE.with(|flag| flag.set(true));
}

/// Persist a validated configuration atomically.
fn write_config_atomically(path: &std::path::Path, content: &str) -> Result<(), AppError> {
    #[cfg(test)]
    if FAIL_NEXT_CONFIG_WRITE.with(|flag| flag.replace(false)) {
        return Err(AppError::Runtime(
            "injected configuration write failure".to_string(),
        ));
    }
    crate::config::write_config_file(path, content)
}

pub fn config_set(ctx: &Context, key: &str, value: &str) -> Result<CommandOutput, AppError> {
    if !valid_config_selector(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    if selector_targets_field(key) {
        return Err(AppError::Validation(format!(
            "{}: fields are configuration file data and cannot be changed by config set",
            key
        )));
    }
    reject_declarative_data_selector(key, "set")?;
    reject_bracketed_selector(key, "set")?;
    // The key is classified before the value is parsed (ADR-0024 decision 1).
    // A list-valued key takes the comma-separated form and never enters the
    // YAML scalar path, so a comma stays separators here and ordinary data for
    // every other key (decision 4).
    let parsed = if is_list_valued_key(key) {
        let values = parse_list_value(key, value).map_err(AppError::Validation)?;
        yaml_serde::Value::Sequence(values.into_iter().map(yaml_serde::Value::String).collect())
    } else {
        parse_scalar_value(key, value)?
    };

    // load file value (or empty mapping)
    let mut file_val = if ctx.config_path.exists() {
        let raw = std::fs::read_to_string(&ctx.config_path)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        yaml_serde::from_str(&raw).map_err(|e| AppError::Validation(e.to_string()))?
    } else {
        yaml_serde::Value::Mapping(yaml_serde::Mapping::new())
    };
    if !matches!(file_val, yaml_serde::Value::Mapping(_)) {
        file_val = yaml_serde::Value::Mapping(yaml_serde::Mapping::new());
    }

    // Materialize only the target entry when the file declares no
    // `document_types` map. The entry carries its seed prefix and directory,
    // which are never filled (REQ-07.002, TC-114).
    if key.starts_with("document_types.") {
        let slug = key.split('.').nth(1).unwrap_or_default();
        let entry_key = format!("document_types.{}", slug);
        if yaml_get(&file_val, &entry_key).is_none() {
            materialize_target_entry(&mut file_val, key)?;
        }
    }

    yaml_set(&mut file_val, key, parsed.clone()).map_err(AppError::Validation)?;

    validate_candidate(&file_val, &ctx.project_dir)?;

    // Folder relocation, if applicable, runs before persistence. Its failure
    // leaves the old file untouched; a failed persistence rolls it back.
    let mut relocation_cross_device = false;
    let mut relocation: Option<(std::path::PathBuf, std::path::PathBuf)> = None;
    if Config::is_folder_key(key) {
        let old_cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;
        let new_value = match &parsed {
            yaml_serde::Value::String(s) => s.clone(),
            _ => {
                return Err(AppError::Validation(format!(
                    "{}: value must be a string",
                    key
                )));
            }
        };
        let (source, destination) = folder_paths(ctx, &old_cfg, key, &new_value)?;
        let report = crate::record::relocate::relocate(ctx, &source, &destination)?;
        relocation_cross_device = report.cross_device;
        relocation = Some((source, destination));
    }

    let out_str = to_config_yaml(&file_val).map_err(AppError::Runtime)?;
    if let Err(error) = write_config_atomically(&ctx.config_path, &out_str) {
        if let Some((source, destination)) = relocation
            && let Err(rollback) = crate::record::relocate::relocate(ctx, &destination, &source)
        {
            return Err(AppError::Runtime(format!(
                "{}; failed to roll back relocation {} -> {}: {}",
                error,
                destination.display(),
                source.display(),
                rollback
            )));
        }
        return Err(error);
    }

    let text = if relocation_cross_device {
        "relocation: non-atomic operation (a single rename was impossible; used copy-then-remove)"
            .to_string()
    } else {
        String::new()
    };
    let json = if ctx.json {
        Some(serde_json::json!({"status":"ok"}))
    } else {
        None
    };
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

pub fn config_unset(ctx: &Context, key: &str) -> Result<CommandOutput, AppError> {
    if !valid_config_selector(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    if selector_targets_field(key) {
        return Err(AppError::Validation(format!(
            "{}: fields are configuration file data and cannot be changed by config unset",
            key
        )));
    }
    reject_declarative_data_selector(key, "unset")?;
    reject_bracketed_selector(key, "unset")?;
    let mut file_val = if ctx.config_path.exists() {
        let raw = std::fs::read_to_string(&ctx.config_path)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        yaml_serde::from_str(&raw).map_err(|e| AppError::Validation(e.to_string()))?
    } else {
        yaml_serde::Value::Mapping(yaml_serde::Mapping::new())
    };
    if !matches!(file_val, yaml_serde::Value::Mapping(_)) {
        file_val = yaml_serde::Value::Mapping(yaml_serde::Mapping::new());
    }

    yaml_unset(&mut file_val, key).map_err(AppError::Validation)?;

    validate_candidate(&file_val, &ctx.project_dir)?;

    // write or remove
    let is_empty_map = matches!(&file_val, yaml_serde::Value::Mapping(m) if m.is_empty());
    if is_empty_map {
        let _ = std::fs::remove_file(&ctx.config_path);
    } else {
        let out_str = to_config_yaml(&file_val).map_err(AppError::Runtime)?;
        write_config_atomically(&ctx.config_path, &out_str)?;
    }

    let json = if ctx.json {
        Some(serde_json::json!({"status":"ok"}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

/// Whether a selector addresses declarative file data (`api_aliases` or
/// `slots`).
fn selector_targets_declarative_data(key: &str) -> bool {
    parse_selector(key).is_ok_and(|segments| {
        segments
            .iter()
            .any(|segment| matches!(segment.key.as_str(), "api_aliases" | "slots"))
    })
}

/// Reject `api_aliases` and `slots` as `config set`/`unset` targets.
///
/// Both are declarative file data (ADR-042 ruling 6): a comma-separated scalar
/// cannot express a slot mapping, and neither is a scalar leaf. The selector
/// path is reported so the caller knows which entry to edit in the file.
fn reject_declarative_data_selector(key: &str, command: &str) -> Result<(), AppError> {
    if selector_targets_declarative_data(key) {
        return Err(AppError::Validation(format!(
            "{}: declarative file data and cannot be changed by config {}",
            key, command
        )));
    }
    Ok(())
}

/// Reject an indexed or keyed sequence selector for `config set`/`unset`.
///
/// The YAML mutators operate on dotted mapping keys only, so a bracketed
/// selector would otherwise create a literal `slots[0]` key (L-6). `config get`
/// still accepts both forms.
fn reject_bracketed_selector(key: &str, command: &str) -> Result<(), AppError> {
    if key.contains('[') || key.contains(']') {
        return Err(AppError::Validation(format!(
            "{}: bracketed selectors are not supported by config {}; use the dotted mapping key without an index",
            key, command
        )));
    }
    Ok(())
}

/// Parse and sanitize a scalar `config set` value (specification section 8.2).
///
/// Kept separate from the list path so the two value forms never share
/// semantics: a sequence or mapping written as a scalar value is still refused,
/// and commas stay ordinary characters here.
fn parse_scalar_value(key: &str, value: &str) -> Result<yaml_serde::Value, AppError> {
    let parsed_raw: yaml_serde::Value =
        yaml_serde::from_str(value).map_err(|e| AppError::Validation(e.to_string()))?;
    // require scalar
    match parsed_raw {
        yaml_serde::Value::Mapping(_) | yaml_serde::Value::Sequence(_) => {
            return Err(AppError::Validation(
                "value must be a YAML scalar".to_string(),
            ));
        }
        yaml_serde::Value::Null => {
            return Err(AppError::Validation(
                "value cannot be normalized".to_string(),
            ));
        }
        _ => {}
    }
    // Browser keys are validated here so the message names the offending key
    // (specification section 8.2). `browser_port` is a `u16`, so an
    // out-of-range value would otherwise fail deserialization with a message
    // that does not name the key.
    if key == "browser_port" {
        let in_range = match &parsed_raw {
            yaml_serde::Value::Number(number) => number
                .as_u64()
                .map(|port| (1..=65535).contains(&port))
                .unwrap_or(false),
            _ => false,
        };
        if !in_range {
            return Err(AppError::Validation(
                "browser_port: must be between 1 and 65535".to_string(),
            ));
        }
    }
    if key == "browser_host"
        && let yaml_serde::Value::String(host) = &parsed_raw
        && let Err(reason) = validate_browser_host(host)
    {
        return Err(AppError::Validation(format!("browser_host: {}", reason)));
    }

    // Sanitize string values.
    Ok(match parsed_raw {
        yaml_serde::Value::String(s) => {
            if key == "browser_host" {
                // Host validation above is authoritative; the path-component
                // sanitizer would reject characters such as `/` with a generic
                // message that does not name the key.
                yaml_serde::Value::String(s)
            } else if Config::is_folder_key(key) {
                let sanitized = crate::sanitize::sanitize_path_string(&s).ok_or_else(|| {
                    AppError::Validation(format!("{}: value cannot be normalized", key))
                })?;
                yaml_serde::Value::String(sanitized)
            } else {
                let sanitized = crate::sanitize::sanitize_path_component(&s).ok_or_else(|| {
                    AppError::Validation(format!("{}: value cannot be normalized", key))
                })?;
                yaml_serde::Value::String(sanitized)
            }
        }
        other => other,
    })
}

fn folder_paths(
    ctx: &Context,
    cfg: &Config,
    key: &str,
    new_value: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), AppError> {
    let (source_raw, destination_raw) = match key {
        "backlog_directory" => (
            ctx.project_dir.join(&cfg.backlog_directory),
            ctx.project_dir.join(new_value),
        ),
        "decision_directory" => (
            ctx.project_dir.join(&cfg.decision_directory),
            ctx.project_dir.join(new_value),
        ),
        "doc_directory" => (
            ctx.project_dir.join(&cfg.doc_directory),
            ctx.project_dir.join(new_value),
        ),
        _ if key.starts_with("document_types.") && key.ends_with(".directory") => {
            let type_key = key
                .trim_start_matches("document_types.")
                .trim_end_matches(".directory");
            let old_entry = cfg.document_types.get(type_key).ok_or_else(|| {
                AppError::Validation(format!("unknown document type: {}", type_key))
            })?;
            let doc_dir = &cfg.doc_directory;
            (
                ctx.project_dir.join(doc_dir).join(&old_entry.directory),
                ctx.project_dir.join(doc_dir).join(new_value),
            )
        }
        _ => {
            return Err(AppError::Validation("not a folder key".to_string()));
        }
    };
    let source = Context::ensure_within(&ctx.project_dir, &source_raw).map_err(|_| {
        AppError::Runtime("source path resolves outside project directory".to_string())
    })?;
    let destination = Context::ensure_within(&ctx.project_dir, &destination_raw).map_err(|_| {
        AppError::Runtime("destination path resolves outside project directory".to_string())
    })?;
    Ok((source, destination))
}

/// Materialize the single target entry from the seed prefix and directory.
fn materialize_target_entry(file_val: &mut yaml_serde::Value, key: &str) -> Result<(), AppError> {
    let slug = key
        .split('.')
        .nth(1)
        .filter(|slug| !slug.is_empty())
        .ok_or_else(|| AppError::Validation("invalid key".to_string()))?;
    let defaults = Config::default();
    let seed = defaults.document_types.get(slug).ok_or_else(|| {
        AppError::Validation(format!(
            "document_types.{}.directory: cannot be introduced by config set; add the entry and its prefix and directory to the configuration file",
            slug
        ))
    })?;
    let mut entry = yaml_serde::Mapping::new();
    entry.insert(
        yaml_serde::Value::String("prefix".to_string()),
        yaml_serde::Value::String(seed.prefix.clone()),
    );
    entry.insert(
        yaml_serde::Value::String("directory".to_string()),
        yaml_serde::Value::String(seed.directory.clone()),
    );
    let mut document_types = match yaml_get(file_val, "document_types") {
        Some(yaml_serde::Value::Mapping(existing)) => existing.clone(),
        _ => yaml_serde::Mapping::new(),
    };
    document_types.insert(
        yaml_serde::Value::String(slug.to_string()),
        yaml_serde::Value::Mapping(entry),
    );
    yaml_set(
        file_val,
        "document_types",
        yaml_serde::Value::Mapping(document_types),
    )
    .map_err(AppError::Validation)
}

fn effective_project_name_from_dir(project_dir: &std::path::Path) -> String {
    // Mirror Config::effective_project_name without borrowing Config.
    project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}
