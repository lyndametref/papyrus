//! Browser record view model built from the shared record modules.
//!
//! Reading, parsing, identity, projection, and writing all live under
//! `crate::record/`. This module only reshapes the shared [`Document`] and
//! [`RecordView`] into the render-only [`Record`] view model and keeps the
//! browser's validation rules. The owning [`TypeDescriptor`] is carried on
//! every record; behaviour is selected by `DocumentModel`/`TypedHandler`, never
//! by a browser-local category enum or a slug literal.

use crate::config::{Config, FieldKind, FieldSpec, FieldValues};
use crate::context::Context;
use crate::error::AppError;
use crate::record::document::{self, Document};
use crate::record::field_map::{DocumentFieldMap, ResolvedField};
use crate::record::identity;
use crate::record::migrate::MigrationConflict;
use crate::record::view::RecordView;
use crate::registry::{
    ArtifactCategory as CategoryKind, ArtifactRegistry, DocumentModel, IdShape, StatusModel,
    TypeDescriptor, TypedHandler,
};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum accepted record ID length in bytes (specification section 7.1).
pub const MAX_ID_BYTES: usize = 128;
/// Maximum accepted title length in Unicode scalar values (section 7.3).
pub const MAX_TITLE_CHARS: usize = 200;
/// Maximum accepted story or description length (section 7.3).
pub const MAX_TEXT_CHARS: usize = 20000;

/// A loaded record with its owning descriptor.
#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub descriptor: TypeDescriptor,
    /// The model-neutral content projection of a document record.
    pub field_map: DocumentFieldMap,
    /// A grouped family's group record (`REQ-001`) rather than a member.
    pub is_group: bool,
    pub title: String,
    pub group_id: Option<String>,
    pub group_title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub ordinal: Option<i64>,
    pub milestone_id: Option<String>,
    pub milestone_title: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub comments: Vec<(String, String)>,
    pub release_id: Option<String>,
    pub release_title: Option<String>,
    pub decision: Option<String>,
    pub context: Option<String>,
    pub consequences: Option<String>,
    pub decision_type: Option<String>,
    pub superseded_by: Option<String>,
    pub superseded_title: Option<String>,
    pub refs: Vec<String>,
    pub doc: Document,
    /// Record content exactly as stored on disk, before line-ending
    /// normalization. It is the byte comparison baseline for a save.
    pub stored_source: String,
    pub path: PathBuf,
    pub rel_path: String,
    pub version: String,
}

impl Record {
    /// Registry slug of the record's type.
    pub fn slug(&self) -> &str {
        &self.descriptor.slug
    }

    /// Typed handler for built-in types; `None` for generic config types.
    pub fn handler(&self) -> Option<TypedHandler> {
        self.descriptor.handler()
    }

    pub fn is_generic(&self) -> bool {
        self.descriptor.handler().is_none()
    }

    pub fn is_html_mockup(&self) -> bool {
        self.descriptor.document_model == DocumentModel::SingleHtmlMockup
    }

    pub fn type_label(&self) -> String {
        crate::browser::routes::label(&self.descriptor)
    }

    /// Stored status rendered read-only, or `(none)` when absent.
    pub fn status_text(&self) -> String {
        self.status
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("(none)")
            .to_string()
    }
}

/// An indexed document type. Records are sorted by ID ascending.
#[derive(Debug, Default)]
pub struct Index {
    pub records: Vec<Record>,
    pub groups: Vec<Record>,
    pub requirements: Vec<Record>,
    /// Member requirement indexes keyed by resolved group ID.
    pub members: BTreeMap<String, Vec<usize>>,
    /// Requirement indexes whose stored group reference is absent or unresolved.
    pub other: Vec<usize>,
}

impl Index {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// A record resolved by exact ID match.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub record: Record,
}

/// A request-scoped, display-only record snapshot.
///
/// The snapshot indexes the enabled descriptors' active directories once, on
/// the first ID lookup, and records file names only. Resolving an ID reads the
/// file-name map and parses a matching record at most once, memoising it for
/// the rest of the request. It lives only for the request that owns it and is
/// never stored in `ServerState`.
pub struct RecordSnapshot<'a> {
    ctx: &'a Context,
    cfg: &'a Config,
    registry: &'a ArtifactRegistry,
    descriptors: Vec<&'a TypeDescriptor>,
    listing: RefCell<Option<SnapshotListing>>,
    records: RefCell<BTreeMap<PathBuf, Option<Record>>>,
    /// Front-matter-only records used by list, group and reference read paths.
    /// A record already parsed in full by `records` is reused verbatim.
    light_records: RefCell<BTreeMap<PathBuf, Option<Record>>>,
    conflicts: RefCell<BTreeMap<String, Vec<MigrationConflict>>>,
    listing_count: Cell<usize>,
    parse_count: Cell<usize>,
    conflict_scan_count: Cell<usize>,
}

#[derive(Default)]
struct SnapshotListing {
    /// The canonical numeric key of every listed record, per descriptor index.
    by_descriptor: BTreeMap<usize, BTreeMap<String, Vec<PathBuf>>>,
}

/// Display-only title lookup used while projecting one record.
trait RecordTitles {
    fn title(&self, id: &str) -> Option<String>;
}

struct LegacyTitles<'a> {
    ctx: &'a Context,
    cfg: &'a Config,
    registry: &'a ArtifactRegistry,
}

impl RecordTitles for LegacyTitles<'_> {
    fn title(&self, id: &str) -> Option<String> {
        resolve_title(self.ctx, self.cfg, self.registry, id)
    }
}

impl RecordTitles for RecordSnapshot<'_> {
    fn title(&self, id: &str) -> Option<String> {
        self.resolve_title(id)
    }
}

impl<'a> RecordSnapshot<'a> {
    pub fn new(ctx: &'a Context, cfg: &'a Config, registry: &'a ArtifactRegistry) -> Self {
        RecordSnapshot {
            ctx,
            cfg,
            registry,
            descriptors: registry.descriptors().collect(),
            listing: RefCell::new(None),
            records: RefCell::new(BTreeMap::new()),
            light_records: RefCell::new(BTreeMap::new()),
            conflicts: RefCell::new(BTreeMap::new()),
            listing_count: Cell::new(0),
            parse_count: Cell::new(0),
            conflict_scan_count: Cell::new(0),
        }
    }

    /// Number of `read_dir` attempts made while building the file-name index.
    pub fn listing_count(&self) -> usize {
        self.listing_count.get()
    }

    /// Number of distinct record files parsed through this snapshot.
    pub fn parse_count(&self) -> usize {
        self.parse_count.get()
    }

    /// Number of descriptor-family conflict scans made through this snapshot.
    pub fn conflict_scan_count(&self) -> usize {
        self.conflict_scan_count.get()
    }

    /// Whether the file-name index has been built.
    pub fn is_built(&self) -> bool {
        self.listing.borrow().is_some()
    }

