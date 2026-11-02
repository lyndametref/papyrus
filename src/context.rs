use crate::error::AppError;
use crate::sanitize;
use std::path::{Path, PathBuf};

pub struct Context {
    pub project_dir: PathBuf,
    pub config_path: PathBuf,
    pub json: bool,
    pub no_color: bool,
}

impl Context {
    /// Select project dir from --dir, PAPYRUS_DIR, cwd.
    ///
    /// Security: each path component is sanitized via `sanitize_path_component`;
    /// values that cannot be normalized are rejected (exit code 4).
    pub fn select_project_dir(dir_opt: Option<PathBuf>) -> Result<PathBuf, AppError> {
        let raw = if let Some(d) = dir_opt {
            d
        } else if let Ok(env) = std::env::var("PAPYRUS_DIR")
            && !env.is_empty()
        {
            PathBuf::from(env)
        } else {
            std::env::current_dir().map_err(|e| AppError::Runtime(e.to_string()))?
        };
        sanitize::sanitize_path(&raw).ok_or_else(|| {
            AppError::Validation("project directory path cannot be normalized".to_string())
        })
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
    ///
    /// Security: resolves symbolic links via `canonicalize` and rejects any
    /// target that escapes the selected project directory (exit code 1).
    pub fn ensure_within(project_dir: &Path, target: &Path) -> Result<PathBuf, AppError> {
        // Canonicalize the project directory itself (resolves symlinks).
        let project_canon = std::fs::canonicalize(project_dir)
            .map_err(|e| AppError::Runtime(format!("project directory: {}", e)))?;
        // Canonicalize if exists; otherwise canonicalize the nearest existing
        // parent and re-attach the remaining (sanitized) components.
        let canon = if target.exists() {
            std::fs::canonicalize(target).map_err(|e| AppError::Runtime(e.to_string()))?
        } else if let Ok(link_target) = std::fs::read_link(target) {
            // Resolve symlinks even when their target does not exist.
            let base = target.parent().unwrap_or(project_dir);
            let resolved = if link_target.is_absolute() {
                link_target
            } else {
                base.join(link_target)
            };
            if resolved.exists() {
                std::fs::canonicalize(&resolved).map_err(|e| AppError::Runtime(e.to_string()))?
            } else {
                let parent = resolved.parent().unwrap_or(project_dir);
                let canon_parent =
                    std::fs::canonicalize(parent).map_err(|e| AppError::Runtime(e.to_string()))?;
                canon_parent.join(resolved.file_name().unwrap_or_default())
            }
        } else {
            let parent = target.parent().unwrap_or(project_dir);
            let canon_parent =
                std::fs::canonicalize(parent).map_err(|e| AppError::Runtime(e.to_string()))?;
            canon_parent.join(target.file_name().unwrap_or_default())
        };
        if !canon.starts_with(&project_canon) {
            return Err(AppError::Runtime(format!(
                "path resolves outside project directory: {}",
                target.display()
            )));
        }
        Ok(canon)
    }
}
