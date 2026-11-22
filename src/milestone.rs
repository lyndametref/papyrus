use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum MilestoneCmd {
    /// Create a milestone.
    Create {
        /// Milestone title.
        #[arg(value_name = "title")]
        title: String,
        /// Milestone description.
        #[arg(
            short = 'd',
            long = "description",
            help = crate::sanitize::MULTILINE_STDIN_HELP
        )]
        description: Option<String>,
        /// Assign the milestone to a release.
        #[arg(long = "release")]
        release: Option<String>,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// List milestones.
    List,
    /// View a milestone.
    View { id: String },
    /// Edit a milestone.
    Edit {
        id: String,
        /// Update the title.
        #[arg(long = "title")]
        title: Option<String>,
        /// Update the description.
        #[arg(
            short = 'd',
            long = "description",
            help = crate::sanitize::MULTILINE_STDIN_HELP
        )]
        description: Option<String>,
        /// Assign the milestone to a release.
        #[arg(long = "release")]
        release: Option<String>,
        /// Remove the release assignment.
        #[arg(long = "no-release", num_args = 0..=1)]
        no_release: Option<Option<String>>,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// Close a milestone (`open -> closed`).
    Close {
        /// Milestone ID.
        id: String,
    },
    /// Archive a milestone (`open -> closed -> archived`).
    Archive {
        /// Milestone ID.
        id: String,
    },
    /// Permanently delete a milestone.
    Delete {
        id: String,
        /// Skip confirmation prompt.
        #[arg(long)]
        force: bool,
    },
}

pub fn run(ctx: &Context, cmd: MilestoneCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        MilestoneCmd::Create {
            title,
            description,
            release,
            label,
            reference,
        } => {
            let description = sanitize::resolve_text_field("description", description)?;
            let id = create(
                ctx,
                &title,
                description,
                release.as_deref(),
                &label,
                &reference,
            )?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput {
                text,
                json,
                ..Default::default()
            })
        }
        MilestoneCmd::List => list(ctx),
        MilestoneCmd::View { id } => view(ctx, &id),
        MilestoneCmd::Edit {
            id,
            title,
            description,
            release,
            no_release,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            sanitize::resolve_text_field("description", description)?,
            release.as_deref(),
            no_release,
            &label,
            &reference,
        ),
        MilestoneCmd::Close { id } => close(ctx, &id),
        MilestoneCmd::Archive { id } => archive(ctx, &id),
        MilestoneCmd::Delete { id, force } => delete(ctx, &id, force),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn unique_push_ms(vec: &mut Vec<String>, val: String) {
    if !vec.contains(&val) {
        vec.push(val);
    }
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn milestone_descriptor() -> crate::record::descriptor::RecordDescriptor {
    crate::record::descriptor::descriptor_for("milestone").expect("milestone descriptor")
}

fn milestone_id_spec(cfg: &Config) -> crate::record::identity::IdSpec {
    crate::record::identity::IdSpec::new(cfg.milestone_prefix.clone(), cfg.id_padding as usize)
}

fn active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    milestone_descriptor().resolve_directory(cfg, &ctx.project_dir)
}

fn read_dirs(ctx: &Context, cfg: &Config) -> Vec<PathBuf> {
    milestone_descriptor().resolve_read_directories(cfg, &ctx.project_dir)
}

/// Resolve a milestone by its Papyrus ID in the configured read locations.
pub(crate) fn resolve_milestone_entry(ctx: &Context, cfg: &Config, id: &str) -> Option<PathBuf> {
    let spec = milestone_id_spec(cfg);
    let wanted = spec.parse_id(id).ok()?;
    for dir in read_dirs(ctx, cfg) {
        if let Some(path) = find_milestone_file(&dir, &spec, wanted) {
            return Some(path);
        }
    }
    None
}

fn resolve_milestone_path(ctx: &Context, cfg: &Config, id: &str) -> Result<PathBuf, AppError> {
    resolve_milestone_entry(ctx, cfg, id)
        .ok_or_else(|| AppError::NotFound(format!("not found: {}", id)))
}