    fn ensure_listing(&self) {
        if self.listing.borrow().is_some() {
            return;
        }
        let mut listing = SnapshotListing::default();
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            if !descriptor.enabled {
                continue;
            }
            let Some(dir) = descriptor_directory(self.ctx, self.cfg, descriptor) else {
                continue;
            };
            let extension = content_extension(descriptor);
            self.listing_count.set(self.listing_count.get() + 1);
            let Ok(read_dir) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some(extension) {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                if !matches_stem(descriptor, stem) {
                    continue;
                }
                for key in lookup_keys(descriptor, stem) {
                    listing
                        .by_descriptor
                        .entry(index)
                        .or_default()
                        .entry(key)
                        .or_default()
                        .push(path.clone());
                }
            }
        }
        *self.listing.borrow_mut() = Some(listing);
    }

    fn candidates(&self, id: &str) -> Vec<(usize, PathBuf)> {
        let listing = self.listing.borrow();
        let Some(listing) = listing.as_ref() else {
            return Vec::new();
        };
        let mut candidates: Vec<(usize, PathBuf)> = Vec::new();
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            if !descriptor.enabled {
                continue;
            }
            let keys = lookup_keys(descriptor, id);
            if keys.is_empty() {
                continue;
            }
            let Some(by_key) = listing.by_descriptor.get(&index) else {
                continue;
            };
            for key in keys {
                let Some(paths) = by_key.get(&key) else {
                    continue;
                };
                // Several files with the same canonical ID are the existing
                // duplicate-ID outcome: unresolved, never a directory rescan.
                if paths.len() == 1 && !candidates.iter().any(|(_, path)| path == &paths[0]) {
                    candidates.push((index, paths[0].clone()));
                }
            }
        }
        candidates
    }

    /// One migration-conflict scan per descriptor family per snapshot.
    ///
    /// Every record in a family shares the same listing and conflict result,
    /// so a request never rescans the family per record.
    fn migration_conflicts(&self, path: &Path) -> Vec<MigrationConflict> {
        let Some(descriptor) = crate::record::migrate::managed_descriptor(
            self.cfg,
            self.registry,
            &self.ctx.project_dir,
            path,
        ) else {
            return Vec::new();
        };
        let key = descriptor.name.to_string();
        if let Some(cached) = self.conflicts.borrow().get(&key) {
            return cached.clone();
        }
        self.conflict_scan_count
            .set(self.conflict_scan_count.get() + 1);
        let conflicts = crate::record::migrate::scan_migration_conflicts(
            self.ctx,
            self.cfg,
            self.registry,
            &descriptor,
        );
        self.conflicts.borrow_mut().insert(key, conflicts.clone());
        conflicts
    }

    fn load_cached(
        &self,
        descriptor: &TypeDescriptor,
        path: &Path,
    ) -> Result<Option<Record>, AppError> {
        if let Some(cached) = self.records.borrow().get(path) {
            return Ok(cached.clone());
        }
        self.parse_count.set(self.parse_count.get() + 1);
        let conflicts = self.migration_conflicts(path);
        let record = load_record_with(
            self.ctx,
            self.cfg,
            self.registry,
            descriptor,
            path,
            self,
            Some(&conflicts),
        )?;
        self.records
            .borrow_mut()
            .insert(path.to_path_buf(), record.clone());
        Ok(record)
    }

    /// Parse only the front matter of one record for a list or member row.
    ///
    /// The full parse cache is consulted first, so a record already read for
    /// the page itself is never parsed twice. Light records are memoised for
    /// the rest of the request in their own cache: they are not interchangeable
    /// with a full [`Record`] (no body sections and no field map), so the two
    /// caches stay separate.
    fn load_light_cached(
        &self,
        descriptor: &TypeDescriptor,
        path: &Path,
    ) -> Result<Option<Record>, AppError> {
        if let Some(cached) = self.records.borrow().get(path) {
            return Ok(cached.clone());
        }
        if let Some(cached) = self.light_records.borrow().get(path) {
            return Ok(cached.clone());
        }
        self.parse_count.set(self.parse_count.get() + 1);
        let record = load_light_record(self, descriptor, path)?;
        self.light_records
            .borrow_mut()
            .insert(path.to_path_buf(), record.clone());
        Ok(record)
    }

    /// Whether one listed path contributes a record to its descriptor's index.
    ///
    /// This mirrors [`load_record_with`]'s acceptance rules (readable, UTF-8,
    /// valid front matter, declared decision type, still present after any
    /// read-path relocation) without parsing the document body, the CQRS read
    /// projection, or the field map.
    fn record_counted(&self, descriptor: &TypeDescriptor, path: &Path) -> bool {
        if descriptor.document_model == DocumentModel::SingleHtmlMockup {
            return matches!(
                load_ui_mockup(self.ctx, self.cfg, descriptor, path),
                Ok(Some(_))
            );
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            return false;
        };
        if !matches_stem(descriptor, stem) {
            return false;
        }
        if descriptor.category == CategoryKind::Decision
            && crate::decision::type_key_for_id(self.cfg, stem).is_none()
        {
            return false;
        }
        let Ok((safe, _source, _front_matter)) = self.read_snapshot_source(path) else {
            return false;
        };
        safe.exists()
    }

    /// Fallback store read for a rare or stateful record, returning the parsed
    /// front matter so the caller never parses it a second time.
    fn slow_snapshot_source(
        &self,
        target: &Path,
        conflicts: Option<&[MigrationConflict]>,
    ) -> Result<
        (
            PathBuf,
            String,
            Option<crate::record::frontmatter::FrontMatter>,
        ),
        AppError,
    > {
        let (safe, normalized) = crate::record::store::read_normalized_source_with_conflicts(
            self.ctx,
            self.cfg,
            self.registry,
            target,
            conflicts,
        )?;
        let front_matter = crate::record::frontmatter::parse(&normalized)?;
        Ok((safe, normalized, front_matter))
    }

    /// Read one record for a display list without repeating the store reads.
    ///
    /// The shared store read performs a relocation probe and an adoption read
    /// before reading the record again. A listed record that already carries
    /// front matter, is not being relocated, and has no blocking conflict can
    /// be read once and normalized directly. Every rare or stateful case (no
    /// front matter, an archived record still in its active folder, a blocking
    /// conflict, malformed front matter, an unreadable file) falls back to the
    /// shared store so its writes, errors and warnings stay unchanged.
    fn read_snapshot_source(
        &self,
        target: &Path,
    ) -> Result<
        (
            PathBuf,
            String,
            Option<crate::record::frontmatter::FrontMatter>,
        ),
        AppError,
    > {
        let safe = Context::ensure_within(&self.ctx.project_dir, target)?;
        let Ok(content) = fs::read_to_string(&safe) else {
            return self.slow_snapshot_source(target, None);
        };
        let Ok(Some(front_matter)) = crate::record::frontmatter::parse(&content) else {
            return self.slow_snapshot_source(target, None);
        };
        if let Some(descriptor) = crate::record::migrate::managed_descriptor(
            self.cfg,
            self.registry,
            &self.ctx.project_dir,
            &safe,
        ) {
            let status = crate::record::frontmatter::value_string(&front_matter.data, "status");
            let Ok(active) = descriptor.resolve_directory(self.cfg, &self.ctx.project_dir) else {
                return self.slow_snapshot_source(target, None);
            };
            if status.as_deref() == Some("archived") && safe.parent() == Some(active.as_path()) {
                return self.slow_snapshot_source(target, None);
            }
            let conflicts = self.migration_conflicts(target);
            if crate::record::migrate::blocking_conflict_for_path(
                self.ctx,
                &safe,
                &descriptor,
                &content,
                &conflicts,
            )
            .is_some()
            {
                return self.slow_snapshot_source(target, Some(&conflicts));
            }
            let declared = descriptor.declared_statuses(self.cfg);
            if let Some(status) = status
                && !declared.is_empty()
                && !declared.iter().any(|declared| declared == &status)
            {
                eprintln!(
                    "warning: undeclared status '{}' at {}",
                    status,
                    safe.display()
                );
            }
        }
        Ok((
            safe,
            crate::record::line_endings::normalize(&content),
            Some(front_matter),
        ))
    }

    /// Count each enabled descriptor's listed records without parsing bodies.
    ///
    /// Inventory only needs one count per card. The file-name listing is built
    /// once, and each listed file is read and validated the same way a full
    /// load would reject it, but only its front matter is parsed.
    pub fn scan_counts(&self) -> BTreeMap<String, usize> {
        self.ensure_listing();
        let listing = self.listing.borrow();
        let Some(listing) = listing.as_ref() else {
            return BTreeMap::new();
        };
        let mut counts = BTreeMap::new();
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            if !descriptor.enabled {
                continue;
            }
            let mut paths: BTreeSet<&PathBuf> = BTreeSet::new();
            if let Some(by_key) = listing.by_descriptor.get(&index) {
                for path in by_key.values().flatten() {
                    paths.insert(path);
                }
            }
            let count = paths
                .into_iter()
                .filter(|path| self.record_counted(descriptor, path))
                .count();
            counts.insert(descriptor.slug.clone(), count);
        }
        counts
    }

    /// Resolve one record ID against the snapshot, memoising the parsed record.
    ///
    /// Unknown, archived, and disabled IDs stay absent from the file-name map
    /// and return `Ok(None)`. A listed record that cannot be read or parsed
    /// keeps the existing diagnostic.
    pub fn resolve(&self, id: &str) -> Result<Option<Resolved>, AppError> {
        self.resolve_inner(id, true)
    }

    /// Resolve one record ID for a write path: same identity rules and file
    /// names, but the returned record is a fresh live read. The snapshot
    /// supplies no bytes or version token to this result.
    pub fn resolve_live(&self, id: &str) -> Result<Option<Resolved>, AppError> {
        self.resolve_inner(id, false)
    }

    fn resolve_inner(&self, id: &str, cached: bool) -> Result<Option<Resolved>, AppError> {
        if !identity::is_valid_id_component(id) {
            return Ok(None);
        }
        self.ensure_listing();
        for (index, path) in self.candidates(id) {
            let descriptor = self.descriptors[index];
            let loaded = if cached {
                self.load_cached(descriptor, &path)?
            } else {
                load_record_with(
                    self.ctx,
                    self.cfg,
                    self.registry,
                    descriptor,
                    &path,
                    self,
                    None,
                )?
            };
            if let Some(record) = loaded {
                // A record auto-relocated out of the active directory by the
                // read path is no longer part of the display snapshot.
                if !record.path.exists() {
                    continue;
                }
                return Ok(Some(Resolved { record }));
            }
            // A candidate that resolves outside the project is invisible,
            // exactly as it is for the pre-snapshot resolver.
            let Ok(safe) = Context::ensure_within(&self.ctx.project_dir, &path) else {
                continue;
            };
            if fs::read_to_string(&safe).is_err() {
                return Err(AppError::Runtime("record is not valid UTF-8".to_string()));
            }
        }
        Ok(None)
    }

    /// The title of a resolved record, or `None` when it is unknown.
    pub fn resolve_title(&self, id: &str) -> Option<String> {
        self.resolve(id)
            .ok()
            .flatten()
            .map(|resolved| resolved.record.title)
    }

    /// Build every enabled descriptor's index from one registry-wide scan.
    ///
    /// The file-name listing lists each enabled descriptor's active directory
    /// at most once, and each listed record is parsed at most once through the
    /// shared parse cache. Every inventory card count and aggregate count is
    /// derived from this one result; no card or aggregate rescans a directory.
    pub fn scan_indexes(&self) -> BTreeMap<String, Index> {
        self.ensure_listing();
        let listing = self.listing.borrow();
        let Some(listing) = listing.as_ref() else {
            return BTreeMap::new();
        };
        let mut indexes = BTreeMap::new();
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            if !descriptor.enabled {
                continue;
            }
            // A grouped stem can appear under both its group and its member
            // lookup key; each file still belongs to the index exactly once.
            let mut paths: BTreeSet<PathBuf> = BTreeSet::new();
            if let Some(by_key) = listing.by_descriptor.get(&index) {
                for path in by_key.values().flatten() {
                    paths.insert(path.clone());
                }
            }
            let mut records = Vec::new();
            for path in paths {
                if let Ok(Some(record)) = self.load_cached(descriptor, &path)
                    && record.path.exists()
                {
                    records.push(record);
                }
            }
            indexes.insert(descriptor.slug.clone(), finalize_index(records, descriptor));
        }
        indexes
    }
}

