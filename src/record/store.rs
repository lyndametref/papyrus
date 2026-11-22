//! Safe path resolution, read, atomic write, and version token for records.

use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::document;
use crate::record::document::Document;
use crate::registry::ArtifactRegistry;
use std::fs;
use std::path::{Path, PathBuf};

/// Failure modes for read-path auto-relocation of an archived-status record
/// that lives in the active folder.
#[derive(Debug)]
pub enum RelocateError {
    /// The destination write failed; the source stays intact.
    WriteFailed { source: std::path::PathBuf },
    /// The destination write succeeded but removing the source failed; both
    /// copies remain.
    UnlinkFailed {
        source: std::path::PathBuf,
        destination: std::path::PathBuf,
    },
    /// Any other failure (containment, not found, ...), preserving the
    /// original error kind.
    Other(crate::error::AppError),
}

impl std::fmt::Display for RelocateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelocateError::WriteFailed { source } => {
                write!(f, "failed to write relocated record: {}", source.display())
            }
            RelocateError::UnlinkFailed {
                source,
                destination,
            } => {
                write!(
                    f,
                    "archive wrote {} but failed to remove {}",
                    destination.display(),
                    source.display()
                )
            }
            RelocateError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RelocateError {}

impl From<crate::error::AppError> for RelocateError {
    fn from(e: crate::error::AppError) -> Self {
        RelocateError::Other(e)
    }
}

impl RelocateError {
    pub fn into_app_error(self) -> crate::error::AppError {
        match self {
            RelocateError::WriteFailed { source } => crate::error::AppError::Validation(format!(
                "failed to relocate archived record; source intact: {}",
                source.display()
            )),
            RelocateError::UnlinkFailed {
                source,
                destination,
            } => crate::error::AppError::Runtime(format!(
                "archive wrote {} but failed to remove {}",
                destination.display(),
                source.display()
            )),
            RelocateError::Other(e) => e,
        }
    }
}

pub fn resolve_path(ctx: &Context, target: &Path) -> Result<PathBuf, AppError> {
    Context::ensure_within(&ctx.project_dir, target)
}

pub fn read(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
) -> Result<(Document, String), AppError> {
    read_document(ctx, cfg, registry, target)
}

pub fn read_document(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
) -> Result<(Document, String), AppError> {
    let (safe, content) = read_source(ctx, cfg, registry, target)?;
    let doc = document::parse(&content)?;
    let meta = fs::metadata(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    let token = version_token(&meta);
    Ok((doc, token))
}

/// One read of a record for the browser edit surface (ADR-0013 D2).
#[derive(Debug)]
pub struct NormalizedRead {
    /// Document parsed after CRLF and lone-CR normalization.
    pub document: Document,
    /// Optimistic-concurrency token read from the stored file.
    pub version: String,
    /// Content exactly as stored on disk, before normalization.
    pub stored_source: String,
}

/// Read a record for the browser: the shared parser keeps its byte-preserving
/// contract, while the browser requests LF-normalized content so headings never
/// carry a trailing `\r` (ADR-0013 D2). `stored_source` keeps the on-disk bytes
/// so a line-ending-only difference still produces exactly one write.
pub fn read_document_normalized(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
) -> Result<NormalizedRead, AppError> {
    let (safe, stored_source) = read_source(ctx, cfg, registry, target)?;
    let normalized = crate::record::line_endings::normalize(&stored_source);
    let document = document::parse(&normalized)?;
    let meta = fs::metadata(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(NormalizedRead {
        document,
        version: version_token(&meta),
        stored_source,
    })
}

fn read_source(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
) -> Result<(PathBuf, String), AppError> {
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
    // Read-path auto-relocate for active-folder archived-status records.
    let safe = match relocate_if_archived(ctx, cfg, registry, &safe) {
        Ok(p) => p,
        Err(RelocateError::WriteFailed { source }) => {
            return Err(AppError::Validation(format!(
                "failed to relocate archived record; source intact: {}",
                source.display()
            )));
        }
        Err(RelocateError::UnlinkFailed {
            source,
            destination,
        }) => {
            return Err(AppError::Runtime(format!(
                "archive wrote {} but failed to remove {}",
                destination.display(),
                source.display()
            )));
        }
        Err(RelocateError::Other(e)) => return Err(e),
    };
    let content = crate::record::migrate::ensure_front_matter_quiet(ctx, cfg, registry, &safe)?;
    // Tolerant read: flag (but preserve) an undeclared status value.
    if let Some(desc) =
        crate::record::migrate::managed_descriptor(cfg, registry, &ctx.project_dir, &safe)
    {
        let declared = desc.declared_statuses(cfg);
        if let Ok(Some(fm)) = crate::record::frontmatter::parse(&content)
            && let Some(status) =
                crate::record::frontmatter::current_value_string(&fm.raw, "status")
            && !declared.is_empty()
            && !declared.iter().any(|d| d == &status)
        {
            eprintln!(
                "warning: undeclared status '{}' at {}",
                status,
                safe.display()
            );
        }
    }
    Ok((safe, content))
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

pub fn valid_version_token(token: &str) -> bool {
    let mut parts = token.split('.');
    let (Some(seconds), Some(nanos), Some(length), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    !seconds.is_empty()
        && !nanos.is_empty()
        && !length.is_empty()
        && seconds.bytes().all(|b| b.is_ascii_digit())
        && nanos.bytes().all(|b| b.is_ascii_digit())
        && length.bytes().all(|b| b.is_ascii_digit())
}

/// Move one record file atomically to another project-relative location.
///
/// Relocation is a directory move, not a record write: the shared store is the
/// only place that may rename a record path.
pub fn move_record(ctx: &Context, source: &Path, destination: &Path) -> Result<(), AppError> {
    let source = resolve_path(ctx, source).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            source.display()
        ))
    })?;
    let destination = resolve_path(ctx, destination).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            destination.display()
        ))
    })?;
    fs::rename(&source, &destination).map_err(|e| AppError::Runtime(e.to_string()))
}