fn resolve_milestone_path_migrated(
    ctx: &Context,
    cfg: &Config,
    id: &str,
) -> Result<PathBuf, AppError> {
    let path = resolve_milestone_path(ctx, cfg, id)?;
    crate::record::migrate::ensure_front_matter_canonical(ctx, &path)
}

fn milestone_frontmatter_id(content: &str) -> Option<String> {
    if !crate::record::frontmatter::has_front_matter(content) {
        return None;
    }
    let fm = crate::record::frontmatter::parse(content).ok().flatten()?;
    let id = crate::record::frontmatter::current_value_string(&fm.raw, "id")?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(id.to_string())
}

fn reject_milestone_write_conflict(ctx: &Context, path: &std::path::Path) -> Result<(), AppError> {
    let content = fs::read_to_string(path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let desc = milestone_descriptor();
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, &desc);
    if let Some(reason) =
        crate::record::migrate::check_conflicts_for_path(ctx, path, &desc, &content, &conflicts)
    {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            crate::record::migrate::rel_path(&ctx.project_dir, path),
            reason
        )));
    }
    Ok(())
}

/// Find a milestone file by its Papyrus number under the configured prefix.
///
/// Backlog-shaped stems (for example `m-0`) resolve to no record. The
/// front-matter `id` is authoritative on a stem mismatch, but only for a
/// native stem.
fn find_milestone_file(
    dir: &std::path::Path,
    spec: &crate::record::identity::IdSpec,
    wanted: i64,
) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    let rd = fs::read_dir(dir).ok()?;
    let mut entries: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        entries.push(path);
    }
    for path in &entries {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if crate::record::identity::id_number(stem, &spec.prefix)
            .is_some_and(|number| i64::from(number) == wanted)
        {
            return Some(path.clone());
        }
    }
    // ADR-0008 decision 3/9: front-matter id is canonical; resolve a record
    // addressed by its front-matter ID even on a stem mismatch. Reads succeed;
    // writes refuse via reject_milestone_write_conflict.
    for path in &entries {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if crate::record::identity::id_number(stem, &spec.prefix).is_none() {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if let Some(fm_id) = milestone_frontmatter_id(&content)
            && spec.parse_id(&fm_id).ok() == Some(wanted)
        {
            return Some(path.clone());
        }
    }
    None
}

fn releases_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let desc = crate::record::descriptor::descriptor_for("release")
        .ok_or_else(|| AppError::Runtime("release descriptor is unavailable".to_string()))?;
    desc.resolve_directory(cfg, &ctx.project_dir)
}

fn ensure_dir(dir: &std::path::Path, project_dir: &std::path::Path) -> Result<(), AppError> {
    let safe = Context::ensure_within(project_dir, dir)?;
    fs::create_dir_all(&safe).map_err(|e| AppError::Runtime(e.to_string()))
}