fn lookup_keys(descriptor: &TypeDescriptor, value: &str) -> Vec<String> {
    let shapes: &[bool] = match descriptor.id_shape {
        IdShape::Grouped => &[true, false],
        IdShape::Dotted => &[true],
        IdShape::Plain => &[false],
    };
    let mut keys: Vec<String> = Vec::new();
    for &dotted in shapes {
        if let Some(parts) = identity::parse_id_parts(value, &descriptor.prefix, dotted) {
            let key = parts.key(&descriptor.prefix);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

/// Resolve a descriptor's active directory against the project boundary.
pub fn descriptor_directory(
    ctx: &Context,
    cfg: &Config,
    descriptor: &TypeDescriptor,
) -> Option<PathBuf> {
    if !descriptor.enabled {
        return None;
    }
    let base = match descriptor.directory.config_key {
        "backlog_directory" => ctx.project_dir.join(&cfg.backlog_directory),
        "decision_directory" => ctx.project_dir.join(&cfg.decision_directory),
        "doc_directory" => ctx.project_dir.join(&cfg.doc_directory),
        _ => return None,
    };
    let raw = if descriptor.directory.relative.is_empty() {
        base
    } else {
        base.join(&descriptor.directory.relative)
    };
    Context::ensure_within(&ctx.project_dir, &raw).ok()
}

fn relative_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn section_or_none(doc: &Document, name: &str) -> Option<String> {
    document::section_text(doc, name)
}

fn heading_title(doc: &Document) -> String {
    document::heading_id_title(doc).1
}

fn frontmatter_title(doc: &Document) -> String {
    document::heading_id_title_with_frontmatter(doc).1
}

fn non_empty_or(value: String, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn is_group_id(descriptor: &TypeDescriptor, stem: &str) -> bool {
    descriptor.id_shape == IdShape::Grouped
        && identity::parse_id_parts(stem, &descriptor.prefix, true).is_none()
        && identity::parse_id_parts(stem, &descriptor.prefix, false).is_some()
}

fn canonical_id(descriptor: &TypeDescriptor, value: &str) -> Option<String> {
    match descriptor.id_shape {
        IdShape::Grouped => identity::parse_id_parts(value, &descriptor.prefix, true)
            .or_else(|| identity::parse_id_parts(value, &descriptor.prefix, false))
            .map(|parts| parts.display),
        IdShape::Dotted => {
            identity::parse_id_parts(value, &descriptor.prefix, true).map(|parts| parts.display)
        }
        IdShape::Plain => {
            identity::parse_id_parts(value, &descriptor.prefix, false).map(|parts| parts.display)
        }
    }
}

/// Whether a file stem denotes a record of the descriptor's type.
pub fn matches_stem(descriptor: &TypeDescriptor, stem: &str) -> bool {
    identity::is_valid_id_component(stem) && canonical_id(descriptor, stem).is_some()
}

fn content_extension(descriptor: &TypeDescriptor) -> &'static str {
    match descriptor.document_model {
        DocumentModel::SingleHtmlMockup => "html",
        _ => "md",
    }
}

fn default_record(
    id: String,
    descriptor: TypeDescriptor,
    is_group: bool,
    doc: Document,
    path: PathBuf,
    rel_path: String,
    version: String,
) -> Record {
    Record {
        id,
        descriptor,
        field_map: DocumentFieldMap::default(),
        is_group,
        title: String::new(),
        group_id: None,
        group_title: None,
        description: None,
        status: None,
        ordinal: None,
        milestone_id: None,
        milestone_title: None,
        assignee: None,
        labels: Vec::new(),
        dependencies: Vec::new(),
        acceptance_criteria: Vec::new(),
        comments: Vec::new(),
        release_id: None,
        release_title: None,
        decision: None,
        context: None,
        consequences: None,
        decision_type: None,
        superseded_by: None,
        superseded_title: None,
        refs: Vec::new(),
        doc,
        stored_source: String::new(),
        path,
        rel_path,
        version,
    }
}

/// Load one managed record from the shared document parser and projection.
pub fn load_record(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    let titles = LegacyTitles { ctx, cfg, registry };
    load_record_with(ctx, cfg, registry, descriptor, path, &titles, None)
}

fn load_record_with(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    path: &Path,
    titles: &dyn RecordTitles,
    conflicts: Option<&[MigrationConflict]>,
) -> Result<Option<Record>, AppError> {
    if descriptor.document_model == DocumentModel::SingleHtmlMockup {
        return load_ui_mockup(ctx, cfg, descriptor, path);
    }
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        Err(_) => return Ok(None),
    };
    let Some(stem) = safe
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
    else {
        return Ok(None);
    };
    if !matches_stem(descriptor, &stem) {
        return Ok(None);
    }
    if descriptor.category == CategoryKind::Decision
        && crate::decision::type_key_for_id(cfg, &stem).is_none()
    {
        return Ok(None);
    }
    let Ok(read) = crate::record::store::read_document_normalized_with_conflicts(
        ctx, cfg, registry, &safe, conflicts,
    ) else {
        return Ok(None);
    };
    let id = canonical_id(descriptor, &stem).unwrap_or_else(|| stem.clone());
    let view = RecordView::from_content(&read.document.source, &id, &read.version).ok();
    let rel_path = relative_path(&ctx.project_dir, &safe);
    let version = read.version;
    let is_group = is_group_id(descriptor, &stem);
    let mut record = default_record(
        id,
        descriptor.clone(),
        is_group,
        read.document,
        safe,
        rel_path,
        version,
    );
    record.stored_source = read.stored_source;

    match record.handler() {
        Some(TypedHandler::Task) => {
            let td = crate::task::task_from_document(&record.doc);
            record.title = if td.title.is_empty() {
                stem.to_string()
            } else {
                td.title
            };
            record.description = td.description;
            record.status = (!td.status.is_empty()).then_some(td.status);
            record.ordinal = td.ordinal;
            record.milestone_id = td.milestone;
            record.assignee = td.assignee;
            record.labels = td.labels;
            record.dependencies = td.deps;
            record.acceptance_criteria = td.acs;
            record.comments = td.comments;
        }
        Some(TypedHandler::Milestone) => {
            record.title = non_empty_or(frontmatter_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.status = record
                .doc
                .front_matter
                .as_ref()
                .and_then(|fm| {
                    if crate::record::frontmatter::contains_key(&fm.raw, "status") {
                        crate::record::frontmatter::current_value_string(&fm.raw, "status")
                    } else {
                        None
                    }
                })
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| Some("open".to_string()));
            record.release_id = match record.doc.front_matter.as_ref() {
                Some(fm) if crate::record::frontmatter::contains_key(&fm.raw, "release") => {
                    crate::record::frontmatter::current_value_string(&fm.raw, "release")
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                }
                _ => section_or_none(&record.doc, "Release").map(|s| s.trim().to_string()),
            };
        }
        Some(TypedHandler::Release) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            record.description = section_or_none(&record.doc, "Description");
            record.status = record
                .doc
                .front_matter
                .as_ref()
                .and_then(|fm| {
                    if crate::record::frontmatter::contains_key(&fm.raw, "status") {
                        crate::record::frontmatter::current_value_string(&fm.raw, "status")
                    } else {
                        None
                    }
                })
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| Some("planned".to_string()));
        }
        Some(TypedHandler::Decision) => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
            let status = section_or_none(&record.doc, "Status").unwrap_or_default();
            record.status = Some(if status.trim().is_empty() {
                "proposed".to_string()
            } else {
                status.trim().to_string()
            });
            record.superseded_by = record
                .status
                .as_deref()
                .and_then(|s| s.strip_prefix("Superseded by "))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            record.decision_type = crate::decision::type_key_for_id(cfg, &stem);
            record.decision = section_or_none(&record.doc, "Decision");
            record.context = section_or_none(&record.doc, "Context");
            record.consequences = section_or_none(&record.doc, "Consequences");
        }
        None => {
            record.title = non_empty_or(heading_title(&record.doc), &stem);
        }
    }

    // The group link is descriptor-driven for every grouped member, not a
    // per-type branch: a record's group is the reserved engine-owned
    // front-matter key, with a legacy `## Group` section as fallback.
    if record.descriptor.document_model == DocumentModel::GroupedMarkdown && !is_group {
        let frontmatter_group = record
            .doc
            .front_matter
            .as_ref()
            .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "group"))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let Some(group_id) = frontmatter_group {
            record.group_id = Some(group_id);
        } else if let Some(group) = section_or_none(&record.doc, "Group") {
            let mut parts = group
                .lines()
                .next()
                .unwrap_or("")
                .splitn(2, char::is_whitespace);
            record.group_id = parts
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            record.group_title = parts
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
    }

    // Every document type, including generic config-defined types, projects
    // the front-matter `labels` and `references` lists. A handler that already
    // owns a field (task labels, UI mockup references) keeps its value.
    if record.labels.is_empty() {
        record.labels = view
            .as_ref()
            .map(|v| v.list_value("labels"))
            .unwrap_or_default();
    }
    if record.refs.is_empty() {
        record.refs = view
            .as_ref()
            .map(|v| v.list_value("references"))
            .unwrap_or_default();
    }
    if record.status.is_none() {
        record.status = view
            .as_ref()
            .and_then(|v| v.str_value("status"))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    if record.descriptor.category == CategoryKind::Document {
        record.field_map =
            DocumentFieldMap::build(&record.descriptor, is_group, &record.doc, view.as_ref());
    }
    if let Some(mid) = record.milestone_id.clone() {
        record.milestone_title = titles.title(&mid);
    }
    if let Some(rid) = record.release_id.clone() {
        record.release_title = titles.title(&rid);
    }
    if let Some(sid) = record.superseded_by.clone() {
        record.superseded_title = titles.title(&sid);
    }
    Ok(Some(record))
}

