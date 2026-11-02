use crate::CommandOutput;
use crate::config::{Config, merge_yaml, validate_key_path, yaml_get, yaml_set, yaml_unset};
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
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

pub fn load_effective_config(config_path: &Path, project_dir: &Path) -> Result<Config, AppError> {
    let val = load_effective_value(config_path)?;
    let cfg: Config =
        yaml_serde::from_value(val).map_err(|e| AppError::Validation(e.to_string()))?;
    cfg.validate().map_err(AppError::Validation)?;
    // resolve project_name default handled in methods; store resolved if None? leave None.
    let _ = (project_dir, sanitize::sanitize_name);
    Ok(cfg)
}

pub fn config_list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;
    // resolve project_name into a cloned config for json
    let mut cfg_json = cfg.clone();
    if cfg_json.project_name.is_none() {
        cfg_json.project_name = Some(cfg_json.effective_project_name(&ctx.project_dir));
    }
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
    if !validate_key_path(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    let val = load_effective_value(&ctx.config_path)?;
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
    if !validate_key_path(key) {
        return Err(AppError::Validation("invalid key".to_string()));
    }
    // parse value as YAML
    let parsed: yaml_serde::Value =
        yaml_serde::from_str(value).map_err(|e| AppError::Validation(e.to_string()))?;
    // require scalar
    match parsed {
        yaml_serde::Value::Mapping(_) | yaml_serde::Value::Sequence(_) => {
            return Err(AppError::Validation(
                "value must be a YAML scalar".to_string(),
            ));
        }
        _ => {}
    }

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
    let cfg: Config =
        yaml_serde::from_value(base).map_err(|e| AppError::Validation(e.to_string()))?;
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
    if !validate_key_path(key) {
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