fn release_exists(ctx: &Context, cfg: &Config, id: &str) -> bool {
    let dir = match releases_dir(ctx, cfg) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let path = dir.join(format!("{}.md", id));
    Context::ensure_within(&ctx.project_dir, &path)
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn frontmatter_list_ms(content: &str, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(Some(fm)) = crate::record::frontmatter::parse(content)
        && let Some(yaml_serde::Value::Sequence(seq)) =
            fm.data.get(yaml_serde::Value::String(key.to_string()))
    {
        for v in seq {
            if let yaml_serde::Value::String(s) = v {
                out.push(s.clone());
            }
        }
    }
    out
}

fn milestone_from_document(
    doc: &crate::record::document::Document,
) -> (String, String, Option<String>, Option<String>) {
    let (id, title) = crate::record::document::heading_id_title_with_frontmatter(doc);
    let desc = crate::record::document::section_text(doc, "Description");
    let release = match doc.front_matter.as_ref() {
        Some(fm) if crate::record::frontmatter::contains_key(&fm.raw, "release") => {
            crate::record::frontmatter::current_value_string(&fm.raw, "release")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        }
        _ => crate::record::document::section_text(doc, "Release").map(|s| s.trim().to_string()),
    };
    (id, title, desc, release)
}

fn body_document(
    _id: &str,
    _title: &str,
    desc: Option<&str>,
    _release: Option<&str>,
) -> crate::record::document::Document {
    let mut sections = Vec::new();
    if let Some(d) = desc {
        sections.push(crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![d.to_string(), String::new()],
        });
    }
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: None,
        preamble: vec![String::new()],
        sections,
    }
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    desc_raw: Option<sanitize::Text>,
    release_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let desc = match &desc_raw {
        Some(text) => Some(text.resolve()?),
        None => None,
    };
    let release = match release_raw {
        Some(s) => {
            let rid = sanitize_text(s)?;
            if !release_exists(ctx, &cfg, &rid) {
                return Err(AppError::NotFound(format!("release not found: {}", rid)));
            }
            Some(rid)
        }
        None => None,
    };

    let mut labels: Vec<String> = Vec::new();
    for l in labels_raw {
        unique_push_ms(&mut labels, sanitize_text(l)?);
    }
    let mut references: Vec<String> = Vec::new();
    for r in references_raw {
        unique_push_ms(
            &mut references,
            sanitize::sanitize_path_string(r)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?,
        );
    }

    let active = active_dir(ctx, &cfg)?;
    ensure_dir(&active, &ctx.project_dir)?;
    let dirs = read_dirs(ctx, &cfg);
    let dir_refs: Vec<&std::path::Path> = dirs.iter().map(PathBuf::as_path).collect();
    let next = crate::record::identity::next_number_in_dirs(&dir_refs, &cfg.milestone_prefix);
    let id = crate::record::identity::format_id(
        &cfg.milestone_prefix,
        next as i64,
        cfg.id_padding as usize,
    );

    let body_doc = body_document(&id, &title, desc.as_deref(), release.as_deref());
    let body = crate::record::document::render(&body_doc);
    let parsed = crate::record::document::parse(&body)?;
    let now = crate::record::frontmatter::utc_now();
    let labels_value = if labels.is_empty() {
        crate::record::frontmatter::FrontMatterValue::String("[]".to_string())
    } else {
        crate::record::frontmatter::FrontMatterValue::String(labels.join("\n"))
    };
    let refs_value = if references.is_empty() {
        crate::record::frontmatter::FrontMatterValue::String("[]".to_string())
    } else {
        crate::record::frontmatter::FrontMatterValue::String(references.join("\n"))
    };
    let release_value = if let Some(r) = &release {
        crate::record::frontmatter::FrontMatterValue::String(r.clone())
    } else {
        crate::record::frontmatter::FrontMatterValue::Null
    };
    let ops = vec![
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "id".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(id.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "title".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(title.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "created_date".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(now.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "updated_date".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(now),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue("labels".to_string(), labels_value),
        crate::record::patch::PatchOp::SetFrontMatterValue("references".to_string(), refs_value),
        crate::record::patch::PatchOp::SetFrontMatterValue("release".to_string(), release_value),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "status".to_string(),
            crate::record::frontmatter::FrontMatterValue::String("open".to_string()),
        ),
    ];
    let file_content = crate::record::patch::apply_patch(&parsed, &ops)?;
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &file_content)?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let mut items: Vec<(String, String, String)> = Vec::new();
    let desc = crate::record::descriptor::descriptor_for("milestone").unwrap();
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, &desc);
    if let Ok(rd) = fs::read_dir(&active) {
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            if crate::record::identity::id_number(&id, &cfg.milestone_prefix).is_none() {
                // Backlog-shaped stems are not Papyrus records.
                continue;
            }
            let safe = Context::ensure_within(&ctx.project_dir, &path)?;
            let content = fs::read_to_string(&safe).unwrap_or_default();
            if let Some(conflict) = crate::record::migrate::path_migration_conflict(
                ctx, &safe, &desc, &content, &conflicts,
            ) {
                eprintln!("warning: {}", conflict);
                continue;
            }
            let (fm_id, parsed_title) = crate::record::document::parse(&content)
                .ok()
                .map(|d| crate::record::document::heading_id_title_with_frontmatter(&d))
                .unwrap_or_default();
            let display_id = if milestone_id_spec(&cfg).parse_id(&fm_id).is_ok() {
                fm_id
            } else {
                id.clone()
            };
            let title = if parsed_title.is_empty() {
                display_id.clone()
            } else {
                parsed_title
            };
            items.push((display_id, title, content));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, title, content)| {
            crate::record::view::RecordView::lenient(content, id).list_line(title)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, title, content)| {
                serde_json::json!({
                    "id": id,
                    "title": title,
                    "metadata": crate::record::view::RecordView::lenient(content, id).metadata_json(),
                })
            })
            .collect();
        Some(serde_json::Value::Array(arr))
    } else {
        None
    };

    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