/// Load one record with its front matter only for a list, member or group row.
///
/// The source is still read, migrated and validated through the same store
/// contract as a full load, so identity, titles, statuses and owning-group
/// links are identical. The document body is kept as source (the display
/// projection reads it lazily) but is not split into sections, and no typed
/// handler projection or field map is built.
fn load_light_record(
    snapshot: &RecordSnapshot<'_>,
    descriptor: &TypeDescriptor,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    if descriptor.document_model == DocumentModel::SingleHtmlMockup {
        return load_ui_mockup(snapshot.ctx, snapshot.cfg, descriptor, path);
    }
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return Ok(None);
    };
    if !matches_stem(descriptor, stem) {
        return Ok(None);
    }
    if descriptor.category == CategoryKind::Decision
        && crate::decision::type_key_for_id(snapshot.cfg, stem).is_none()
    {
        return Ok(None);
    }
    let Ok((safe, normalized, front_matter)) = snapshot.read_snapshot_source(path) else {
        return Ok(None);
    };
    // A record auto-relocated out of the active directory by the read path is
    // no longer part of the display snapshot.
    if !safe.exists() {
        return Ok(None);
    }
    let id = canonical_id(descriptor, stem).unwrap_or_else(|| stem.to_string());
    let is_group = is_group_id(descriptor, stem);
    let rel_path = relative_path(&snapshot.ctx.project_dir, &safe);
    let version = fs::metadata(&safe)
        .map(|meta| crate::record::store::version_token(&meta))
        .unwrap_or_default();
    let heading = heading_line(&normalized);
    let title = front_matter
        .as_ref()
        .and_then(|front_matter| {
            crate::record::frontmatter::value_string(&front_matter.data, "title")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| heading_title_from(heading.as_deref()));
    let group = front_matter
        .as_ref()
        .and_then(|front_matter| {
            crate::record::frontmatter::value_string(&front_matter.data, "group")
        })
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let document = Document {
        source: normalized,
        front_matter: front_matter.clone(),
        frontmatter: front_matter,
        heading,
        preamble: Vec::new(),
        sections: Vec::new(),
    };
    let mut record = default_record(
        id,
        descriptor.clone(),
        is_group,
        document,
        safe,
        rel_path,
        version,
    );
    record.title = non_empty_or(title, stem);
    let view = record.doc.front_matter.as_ref().map(|front_matter| {
        RecordView::from_front_matter(front_matter, &record.id, &record.version)
    });
    if let Some(view) = &view {
        record.labels = view.list_value("labels");
        record.refs = view.list_value("references");
        record.status = view
            .str_value("status")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    if record.descriptor.document_model == DocumentModel::GroupedMarkdown && !is_group {
        if let Some(group_id) = group {
            record.group_id = Some(group_id);
        } else if let Some(group) = group_section_line(&record.doc.source) {
            let mut parts = group.splitn(2, char::is_whitespace);
            record.group_id = parts
                .next()
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty());
            record.group_title = parts
                .next()
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty());
        }
    }
    Ok(Some(record))
}

/// The title part of a heading line, using the same split as
/// [`document::heading_id_title`].
fn heading_title_from(heading: Option<&str>) -> String {
    let rest = heading.unwrap_or("").trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let _id = parts.next();
    parts.next().unwrap_or("").trim().to_string()
}

/// The heading line of a document, using the same leading-blank and `# `
/// rules as [`document::parse`].
fn heading_line(content: &str) -> Option<String> {
    let start = crate::record::frontmatter::find_block(content)
        .map(|(_, _, body_start)| body_start)
        .unwrap_or(0);
    for line in content[start..].split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.trim().is_empty() {
            continue;
        }
        return line.starts_with("# ").then(|| line.to_string());
    }
    None
}

/// The first non-empty line of a legacy `## Group` section, if present.
fn group_section_line(content: &str) -> Option<String> {
    let mut inside = false;
    for line in content.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if inside {
            if line.starts_with("## ") {
                break;
            }
            let value = line.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        } else if line.trim() == "## Group" {
            inside = true;
        }
    }
    None
}

fn resolve_title(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    id: &str,
) -> Option<String> {
    resolve_id(ctx, cfg, registry, id)
        .ok()
        .flatten()
        .map(|r| r.record.title)
}

fn load_ui_mockup(
    ctx: &Context,
    cfg: &Config,
    descriptor: &TypeDescriptor,
    path: &Path,
) -> Result<Option<Record>, AppError> {
    let safe = match Context::ensure_within(&ctx.project_dir, path) {
        Ok(safe) => safe,
        Err(_) => return Ok(None),
    };
    let content = match fs::read_to_string(&safe) {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };
    let meta = match fs::metadata(&safe) {
        Ok(meta) => meta,
        Err(_) => return Ok(None),
    };
    let Some(stem) = safe.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    if !identity::is_valid_id_component(stem) || !matches_stem(descriptor, stem) {
        return Ok(None);
    }
    let meta_record = crate::ui_mockup::parse_browser_metadata(&content, stem);
    let version = crate::record::store::version_token(&meta);
    let id = canonical_id(descriptor, stem).unwrap_or_else(|| stem.to_string());
    let doc = Document {
        source: content,
        front_matter: None,
        frontmatter: None,
        heading: None,
        preamble: Vec::new(),
        sections: Vec::new(),
    };
    let rel_path = relative_path(&ctx.project_dir, &safe);
    let mut record = default_record(id, descriptor.clone(), false, doc, safe, rel_path, version);
    record.title = meta_record.title.clone();
    record.status = (!meta_record.status.is_empty()).then_some(meta_record.status.clone());
    record.superseded_by = meta_record.superseded_by.clone();
    record.refs = meta_record.refs.clone();
    record.field_map = DocumentFieldMap::for_mockup(
        &record.descriptor,
        vec![
            (
                "status".to_string(),
                record.status.clone().unwrap_or_default(),
            ),
            ("task".to_string(), meta_record.task.clone()),
            ("specs".to_string(), meta_record.specs.join(" ")),
            ("refs".to_string(), meta_record.refs.join(" ")),
            (
                "superseded_by".to_string(),
                record.superseded_by.clone().unwrap_or_default(),
            ),
        ],
    );
    record.stored_source = record.doc.source.clone();
    let _ = cfg;
    Ok(Some(record))
}

/// The filterable status values offered by a descriptor, in order.
pub fn filter_statuses(descriptor: &TypeDescriptor) -> Vec<String> {
    match &descriptor.status_model {
        StatusModel::DocumentMain { custom, .. } => {
            let mut values = vec!["active".to_string()];
            values.extend(custom.iter().cloned());
            values
        }
        StatusModel::HtmlMetadata { states } => states
            .iter()
            .filter(|value| **value != "archived")
            .map(|value| (*value).to_string())
            .collect(),
        StatusModel::TaskConfigurable { statuses, .. } => statuses.clone(),
        StatusModel::Fixed { states } => states.iter().map(|value| (*value).to_string()).collect(),
    }
}

/// Build the index for one descriptor. Only the active directory is read.
pub fn build_index(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    status: Option<&str>,
) -> Index {
    build_index_impl(ctx, cfg, registry, descriptor, status, None, false)
}

/// Build one descriptor's index, sharing the request record snapshot's parse
/// cache and title resolution.
pub fn build_index_with_snapshot(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    status: Option<&str>,
    snapshot: &RecordSnapshot<'_>,
) -> Index {
    build_index_impl(
        ctx,
        cfg,
        registry,
        descriptor,
        status,
        Some(snapshot),
        false,
    )
}

/// Build one grouped descriptor's index from front-matter-only records.
///
/// The group and member view and edit pages only need each member's identity,
/// title, status and owning group for their row links. Parsing the document
/// body, the read projection and the field map for hundreds of siblings is
/// wasted work on that path; the page record itself is still read in full.
pub fn build_grouped_index_with_snapshot(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    snapshot: &RecordSnapshot<'_>,
) -> Index {
    build_index_impl(ctx, cfg, registry, descriptor, None, Some(snapshot), true)
}

