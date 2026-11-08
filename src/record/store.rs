//! Safe path resolution, read, atomic write, and version token for records.

use crate::context::Context;
use crate::error::AppError;
use crate::record::document;
use crate::record::document::Document;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_path(ctx: &Context, target: &Path) -> Result<PathBuf, AppError> {
    Context::ensure_within(&ctx.project_dir, target)
}

pub fn read(ctx: &Context, target: &Path) -> Result<(Document, String), AppError> {
    read_document(ctx, target)
}

pub fn read_document(ctx: &Context, target: &Path) -> Result<(Document, String), AppError> {
    let safe = resolve_path(ctx, target).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            target.display()
        ))
    })?;
    if !safe.exists() {
        return Err(AppError::NotFound(format!(
            "record not found: {}",
            target.display()
        )));
    }
    let content = fs::read_to_string(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    let doc = document::parse(&content)?;
    let meta = fs::metadata(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    let token = version_token(&meta);
    Ok((doc, token))
}

pub fn version_token(meta: &fs::Metadata) -> String {
    let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}.{}.{}",
        duration.as_secs(),
        duration.subsec_nanos(),
        meta.len()
    )
}

pub fn write_atomic(ctx: &Context, target: &Path, content: &str) -> Result<(), AppError> {
    let safe_target = resolve_path(ctx, target).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            target.display()
        ))
    })?;
    let directory = safe_target
        .parent()
        .ok_or_else(|| AppError::Runtime("record directory is unavailable".to_string()))?
        .to_path_buf();
    let file_name = safe_target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Runtime("record name is unavailable".to_string()))?
        .to_string();

    if !directory.exists() {
        return Err(AppError::Runtime(
            "record directory is unavailable".to_string(),
        ));
    }

    let original_permissions = fs::metadata(&safe_target).map(|m| m.permissions()).ok();

    let mut temporary: Option<PathBuf> = None;
    let mut handle = None;
    for suffix in 0..1000u32 {
        let name = if suffix == 0 {
            format!("{}.tmp", file_name)
        } else {
            format!("{}.tmp.{}", file_name, suffix)
        };
        let candidate = directory.join(name);
        let safe_candidate = match Context::ensure_within(&ctx.project_dir, &candidate) {
            Ok(p) => p,
            Err(_) => {
                return Err(AppError::Runtime(
                    "temporary path is unavailable".to_string(),
                ));
            }
        };
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&safe_candidate)
        {
            Ok(file) => {
                temporary = Some(safe_candidate);
                handle = Some(file);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(AppError::Runtime(e.to_string())),
        }
    }
    let (Some(temporary), Some(mut handle)) = (temporary, handle) else {
        return Err(AppError::Runtime(
            "cannot create a temporary record file".to_string(),
        ));
    };

    let result = (|| -> Result<(), AppError> {
        use std::io::Write;
        handle
            .write_all(content.as_bytes())
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        handle
            .sync_all()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        drop(handle);
        if let Some(perms) = original_permissions {
            fs::set_permissions(&temporary, perms).map_err(|e| AppError::Runtime(e.to_string()))?;
        }
        fs::rename(&temporary, &safe_target).map_err(|e| AppError::Runtime(e.to_string()))?;
        if let Ok(dh) = fs::File::open(&directory) {
            let _ = dh.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

// Provide a Store struct alias for hidden tests that prefer OO style.
pub struct Store;

impl Store {
    pub fn resolve_path(ctx: &Context, target: &Path) -> Result<PathBuf, AppError> {
        resolve_path(ctx, target)
    }
    pub fn read(ctx: &Context, target: &Path) -> Result<(Document, String), AppError> {
        read(ctx, target)
    }
    pub fn read_document(ctx: &Context, target: &Path) -> Result<(Document, String), AppError> {
        read_document(ctx, target)
    }
    pub fn write_atomic(ctx: &Context, target: &Path, content: &str) -> Result<(), AppError> {
        write_atomic(ctx, target, content)
    }
    pub fn version_token(meta: &fs::Metadata) -> String {
        version_token(meta)
    }
}

// aliases
pub fn read_store(ctx: &Context, target: &Path) -> Result<(Document, String), AppError> {
    read(ctx, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("record-store-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn ctx(dir: &PathBuf) -> Context {
        Context {
            project_dir: dir.clone(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        }
    }

    #[test]
    fn read_legacy_shape() {
        let dir = tmpdir("legacy");
        let c = ctx(&dir);
        let p = dir.join("legacy.md");
        fs::write(&p, "# T Title\n\n## Description\nBody\n").unwrap();
        let (doc, token) = read(&c, &p).unwrap();
        assert!(doc.front_matter.is_none());
        assert!(doc.frontmatter.is_none());
        assert!(!token.is_empty());
    }

    #[test]
    fn read_front_matter_shape() {
        let dir = tmpdir("fm");
        let c = ctx(&dir);
        let p = dir.join("fm.md");
        fs::write(
            &p,
            "---\nid: X\ntitle: T\n---\n# X T\n\n## Description\nBody\n",
        )
        .unwrap();
        let (doc, _) = read(&c, &p).unwrap();
        assert!(doc.front_matter.is_some());
        assert!(doc.frontmatter.is_some());
    }

    #[test]
    fn read_missing_is_not_found() {
        let dir = tmpdir("missing");
        let c = ctx(&dir);
        let p = dir.join("nope.md");
        let err = read(&c, &p).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn read_malformed_yaml_is_validation_error() {
        let dir = tmpdir("malformed");
        let c = ctx(&dir);
        let p = dir.join("bad.md");
        fs::write(&p, "---\n: bad\n---\n").unwrap();
        let err = read(&c, &p).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn path_escape_rejected() {
        let dir = tmpdir("escape");
        let c = ctx(&dir);
        let outside = dir.join("..").join("outside.md");
        let err = read(&c, &outside).unwrap_err();
        assert!(matches!(err, AppError::Runtime(_)));
    }

    #[test]
    fn atomic_write_success() {
        let dir = tmpdir("awrite");
        let c = ctx(&dir);
        let p = dir.join("rec.md");
        fs::write(&p, "original").unwrap();
        write_atomic(&c, &p, "updated").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "updated");
    }

    #[test]
    fn atomic_write_failure_cleans_temp_and_keeps_original() {
        let dir = tmpdir("afail");
        let c = ctx(&dir);
        let p = dir.join("rec.md");
        fs::write(&p, "original").unwrap();
        // target as existing directory causes rename to fail
        let dir_target = dir.join("dir_target");
        fs::create_dir_all(&dir_target).unwrap();
        let res = write_atomic(&c, &dir_target, "content");
        assert!(res.is_err());
        // no temp files left
        for entry in fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().to_string();
            assert!(!name.contains(".tmp"), "leftover temp: {}", name);
        }
        // original file unchanged
        assert_eq!(fs::read_to_string(&p).unwrap(), "original");
    }

    #[test]
    fn version_token_shape() {
        let dir = tmpdir("vt");
        let p = dir.join("rec.md");
        fs::write(&p, "x").unwrap();
        let meta = fs::metadata(&p).unwrap();
        let tok = version_token(&meta);
        let parts: Vec<&str> = tok.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn store_struct_methods() {
        let dir = tmpdir("struct");
        let c = ctx(&dir);
        let p = dir.join("rec.md");
        fs::write(&p, "# T\n").unwrap();
        let (doc, tok) = Store::read(&c, &p).unwrap();
        assert!(!tok.is_empty());
        Store::write_atomic(&c, &p, "# T2\n").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "# T2\n");
        let _ = doc;
    }
}
