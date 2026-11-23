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

#[derive(Debug)]
enum MovedEntry {
    File { src: PathBuf, dst: PathBuf },
    Dir { src: PathBuf, dst: PathBuf },
}

#[cfg(test)]
thread_local! {
    static TEST_FAIL_AFTER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TEST_MOVED_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TEST_FORCE_CROSS_DEVICE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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

    let mut moved_entries_log: Vec<MovedEntry> = Vec::new();
    let mut report = RelocationReport {
        source: source.clone(),
        destination: destination.clone(),
        moved_entries: 0,
        moved: Vec::new(),
        cross_device: false,
        single_rename_possible: true,
        atomic: true,
    };

    if let Err(e) = move_dir_contents(&source, &destination, &mut moved_entries_log, &mut report) {
        rollback(&moved_entries_log);
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

fn preflight(source_dir: &Path, destination_dir: &Path) -> Result<(), AppError> {
    let read_dir = fs::read_dir(source_dir).map_err(|e| AppError::Validation(e.to_string()))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| AppError::Validation(e.to_string()))?;
        let src = entry.path();
        let name = entry.file_name();
        let dst = destination_dir.join(&name);
        if dst.exists() {
            return Err(AppError::Validation(format!(
                "destination entry conflict: {}",
                name.to_string_lossy()
            )));
        }
        let ft = entry
            .file_type()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        if ft.is_dir() && !ft.is_symlink() {
            let _ = fs::read_dir(&src).map_err(|e| AppError::Validation(e.to_string()))?;
            preflight(&src, &dst)?;
        } else {
            let _ = fs::metadata(&src).map_err(|e| AppError::Validation(e.to_string()))?;
        }
    }
    Ok(())
}

fn move_dir_contents(
    source: &Path,
    destination: &Path,
    moved_log: &mut Vec<MovedEntry>,
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
            // Record the directory move before recursing so a later failure
            // can remove the created destination directory and restore the
            // (possibly emptied) source directory.
            let rel = dst
                .strip_prefix(&report.destination)
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|_| dst.clone());
            report.moved.push(rel);
            report.moved_entries += 1;
            moved_log.push(MovedEntry::Dir {
                src: src.clone(),
                dst: dst.clone(),
            });

            move_dir_contents(&src, &dst, moved_log, report)?;
            let _ = fs::remove_dir(&src);
        } else {
            move_single_file(&src, &dst, moved_log, report)?;
        }
        {
            #[cfg(test)]
            {
                let threshold = TEST_FAIL_AFTER.with(|c| c.get());
                if threshold > 0 {
                    let n = TEST_MOVED_COUNT.with(|c| {
                        let v = c.get() + 1;
                        c.set(v);
                        v
                    });
                    if n >= threshold {
                        return Err(AppError::Validation("injected partial failure".to_string()));
                    }
                }
            }
        }
    }
    Ok(())
}