fn build_index_impl(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    descriptor: &TypeDescriptor,
    status: Option<&str>,
    snapshot: Option<&RecordSnapshot<'_>>,
    light: bool,
) -> Index {
    let mut index = Index::default();
    if !descriptor.enabled {
        return index;
    }
    let Some(dir) = descriptor_directory(ctx, cfg, descriptor) else {
        return index;
    };
    let extension = content_extension(descriptor);
    let Ok(read_dir) = fs::read_dir(&dir) else {
        return index;
    };
    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some(extension) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !matches_stem(descriptor, stem) {
            continue;
        }
        let loaded = match (snapshot, light) {
            (Some(snapshot), true) => snapshot.load_light_cached(descriptor, &path),
            (Some(snapshot), false) => snapshot.load_cached(descriptor, &path),
            (None, _) => load_record(ctx, cfg, registry, descriptor, &path),
        };
        if let Ok(Some(record)) = loaded {
            // A record auto-relocated out of the active directory by the read
            // path is no longer listed.
            if !record.path.exists() {
                continue;
            }
            if let Some(wanted) = status
                && record.status.as_deref() != Some(wanted)
            {
                continue;
            }
            index.records.push(record);
        }
    }
    finalize_index(index.records, descriptor)
}

/// Sort one descriptor's records by ID and derive its grouped-family views.
fn finalize_index(mut records: Vec<Record>, descriptor: &TypeDescriptor) -> Index {
    records.sort_by(|a, b| a.id.cmp(&b.id));
    let mut index = Index {
        records,
        ..Index::default()
    };
    if descriptor.id_shape != IdShape::Grouped {
        return index;
    }
    index.groups = index
        .records
        .iter()
        .filter(|r| r.is_group)
        .cloned()
        .collect();
    index.requirements = index
        .records
        .iter()
        .filter(|r| !r.is_group)
        .cloned()
        .collect();

    let group_ids: BTreeMap<String, usize> = index
        .groups
        .iter()
        .enumerate()
        .map(|(position, group)| (group.id.clone(), position))
        .collect();
    for (position, requirement) in index.requirements.iter().enumerate() {
        let group_id = requirement.group_id.clone().unwrap_or_default();
        if group_id.is_empty() || !group_ids.contains_key(&group_id) {
            index.other.push(position);
        } else {
            index.members.entry(group_id).or_default().push(position);
        }
    }
    index
}

/// Resolve an ID by exact canonical match against the enabled descriptors.
pub fn resolve_id(
    ctx: &Context,
    cfg: &Config,
    registry: &ArtifactRegistry,
    id: &str,
) -> Result<Option<Resolved>, AppError> {
    if !identity::is_valid_id_component(id) {
        return Ok(None);
    }
    for descriptor in registry.descriptors() {
        if !descriptor.enabled {
            continue;
        }
        let Some(dir) = descriptor_directory(ctx, cfg, descriptor) else {
            continue;
        };
        if descriptor.document_model == DocumentModel::SingleHtmlMockup {
            let Some(wanted) = canonical_id(descriptor, id) else {
                continue;
            };
            for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("html") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if canonical_id(descriptor, stem).as_ref() == Some(&wanted)
                    && Context::ensure_within(&ctx.project_dir, &path).is_ok()
                    && let Some(record) = load_record(ctx, cfg, registry, descriptor, &path)?
                {
                    if !record.path.exists() {
                        continue;
                    }
                    return Ok(Some(Resolved { record }));
                }
            }
            continue;
        }
        if canonical_id(descriptor, id).is_none() {
            continue;
        }
        let dirs = vec![dir];
        let prefixes = vec![descriptor.prefix.clone()];
        let dotted = descriptor.id_shape != IdShape::Plain;
        let mut resolved = crate::record::resolve::resolve_in_dirs(
            ctx, cfg, registry, &dirs, id, &prefixes, dotted,
        )
        .ok()
        .flatten();
        if resolved.is_none() && dotted {
            resolved = crate::record::resolve::resolve_in_dirs(
                ctx, cfg, registry, &dirs, id, &prefixes, false,
            )
            .ok()
            .flatten();
        }
        if let Some(found) = resolved {
            if let Some(record) = load_record(ctx, cfg, registry, descriptor, &found.path)? {
                if !record.path.exists() {
                    continue;
                }
                return Ok(Some(Resolved { record }));
            }
            let safe = Context::ensure_within(&ctx.project_dir, &found.path)
                .map_err(|_| AppError::Runtime("record is not readable".to_string()))?;
            if fs::read_to_string(&safe).is_err() {
                return Err(AppError::Runtime("record is not valid UTF-8".to_string()));
            }
        }
    }
    Ok(None)
}

/// Validate a title and return the stored value (specification section 7.3).
pub fn validate_title(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    let chars = value.chars().count();
    if chars == 0 {
        return Err("Title is required.".to_string());
    }
    if chars > MAX_TITLE_CHARS {
        return Err(format!(
            "Title must be at most {} characters.",
            MAX_TITLE_CHARS
        ));
    }
    if value.chars().any(|c| c.is_control()) {
        return Err("Title cannot contain control characters.".to_string());
    }
    Ok(value.to_string())
}

/// Validate a single-line field and return the stored value, or `None` when the
/// value is empty. A single-line value rejects every line break and control
/// character, and never injects a section heading (TECH-040 section 7).
pub fn validate_single_line(label: &str, raw: &str) -> Result<Option<String>, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{} must be at most 20000 characters.", label));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("{} cannot contain control characters.", label));
    }
    if value.starts_with("## ") {
        return Err(format!("{} contains a line starting with ## .", label));
    }
    Ok(Some(value.to_string()))
}

/// Validate a multi-line field and return the stored value, or `None` when the
/// value is empty (specification sections 7.3 and 7.4).
pub fn validate_multiline(label: &str, raw: &str) -> Result<Option<String>, String> {
    let normalized = crate::record::line_endings::unify(raw);
    let value = normalized.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{} must be at most 20000 characters.", label));
    }
    if value
        .chars()
        .any(|c| c != '\n' && c != '\t' && c.is_control())
    {
        return Err(format!("{} cannot contain control characters.", label));
    }
    if value.lines().any(|line| line.starts_with("## ")) {
        return Err(format!("{} contains a line starting with ## .", label));
    }
    Ok(Some(value.to_string()))
}

/// Validate a list field (one value per line) and return the parsed items.
pub fn validate_list_items(label: &str, raw: &str) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    let normalized = crate::record::line_endings::unify(raw);
    for line in normalized.lines() {
        let value = line.trim();
        if value.is_empty() {
            continue;
        }
        let chars = value.chars().count();
        if chars == 0 || chars > MAX_TITLE_CHARS {
            return Err(format!("{} must be at most 200 characters.", label));
        }
        if value.chars().any(|c| c.is_control()) {
            return Err(format!("{} cannot contain control characters.", label));
        }
        if value.starts_with("## ") {
            return Err(format!("{} contains a line starting with ## .", label));
        }
        if !items.contains(&value.to_string()) {
            items.push(value.to_string());
        }
    }
    Ok(items)
}

fn html_escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn title_inner_span(content: &str) -> Option<(usize, usize, bool)> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<title") {
        let open_start = search + rel;
        let after_open = open_start + "<title".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<title".len();
            continue;
        }
        let gt = content[open_start..].find('>')?;
        let inner_start = open_start + gt + 1;
        let close_rel = content[inner_start..].find("</title>")?;
        let inner_end = inner_start + close_rel;
        let raw = &content[inner_start..inner_end];
        let dash = raw.find('—');
        let entity = raw.find("&mdash;");
        let uses_entity = match (dash, entity) {
            (Some(d), Some(e)) => e < d,
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => false,
        };
        return Some((inner_start, inner_end, uses_entity));
    }
    None
}

fn stub_h1_inner_span(content: &str, id: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<h1") {
        let open_start = search + rel;
        let after_open = open_start + "<h1".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<h1".len();
            continue;
        }
        let gt_rel = content[open_start..].find('>')?;
        let inner_start = open_start + gt_rel + 1;
        let close_rel = content[inner_start..].find("</h1>")?;
        let inner_end = inner_start + close_rel;
        let inner = &content[inner_start..inner_end];
        let follows = inner
            .strip_prefix(id)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| c.is_whitespace());
        if follows {
            return Some((inner_start, inner_end));
        }
        search = inner_end + "</h1>".len();
    }
    None
}

fn comment_span(content: &str, key: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    let prefix = format!("{}:", key);
    while let Some(rel) = content[search..].find("<!--") {
        let start = search + rel;
        let end_rel = content[start..].find("-->")?;
        let inner_start = start + "<!--".len();
        let inner_end = start + end_rel;
        let inner = content[inner_start..inner_end].trim();
        if inner.starts_with(&prefix) {
            return Some((start, inner_end + "-->".len()));
        }
        search = inner_end + "-->".len();
    }
    None
}

