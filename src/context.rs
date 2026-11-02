use crate::error::AppError;
use std::path::{Path, PathBuf};

pub struct Context {
    pub project_dir: PathBuf,
    pub config_path: PathBuf,
    pub json: bool,
    pub no_color: bool,
}

impl Context {
    /// Select project dir from --dir, PAPYRUS_DIR, cwd.
    pub fn select_project_dir(dir_opt: Option<PathBuf>) -> Result<PathBuf, AppError> {
        if let Some(d) = dir_opt {
            return Ok(d);
        }
        if let Ok(env) = std::env::var("PAPYRUS_DIR")
            && !env.is_empty()
        {
            return Ok(PathBuf::from(env));
        }
        std::env::current_dir().map_err(|e| AppError::Runtime(e.to_string()))
    }

    /// Canonicalize project dir (must exist).
    pub fn canonical_project_dir(raw: &Path) -> Result<PathBuf, AppError> {
        std::fs::canonicalize(raw)
            .map_err(|e| AppError::Runtime(format!("project directory: {}", e)))
    }

    pub fn select_config_path(config_opt: Option<PathBuf>, project_dir: &Path) -> PathBuf {
        if let Some(c) = config_opt {
            return c;
        }
        if let Ok(env) = std::env::var("PAPYRUS_CONFIG")
            && !env.is_empty()
        {
            return PathBuf::from(env);
        }
        project_dir.join("papyrus.config.yml")
    }

    /// Ensure a path stays within the canonical project dir.
    pub fn ensure_within(project_dir: &Path, target: &Path) -> Result<PathBuf, AppError> {
        // Canonicalize if exists; otherwise canonicalize parent.
        let canon = if target.exists() {
            std::fs::canonicalize(target).map_err(|e| AppError::Runtime(e.to_string()))?
        } else {
            let parent = target.parent().unwrap_or(project_dir);
            let canon_parent =
                std::fs::canonicalize(parent).map_err(|e| AppError::Runtime(e.to_string()))?;
            canon_parent.join(target.file_name().unwrap_or_default())
        };
        if !canon.starts_with(project_dir) {
            return Err(AppError::Runtime(format!(
                "path resolves outside project directory: {}",
                target.display()
            )));
        }
        Ok(canon)
    }
}