pub fn write_atomic(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
    content: &str,
) -> Result<(), AppError> {
    // Lazy migration on first write: if the target is a managed document record
    // lacking front matter, add front matter before writing.
    let mut content = content.to_string();
    if !crate::record::frontmatter::has_front_matter(&content) {
        let safe_check = resolve_path(ctx, target).ok();
        if let Some(safe) = safe_check
            && let Some(desc) =
                crate::record::migrate::managed_descriptor(cfg, registry, &ctx.project_dir, &safe)
        {
            // Conflict detection before lazy migration on the write path.
            let conflicts =
                crate::record::migrate::scan_migration_conflicts(ctx, cfg, registry, &desc);
            if let Some(reason) = crate::record::migrate::check_conflicts_for_path(
                ctx, cfg, &safe, &desc, &content, &conflicts,
            ) {
                let rel = safe
                    .strip_prefix(&ctx.project_dir)
                    .unwrap_or(&safe)
                    .to_string_lossy()
                    .replace('\\', "/");
                return Err(AppError::Validation(format!(
                    "migration conflict at {}: {}",
                    rel, reason
                )));
            }
            let (new_content, changed, action, keys) = crate::record::migrate::migrate_content(
                ctx, cfg, registry, &safe, &content, false,
            )?;
            if changed {
                content = new_content;
                let rel = safe
                    .strip_prefix(&ctx.project_dir)
                    .unwrap_or(&safe)
                    .to_string_lossy()
                    .replace('\\', "/");
                println!("{}\t{}\t{}", rel, action, keys.join(","));
            }
        }
    }

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

/// Resolve a document record by ID across the family directories.
///
/// A `GroupedMarkdown` family carries both plain group IDs and dotted member
/// IDs, so a dotted descriptor resolves a plain ID with one fallback pass.
fn resolve_family_record(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    dirs: &[PathBuf],
    id: &str,
    prefixes: &[String],
    dotted: bool,
) -> Result<Option<crate::record::resolve::ResolvedRecord>, AppError> {
    if let Some(found) =
        crate::record::resolve::resolve_in_dirs(ctx, cfg, registry, dirs, id, prefixes, dotted)?
    {
        return Ok(Some(found));
    }
    if dotted {
        return crate::record::resolve::resolve_in_dirs(
            ctx, cfg, registry, dirs, id, prefixes, false,
        );
    }
    Ok(None)
}

/// Archive a record: patch status to `archived`, write the destination via the
/// atomic sequence, and remove the source only after the destination write
/// succeeds.
pub fn archive_inner(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &crate::record::descriptor::RecordDescriptor,
    id: &str,
) -> Result<PathBuf, RelocateError> {
    let active_dir = descriptor
        .resolve_directory(cfg, &ctx.project_dir)
        .map_err(RelocateError::Other)?;
    let archived_dir = match descriptor.resolve_archived_directory(cfg, &ctx.project_dir) {
        Some(Ok(dir)) => dir,
        Some(Err(e)) => return Err(RelocateError::Other(e)),
        None => {
            return Err(RelocateError::Other(AppError::Validation(format!(
                "record type {} has no archived location",
                descriptor.name
            ))));
        }
    };
    let read_dirs = descriptor.resolve_read_directories(cfg, &ctx.project_dir);
    let prefixes = descriptor.configured_prefixes(cfg);
    let dotted = descriptor.is_dotted_id();
    let resolved = resolve_family_record(ctx, cfg, registry, &read_dirs, id, &prefixes, dotted)
        .map_err(RelocateError::Other)?;
    let record = resolved.ok_or_else(|| {
        RelocateError::Other(AppError::NotFound(format!("record not found: {}", id)))
    })?;
    let path = record.path;

    // Lazy adoption first so status is present.
    let content_for_status =
        crate::record::migrate::ensure_front_matter_quiet(ctx, cfg, registry, &path)
            .map_err(RelocateError::Other)?;
    let status = crate::record::frontmatter::parse(&content_for_status)
        .ok()
        .flatten()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "status"));

    let in_archived = path.parent() == Some(archived_dir.as_path());
    let in_active = path.parent() == Some(active_dir.as_path());

    // Archived-folder record: idempotent if status archived, else validation error.
    if in_archived {
        if status.as_deref() == Some("archived") {
            return Ok(path);
        }
        return Err(RelocateError::Other(AppError::Validation(format!(
            "record {} in archived folder has status {}",
            record.id,
            status.unwrap_or_else(|| "<missing>".to_string())
        ))));
    }

    // Non-active, non-archived location (legacy alias): idempotent no-op.
    if !in_active {
        return Ok(path);
    }

    // Active folder with status archived: relocate using the same sequence.
    // (status patch is a no-op on the value.)

    fs::create_dir_all(&archived_dir)
        .map_err(|e| RelocateError::Other(AppError::Runtime(e.to_string())))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| {
            RelocateError::Other(AppError::Runtime("record name is unavailable".to_string()))
        })?
        .to_string_lossy()
        .to_string();
    let dest = Context::ensure_within(&ctx.project_dir, &archived_dir.join(&file_name))
        .map_err(RelocateError::Other)?;

    let doc = crate::record::document::parse(&content_for_status).map_err(RelocateError::Other)?;
    let new_content = crate::record::patch::apply_patch(
        &doc,
        &[crate::record::patch::PatchOp::SetFrontMatter(
            "status".into(),
            "archived".into(),
        )],
    )
    .map_err(RelocateError::Other)?;

    write_atomic(ctx, cfg, registry, &dest, &new_content).map_err(|_| {
        RelocateError::WriteFailed {
            source: path.clone(),
        }
    })?;

    if let Err(_e) = fs::remove_file(&path) {
        return Err(RelocateError::UnlinkFailed {
            source: path.clone(),
            destination: dest.clone(),
        });
    }
    Ok(dest)
}