pub fn view(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path_migrated(ctx, &cfg, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;

    let text = content.clone();
    let json = if ctx.json {
        Some(serde_json::json!({
            "id": id,
            "path": path.display().to_string(),
            "content": content,
            "metadata": crate::record::view::RecordView::lenient(&content, &id).metadata_json(),
        }))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    desc_raw: Option<sanitize::Text>,
    release_raw: Option<&str>,
    no_release: Option<Option<String>>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    if release_raw.is_some() && no_release.is_some() {
        return Err(AppError::Validation(
            "mutually exclusive flags: --release and --no-release".to_string(),
        ));
    }

    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path_migrated(ctx, &cfg, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let doc = crate::record::document::parse(&content)?;
    let desc = crate::record::descriptor::descriptor_for("milestone").unwrap();
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, &desc);
    if let Some(reason) =
        crate::record::migrate::check_conflicts_for_path(ctx, &path, &desc, &content, &conflicts)
    {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            crate::record::migrate::rel_path(&ctx.project_dir, &path),
            reason
        )));
    }

    let (parsed_id, parsed_title, parsed_desc, parsed_release) = milestone_from_document(&doc);
    let mut title_val = if parsed_title.is_empty() {
        id.clone()
    } else {
        parsed_title
    };
    let mut desc_val = parsed_desc;
    let mut release_val = parsed_release;

    if let Some(t) = title_raw {
        title_val = sanitize_text(t)?;
    }
    if let Some(d) = &desc_raw {
        desc_val = Some(d.resolve()?);
    }
    if let Some(r) = release_raw {
        let rid = sanitize_text(r)?;
        if !release_exists(ctx, &cfg, &rid) {
            return Err(AppError::NotFound(format!("release not found: {}", rid)));
        }
        release_val = Some(rid);
    }
    if let Some(no) = &no_release {
        match no {
            Some(rid_raw) => {
                let rid = sanitize_text(rid_raw)?;
                if release_val.as_deref() != Some(rid.as_str()) {
                    return Err(AppError::Validation(format!(
                        "milestone is not assigned to release: {}",
                        rid
                    )));
                }
                release_val = None;
            }
            None => {
                release_val = None;
            }
        }
    }

    let new_id = if parsed_id.is_empty() {
        id.clone()
    } else {
        parsed_id
    };
    // Retire the legacy title heading and `## Release` section on every
    // write; explicit user operations are applied afterwards so they win.
    let mut ops: Vec<crate::record::patch::PatchOp> =
        crate::record::migrate::retire_legacy_body_ops(&doc, &desc, &path);
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            title_val.clone(),
        ));
    }
    if desc_raw.is_some() {
        if let Some(d) = &desc_val {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Description".into(),
                d.clone(),
            ));
        } else {
            ops.push(crate::record::patch::PatchOp::ClearSection(
                "## Description".into(),
            ));
        }
    }
    if release_raw.is_some()
        && let Some(r) = &release_val
    {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "release".into(),
            r.clone(),
        ));
    }
    if no_release.is_some() {
        ops.push(crate::record::patch::PatchOp::ClearFrontMatter(
            "release".into(),
        ));
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_ms(&content, "labels");
        for l in labels_raw {
            let lv = sanitize_text(l)?;
            if !fm_labels.contains(&lv) {
                fm_labels.push(lv);
            }
        }
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "labels".into(),
            fm_labels.join("\n"),
        ));
    }
    if !references_raw.is_empty() {
        let mut fm_refs = frontmatter_list_ms(&content, "references");
        for r in references_raw {
            let rv = sanitize::sanitize_path_string(r)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
            if !fm_refs.contains(&rv) {
                fm_refs.push(rv);
            }
        }
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "references".into(),
            fm_refs.join("\n"),
        ));
    }

    let base_new = crate::record::patch::apply_patch(&doc, &ops)?;
    let new_content = if ops.is_empty() || base_new == content {
        content.clone()
    } else {
        let mut ops2 = ops.clone();
        let now = crate::record::frontmatter::utc_now();
        ops2.push(crate::record::patch::PatchOp::SetFrontMatter(
            "updated_date".into(),
            now,
        ));
        crate::record::patch::apply_patch(&doc, &ops2)?
    };

    if new_content != content {
        let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
        crate::record::store::write_atomic(ctx, &safe_write, &new_content)?;
    }

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": new_id}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
}

