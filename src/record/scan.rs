//! Read-only enumeration of the files owned by one record family.
//!
//! The diagnostic command must see every active, archived, read/import, and
//! read-only legacy location of a family without writing to any of them. This
//! primitive lists those files and guarantees containment, but it never moves,
//! adopts, or normalizes a record.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::descriptor::RecordDescriptor;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const CANDIDATE_EXTENSIONS: [&str; 2] = ["md", "html"];
const INERT_DIRECTORY: &str = "deprecated";

/// Project-relative path with `/` separators, for stable reporting.
pub fn project_relative(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Every candidate record file of `descriptor` inside the project.
///
/// Directories come from [`RecordDescriptor::resolve_read_directories`]. A
/// missing directory contributes no files; any other read error is an I/O
/// failure (exit 1). Paths are canonicalized through [`Context::ensure_within`],
/// deduplicated, and sorted by project-relative path.
pub fn family_files(
    ctx: &Context,
    cfg: &Config,
    descriptor: &RecordDescriptor,
) -> Result<Vec<PathBuf>, AppError> {
    let mut files: Vec<PathBuf> = Vec::new();
    for directory in descriptor.resolve_read_directories(cfg, &ctx.project_dir) {
        if directory
            .components()
            .any(|component| component.as_os_str() == INERT_DIRECTORY)
        {
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(AppError::Runtime(format!(
                    "cannot read directory {}: {}",
                    directory.display(),
                    error
                )));
            }
        };
        for entry in entries {
            let entry = entry
                .map_err(|error| AppError::Runtime(format!("cannot read directory: {error}")))?;
            let path = entry.path();
            if path
                .components()
                .any(|component| component.as_os_str() == INERT_DIRECTORY)
            {
                continue;
            }
            if !path.is_file() {
                continue;
            }
            let extension = path.extension().and_then(|value| value.to_str());
            if !extension.is_some_and(|value| CANDIDATE_EXTENSIONS.contains(&value)) {
                continue;
            }
            let safe = Context::ensure_within(&ctx.project_dir, &path)?;
            if !files.contains(&safe) {
                files.push(safe);
            }
        }
    }
    files.sort_by(|left, right| {
        project_relative(&ctx.project_dir, left).cmp(&project_relative(&ctx.project_dir, right))
    });
    Ok(files)
}
