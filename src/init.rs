use crate::config::Config;
use crate::error::AppError;
use crate::sanitize::sanitize_name;
use std::fs;
use std::path::PathBuf;

pub struct InitOptions {
    pub name: String,
    pub no_git: bool,
    pub force: bool,
}

pub fn run_init(project_dir_raw: PathBuf, opts: InitOptions) -> Result<(), AppError> {
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

    // Build effective config (defaults) and set project_name.
    let config = Config {
        project_name: Some(name.clone()),
        ..Config::default()
    };

    // Create directories.
    let backlog_dir = project_dir.join(&config.backlog_directory);
    let _ = fs::create_dir_all(&backlog_dir);
    let _ = fs::create_dir_all(backlog_dir.join("tasks"));
    let _ = fs::create_dir_all(backlog_dir.join("milestones"));
    let _ = fs::create_dir_all(backlog_dir.join("releases"));

    let decision_dir = project_dir.join(&config.decision_directory);
    let _ = fs::create_dir_all(&decision_dir);

    let doc_dir = project_dir.join(&config.doc_directory);
    let _ = fs::create_dir_all(&doc_dir);
    for dt in config.document_types.values() {
        let d = doc_dir.join(&dt.directory);
        let _ = fs::create_dir_all(&d);
        let _ = fs::create_dir_all(d.join("deprecated"));
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
