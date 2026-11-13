//! Atomic whole-folder relocation routine (ADR-0010 decision 8, ADR-0013 stage 5).

use crate::context::Context;
use crate::error::AppError;
use std::fs;
use std::path::{Path, PathBuf};

/// Report returned by a successful relocation.
#[derive(Debug, Clone)]
pub struct RelocationReport {
    pub source: PathBuf,
    pub destination: PathBuf,
    /// Number of entries (files and folders) moved.
    pub moved_entries: usize,
    /// Relative paths of the moved entries.
    pub moved: Vec<PathBuf>,
    /// `true` when at least one entry had to fall back to copy-then-remove
    /// because a single rename was impossible (cross-device).
    pub cross_device: bool,
    /// `true` when every entry was moved with a single rename (no fallback).
    pub single_rename_possible: bool,
    /// `true` when the move completed without rolling back.
    pub atomic: bool,
}

/// Relocate the whole content of `source` into `destination`.
pub fn relocate(
    ctx: &Context,
    source: &Path,
    destination: &Path,
) -> Result<RelocationReport, AppError> {
    relocate_impl(ctx, source, destination)
}

/// Alias for [`relocate`].
pub fn relocate_folder(
    ctx: &Context,
    source: &Path,
    destination: &Path,
) -> Result<RelocationReport, AppError> {
    relocate(ctx, source, destination)
}

/// Alias for [`relocate`].
pub fn move_folder(
    ctx: &Context,
    source: &Path,
    destination: &Path,
) -> Result<RelocationReport, AppError> {
    relocate(ctx, source, destination)
}

fn relocate_impl(
    ctx: &Context,
    source_raw: &Path,
    destination_raw: &Path,
) -> Result<RelocationReport, AppError> {
    let source = Context::ensure_within(&ctx.project_dir, source_raw)
        .map_err(|e| AppError::Runtime(format!("source path: {}", e)))?;
    let destination = Context::ensure_within(&ctx.project_dir, destination_raw)
        .map_err(|e| AppError::Runtime(format!("destination path: {}", e)))?;

    if source == destination {
        return Ok(RelocationReport {
            source,
            destination,
            moved_entries: 0,
            moved: Vec::new(),
            cross_device: false,
            single_rename_possible: true,
            atomic: true,
        });
    }

    if !source.exists() {
        if destination.exists() {
            if !destination.is_dir() {
                return Err(AppError::Validation(format!(
                    "destination is not a directory: {}",
                    destination.display()
                )));
            }
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| AppError::Validation(e.to_string()))?;
            }
            fs::create_dir_all(&destination).map_err(|e| AppError::Validation(e.to_string()))?;
        }
        return Ok(RelocationReport {
            source,
            destination,
            moved_entries: 0,
            moved: Vec::new(),
            cross_device: false,
            single_rename_possible: true,
            atomic: true,
        });
    }

    if !source.is_dir() {
        return Err(AppError::Validation(format!(
            "source is not a directory: {}",
            source.display()
        )));
    }

    let created_destination = if destination.exists() {
        if !destination.is_dir() {
            return Err(AppError::Validation(format!(
                "destination is not a directory: {}",
                destination.display()
            )));
        }
        false
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::Validation(e.to_string()))?;
        }
        fs::create_dir_all(&destination).map_err(|e| AppError::Validation(e.to_string()))?;
        true
    };

    if let Err(e) = preflight(&source, &destination) {
        if created_destination {
            let _ = fs::remove_dir_all(&destination);
        }
        return Err(e);
    }

    let mut moved_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut report = RelocationReport {
        source: source.clone(),
        destination: destination.clone(),
        moved_entries: 0,
        moved: Vec::new(),
        cross_device: false,
        single_rename_possible: true,
        atomic: true,
    };

    if let Err(e) = move_dir_contents(&source, &destination, &mut moved_pairs, &mut report) {
        rollback(&moved_pairs);
        if created_destination {
            let _ = fs::remove_dir_all(&destination);
            let _ = fs::create_dir_all(&destination);
        }
        let _ = fs::create_dir_all(&source);
        report.atomic = false;
        return Err(e);
    }

    let _ = fs::remove_dir(&source);

    report.single_rename_possible = !report.cross_device;
    report.atomic = true;
    Ok(report)
}

fn preflight(source_root: &Path, destination_root: &Path) -> Result<(), AppError> {
    let read_dir = fs::read_dir(source_root).map_err(|e| AppError::Validation(e.to_string()))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| AppError::Validation(e.to_string()))?;
        let src = entry.path();
        let rel = src
            .strip_prefix(source_root)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let dst = destination_root.join(rel);
        if dst.exists() {
            return Err(AppError::Validation(format!(
                "destination entry conflict: {}",
                rel.display()
            )));
        }
        let ft = entry
            .file_type()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        if ft.is_dir() {
            let _ = fs::read_dir(&src).map_err(|e| AppError::Validation(e.to_string()))?;
            preflight(&src, destination_root)?;
        } else {
            let _ = fs::metadata(&src).map_err(|e| AppError::Validation(e.to_string()))?;
        }
    }
    Ok(())
}

fn move_dir_contents(
    source: &Path,
    destination: &Path,
    moved_pairs: &mut Vec<(PathBuf, PathBuf)>,
    report: &mut RelocationReport,
) -> Result<(), AppError> {
    let read_dir = fs::read_dir(source).map_err(|e| AppError::Validation(e.to_string()))?;
    let entries: Vec<fs::DirEntry> = read_dir.filter_map(|e| e.ok()).collect();
    for entry in entries {
        let src = entry.path();
        let name = entry.file_name();
        let dst = destination.join(&name);
        let ft = entry
            .file_type()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        if ft.is_dir() && !ft.is_symlink() {
            fs::create_dir_all(&dst).map_err(|e| AppError::Validation(e.to_string()))?;
            move_dir_contents(&src, &dst, moved_pairs, report)?;
            let _ = fs::remove_dir(&src);
        } else {
            move_single_file(&src, &dst, moved_pairs, report)?;
        }
    }
    Ok(())
}

fn move_single_file(
    src: &Path,
    dst: &Path,
    moved_pairs: &mut Vec<(PathBuf, PathBuf)>,
    report: &mut RelocationReport,
) -> Result<(), AppError> {
    match fs::rename(src, dst) {
        Ok(()) => {}
        Err(e) if is_exdev(&e) => {
            fs::copy(src, dst).map_err(|e| AppError::Validation(e.to_string()))?;
            fs::remove_file(src).map_err(|e| AppError::Validation(e.to_string()))?;
            report.cross_device = true;
        }
        Err(e) => return Err(AppError::Validation(e.to_string())),
    }

    moved_pairs.push((src.to_path_buf(), dst.to_path_buf()));
    let rel = dst
        .strip_prefix(&report.destination)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| dst.to_path_buf());
    report.moved.push(rel);
    report.moved_entries += 1;
    Ok(())
}

fn is_exdev(e: &std::io::Error) -> bool {
    if e.kind() == std::io::ErrorKind::CrossesDevices {
        return true;
    }
    matches!(e.raw_os_error(), Some(18))
}

fn rollback(pairs: &[(PathBuf, PathBuf)]) {
    for (src, dst) in pairs.iter().rev() {
        if let Some(parent) = src.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::rename(dst, src).is_ok() {
            continue;
        }
        if fs::copy(dst, src).is_ok() {
            let _ = fs::remove_file(dst);
        }
    }
}
