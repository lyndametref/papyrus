use crate::config::Config;
use crate::error::AppError;
use crate::sanitize::{sanitize_name, sanitize_path_string};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;

pub struct InitOptions {
    pub name: String,
    pub no_git: bool,
    pub force: bool,
}

pub fn run_init(project_dir_raw: PathBuf, opts: InitOptions) -> Result<(), AppError> {
    let interactive = std::io::stdin().is_terminal();
    if interactive {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        run_init_with_io(project_dir_raw, opts, true, &mut reader, &mut writer)
    } else {
        let mut reader = std::io::empty();
        let mut writer = std::io::sink();
        run_init_with_io(project_dir_raw, opts, false, &mut reader, &mut writer)
    }
}

pub fn run_init_with_io<R: BufRead, W: Write>(
    project_dir_raw: PathBuf,
    opts: InitOptions,
    interactive: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<(), AppError> {
    let name = sanitize_name(&opts.name).ok_or_else(|| {
        AppError::Validation("project name cannot be normalized into a valid value".to_string())
    })?;

    // Create project dir if absent.
    if !project_dir_raw.exists() {
        fs::create_dir_all(&project_dir_raw).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
    let project_dir =
        fs::canonicalize(&project_dir_raw).map_err(|e| AppError::Runtime(e.to_string()))?;

    let config_path = project_dir.join("papyrus.config.yml");

    // Conflict check: existing config blocks init without --force; writes nothing.
    if config_path.exists() && !opts.force {
        return Err(AppError::Validation(
            "papyrus.config.yml already exists; use --force to replace it".to_string(),
        ));
    }

    // Build effective config.
    let config = if interactive {
        build_config_interactive(name.clone(), reader, writer)?
    } else {
        Config {
            project_name: Some(name.clone()),
            id_padding: 3,
            ..Config::default()
        }
    };

    // Validate before writing.
    config.validate().map_err(AppError::Validation)?;

    // Create directories through the shared record descriptors so the layout
    // has one source of truth. Directory creation follows activation.
    for name in ["task", "milestone", "release"] {
        if let Some(desc) = crate::record::descriptor::descriptor_for(name)
            && let Ok(dir) = desc.resolve_directory(&config, &project_dir)
        {
            let _ = fs::create_dir_all(dir);
        }
    }

    // Always ensure the documentation root exists.
    let _ = fs::create_dir_all(project_dir.join(&config.doc_directory));

    let any_decision = config.decision_types.values().any(|v| v.enabled);
    if any_decision
        && let Some(desc) = crate::record::descriptor::descriptor_for("decision")
        && let Ok(dir) = desc.resolve_directory(&config, &project_dir)
    {
        let _ = fs::create_dir_all(dir);
    }

    for (key, dt) in &config.document_types {
        if !dt.enabled {
            continue;
        }
        let Some(desc) = crate::record::descriptor::descriptor_for_doc_type(key) else {
            continue;
        };
        if let Ok(dir) = desc.resolve_directory(&config, &project_dir) {
            let _ = fs::create_dir_all(&dir);
        }
        if let Some(Ok(deprecated)) = desc.resolve_deprecated_directory(&config, &project_dir) {
            let _ = fs::create_dir_all(deprecated);
        }
    }

    // Write config file (full replace under --force or when absent).
    let yaml = yaml_serde::to_string(&config).map_err(|e| AppError::Runtime(e.to_string()))?;
    fs::write(&config_path, yaml.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

    // Git init unless suppressed.
    if !opts.no_git {
        let status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                return Err(AppError::Runtime(format!("git init failed with {}", s)));
            }
            Err(e) => {
                return Err(AppError::Runtime(format!("failed to invoke git: {}", e)));
            }
        }
    }

    Ok(())
}

fn build_config_interactive<R: BufRead, W: Write>(
    name: String,
    reader: &mut R,
    writer: &mut W,
) -> Result<Config, AppError> {
    let defaults = Config::default();
    let mut config = Config {
        project_name: Some(name),
        id_padding: 3,
        ..Config::default()
    };

    // Default activation: everything active.
    let mut active_decisions: BTreeMap<String, bool> = defaults
        .decision_types
        .keys()
        .map(|k| (k.clone(), true))
        .collect();
    let mut active_docs: BTreeMap<String, bool> = defaults
        .document_types
        .keys()
        .map(|k| (k.clone(), true))
        .collect();

    // Single artifact list presentation.
    let _ = writeln!(
        writer,
        "Artifacts (space to toggle; Enter to accept defaults):"
    );
    for key in ["task", "milestone", "release"] {
        let _ = writeln!(writer, "  [x] {} (always active)", key);
    }
    for key in defaults.decision_types.keys() {
        let _ = writeln!(writer, "  [x] {}", key);
    }
    for key in defaults.document_types.keys() {
        let _ = writeln!(writer, "  [x] {}", key);
    }

    // Space-toggle input (line-based): tokens toggle activation.
    let _ = write!(
        writer,
        "Toggle artifacts (space-separated, Enter to keep defaults): "
    );
    let _ = writer.flush();
    let mut toggle_line = String::new();
    let _ = reader.read_line(&mut toggle_line);
    for token in toggle_line.split_whitespace() {
        let t = token.to_lowercase();
        if let Some(v) = active_decisions.get_mut(&t) {
            *v = !*v;
        } else if let Some(v) = active_docs.get_mut(&t) {
            *v = !*v;
        }
    }

    let default_task_prefix = defaults.task_prefix.clone();
    let task_prefix = prompt_value(
        reader,
        writer,
        &format!("Task ID prefix [{}]: ", default_task_prefix),
        &default_task_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_task_prefix),
    )?;
    let default_milestone_prefix = defaults.milestone_prefix.clone();
    let milestone_prefix = prompt_value(
        reader,
        writer,
        &format!("Milestone ID prefix [{}]: ", default_milestone_prefix),
        &default_milestone_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_milestone_prefix),
    )?;
    let default_release_prefix = defaults.release_prefix.clone();
    let release_prefix = prompt_value(
        reader,
        writer,
        &format!("Release ID prefix [{}]: ", default_release_prefix),
        &default_release_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_release_prefix),
    )?;
    let default_backlog_directory = defaults.backlog_directory.clone();
    let backlog_directory = prompt_value(
        reader,
        writer,
        &format!("Backlog directory [{}]: ", default_backlog_directory),
        &default_backlog_directory,
        sanitize_path_string,
        &format!("invalid folder; use {}", default_backlog_directory),
    )?;

    // Decision types.
    let any_decision = active_decisions.values().any(|v| *v);
    let default_decision_directory = defaults.decision_directory.clone();
    let decision_directory = if any_decision {
        prompt_value(
            reader,
            writer,
            &format!("Decision directory [{}]: ", default_decision_directory),
            &default_decision_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", default_decision_directory),
        )?
    } else {
        default_decision_directory.clone()
    };

    for (key, entry) in &defaults.decision_types {
        let activated = active_decisions.get(key).copied().unwrap_or(true);
        if activated {
            let prefix = prompt_value(
                reader,
                writer,
                &format!("  {} ID prefix [{}]: ", key, entry.prefix),
                &entry.prefix,
                sanitize_name,
                &format!("invalid prefix; use {}", entry.prefix),
            )?;
            if let Some(e) = config.decision_types.get_mut(key) {
                e.prefix = prefix;
                e.enabled = true;
            }
        } else if let Some(e) = config.decision_types.get_mut(key) {
            e.enabled = false;
        }
    }

    // Document types.
    let any_doc = active_docs.values().any(|v| *v);
    let default_doc_directory = defaults.doc_directory.clone();
    let doc_directory = if any_doc {
        prompt_value(
            reader,
            writer,
            &format!("Documentation root [{}]: ", default_doc_directory),
            &default_doc_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", default_doc_directory),
        )?
    } else {
        default_doc_directory.clone()
    };

    for (key, entry) in &defaults.document_types {
        let activated = active_docs.get(key).copied().unwrap_or(true);
        if activated {
            let prefix = prompt_value(
                reader,
                writer,
                &format!("  {} ID prefix [{}]: ", key, entry.prefix),
                &entry.prefix,
                sanitize_name,
                &format!("invalid prefix; use {}", entry.prefix),
            )?;
            let directory = prompt_value(
                reader,
                writer,
                &format!("  {} directory [{}]: ", key, entry.directory),
                &entry.directory,
                sanitize_path_string,
                &format!("invalid folder; use {}", entry.directory),
            )?;
            if let Some(e) = config.document_types.get_mut(key) {
                e.prefix = prefix;
                e.directory = directory;
                e.enabled = true;
            }
        } else if let Some(e) = config.document_types.get_mut(key) {
            e.enabled = false;
        }
    }

    config.task_prefix = task_prefix;
    config.milestone_prefix = milestone_prefix;
    config.release_prefix = release_prefix;
    config.backlog_directory = backlog_directory;
    config.decision_directory = decision_directory;
    config.doc_directory = doc_directory;

    Ok(config)
}

fn prompt_value<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    default: &str,
    validate: fn(&str) -> Option<String>,
    hint: &str,
) -> Result<String, AppError> {
    loop {
        write!(writer, "{}", prompt).map_err(|e| AppError::Runtime(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let trimmed = input.trim();
        let candidate = if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        };
        if let Some(v) = validate(&candidate) {
            return Ok(v);
        }
        writeln!(writer, "{}", hint).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
}