/// Close a milestone: the only `open -> closed` transition.
///
/// A missing or empty stored `status` is treated as `open` so imported and
/// legacy records remain closable. Any other status is rejected with a
/// validation outcome (exit 4) before any write, leaving the file untouched.
pub fn close(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path(ctx, &cfg, &requested)?;
    reject_milestone_write_conflict(ctx, &path)?;
    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let doc = crate::record::document::parse(&content)?;
    let id = doc
        .front_matter
        .as_ref()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "id"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(requested);
    let current = doc
        .front_matter
        .as_ref()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "status"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "open".to_string());
    if current != "open" {
        return Err(AppError::Validation(format!(
            "milestone {} is already {}; only open milestones can be closed",
            id, current
        )));
    }

    let now = crate::record::frontmatter::utc_now();
    let new_content = crate::record::patch::apply_patch(
        &doc,
        &[
            crate::record::patch::PatchOp::SetFrontMatter("status".into(), "closed".into()),
            crate::record::patch::PatchOp::SetFrontMatter("updated_date".into(), now),
        ],
    )?;
    let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
    crate::record::store::write_atomic(ctx, &safe_write, &new_content)?;

    let json = ctx
        .json
        .then(|| serde_json::json!({"id": id, "status": "closed"}));
    Ok(crate::CommandOutput {
        text: id,
        json,
        ..Default::default()
    })
}