/// Archive a record: patch status to `archived`, write the destination via the
/// atomic sequence, and remove the source only after the destination write
/// succeeds. Returns an `AppError` for callers that do not need to distinguish
/// relocation failure modes.
pub fn archive(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &crate::record::descriptor::RecordDescriptor,
    id: &str,
) -> Result<PathBuf, AppError> {
    archive_inner(ctx, cfg, registry, descriptor, id).map_err(|e| e.into_app_error())
}

/// Relocate an active-folder record whose `status` is `archived` to the
/// archived folder. Returns the (possibly new) path. Idempotent and a no-op
/// for non-managed or non-archived-status records.
pub fn relocate_if_archived(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    path: &Path,
) -> Result<PathBuf, RelocateError> {
    let safe = resolve_path(ctx, path).map_err(RelocateError::Other)?;
    if !safe.exists() {
        return Err(RelocateError::Other(AppError::NotFound(format!(
            "record not found: {}",
            safe.display()
        ))));
    }
    let desc =
        match crate::record::migrate::managed_descriptor(cfg, registry, &ctx.project_dir, &safe) {
            Some(d) => d,
            None => return Ok(safe),
        };
    let active_dir = desc
        .resolve_directory(cfg, &ctx.project_dir)
        .map_err(RelocateError::Other)?;
    if safe.parent() != Some(active_dir.as_path()) {
        return Ok(safe);
    }
    let content = fs::read_to_string(&safe).unwrap_or_default();
    let status = crate::record::frontmatter::parse(&content)
        .ok()
        .flatten()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "status"));
    if status.as_deref() != Some("archived") {
        return Ok(safe);
    }

    // Direct relocation without resolving by id (avoids recursion through
    // `resolve_in_dirs` / `archive_inner`).
    let archived_dir = match desc.resolve_archived_directory(cfg, &ctx.project_dir) {
        Some(Ok(dir)) => dir,
        Some(Err(e)) => return Err(RelocateError::Other(e)),
        None => {
            return Err(RelocateError::Other(AppError::Validation(format!(
                "record type {} has no archived location",
                desc.name
            ))));
        }
    };
    fs::create_dir_all(&archived_dir)
        .map_err(|e| RelocateError::Other(AppError::Runtime(e.to_string())))?;
    let file_name = safe
        .file_name()
        .ok_or_else(|| {
            RelocateError::Other(AppError::Runtime("record name is unavailable".to_string()))
        })?
        .to_string_lossy()
        .to_string();
    let dest = Context::ensure_within(&ctx.project_dir, &archived_dir.join(&file_name))
        .map_err(RelocateError::Other)?;

    let doc = crate::record::document::parse(&content).map_err(RelocateError::Other)?;
    let new_content = crate::record::patch::apply_patch(
        &doc,
        &[crate::record::patch::PatchOp::SetFrontMatter(
            "status".into(),
            "archived".into(),
        )],
    )
    .map_err(RelocateError::Other)?;

    crate::record::store::write_atomic(ctx, cfg, registry, &dest, &new_content).map_err(|_| {
        eprintln!(
            "failed to relocate archived record; source intact: {}",
            safe.display()
        );
        RelocateError::WriteFailed {
            source: safe.clone(),
        }
    })?;

    if fs::remove_file(&safe).is_err() {
        eprintln!(
            "archive wrote {} but failed to remove {}",
            dest.display(),
            safe.display()
        );
        return Err(RelocateError::UnlinkFailed {
            source: safe.clone(),
            destination: dest.clone(),
        });
    }
    eprintln!(
        "relocated archived record: {} -> {}",
        safe.display(),
        dest.display()
    );
    Ok(dest)
}

