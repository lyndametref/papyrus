use crate::CommandOutput;
use crate::config::{
    Config, merge_yaml, validate_browser_host, validate_key_path, yaml_get, yaml_set, yaml_unset,
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

pub fn load_effective_value(config_path: &Path) -> Result<yaml_serde::Value, AppError> {
    let default_cfg = Config::default();
    let mut base =
        yaml_serde::to_value(&default_cfg).map_err(|e| AppError::Runtime(e.to_string()))?;
    if config_path.exists() {
        let raw =
            std::fs::read_to_string(config_path).map_err(|e| AppError::Runtime(e.to_string()))?;
        let overlay: yaml_serde::Value =
            yaml_serde::from_str(&raw).map_err(|e| AppError::Validation(e.to_string()))?;
        merge_yaml(&mut base, &overlay);
    }
    Ok(base)
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
    let val = load_effective_value(config_path)?;
    validate_browser_yaml(&val).map_err(AppError::Validation)?;
    let cfg: Config =
        yaml_serde::from_value(val).map_err(|e| AppError::Validation(e.to_string()))?;
    cfg.validate().map_err(AppError::Validation)?;
    let _ = project_dir;
    Ok(cfg)
}

pub fn config_list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;
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
    Ok(CommandOutput { text, json })
}

pub fn config_get(ctx: &Context, key: &str) -> Result<CommandOutput, AppError> {
    if !valid_sanitized_key(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    let val = load_effective_value(&ctx.config_path)?;

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
            return Ok(CommandOutput { text: name, json });
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
            return Ok(CommandOutput { text, json });
        }
    }

    let found =
        yaml_get(&val, key).ok_or_else(|| AppError::NotFound(format!("key not found: {}", key)))?;
    let text = yaml_scalar_to_string(found);
    let json = if ctx.json {
        Some(yaml_to_json(found))
    } else {
        None
    };
    Ok(CommandOutput { text, json })
}

pub fn config_set(ctx: &Context, key: &str, value: &str) -> Result<CommandOutput, AppError> {
    if !valid_sanitized_key(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    // parse value as YAML
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
    let parsed = match parsed_raw {
        yaml_serde::Value::String(s) => {
            if key == "browser_host" {
                // Host validation above is authoritative; the path-component
                // sanitizer would reject characters such as `/` with a generic
                // message that does not name the key.
                yaml_serde::Value::String(s)
            } else {
                let sanitized = crate::sanitize::sanitize_path_component(&s).ok_or_else(|| {
                    AppError::Validation(format!("{}: value cannot be normalized", key))
                })?;
                yaml_serde::Value::String(sanitized)
            }
        }
        other => other,
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

    yaml_set(&mut file_val, key, parsed).map_err(AppError::Validation)?;

    // validate merged effective
    let default_cfg = Config::default();
    let mut base = yaml_serde::to_value(&default_cfg).unwrap();
    merge_yaml(&mut base, &file_val);
    let cfg: Config = yaml_serde::from_value(base)
        .map_err(|e| AppError::Validation(format!("{}: {}", key, e)))?;
    cfg.validate().map_err(AppError::Validation)?;

    // boundary check for config path? skip
    // write
    let parent = ctx.config_path.parent();
    if let Some(p) = parent {
        let _ = std::fs::create_dir_all(p);
    }
    let out_str = yaml_serde::to_string(&file_val).map_err(|e| AppError::Runtime(e.to_string()))?;
    std::fs::write(&ctx.config_path, out_str.as_bytes())
        .map_err(|e| AppError::Runtime(e.to_string()))?;

    let json = if ctx.json {
        Some(serde_json::json!({"status":"ok"}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
    })
}

pub fn config_unset(ctx: &Context, key: &str) -> Result<CommandOutput, AppError> {
    if !valid_sanitized_key(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
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

    // validate merged
    let default_cfg = Config::default();
    let mut base = yaml_serde::to_value(&default_cfg).unwrap();
    merge_yaml(&mut base, &file_val);
    let cfg: Config =
        yaml_serde::from_value(base).map_err(|e| AppError::Validation(e.to_string()))?;
    cfg.validate().map_err(AppError::Validation)?;

    // write or remove
    let is_empty_map = matches!(&file_val, yaml_serde::Value::Mapping(m) if m.is_empty());
    if is_empty_map {
        let _ = std::fs::remove_file(&ctx.config_path);
    } else {
        let out_str =
            yaml_serde::to_string(&file_val).map_err(|e| AppError::Runtime(e.to_string()))?;
        std::fs::write(&ctx.config_path, out_str.as_bytes())
            .map_err(|e| AppError::Runtime(e.to_string()))?;
    }

    let json = if ctx.json {
        Some(serde_json::json!({"status":"ok"}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
    })
}

fn valid_sanitized_key(key: &str) -> bool {
    if !validate_key_path(key) {
        return false;
    }
    key.split('.')
        .all(|p| crate::sanitize::sanitize_path_component(p).is_some())
}

fn effective_project_name_from_dir(project_dir: &std::path::Path) -> String {
    // Mirror Config::effective_project_name without borrowing Config.
    project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}