/// Archive a milestone: the only transition to `archived`.
///
/// An open milestone is archived in a single write that sets its status to
/// `archived`; a closed milestone is archived directly. An already archived
/// milestone is a no-op success. Any other stored status is rejected with a
/// validation outcome (exit 4) before any write, leaving the file untouched.
/// The record is archived by writing the updated content to the milestone
/// lifecycle archive folder with the shared atomic writer and then unlinking
/// the active-folder source. If that unlink fails, the written destination is
/// removed again before returning an error, so only the source copy remains.
/// Title, description, labels, references, and release assignment are
/// preserved.
pub fn archive(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path(ctx, &cfg, &requested)?;
    reject_milestone_write_conflict(ctx, &path)?;
    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let doc = crate::record::document::parse(&content)?;
    let id = doc
        .front_matter
        .as_ref()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "id"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(requested);
    let current = doc
        .front_matter
        .as_ref()
        .and_then(|fm| crate::record::frontmatter::current_value_string(&fm.raw, "status"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "open".to_string());

    match current.as_str() {
        "archived" => {
            let json = ctx
                .json
                .then(|| serde_json::json!({"id": id, "status": "archived"}));
            return Ok(crate::CommandOutput {
                text: id,
                json,
                ..Default::default()
            });
        }
        "open" | "closed" => {}
        _ => {
            return Err(AppError::Validation(format!(
                "milestone {} has status {}; only open, closed, or archived milestones can be archived",
                id, current
            )));
        }
    }

    let now = crate::record::frontmatter::utc_now();
    let new_content = crate::record::patch::apply_patch(
        &doc,
        &[
            crate::record::patch::PatchOp::SetFrontMatter("status".into(), "archived".into()),
            crate::record::patch::PatchOp::SetFrontMatter("updated_date".into(), now),
        ],
    )?;

    let archive_dir = milestone_descriptor()
        .resolve_archive_directory(&cfg, &ctx.project_dir)
        .ok_or_else(|| {
            AppError::Runtime("milestone has no lifecycle archive location".to_string())
        })??;
    fs::create_dir_all(&archive_dir).map_err(|e| AppError::Runtime(e.to_string()))?;
    let archive_dir = Context::ensure_within(&ctx.project_dir, &archive_dir)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?
        .to_owned();
    let dest = Context::ensure_within(&ctx.project_dir, &archive_dir.join(&file_name))?;

    if dest == path {
        let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
        crate::record::store::write_atomic(ctx, &safe_write, &new_content)?;
    } else {
        if dest.exists() {
            return Err(AppError::Validation(format!("already archived: {}", id)));
        }
        crate::record::store::write_atomic(ctx, &dest, &new_content)?;
        if let Err(err) = fs::remove_file(&path) {
            // Roll back the destination so a failed archive leaves exactly one
            // copy of the record: the still-valid source.
            let _ = fs::remove_file(&dest);
            return Err(AppError::Runtime(err.to_string()));
        }
    }

    let json = ctx
        .json
        .then(|| serde_json::json!({"id": id, "status": "archived"}));
    Ok(crate::CommandOutput {
        text: id,
        json,
        ..Default::default()
    })
}

pub fn delete(ctx: &Context, id_raw: &str, force: bool) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path(ctx, &cfg, &id)?;
    reject_milestone_write_conflict(ctx, &path)?;
    let path = crate::record::migrate::ensure_front_matter_canonical(ctx, &path)?;

    if !force {
        print!("Delete {}? [y/N] ", id);
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let ans = input.trim().to_lowercase();
        if ans != "y" && ans != "yes" {
            let json = if ctx.json {
                Some(serde_json::json!({"status": "aborted", "id": id}))
            } else {
                None
            };
            return Ok(crate::CommandOutput {
                text: String::new(),
                json,
                ..Default::default()
            });
        }
    }

    fs::remove_file(&path).map_err(|e| AppError::Runtime(e.to_string()))?;

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    })
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
            .join(format!("milestone-{}", name));
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

    #[test]
    fn view_succeeds_via_canonical_frontmatter_id_on_mismatch() {
        let dir = tmpdir("mismatch-canonical-view");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: MS-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/MS-2 - MileTwo.md"), content).unwrap();
        // Canonical front-matter id m-99 -> display MS-100.
        let out = view(&c, "MS-99").unwrap_or_else(|e| panic!("canonical view failed: {}", e));
        assert!(out.text.contains("id: MS-99"), "got: {}", out.text);
        let out2 = view(&c, "MS-2").unwrap_or_else(|e| panic!("filename view failed: {}", e));
        assert!(out2.text.contains("id: MS-99"), "got: {}", out2.text);
    }

    #[test]
    fn edit_refuses_mismatch_via_canonical_id() {
        let dir = tmpdir("mismatch-canonical-edit");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: MS-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/MS-2 - MileTwo.md"), content).unwrap();
        let err = edit(&c, "MS-99", Some("NewMile"), None, None, None, &[], &[])
            .err()
            .expect("expected error");
        let msg = err.to_string();
        assert!(
            msg.contains("migration conflict"),
            "expected conflict, got: {}",
            msg
        );
        assert_eq!(
            fs::read_to_string(dir.join("backlog/milestones/MS-2 - MileTwo.md")).unwrap(),
            content
        );
    }

    #[test]
    fn close_refuses_mismatch() {
        let dir = tmpdir("mismatch-close");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: MS-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/MS-2 - MileTwo.md"), content).unwrap();
        let err = close(&c, "MS-2").err().expect("expected error");
        assert!(
            err.to_string().contains("migration conflict"),
            "got: {}",
            err
        );
        assert!(dir.join("backlog/milestones/MS-2 - MileTwo.md").exists());
    }

    #[test]
    fn partial_front_matter_is_not_completed_on_view() {
        let dir = tmpdir("interop-readd");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let body = "\n## Description\n\nMile body\n";
        let content = format!("---\nid: MS-2\ntitle: MileTwo\n---\n{}", body);
        let path = dir.join("backlog/milestones/MS-2 - MileTwo.md");
        fs::write(&path, &content).unwrap();
        let out = view(&c, "MS-2").unwrap();
        assert_eq!(out.text, content, "view must not complete missing keys");
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn create_writes_front_matter_only_without_release_section() {
        let dir = tmpdir("create-layout");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/releases")).unwrap();
        fs::write(
            dir.join("backlog/releases/REL-001.md"),
            "---\nid: REL-001\ntitle: R1\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\n---\n\n## Description\nRelease\n",
        )
        .unwrap();
        let id = create(
            &c,
            "New Milestone",
            Some(sanitize::Text::literal("Milestone desc")),
            Some("REL-001"),
            &["m16".to_string()],
            &[],
        )
        .unwrap();
        assert_eq!(id, "MS-001");
        let content = fs::read_to_string(dir.join("backlog/milestones/MS-001.md")).unwrap();
        assert!(content.starts_with("---\n"), "{}", content);
        assert!(
            !content.lines().any(|line| line.starts_with("# ")),
            "new milestones must not have a title heading: {}",
            content
        );
        assert!(!content.contains("## Release"), "{}", content);
        assert!(content.contains("release: REL-001"), "{}", content);
        let order = [
            "id:",
            "title:",
            "created_date:",
            "updated_date:",
            "labels:",
            "references:",
            "release:",
            "status:",
        ];
        let mut last = 0usize;
        for key in order {
            let position = content
                .find(key)
                .unwrap_or_else(|| panic!("missing {key}: {content}"));
            assert!(position >= last, "key {key} out of order: {content}");
            last = position;
        }
    }

    #[test]
    fn title_edit_preserves_body_bytes() {
        let dir = tmpdir("title-edit-preserve");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let body = "\n## Description\n\nMile body UNIQUE-M\n";
        let content = format!(
            "---\nid: MS-2\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n{}",
            body
        );
        let path = dir.join("backlog/milestones/MS-2 - MileTwo.md");
        fs::write(&path, &content).unwrap();
        edit(&c, "MS-2", Some("NewMile"), None, None, None, &[], &[]).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("title: NewMile"), "got: {}", after);
        let (_, after_body) = crate::record::frontmatter::split(&after).unwrap();
        assert_eq!(after_body, body);
    }

    fn milestone_record(id: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: {title}\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nBody\n"
        )
    }

    #[test]
    fn no_op_edit_does_not_write_or_refresh_updated_date() {
        let dir = tmpdir("no-op-edit");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = milestone_record("MS-001", "One");
        let path = dir.join("backlog/milestones/MS-001.md");
        fs::write(&path, &content).unwrap();

        edit(&c, "MS-001", None, None, None, None, &[], &[]).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            content,
            "an edit that changes nothing must not write or refresh updated_date"
        );
    }

    #[test]
    fn close_sets_status_in_place_and_refreshes_updated_date() {
        let dir = tmpdir("close-canonical");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let path = dir.join("backlog/milestones/MS-001.md");
        fs::write(&path, milestone_record("MS-001", "One")).unwrap();

        let out = close(&c, "MS-001").unwrap();
        assert_eq!(out.text, "MS-001");
        assert!(path.exists(), "close must not move the record");
        assert!(!dir.join("backlog/archive/milestones/MS-001.md").exists());
        let closed = fs::read_to_string(&path).unwrap();
        assert!(closed.contains("status: closed"), "{}", closed);
        assert!(
            !closed.contains("updated_date: '2026-01-01T00:00:00Z'"),
            "close must refresh updated_date: {}",
            closed
        );
        assert!(
            closed.contains("created_date: '2026-01-01T00:00:00Z'"),
            "close must not change created_date: {}",
            closed
        );
    }

    #[test]
    fn close_of_closed_is_validation_without_a_write() {
        let dir = tmpdir("close-closed");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let path = dir.join("backlog/milestones/MS-001.md");
        close_seed(&path, "MS-001", "closed");
        let before = fs::read_to_string(&path).unwrap();

        let err = close(&c, "MS-001").err().expect("closed close must fail");
        assert!(
            matches!(&err, AppError::Validation(message) if message.contains("closed")),
            "unexpected error: {err:?}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }

    fn close_seed(path: &std::path::Path, id: &str, status: &str) {
        let content = milestone_record(id, "Seed").replace(
            "release: null\n",
            &format!("release: null\nstatus: {}\n", status),
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn close_unknown_id_is_not_found() {
        let dir = tmpdir("close-unknown");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        assert!(matches!(close(&c, "MS-999"), Err(AppError::NotFound(_))));
    }

    #[test]
    fn backlog_shaped_milestone_stem_is_invisible() {
        let dir = tmpdir("backlog-shape");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        fs::write(
            dir.join("backlog/milestones/m-0 - Foundation.md"),
            milestone_record("MS-0", "Foundation"),
        )
        .unwrap();
        assert!(matches!(view(&c, "MS-000"), Err(AppError::NotFound(_))));
        let out = list(&c).unwrap();
        assert!(out.text.is_empty(), "got: {}", out.text);
    }
}

#[cfg(test)]
mod native_layout_tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("milestone-native-{}", name));
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

    fn record(id: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: {title}\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nBody\n"
        )
    }

    #[test]
    fn legacy_archived_milestone_remains_readable() {
        let dir = tmpdir("legacy-archive-readable");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones/archived")).unwrap();
        let path = dir.join("backlog/milestones/archived/MS-005.md");
        fs::write(&path, record("MS-005", "Five")).unwrap();

        let out = view(&c, "MS-005").unwrap();
        assert!(out.text.contains("MS-005"), "got: {}", out.text);
        assert!(
            matches!(resolve_milestone_entry(&c, &Config::default(), "MS-005"), Some(found) if found == path)
        );
    }

    #[test]
    fn read_import_archive_milestone_is_readable_and_reserves_its_number() {
        let dir = tmpdir("read-import-archive");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        fs::create_dir_all(dir.join("backlog/archive/milestones")).unwrap();
        let path = dir.join("backlog/archive/milestones/MS-005.md");
        fs::write(&path, record("MS-005", "Five")).unwrap();

        let out = view(&c, "MS-005").unwrap();
        assert!(out.text.contains("MS-005"), "got: {}", out.text);
        assert!(
            matches!(resolve_milestone_entry(&c, &Config::default(), "MS-005"), Some(found) if found == path),
            "the read/import archive folder must stay resolvable"
        );

        let created = create(&c, "Six", None, None, &[], &[]).unwrap();
        assert_eq!(
            created, "MS-006",
            "ID allocation must observe the read/import archive folder"
        );
    }
}