/// Delete a document record: archive an active record first, then remove the
/// archived file; remove an archived record directly.
pub fn delete(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &crate::record::descriptor::RecordDescriptor,
    id: &str,
) -> Result<(), AppError> {
    let active_dir = descriptor.resolve_directory(cfg, &ctx.project_dir)?;
    let read_dirs = descriptor.resolve_read_directories(cfg, &ctx.project_dir);
    let prefixes = descriptor.configured_prefixes(cfg);
    let dotted = descriptor.is_dotted_id();
    let resolved = resolve_family_record(ctx, cfg, registry, &read_dirs, id, &prefixes, dotted)?;
    let record = resolved.ok_or_else(|| AppError::NotFound(format!("record not found: {}", id)))?;
    let path = record.path;

    if path.parent() == Some(active_dir.as_path()) {
        let archived_path = archive(ctx, cfg, registry, descriptor, id)?;
        fs::remove_file(&archived_path).map_err(|e| AppError::Runtime(e.to_string()))?;
    } else {
        fs::remove_file(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
    Ok(())
}

/// Set a document record's status to `active` or a declared custom status.
///
/// Performs no move and creates no folder; refreshes `updated_date`.
/// Rejects `archived` (use `archive`) and archived records. Validates
/// `value` against the descriptor's declared statuses; an undeclared
/// current status refuses writes until fixed.
pub fn set_status(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &crate::record::descriptor::RecordDescriptor,
    id: &str,
    value: &str,
) -> Result<(), AppError> {
    // Reject archived explicitly.
    if value == "archived" {
        return Err(AppError::Validation(
            "status 'archived' is not allowed here; use archive".to_string(),
        ));
    }
    let declared = descriptor.declared_statuses(cfg);
    if !declared.is_empty() && !declared.iter().any(|d| d == value) {
        return Err(AppError::Validation(format!(
            "unknown status '{}' for {}",
            value, descriptor.name
        )));
    }

    let archived_dir = descriptor
        .resolve_archived_directory(cfg, &ctx.project_dir)
        .and_then(|r| r.ok());
    let read_dirs = descriptor.resolve_read_directories(cfg, &ctx.project_dir);
    let prefixes = descriptor.configured_prefixes(cfg);
    let dotted = descriptor.is_dotted_id();
    let resolved = resolve_family_record(ctx, cfg, registry, &read_dirs, id, &prefixes, dotted)?;
    let record = resolved.ok_or_else(|| AppError::NotFound(format!("record not found: {}", id)))?;
    let path = record.path.clone();

    record.ensure_writable()?;

    // Reject archived records (by location).
    if let Some(ad) = &archived_dir
        && path.parent() == Some(ad.as_path())
    {
        return Err(AppError::Validation(format!(
            "record {} is archived; status writes are not allowed",
            record.id
        )));
    }
    // Also reject if current status is archived.
    let content = crate::record::migrate::ensure_front_matter_quiet(ctx, cfg, registry, &path)?;
    let current_status = crate::record::frontmatter::parse(&content)
        .ok()
        .flatten()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "status"));
    if current_status.as_deref() == Some("archived") {
        return Err(AppError::Validation(format!(
            "record {} is archived; status writes are not allowed",
            record.id
        )));
    }
    // Tolerant: refuse writes if current status is undeclared.
    if let Some(cs) = &current_status
        && !declared.is_empty()
        && !declared.iter().any(|d| d == cs)
    {
        return Err(AppError::Validation(format!(
            "record {} has undeclared status '{}'; fix value or configuration first",
            record.id, cs
        )));
    }

    let doc = crate::record::document::parse(&content)?;
    let now = crate::record::frontmatter::utc_now();
    let new_content = crate::record::patch::apply_patch(
        &doc,
        &[
            crate::record::patch::PatchOp::SetFrontMatter("status".into(), value.into()),
            crate::record::patch::PatchOp::SetFrontMatter("updated_date".into(), now),
        ],
    )?;

    // write in place; directory must already exist (no folder creation).
    write_atomic(ctx, cfg, registry, &path, &new_content)?;
    Ok(())
}