fn visible_para_inner_span(content: &str, label: &str) -> Option<(usize, usize)> {
    let mut search = 0usize;
    let prefix = format!("{}:", label);
    while let Some(rel) = content[search..].find("<p") {
        let open_start = search + rel;
        let after_open = open_start + "<p".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<p".len();
            continue;
        }
        let gt_rel = content[open_start..].find('>')?;
        let inner_start = open_start + gt_rel + 1;
        let close_rel = content[inner_start..].find("</p>")?;
        let inner_end = inner_start + close_rel;
        let inner = content[inner_start..inner_end].trim();
        if inner.starts_with(&prefix) {
            return Some((inner_start, inner_end));
        }
        search = inner_end + "</p>".len();
    }
    None
}

fn body_open_end(content: &str) -> Option<usize> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<body") {
        let open_start = search + rel;
        let after_open = open_start + "<body".len();
        let after = &content[after_open..];
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(|c| c.is_whitespace()) {
            search = open_start + "<body".len();
            continue;
        }
        let gt = content[open_start..].find('>')?;
        return Some(open_start + gt + 1);
    }
    None
}

/// Field-preserving serialization for UI mockups (specification section 8).
pub fn edit_ui_mockup(
    original: &str,
    id: &str,
    title: &str,
    specs: &[String],
    refs: &[String],
) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    if let Some((start, end, uses_entity)) = title_inner_span(original) {
        let separator = if uses_entity { "&mdash;" } else { "—" };
        edits.push((
            start,
            end,
            format!("{} {} {}", id, separator, html_escape_text(title)),
        ));
    }
    if let Some((start, end)) = stub_h1_inner_span(original, id) {
        edits.push((start, end, format!("{} {}", id, html_escape_text(title))));
    }
    let specs_comment = comment_span(original, "specs");
    let refs_comment = comment_span(original, "refs");
    if let Some((start, end)) = specs_comment {
        edits.push((start, end, format!("<!-- specs: {} -->", specs.join(" "))));
    }
    if let Some((start, end)) = refs_comment {
        edits.push((start, end, format!("<!-- refs: {} -->", refs.join(" "))));
    }
    if let Some((start, end)) = visible_para_inner_span(original, "UI specifications") {
        edits.push((
            start,
            end,
            format!("UI specifications: {}", html_escape_text(&specs.join(" "))),
        ));
    }
    if let Some((start, end)) = visible_para_inner_span(original, "References") {
        edits.push((
            start,
            end,
            format!("References: {}", html_escape_text(&refs.join(" "))),
        ));
    }
    let missing_specs = specs_comment.is_none();
    let missing_refs = refs_comment.is_none();
    if missing_specs || missing_refs {
        let mut block = String::new();
        if missing_specs {
            block.push_str(&format!("<!-- specs: {} -->\n", specs.join(" ")));
        }
        if missing_refs {
            block.push_str(&format!("<!-- refs: {} -->\n", refs.join(" ")));
        }
        match body_open_end(original) {
            Some(pos) => edits.push((pos, pos, format!("\n{}", block))),
            None => {
                let separator = if original.is_empty() || original.ends_with('\n') {
                    String::new()
                } else {
                    "\n".to_string()
                };
                edits.push((
                    original.len(),
                    original.len(),
                    format!("{}{}", separator, block),
                ));
            }
        }
    }
    edits.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut output = original.to_string();
    for (start, end, replacement) in edits {
        output.replace_range(start..end, &replacement);
    }
    output
}

