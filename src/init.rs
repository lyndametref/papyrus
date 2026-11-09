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

    // Non-empty check.
    let is_empty = fs::read_dir(&project_dir)
        .map_err(|e| AppError::Runtime(e.to_string()))?
        .next()
        .is_none();
    if !is_empty && !opts.force {
        return Err(AppError::Validation(
            "target directory is not empty; use --force to initialize anyway".to_string(),
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

    // Create directories.
    let backlog_dir = project_dir.join(&config.backlog_directory);
    let _ = fs::create_dir_all(&backlog_dir);
    let _ = fs::create_dir_all(backlog_dir.join("tasks"));
    let _ = fs::create_dir_all(backlog_dir.join("milestones"));
    let _ = fs::create_dir_all(backlog_dir.join("releases"));

    let any_decision = config.decision_types.values().any(|v| v.enabled);
    if any_decision {
        let decision_dir = project_dir.join(&config.decision_directory);
        let _ = fs::create_dir_all(&decision_dir);
    }

    let doc_dir = project_dir.join(&config.doc_directory);
    let _ = fs::create_dir_all(&doc_dir);
    for dt in config.document_types.values() {
        if dt.enabled {
            let d = doc_dir.join(&dt.directory);
            let _ = fs::create_dir_all(&d);
            let _ = fs::create_dir_all(d.join("deprecated"));
        }
    }

    // Write config file if absent.
    let config_path = project_dir.join("papyrus.config.yml");
    if !config_path.exists() {
        let yaml = yaml_serde::to_string(&config).map_err(|e| AppError::Runtime(e.to_string()))?;
        fs::write(&config_path, yaml.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;
    }

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

    let _ = writeln!(
        writer,
        "Core artifacts (always activated): task, milestone, release"
    );
    let task_prefix = prompt_value(
        reader,
        writer,
        "Task ID prefix [TASK]: ",
        &defaults.task_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", defaults.task_prefix),
    )?;
    let milestone_prefix = prompt_value(
        reader,
        writer,
        "Milestone ID prefix [MS]: ",
        &defaults.milestone_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", defaults.milestone_prefix),
    )?;
    let release_prefix = prompt_value(
        reader,
        writer,
        "Release ID prefix [REL]: ",
        &defaults.release_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", defaults.release_prefix),
    )?;
    let backlog_directory = prompt_value(
        reader,
        writer,
        "Backlog directory [backlog/]: ",
        &defaults.backlog_directory,
        sanitize_path_string,
        &format!("invalid folder; use {}", defaults.backlog_directory),
    )?;

    let _ = writeln!(writer, "Decision types:");
    let mut decision_activated: BTreeMap<String, bool> = BTreeMap::new();
    for (key, entry) in &defaults.decision_types {
        let _ = writeln!(writer, "  {} (default prefix: {})", key, entry.prefix);
        let activated = prompt_activation(reader, writer, "  Activate? [Y/n]: ")?;
        decision_activated.insert(key.clone(), activated);
    }
    let any_decision = decision_activated.values().any(|v| *v);
    let decision_directory = if any_decision {
        prompt_value(
            reader,
            writer,
            "Decision directory [decisions/]: ",
            &defaults.decision_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", defaults.decision_directory),
        )?
    } else {
        defaults.decision_directory.clone()
    };

    for (key, entry) in &defaults.decision_types {
        let activated = decision_activated.get(key).copied().unwrap_or(true);
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

    let _ = writeln!(writer, "Document types:");
    let mut doc_activated: BTreeMap<String, bool> = BTreeMap::new();
    for (key, entry) in &defaults.document_types {
        let _ = writeln!(
            writer,
            "  {} (default prefix: {}, directory: {})",
            key, entry.prefix, entry.directory
        );
        let activated = prompt_activation(reader, writer, "  Activate? [Y/n]: ")?;
        doc_activated.insert(key.clone(), activated);
    }
    let any_doc = doc_activated.values().any(|v| *v);
    let doc_directory = if any_doc {
        prompt_value(
            reader,
            writer,
            "Documentation root [docs/]: ",
            &defaults.doc_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", defaults.doc_directory),
        )?
    } else {
        defaults.doc_directory.clone()
    };

    for (key, entry) in &defaults.document_types {
        let activated = doc_activated.get(key).copied().unwrap_or(true);
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

fn prompt_activation<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
) -> Result<bool, AppError> {
    loop {
        write!(writer, "{}", prompt).map_err(|e| AppError::Runtime(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let ans = input.trim().to_lowercase();
        if ans.is_empty() || ans == "y" || ans == "yes" {
            return Ok(true);
        }
        if ans == "n" || ans == "no" {
            return Ok(false);
        }
        writeln!(writer, "invalid choice; enter y, yes, n, or no")
            .map_err(|e| AppError::Runtime(e.to_string()))?;
    }
}