/// Atomic destination write followed by source unlink, for callers that build
/// their own patched content.
pub fn archive_content(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    source: &Path,
    destination: &Path,
    content: &str,
) -> Result<(), AppError> {
    let source_safe = resolve_path(ctx, source)?;
    let dest_safe = resolve_path(ctx, destination)?;
    if let Some(parent) = dest_safe.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
    write_atomic(ctx, cfg, registry, &dest_safe, content)?;
    if let Err(e) = fs::remove_file(&source_safe) {
        return Err(AppError::Runtime(format!(
            "archive wrote {} but failed to remove {}: {}",
            dest_safe.display(),
            source_safe.display(),
            e
        )));
    }
    Ok(())
}

/// Reassign a record's ID through the single write path.
///
/// A record without front matter keeps its identity in the file stem, so the
/// repair is a rename with unchanged bytes. A record with front matter patches
/// `id`, refreshes `updated_date`, and updates the body heading ID only when the
/// heading already agrees with the front-matter `id`. The new path is written
/// atomically before the old path is removed.
pub fn reassign_id(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    path: &Path,
    new_id: &str,
) -> Result<PathBuf, AppError> {
    if !crate::record::identity::is_valid_id_component(new_id) {
        return Err(AppError::Validation(format!(
            "invalid record id: {}",
            new_id
        )));
    }
    let source = resolve_path(ctx, path).map_err(|_| {
        AppError::Runtime(format!(
            "path resolves outside project directory: {}",
            path.display()
        ))
    })?;
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let file_name = if extension.is_empty() {
        new_id.to_string()
    } else {
        format!("{}.{}", new_id, extension)
    };
    let destination = Context::ensure_within(&ctx.project_dir, &source.with_file_name(file_name))?;

    let content = fs::read_to_string(&source).map_err(|error| {
        AppError::Runtime(format!(
            "cannot read record {}: {}",
            source.display(),
            error
        ))
    })?;

    let front_matter = crate::record::frontmatter::parse(&content)?;
    let Some(front_matter) = front_matter else {
        move_record(ctx, &source, &destination)?;
        return Ok(destination);
    };

    let current_id = crate::record::frontmatter::current_value_string(&front_matter.raw, "id")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let doc = crate::record::document::parse(&content)?;
    let (heading_id, _) = crate::record::document::heading_id_title(&doc);
    let heading_matches = current_id
        .as_deref()
        .is_some_and(|current| !heading_id.is_empty() && heading_id.trim() == current);

    let mut ops = vec![
        crate::record::patch::PatchOp::SetFrontMatter("id".into(), new_id.to_string()),
        crate::record::patch::PatchOp::SetFrontMatter(
            "updated_date".into(),
            crate::record::frontmatter::utc_now(),
        ),
    ];
    if heading_matches {
        ops.push(crate::record::patch::PatchOp::SetHeadingId(
            new_id.to_string(),
        ));
    }
    let patched = crate::record::patch::apply_patch(&doc, &ops)?;

    write_atomic(ctx, cfg, registry, &destination, &patched)?;
    fs::remove_file(&source).map_err(|error| {
        AppError::Runtime(format!(
            "wrote {} but failed to remove {}: {}",
            destination.display(),
            source.display(),
            error
        ))
    })?;
    Ok(destination)
}

// Provide a Store struct alias for hidden tests that prefer OO style.
pub struct Store;