/// Trimmed, non-empty submitted tag values in order (for 422 preservation).
pub fn submitted_tag_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Validate `specs` values (specification section 7.2).
pub fn validate_specs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    if trimmed.is_empty() {
        return Err("At least one UI specification id is required.".to_string());
    }
    for value in &trimmed {
        if !identity::is_valid_id_component(value) {
            return Err("UI specification ids must be valid record IDs.".to_string());
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for value in &trimmed {
        if seen.contains(value) {
            return Err(format!("Duplicate UI specification id: {}.", value));
        }
        seen.push(value.clone());
    }
    Ok(trimmed)
}

/// Validate `refs` values (specification section 7.3).
pub fn validate_refs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    for value in &trimmed {
        if value.chars().count() > 200 {
            return Err("References must be at most 200 characters.".to_string());
        }
        if value.chars().any(|c| c.is_control()) {
            return Err("References cannot contain control characters.".to_string());
        }
        if value.chars().any(|c| c.is_whitespace()) {
            return Err("References cannot contain whitespace.".to_string());
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for value in &trimmed {
        if seen.contains(value) {
            return Err(format!("Duplicate reference: {}.", value));
        }
        seen.push(value.clone());
    }
    Ok(trimmed)
}

/// Validate labels (front-matter list): collapse duplicates, 1-200 chars, no control.
pub fn validate_labels(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    let mut out: Vec<String> = Vec::new();
    for value in trimmed {
        if value.chars().count() > MAX_TITLE_CHARS {
            return Err("Labels must be at most 200 characters.".to_string());
        }
        if value.chars().any(|c| c.is_control()) {
            return Err("Labels cannot contain control characters.".to_string());
        }
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub fn validate_requirement_refs(values: &[String]) -> Result<Vec<String>, String> {
    let trimmed = submitted_tag_values(values);
    for value in &trimmed {
        if !identity::is_valid_id_component(value) {
            return Err("Requirement references must be valid record IDs.".to_string());
        }
    }
    let mut out: Vec<String> = Vec::new();
    for value in trimmed {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

/// The descriptor field spec that backs a resolved field, across both roles.
pub fn document_field_spec<'a>(record: &'a Record, key: &str) -> Option<&'a FieldSpec> {
    let descriptor = &record.descriptor;
    descriptor
        .fields
        .iter()
        .chain(
            descriptor
                .group
                .iter()
                .flat_map(|role| role.fields.iter().flatten()),
        )
        .chain(
            descriptor
                .member
                .iter()
                .flat_map(|role| role.fields.iter().flatten()),
        )
        .find(|spec| spec.key == key)
}

/// The one validator for a submitted scalar document-field value (AUD-145 M1).
///
/// The field domain comes from the descriptor: an allowed-value domain rejects
/// every value outside its list, and the remaining fields follow their kind.
pub fn validate_document_field_value(
    record: &Record,
    field: &ResolvedField,
    raw: &str,
) -> Result<Option<String>, String> {
    if let Some(spec) = document_field_spec(record, &field.key)
        && let FieldValues::Allowed(values) = &spec.values
    {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(format!("{} is required.", field.label));
        }
        if !values.iter().any(|value| value == trimmed) {
            return Err(allowed_values_error(&field.label, values));
        }
        return Ok(Some(trimmed.to_string()));
    }
    if field.kind == FieldKind::Text {
        validate_single_line(&field.label, raw)
    } else {
        validate_multiline(&field.label, raw)
    }
}

/// The one validator for a submitted list document-field value (AUD-145 M1).
pub fn validate_document_field_list(
    record: &Record,
    field: &ResolvedField,
    raw: &[String],
) -> Result<Vec<String>, String> {
    if submitted_tag_values(raw)
        .iter()
        .any(|value| value.chars().any(char::is_control))
    {
        return Err(format!(
            "{} cannot contain control characters.",
            field.label
        ));
    }
    match document_field_spec(record, &field.key).map(|spec| &spec.values) {
        Some(FieldValues::RecordIds) => validate_requirement_refs(raw),
        _ => validate_list_items(&field.label, &raw.join("\n")),
    }
}

/// The message for a scalar outside its declared allowed-value domain.
fn allowed_values_error(label: &str, values: &[String]) -> String {
    let last = values.last().cloned().unwrap_or_default();
    let head = values
        .iter()
        .take(values.len().saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let joined = if head.is_empty() {
        last
    } else {
        format!("{head}, or {last}")
    };
    format!("{label} must be one of {joined}.")
}

/// Whether a decision ID exists in the active or archived store.
pub fn decision_exists(ctx: &Context, cfg: &Config, registry: &ArtifactRegistry, id: &str) -> bool {
    registry.descriptors().any(|descriptor| {
        descriptor.category == CategoryKind::Decision
            && descriptor.enabled
            && descriptor_directory(ctx, cfg, descriptor).is_some_and(|active| {
                let archived = active.join("archived");
                let dirs = if Context::ensure_within(&ctx.project_dir, &archived).is_ok() {
                    vec![active, archived]
                } else {
                    vec![active]
                };
                crate::record::resolve::resolve_in_dirs(
                    ctx,
                    cfg,
                    registry,
                    &dirs,
                    id,
                    std::slice::from_ref(&descriptor.prefix),
                    descriptor.id_shape != IdShape::Plain,
                )
                .ok()
                .flatten()
                .is_some()
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct SnapshotProject {
        root: PathBuf,
        ctx: Context,
        cfg: Config,
        registry: ArtifactRegistry,
    }

    impl SnapshotProject {
        fn new(name: &str) -> SnapshotProject {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".tmp")
                .join(format!(
                    "record-snapshot-{}-{}-{}",
                    name,
                    std::process::id(),
                    stamp
                ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("backlog/tasks")).unwrap();
            fs::create_dir_all(root.join("backlog/milestones")).unwrap();
            fs::create_dir_all(root.join("backlog/releases")).unwrap();
            fs::write(
                root.join("backlog/milestones/MS-001.md"),
                "---\nid: MS-001\ntitle: \"Milestone one\"\ncreated_date: '2026-01-02T03:04:05Z'\nupdated_date: '2026-01-02T03:04:05Z'\nlabels: []\nreferences: []\nstatus: open\nrelease: \"\"\n---\n# MS-001 Milestone one\n\n## Description\nMilestone.\n",
            )
            .unwrap();
            for (id, title) in [("TASK-001", "Task one"), ("TASK-002", "Task two")] {
                fs::write(
                    root.join(format!("backlog/tasks/{}.md", id)),
                    format!(
                        "---\nid: {id}\ntitle: \"{title}\"\ncreated_date: '2026-01-03T04:05:06Z'\nupdated_date: '2026-01-03T04:05:06Z'\nlabels: []\nreferences: []\nstatus: ToDo\nassignee: []\nmilestone: MS-001\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n{title}.\n"
                    ),
                )
                .unwrap();
            }
            let cfg = Config::default();
            let registry = ArtifactRegistry::build(&cfg).unwrap();
            let ctx = Context {
                project_dir: root.clone(),
                config_path: root.join("papyrus.config.yml"),
                json: false,
                no_color: true,
            };
            SnapshotProject {
                root,
                ctx,
                cfg,
                registry,
            }
        }

        fn enabled_directory_count(&self) -> usize {
            self.registry
                .descriptors()
                .filter(|descriptor| {
                    descriptor.enabled
                        && descriptor_directory(&self.ctx, &self.cfg, descriptor).is_some()
                })
                .count()
        }
    }

    impl Drop for SnapshotProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn snapshot_lists_once_and_parses_each_record_once() {
        let project = SnapshotProject::new("counts");
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        assert_eq!(snapshot.parse_count(), 0);

        let task = snapshot.resolve("TASK-001").unwrap().expect("task");
        assert_eq!(
            task.record.milestone_title.as_deref(),
            Some("Milestone one")
        );
        let listings = snapshot.listing_count();
        assert_eq!(listings, project.enabled_directory_count());
        let parses = snapshot.parse_count();
        assert!(parses >= 2, "task and milestone parsed: {parses}");

        // Memoised for the rest of the request: no listing and no re-parse.
        snapshot.resolve("TASK-001").unwrap().expect("task again");
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses);

        // A second distinct record parses once; its milestone is already cached.
        snapshot.resolve("TASK-002").unwrap().expect("second task");
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses + 1);

        // Unknown IDs are one map read: no rescan and no parse.
        assert!(snapshot.resolve("TASK-999").unwrap().is_none());
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses + 1);
    }

    #[test]
    fn snapshot_parse_cache_is_shared_with_index_building() {
        let project = SnapshotProject::new("index");
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let descriptor = project
            .registry
            .descriptor("task")
            .expect("task descriptor")
            .clone();
        let index = build_index_with_snapshot(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &descriptor,
            None,
            &snapshot,
        );
        assert_eq!(index.records.len(), 2);
        let listings = snapshot.listing_count();
        let parses = snapshot.parse_count();
        assert_eq!(listings, project.enabled_directory_count());

        // Records parsed while building the index are already memoised.
        snapshot.resolve("TASK-001").unwrap().expect("task");
        snapshot.resolve("MS-001").unwrap().expect("milestone");
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses);
    }

    #[test]
    fn registry_scan_lists_each_directory_and_parses_each_record_once() {
        let project = SnapshotProject::new("scan");
        let business = project.root.join("docs/requirements/business");
        fs::create_dir_all(&business).unwrap();
        fs::write(
            business.join("REQ-001.md"),
            "---\nid: REQ-001\ntitle: \"Group one\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: []\nreferences: []\nstatus: active\n---\n# REQ-001 Group one\n\n## Description\nGroup.\n",
        )
        .unwrap();
        fs::write(
            business.join("REQ-001.001.md"),
            "---\nid: REQ-001.001\ntitle: \"Member one\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: []\nreferences: []\nstatus: active\ngroup: REQ-001\n---\n# REQ-001.001 Member one\n\n## Description\nMember.\n",
        )
        .unwrap();

        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let indexes = snapshot.scan_indexes();
        let listings = snapshot.listing_count();
        let parses = snapshot.parse_count();
        assert_eq!(listings, project.enabled_directory_count());
        assert_eq!(parses, 5, "two tasks, one milestone, and two requirements");
        let conflict_scans = snapshot.conflict_scan_count();
        assert_eq!(
            conflict_scans, 3,
            "one conflict scan for each of the task, milestone, and req families"
        );
        assert_eq!(indexes["task"].records.len(), 2);
        assert_eq!(indexes["milestone"].records.len(), 1);
        assert_eq!(indexes["req"].records.len(), 2);
        assert_eq!(indexes["req"].groups.len(), 1);
        assert_eq!(indexes["req"].requirements.len(), 1);
        assert_eq!(indexes["req"].members["REQ-001"], vec![0]);

        // A grouped stem appears under both lookup keys; the file still
        // contributes exactly one record and is parsed once.
        let ids: Vec<&str> = indexes["req"]
            .records
            .iter()
            .map(|record| record.id.as_str())
            .collect();
        assert_eq!(ids, vec!["REQ-001", "REQ-001.001"]);

        // Repeated inventory scans reuse the listing and parse cache.
        let again = snapshot.scan_indexes();
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses);
        assert_eq!(snapshot.conflict_scan_count(), conflict_scans);
        assert_eq!(again["req"].records.len(), 2);
        snapshot.resolve("TASK-001").unwrap().expect("cached task");
        assert_eq!(snapshot.listing_count(), listings);
        assert_eq!(snapshot.parse_count(), parses);
        assert_eq!(snapshot.conflict_scan_count(), conflict_scans);

        // The scan result matches the per-descriptor index builder exactly.
        let descriptor = project.registry.descriptor("req").expect("req").clone();
        let built = build_index_with_snapshot(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &descriptor,
            None,
            &snapshot,
        );
        let scanned: Vec<&str> = indexes["req"]
            .records
            .iter()
            .map(|record| record.id.as_str())
            .collect();
        let built_ids: Vec<&str> = built
            .records
            .iter()
            .map(|record| record.id.as_str())
            .collect();
        assert_eq!(scanned, built_ids);
    }

    #[test]
    fn snapshot_reports_a_listed_but_unreadable_record() {
        let project = SnapshotProject::new("unreadable");
        fs::write(
            project.root.join("backlog/tasks/TASK-003.md"),
            [0xff, 0xfe, 0x00, 0x01],
        )
        .unwrap();
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let error = snapshot.resolve("TASK-003").expect_err("diagnostic");
        assert_eq!(error.to_string(), "record is not valid UTF-8");
    }

    #[test]
    fn snapshot_is_display_only_and_writes_still_read_live_bytes() {
        let project = SnapshotProject::new("display-only");
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let cached = snapshot.resolve("TASK-001").unwrap().expect("task").record;
        let path = cached.path.clone();
        let before = crate::record::store::read_document_normalized(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &path,
        )
        .unwrap();
        assert_eq!(before.version, cached.version);

        std::thread::sleep(std::time::Duration::from_millis(2));
        fs::write(
            &path,
            "---\nid: TASK-001\ntitle: \"Live title\"\ncreated_date: '2026-01-03T04:05:06Z'\nupdated_date: '2026-01-03T04:05:07Z'\nlabels: []\nreferences: []\nstatus: ToDo\nassignee: []\nmilestone: MS-001\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\nLive.\n",
        )
        .unwrap();

        // The display snapshot keeps the record it parsed earlier.
        let memoised = snapshot
            .resolve("TASK-001")
            .unwrap()
            .expect("memoised")
            .record;
        assert_eq!(memoised.title, "Task one");
        // A write-path read always sees the live bytes and token.
        let live = crate::record::store::read_document_normalized(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &path,
        )
        .unwrap();
        assert_eq!(
            live.version,
            crate::record::store::version_token(&fs::metadata(&path).unwrap())
        );
        assert_ne!(live.version, memoised.version);
        assert!(live.stored_source.contains("Live title"));
    }

    #[test]
    fn title_and_multiline_validation() {
        assert!(validate_title("  Title  ").is_ok());
        assert_eq!(validate_title("").unwrap_err(), "Title is required.");
        assert!(validate_title(&"x".repeat(201)).is_err());
        assert_eq!(
            validate_title(&"x".repeat(201)).unwrap_err(),
            "Title must be at most 200 characters."
        );
        assert_eq!(
            validate_title("a\tb").unwrap_err(),
            "Title cannot contain control characters."
        );
        assert_eq!(
            validate_multiline("Description", "  a\r\nb  ").unwrap(),
            Some("a\nb".to_string())
        );
        assert_eq!(validate_multiline("Description", "   ").unwrap(), None);
        assert_eq!(
            validate_multiline("Description", "a\n## Section\nb").unwrap_err(),
            "Description contains a line starting with ## ."
        );
        assert_eq!(
            validate_multiline("Content", "a\u{0007}b").unwrap_err(),
            "Content cannot contain control characters."
        );
        assert!(validate_multiline("Description", "a\tb").is_ok());
    }

    #[test]
    fn list_validation() {
        assert_eq!(
            validate_list_items("Labels", "one\n two \none\n").unwrap(),
            vec!["one".to_string(), "two".to_string()]
        );
        assert_eq!(
            validate_list_items("Labels", "## section").unwrap_err(),
            "Labels contains a line starting with ## ."
        );
    }

    #[test]
    fn ui_mockup_specs_validation() {
        assert_eq!(
            validate_specs(&["UI-001".to_string(), " UI-002 ".to_string()]).unwrap(),
            vec!["UI-001".to_string(), "UI-002".to_string()]
        );
        assert_eq!(
            validate_specs(&["UI-001".to_string(), "".to_string(), "  ".to_string()]).unwrap(),
            vec!["UI-001".to_string()]
        );
        assert_eq!(
            validate_specs(&[]).unwrap_err(),
            "At least one UI specification id is required."
        );
        assert_eq!(
            validate_specs(&["bad id".to_string()]).unwrap_err(),
            "UI specification ids must be valid record IDs."
        );
        assert_eq!(
            validate_specs(&["UI-001".to_string(), "UI-001".to_string()]).unwrap_err(),
            "Duplicate UI specification id: UI-001."
        );
    }

    #[test]
    fn ui_mockup_refs_validation() {
        assert_eq!(
            validate_refs(&["docs/a.md".to_string()]).unwrap(),
            vec!["docs/a.md".to_string()]
        );
        assert_eq!(
            validate_refs(&["x".repeat(201)]).unwrap_err(),
            "References must be at most 200 characters."
        );
        assert_eq!(
            validate_refs(&["a b".to_string()]).unwrap_err(),
            "References cannot contain whitespace."
        );
        assert_eq!(
            validate_refs(&["docs/a.md".to_string(), "docs/a.md".to_string()]).unwrap_err(),
            "Duplicate reference: docs/a.md."
        );
    }

    #[test]
    fn ui_mockup_stub_round_trip() {
        let original = "<!doctype html>\n<html><head><title>MOCK-001 — Old</title></head><body>\n<!-- mockup-id: MOCK-001 -->\n<!-- status: pending -->\n<!-- task: T -->\n<!-- specs: UI-001 -->\n<!-- refs: docs/old.md -->\n<h1>MOCK-001 Old</h1>\n<p>Status: pending</p>\n<p>Task: T</p>\n<p>UI specifications: UI-001</p>\n<p>References: docs/old.md</p>\n<p>Superseded by: </p>\n</body></html>\n";
        let output = edit_ui_mockup(
            original,
            "MOCK-001",
            "New",
            &["UI-002".to_string()],
            &["docs/new.md".to_string()],
        );
        assert!(output.contains("<title>MOCK-001 — New</title>"));
        assert!(output.contains("<h1>MOCK-001 New</h1>"));
        assert!(output.contains("<!-- specs: UI-002 -->"));
        assert!(output.contains("<!-- refs: docs/new.md -->"));
        assert!(output.contains("<p>UI specifications: UI-002</p>"));
        assert!(output.contains("<p>References: docs/new.md</p>"));
        assert!(output.contains("<!-- mockup-id: MOCK-001 -->"));
        assert!(output.contains("<p>Superseded by: </p>"));
    }

    #[test]
    fn ui_mockup_review_package_preserved() {
        let original = "<!doctype html>\n<html><head><title>MOCK-003 &mdash; Old review</title><style>body{color:red}</style><script>var s=1;</script></head><body class=\"x\">\n<h1>TASK-99 Landing</h1>\n<section>Slides &amp; more</section>\n</body></html>\n";
        let output = edit_ui_mockup(
            original,
            "MOCK-003",
            "New review",
            &["UI-010".to_string()],
            &["docs/x.md".to_string()],
        );
        assert!(output.contains("<title>MOCK-003 &mdash; New review</title>"));
        assert!(output.contains("<style>body{color:red}</style>"));
        assert!(output.contains("<script>var s=1;</script>"));
        assert!(output.contains("<h1>TASK-99 Landing</h1>"));
        assert!(output.contains("<!-- specs: UI-010 -->"));
        assert!(output.contains("<!-- refs: docs/x.md -->"));
    }
    #[test]
    fn scan_counts_match_full_index_record_counts() {
        let project = SnapshotProject::new("counts-match");
        let business = project.root.join("docs/requirements/business");
        fs::create_dir_all(&business).unwrap();
        fs::write(
            business.join("REQ-001.md"),
            "---\nid: REQ-001\ntitle: \"Group one\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: []\nreferences: []\nstatus: active\n---\n# REQ-001 Group one\n\n## Description\nGroup.\n",
        )
        .unwrap();
        fs::write(
            business.join("REQ-001.001.md"),
            "---\nid: REQ-001.001\ntitle: \"Member one\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: []\nreferences: []\nstatus: active\ngroup: REQ-001\n---\n# REQ-001.001 Member one\n\n## Description\nMember.\n",
        )
        .unwrap();
        let mockups = project.root.join("docs/specifications/ui/mockups");
        fs::create_dir_all(&mockups).unwrap();
        fs::write(
            mockups.join("MOCK-001.html"),
            "<!doctype html>\n<html><head><title>MOCK-001 \u{2014} Mock one</title></head><body>\n<!-- mockup-id: MOCK-001 -->\n<!-- status: pending -->\n<!-- task: TASK-001 -->\n<!-- specs: UI-001 -->\n<!-- refs: docs/a.md -->\n<h1>MOCK-001 Mock one</h1>\n</body></html>\n",
        )
        .unwrap();

        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let counts = snapshot.scan_counts();
        let indexes = snapshot.scan_indexes();
        assert_eq!(counts.len(), indexes.len());
        for (slug, index) in &indexes {
            assert_eq!(
                counts.get(slug).copied().unwrap_or(0),
                index.records.len(),
                "count mismatch for {slug}"
            );
        }
        assert_eq!(counts["req"], 2);
        assert_eq!(counts["ui-mockup"], 1);
    }

    #[test]
    fn grouped_light_index_matches_the_full_index_shape() {
        let project = SnapshotProject::new("light-index");
        let business = project.root.join("docs/requirements/business");
        fs::create_dir_all(&business).unwrap();
        fs::write(
            business.join("REQ-001.md"),
            "---\nid: REQ-001\ntitle: \"Group one\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: [group]\nreferences: []\nstatus: active\n---\n# REQ-001 Group one\n\n## Description\nGroup.\n",
        )
        .unwrap();
        for (id, title) in [("REQ-001.001", "Member one"), ("REQ-001.002", "Member two")] {
            fs::write(
                business.join(format!("{id}.md")),
                format!(
                    "---\nid: {id}\ntitle: \"{title}\"\ncreated_date: '2026-01-04T00:00:00Z'\nupdated_date: '2026-01-04T00:00:00Z'\nlabels: [member]\nreferences: []\nstatus: active\n---\n# {id} {title}\n\n## Group\nREQ-001 Group one\n\n## Description\nMember.\n"
                ),
            )
            .unwrap();
        }

        let descriptor = project.registry.descriptor("req").expect("req").clone();
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let full = build_index_with_snapshot(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &descriptor,
            None,
            &snapshot,
        );
        let light = build_grouped_index_with_snapshot(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &descriptor,
            &snapshot,
        );
        assert_eq!(light.records.len(), full.records.len());
        for (light, full) in light.records.iter().zip(&full.records) {
            assert_eq!(light.id, full.id);
            assert_eq!(light.title, full.title);
            assert_eq!(light.status, full.status);
            assert_eq!(light.group_id, full.group_id);
            assert_eq!(light.group_title, full.group_title);
            assert_eq!(light.is_group, full.is_group);
            assert_eq!(light.labels, full.labels);
            assert_eq!(light.refs, full.refs);
        }
        assert_eq!(light.groups.len(), full.groups.len());
        assert_eq!(light.requirements.len(), full.requirements.len());
        assert_eq!(light.members, full.members);
        assert_eq!(light.other, full.other);
    }

    #[test]
    fn snapshot_source_matches_the_shared_store_read() {
        let project = SnapshotProject::new("source-parity");
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let target = project.root.join("backlog/tasks/TASK-001.md");
        let fast = snapshot.read_snapshot_source(&target).expect("fast read");
        let slow = crate::record::store::read_normalized_source_with_conflicts(
            &project.ctx,
            &project.cfg,
            &project.registry,
            &target,
            None,
        )
        .expect("store read");
        assert_eq!(fast.0, slow.0);
        assert_eq!(fast.1, slow.1);
        assert!(fast.2.is_some());
    }

    #[test]
    fn snapshot_source_falls_back_to_adoption_for_legacy_records() {
        let project = SnapshotProject::new("source-adoption");
        let target = project.root.join("backlog/tasks/TASK-009.md");
        fs::write(
            &target,
            "# TASK-009 Legacy task\n\n## Description\nLegacy.\n",
        )
        .unwrap();
        let snapshot = RecordSnapshot::new(&project.ctx, &project.cfg, &project.registry);
        let (_, normalized, front_matter) = snapshot
            .read_snapshot_source(&target)
            .expect("adoption read");
        assert!(normalized.starts_with("---\n"));
        assert!(front_matter.is_some());
        assert_eq!(
            normalized,
            crate::record::store::read_normalized_source_with_conflicts(
                &project.ctx,
                &project.cfg,
                &project.registry,
                &target,
                None,
            )
            .expect("store read")
            .1
        );
    }
}