fn move_single_file(
    src: &Path,
    dst: &Path,
    moved_log: &mut Vec<MovedEntry>,
    report: &mut RelocationReport,
) -> Result<(), AppError> {
    #[cfg(test)]
    let force_cross = TEST_FORCE_CROSS_DEVICE.with(|c| c.get());
    #[cfg(not(test))]
    let force_cross = false;

    let rename_result = if force_cross {
        Err(std::io::Error::new(
            std::io::ErrorKind::CrossesDevices,
            "forced",
        ))
    } else {
        fs::rename(src, dst)
    };

    match rename_result {
        Ok(()) => {}
        Err(e) if is_exdev(&e) => {
            fs::copy(src, dst).map_err(|e| AppError::Validation(e.to_string()))?;
            fs::remove_file(src).map_err(|e| AppError::Validation(e.to_string()))?;
            report.cross_device = true;
        }
        Err(e) => return Err(AppError::Validation(e.to_string())),
    }

    moved_log.push(MovedEntry::File {
        src: src.to_path_buf(),
        dst: dst.to_path_buf(),
    });
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

fn rollback(entries: &[MovedEntry]) {
    for entry in entries.iter().rev() {
        match entry {
            MovedEntry::File { src, dst } => {
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
            MovedEntry::Dir { src, dst } => {
                let _ = fs::create_dir_all(src);
                let _ = fs::remove_dir(dst);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use std::fs;

    fn make_ctx(project: &std::path::Path) -> Context {
        Context {
            project_dir: project.to_path_buf(),
            config_path: {
                let p = project.join("papyrus.config.yml");
                if !p.exists() {
                    let _ = std::fs::write(&p, "project_name: Test\n");
                }
                p
            },
            json: false,
            no_color: false,
        }
    }

    #[test]
    fn partial_failure_rolls_back_entries_and_dirs() {
        let base = std::env::temp_dir().join("papyrus-reloc-unit-partial");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let proj = base.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::create_dir_all(proj.join("src")).unwrap();
        fs::write(proj.join("src/a.txt"), "a").unwrap();
        fs::create_dir_all(proj.join("src/sub")).unwrap();
        fs::write(proj.join("src/sub/b.txt"), "b").unwrap();
        fs::create_dir_all(proj.join("dst")).unwrap();

        let ctx = make_ctx(&proj);
        // fail after moving the first entry
        TEST_FAIL_AFTER.with(|c| c.set(1));
        TEST_MOVED_COUNT.with(|c| c.set(0));

        let source = proj.join("src");
        let destination = proj.join("dst");
        let res = relocate(&ctx, &source, &destination);
        assert!(res.is_err());

        // rollback restored source files/dirs
        assert!(proj.join("src").exists(), "source directory restored");
        assert!(proj.join("src/a.txt").exists());
        assert!(
            proj.join("src/sub").exists(),
            "source subdirectory restored"
        );
        assert!(proj.join("src/sub/b.txt").exists());
        // destination should not retain moved file and remains present
        assert!(proj.join("dst").exists(), "destination directory restored");
        assert!(!proj.join("dst/a.txt").exists());

        TEST_FAIL_AFTER.with(|c| c.set(0));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn empty_source_dir_removed_on_success() {
        let base = std::env::temp_dir().join("papyrus-reloc-unit-empty");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let proj = base.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::create_dir_all(proj.join("src/emptydir")).unwrap();
        fs::create_dir_all(proj.join("dst")).unwrap();

        let ctx = make_ctx(&proj);
        TEST_FAIL_AFTER.with(|c| c.set(0));
        TEST_MOVED_COUNT.with(|c| c.set(0));
        let res = relocate(&ctx, &proj.join("src"), &proj.join("dst"));
        assert!(res.is_ok());
        let report = res.unwrap();
        // directory entry counted
        assert!(report.moved_entries >= 1);
        assert!(
            !proj.join("src").exists() || fs::read_dir(proj.join("src")).unwrap().next().is_none()
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cross_device_fallback_reports_cross_device() {
        let base = std::env::temp_dir().join("papyrus-reloc-unit-xdev");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let proj = base.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::create_dir_all(proj.join("src")).unwrap();
        fs::write(proj.join("src/a.txt"), "a").unwrap();
        fs::create_dir_all(proj.join("dst")).unwrap();

        TEST_FORCE_CROSS_DEVICE.with(|c| c.set(true));
        TEST_FAIL_AFTER.with(|c| c.set(0));
        TEST_MOVED_COUNT.with(|c| c.set(0));

        let ctx = make_ctx(&proj);
        let res = relocate(&ctx, &proj.join("src"), &proj.join("dst"));
        assert!(res.is_ok());
        let report = res.unwrap();
        assert!(report.cross_device);
        assert!(!report.single_rename_possible);
        assert!(proj.join("dst/a.txt").exists());
        assert!(
            !proj.join("src").exists() || fs::read_dir(proj.join("src")).unwrap().next().is_none()
        );

        TEST_FORCE_CROSS_DEVICE.with(|c| c.set(false));
        let _ = fs::remove_dir_all(&base);
    }
}