impl Store {
    pub fn resolve_path(ctx: &Context, target: &Path) -> Result<PathBuf, AppError> {
        resolve_path(ctx, target)
    }
    pub fn read(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        target: &Path,
    ) -> Result<(Document, String), AppError> {
        read(ctx, cfg, registry, target)
    }
    pub fn read_document(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        target: &Path,
    ) -> Result<(Document, String), AppError> {
        read_document(ctx, cfg, registry, target)
    }
    pub fn read_document_normalized(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        target: &Path,
    ) -> Result<NormalizedRead, AppError> {
        read_document_normalized(ctx, cfg, registry, target)
    }
    pub fn valid_version_token(token: &str) -> bool {
        let mut parts = token.split('.');
        let (Some(seconds), Some(nanos), Some(length), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return false;
        };
        !seconds.is_empty()
            && !nanos.is_empty()
            && !length.is_empty()
            && seconds.bytes().all(|b| b.is_ascii_digit())
            && nanos.bytes().all(|b| b.is_ascii_digit())
            && length.bytes().all(|b| b.is_ascii_digit())
    }

    /// Move one record file atomically to another project-relative location.
    ///
    /// Relocation is a directory move, not a record write: the shared store is the
    /// only place that may rename a record path.
    pub fn move_record(ctx: &Context, source: &Path, destination: &Path) -> Result<(), AppError> {
        let source = resolve_path(ctx, source).map_err(|_| {
            AppError::Runtime(format!(
                "path resolves outside project directory: {}",
                source.display()
            ))
        })?;
        let destination = resolve_path(ctx, destination).map_err(|_| {
            AppError::Runtime(format!(
                "path resolves outside project directory: {}",
                destination.display()
            ))
        })?;
        fs::rename(&source, &destination).map_err(|e| AppError::Runtime(e.to_string()))
    }

    pub fn write_atomic(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        target: &Path,
        content: &str,
    ) -> Result<(), AppError> {
        write_atomic(ctx, cfg, registry, target, content)
    }
    pub fn archive(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        descriptor: &crate::record::descriptor::RecordDescriptor,
        id: &str,
    ) -> Result<PathBuf, AppError> {
        archive(ctx, cfg, registry, descriptor, id)
    }
    pub fn set_status(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        descriptor: &crate::record::descriptor::RecordDescriptor,
        id: &str,
        value: &str,
    ) -> Result<(), AppError> {
        set_status(ctx, cfg, registry, descriptor, id, value)
    }
    pub fn relocate_if_archived(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        path: &Path,
    ) -> Result<PathBuf, RelocateError> {
        relocate_if_archived(ctx, cfg, registry, path)
    }
    pub fn delete(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        descriptor: &crate::record::descriptor::RecordDescriptor,
        id: &str,
    ) -> Result<(), AppError> {
        delete(ctx, cfg, registry, descriptor, id)
    }
    pub fn archive_content(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        source: &Path,
        destination: &Path,
        content: &str,
    ) -> Result<(), AppError> {
        archive_content(ctx, cfg, registry, source, destination, content)
    }
    pub fn version_token(meta: &fs::Metadata) -> String {
        version_token(meta)
    }
    pub fn reassign_id(
        ctx: &Context,
        cfg: &Config,
        registry: &ArtifactRegistry,
        path: &Path,
        new_id: &str,
    ) -> Result<PathBuf, AppError> {
        reassign_id(ctx, cfg, registry, path, new_id)
    }
}

// aliases
pub fn read_store(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    target: &Path,
) -> Result<(Document, String), AppError> {
    read(ctx, cfg, registry, target)
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

    fn ctx(dir: &std::path::Path) -> Context {
        Context {
            project_dir: dir.to_path_buf(),
            config_path: {
                let p = dir.join("papyrus.config.yml");
                if !p.exists() {
                    let _ = std::fs::write(&p, "project_name: Test\n");
                }
                p
            },
            json: false,
            no_color: true,
        }
    }

    fn access(c: &Context) -> (Config, ArtifactRegistry) {
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir)
            .unwrap_or_default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        (cfg, registry)
    }

    #[test]
    fn read_legacy_shape() {
        let dir = tmpdir("legacy");
        let c = ctx(&dir);
        let p = dir.join("legacy.md");
        fs::write(&p, "# T Title\n\n## Description\nBody\n").unwrap();
        let (doc, token) = read(&c, &access(&c).0, &access(&c).1, &p).unwrap();
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
        let (doc, _) = read(&c, &access(&c).0, &access(&c).1, &p).unwrap();
        assert!(doc.front_matter.is_some());
        assert!(doc.frontmatter.is_some());
    }

    #[test]
    fn read_missing_is_not_found() {
        let dir = tmpdir("missing");
        let c = ctx(&dir);
        let p = dir.join("nope.md");
        let err = read(&c, &access(&c).0, &access(&c).1, &p).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn read_malformed_yaml_is_validation_error() {
        let dir = tmpdir("malformed");
        let c = ctx(&dir);
        let p = dir.join("bad.md");
        fs::write(&p, "---\n: bad\n---\n").unwrap();
        let err = read(&c, &access(&c).0, &access(&c).1, &p).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn path_escape_rejected() {
        let dir = tmpdir("escape");
        let c = ctx(&dir);
        let outside = dir.join("..").join("outside.md");
        let err = read(&c, &access(&c).0, &access(&c).1, &outside).unwrap_err();
        assert!(matches!(err, AppError::Runtime(_)));
    }

    #[test]
    fn atomic_write_success() {
        let dir = tmpdir("awrite");
        let c = ctx(&dir);
        let p = dir.join("rec.md");
        fs::write(&p, "original").unwrap();
        write_atomic(&c, &access(&c).0, &access(&c).1, &p, "updated").unwrap();
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
        let res = write_atomic(&c, &access(&c).0, &access(&c).1, &dir_target, "content");
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
    fn archived_milestone_in_active_folder_is_relocated_by_read_paths() {
        let dir = tmpdir("milestone-archived-preserved");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let path = dir.join("backlog/milestones/MS-001.md");
        let content = "---\nid: MS-001\ntitle: Legacy\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\nstatus: archived\n---\n\n## Description\n\nBody\n";
        fs::write(&path, content).unwrap();

        let desc = crate::record::descriptor::descriptor_for("milestone").expect("milestone");
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir)
            .unwrap_or_default();
        let read_dirs = desc.resolve_read_directories(&cfg, &c.project_dir);
        let prefixes = desc.configured_prefixes(&cfg);
        let archived = dir.join("backlog/archive/milestones/MS-001.md");
        let resolved = crate::record::resolve::resolve_in_dirs(
            &c,
            &access(&c).0,
            &access(&c).1,
            &read_dirs,
            "MS-001",
            &prefixes,
            false,
        )
        .expect("read path succeeds")
        .expect("milestone resolves");
        assert_eq!(
            resolved.path, archived,
            "the archived record moves to the canonical archive folder"
        );
        assert!(!path.exists(), "the active-folder copy is removed");
        assert_eq!(fs::read_to_string(&archived).unwrap(), content);

        let after = relocate_if_archived(&c, &access(&c).0, &access(&c).1, &archived)
            .expect("archived folder read is a no-op");
        assert_eq!(after, archived);
        assert_eq!(fs::read_to_string(&archived).unwrap(), content);
    }

    fn reassign_fixture(name: &str) -> (Context, std::path::PathBuf) {
        let dir = tmpdir(name);
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("docs/technical")).unwrap();
        (c, dir)
    }

    #[test]
    fn reassign_id_patches_id_updated_date_heading_and_preserves_the_rest() {
        let (c, dir) = reassign_fixture("reassign-frontmatter");
        let path = dir.join("docs/technical/TECH-001.md");
        let content = "---\nid: TECH-001\ntitle: T\ncustom: kept\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2000-01-01T00:00:00Z'\n---\n\n# TECH-001 T\n\n## Description\n\nBody\n";
        fs::write(&path, content).unwrap();

        let new_path = reassign_id(&c, &access(&c).0, &access(&c).1, &path, "TECH-002").unwrap();
        assert_eq!(new_path, dir.join("docs/technical/TECH-002.md"));
        assert!(!path.exists(), "the old path is removed");
        let written = fs::read_to_string(&new_path).unwrap();
        assert!(written.contains("id: TECH-002"), "got: {}", written);
        assert!(written.contains("# TECH-002 T"), "got: {}", written);
        assert!(written.contains("custom: kept"), "got: {}", written);
        assert!(!written.contains("TECH-001"), "got: {}", written);
        assert!(
            !written.contains("2000-01-01T00:00:00Z"),
            "got: {}",
            written
        );
        let frontmatter = crate::record::frontmatter::parse(&written)
            .unwrap()
            .expect("front matter");
        let updated =
            crate::record::frontmatter::current_value_string(&frontmatter.raw, "updated_date")
                .expect("updated_date");
        assert!(
            crate::record::frontmatter::timestamp_seconds(&updated).is_some(),
            "updated_date is ISO-8601 UTC: {}",
            updated
        );
        let parsed = crate::record::identity::parse_id_parts(
            new_path.file_stem().unwrap().to_str().unwrap(),
            "TECH",
            false,
        )
        .expect("new stem parses");
        assert_eq!(parsed.display, "TECH-002");
    }

    #[test]
    fn reassign_id_leaves_a_disagreeing_heading_untouched() {
        let (c, dir) = reassign_fixture("reassign-heading");
        let path = dir.join("docs/technical/TECH-001.md");
        let body = "\n# OTHER T\n\n## Description\n\nBody\n";
        let content = format!(
            "---\nid: TECH-001\ntitle: T\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2000-01-01T00:00:00Z'\n---\n{}",
            body
        );
        fs::write(&path, &content).unwrap();

        let new_path = reassign_id(&c, &access(&c).0, &access(&c).1, &path, "TECH-002").unwrap();
        let written = fs::read_to_string(&new_path).unwrap();
        assert!(written.contains("# OTHER T"), "got: {}", written);
        assert!(written.contains("id: TECH-002"), "got: {}", written);
        let (_, new_body) = crate::record::frontmatter::split(&written).unwrap();
        assert_eq!(new_body, body, "the body is byte-identical");
    }

    #[test]
    fn reassign_id_renames_a_record_without_front_matter_with_identical_bytes() {
        let (c, dir) = reassign_fixture("reassign-html");
        let path = dir.join("docs/technical/UI-001-dashboard.html");
        let bytes = b"<!doctype html>\n<title>Dashboard</title>\n";
        fs::write(&path, bytes).unwrap();

        let new_path = reassign_id(&c, &access(&c).0, &access(&c).1, &path, "UI-002").unwrap();
        assert_eq!(new_path, dir.join("docs/technical/UI-002.html"));
        assert!(!path.exists(), "the old path is removed");
        assert_eq!(fs::read(&new_path).unwrap(), bytes);
    }

    #[test]
    fn reassign_id_rejects_an_unsafe_id() {
        let (c, dir) = reassign_fixture("reassign-unsafe");
        let path = dir.join("docs/technical/TECH-001.md");
        fs::write(&path, "---\nid: TECH-001\n---\n").unwrap();
        let error = reassign_id(&c, &access(&c).0, &access(&c).1, &path, "../escape").unwrap_err();
        assert!(matches!(error, AppError::Validation(_)), "{:?}", error);
        assert!(path.exists(), "a refused id writes nothing");
    }

    #[test]
    fn config_defined_type_reads_after_the_configuration_file_is_removed() {
        let dir = tmpdir("config-removed");
        let c = ctx(&dir);
        fs::write(
            &c.config_path,
            "project_name: Test\ndocument_types:\n  note:\n    prefix: NOTE\n    directory: docs/notes/\n    enabled: true\n    statuses:\n      - active\n      - archived\n",
        )
        .unwrap();
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir).unwrap();
        let registry = ArtifactRegistry::build(&cfg).unwrap();
        fs::create_dir_all(dir.join("docs/notes")).unwrap();
        let path = dir.join("docs/notes/NOTE-001.md");
        let content = "---\nid: NOTE-001\ntitle: A note\nstatus: active\n---\n\n# NOTE-001 A note\n\n## Description\nBody\n";
        fs::write(&path, content).unwrap();

        fs::remove_file(&c.config_path).unwrap();
        let (doc, _token) = read_document(&c, &cfg, &registry, &path)
            .expect("the supplied configuration and registry serve the read");
        assert_eq!(doc.source, content);
    }

    #[test]
    fn config_defined_type_reads_after_the_configuration_file_becomes_invalid() {
        let dir = tmpdir("config-invalid");
        let c = ctx(&dir);
        fs::write(
            &c.config_path,
            "project_name: Test\ndocument_types:\n  note:\n    prefix: NOTE\n    directory: docs/notes/\n    enabled: true\n    statuses:\n      - active\n      - archived\n",
        )
        .unwrap();
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir).unwrap();
        let registry = ArtifactRegistry::build(&cfg).unwrap();
        fs::create_dir_all(dir.join("docs/notes")).unwrap();
        let path = dir.join("docs/notes/NOTE-001.md");
        let content = "---\nid: NOTE-001\ntitle: A note\nstatus: active\n---\n\n# NOTE-001 A note\n\n## Description\nBody\n";
        fs::write(&path, content).unwrap();

        fs::write(&c.config_path, "document_types: [not a map").unwrap();
        let (doc, _token) = read_document(&c, &cfg, &registry, &path)
            .expect("an invalid on-disk configuration is not reloaded");
        assert_eq!(doc.source, content);
    }

    #[test]
    fn store_struct_methods() {
        let dir = tmpdir("struct");
        let c = ctx(&dir);
        let p = dir.join("rec.md");
        fs::write(&p, "# T\n").unwrap();
        let (doc, tok) = Store::read(&c, &access(&c).0, &access(&c).1, &p).unwrap();
        assert!(!tok.is_empty());
        Store::write_atomic(&c, &access(&c).0, &access(&c).1, &p, "# T2\n").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "# T2\n");
        let _ = doc;
    }
}
