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
        #[arg(short = 'd', long = "description")]
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
        #[arg(short = 'd', long = "description")]
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
    /// Archive a milestone.
    Archive { id: String },
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
            let id = create(
                ctx,
                &title,
                description.as_deref(),
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
            Ok(crate::CommandOutput { text, json })
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
            description.as_deref(),
            release.as_deref(),
            no_release,
            &label,
            &reference,
        ),
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

fn backlog_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let raw = ctx.project_dir.join(&cfg.backlog_directory);
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("milestones");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn archived_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = active_dir(ctx, cfg)?;
    let raw = base.join("archived");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn backlog_archive_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("archive/milestones");
    fs::create_dir_all(&raw).map_err(|e| AppError::Runtime(e.to_string()))?;
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn resolve_milestone_path(ctx: &Context, cfg: &Config, id: &str) -> Result<PathBuf, AppError> {
    let storage_id =
        crate::record::identity::milestone_to_storage_id(id).unwrap_or_else(|_| id.to_lowercase());
    for dir in [
        active_dir(ctx, cfg),
        archived_dir(ctx, cfg),
        backlog_archive_dir(ctx, cfg),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(path) = find_milestone_file(&dir, &storage_id) {
            return Ok(path);
        }
    }
    Err(AppError::NotFound(format!("not found: {}", id)))
}

fn resolve_milestone_path_migrated(
    ctx: &Context,
    cfg: &Config,
    id: &str,
) -> Result<PathBuf, AppError> {
    let path = resolve_milestone_path(ctx, cfg, id)?;
    crate::record::migrate::ensure_front_matter_canonical(ctx, &path)
}

fn milestone_frontmatter_storage_id(content: &str) -> Option<String> {
    if !crate::record::frontmatter::has_front_matter(content) {
        return None;
    }
    let fm = crate::record::frontmatter::parse(content).ok().flatten()?;
    let id = crate::record::frontmatter::current_value_string(&fm.raw, "id")?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    if let Ok(sid) = crate::record::identity::milestone_to_storage_id(id) {
        return Some(sid);
    }
    let spec = crate::record::identity::IdSpec::default_milestone();
    spec.parse_storage_number(id).ok()?;
    Some(id.to_lowercase())
}

fn reject_milestone_write_conflict(ctx: &Context, path: &std::path::Path) -> Result<(), AppError> {
    let content = fs::read_to_string(path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let desc = crate::record::descriptor::descriptor_for("milestone").unwrap();
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, desc);
    if let Some(reason) =
        crate::record::migrate::check_conflicts_for_path(ctx, path, desc, &content, &conflicts)
    {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            crate::record::migrate::rel_path(&ctx.project_dir, path),
            reason
        )));
    }
    Ok(())
}

pub(crate) fn find_milestone_file(dir: &std::path::Path, storage_id: &str) -> Option<PathBuf> {
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
        let stem = path.file_stem().and_then(|s| s.to_str())?;
        let base = stem.split(" - ").next().unwrap_or(stem).trim();
        if base.eq_ignore_ascii_case(storage_id) {
            return Some(path.clone());
        }
        if let Ok(sid) = crate::record::identity::milestone_to_storage_id(base)
            && sid.eq_ignore_ascii_case(storage_id)
        {
            return Some(path.clone());
        }
    }
    // ADR-0008 decision 3/9: front-matter id is canonical; resolve a record
    // addressed by its front-matter ID even on mismatch. Reads succeed;
    // writes refuse via reject_milestone_write_conflict.
    for path in &entries {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if let Some(sid) = milestone_frontmatter_storage_id(&content)
            && sid.eq_ignore_ascii_case(storage_id)
        {
            return Some(path.clone());
        }
    }
    None
}

fn releases_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("releases");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn ensure_dir(dir: &std::path::Path, project_dir: &std::path::Path) -> Result<(), AppError> {
    let safe = Context::ensure_within(project_dir, dir)?;
    fs::create_dir_all(&safe).map_err(|e| AppError::Runtime(e.to_string()))
}

fn milestone_number_from_stem(stem: &str) -> Option<u32> {
    let base = stem.split(" - ").next()?.trim();
    let storage_id = crate::record::identity::milestone_to_storage_id(base)
        .unwrap_or_else(|_| base.to_lowercase());
    let num = storage_id.strip_prefix("m-")?;
    num.parse::<u32>().ok().map(|n| n + 1)
}

fn scan_max(
    active: &std::path::Path,
    archived: &std::path::Path,
    archive: &std::path::Path,
) -> u32 {
    let mut max = 0u32;
    for dir in [active, archived, archive] {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = milestone_number_from_stem(stem)
                    && n > max
                {
                    max = n;
                }
            }
        }
    }
    max
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

fn parse_heading(content: &str) -> (String, String) {
    let body = if let Some((_, _, bs)) = crate::record::frontmatter::find_block(content) {
        &content[bs..]
    } else {
        content
    };
    let first = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let title = parts.next().unwrap_or("").trim().to_string();
    // Prefer the canonical title from front matter when present.
    let title = if let Ok(Some(fm)) = crate::record::frontmatter::parse(content) {
        crate::record::frontmatter::current_value_string(&fm.raw, "title")
            .filter(|t| !t.is_empty())
            .unwrap_or(title)
    } else {
        title
    };
    (id, title)
}

pub(crate) fn parse_milestone(content: &str) -> (String, String, Option<String>, Option<String>) {
    let (id, title) = parse_heading(content);
    let mut desc: Option<String> = None;
    let mut release: Option<String> = None;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].starts_with("## ") {
            let section = lines[i].trim().to_string();
            i += 1;
            let mut buf: Vec<&str> = Vec::new();
            while i < lines.len() && !lines[i].starts_with("## ") {
                buf.push(lines[i]);
                i += 1;
            }
            while buf.first().map(|s| s.trim().is_empty()).unwrap_or(false) {
                buf.remove(0);
            }
            while buf.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
                buf.pop();
            }
            let text = buf.join("\n");
            let val = if text.trim().is_empty() {
                None
            } else {
                Some(text)
            };
            match section.as_str() {
                "## Description" => desc = val,
                "## Release" => {
                    if let Some(t) = val {
                        release = Some(t.trim().to_string());
                    }
                }
                _ => {}
            }
        } else {
            i += 1;
        }
    }
    (id, title, desc, release)
}

pub(crate) fn render_milestone(
    id: &str,
    title: &str,
    desc: Option<&str>,
    release: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", id, title));
    if let Some(d) = desc {
        parts.push(format!("## Description\n{}", d));
    }
    if let Some(r) = release {
        parts.push(format!("## Release\n{}", r));
    }
    parts.join("\n\n") + "\n"
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    desc_raw: Option<&str>,
    release_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
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
    let archived = archived_dir(ctx, &cfg)?;
    let archive = backlog_archive_dir(ctx, &cfg)?;
    ensure_dir(&active, &ctx.project_dir)?;
    let _ = ensure_dir(&archived, &ctx.project_dir);

    let max = scan_max(&active, &archived, &archive);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        cfg.milestone_prefix,
        next,
        width = cfg.id_padding as usize
    );

    let now = crate::record::frontmatter::utc_now();
    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", id));
    fm.push_str(&format!(
        "title: {}\n",
        crate::record::frontmatter::format_scalar_pub(&title)
    ));
    fm.push_str(&format!("created_date: '{}'\n", now));
    fm.push_str(&format!("updated_date: '{}'\n", now));
    if labels.is_empty() {
        fm.push_str("labels: []\n");
    } else {
        fm.push_str("labels:\n");
        for l in &labels {
            fm.push_str(&format!(
                "  - {}\n",
                crate::record::frontmatter::format_scalar_pub(l)
            ));
        }
    }
    if references.is_empty() {
        fm.push_str("references: []\n");
    } else {
        fm.push_str("references:\n");
        for r in &references {
            fm.push_str(&format!(
                "  - {}\n",
                crate::record::frontmatter::format_scalar_pub(r)
            ));
        }
    }
    if let Some(r) = &release {
        fm.push_str(&format!(
            "release: {}\n",
            crate::record::frontmatter::format_scalar_pub(r)
        ));
    } else {
        fm.push_str("release: null\n");
    }

    let body = render_milestone(&id, &title, desc.as_deref(), release.as_deref());
    let content = crate::record::frontmatter::wrap(&fm, &body);

    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let mut items: Vec<(String, String, String)> = Vec::new();
    let desc = crate::record::descriptor::descriptor_for("milestone").unwrap();
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, desc);
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
            let safe = Context::ensure_within(&ctx.project_dir, &path)?;
            let content = fs::read_to_string(&safe).unwrap_or_default();
            if let Some(conflict) = crate::record::migrate::path_migration_conflict(
                ctx, &safe, desc, &content, &conflicts,
            ) {
                eprintln!("warning: {}", conflict);
                continue;
            }
            let (_, title) = parse_heading(&content);
            let title = if title.is_empty() { id.clone() } else { title };
            items.push((id, title, content));
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

    Ok(crate::CommandOutput { text, json })
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
    Ok(crate::CommandOutput { text, json })
}

#[allow(clippy::too_many_arguments)]
pub fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    desc_raw: Option<&str>,
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
    let conflicts = crate::record::migrate::scan_migration_conflicts(ctx, desc);
    if let Some(reason) =
        crate::record::migrate::check_conflicts_for_path(ctx, &path, desc, &content, &conflicts)
    {
        return Err(AppError::Validation(format!(
            "migration conflict at {}: {}",
            crate::record::migrate::rel_path(&ctx.project_dir, &path),
            reason
        )));
    }

    let (parsed_id, parsed_title, parsed_desc, parsed_release) = parse_milestone(&content);
    let mut title_val = if parsed_title.is_empty() {
        id.clone()
    } else {
        parsed_title
    };
    let mut desc_val = parsed_desc;
    let mut release_val = parsed_release
        .or_else(|| crate::record::frontmatter::current_value_string(&content, "release"));

    if let Some(t) = title_raw {
        title_val = sanitize_text(t)?;
    }
    if let Some(d) = desc_raw {
        desc_val = Some(sanitize_text(d)?);
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
    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            title_val.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
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
    if release_raw.is_some() {
        if let Some(r) = &release_val {
            ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "release".into(),
                r.clone(),
            ));
            if doc.sections.iter().any(|s| s.heading == "## Release") {
                ops.push(crate::record::patch::PatchOp::SetSection(
                    "## Release".into(),
                    r.clone(),
                ));
            }
        }
    }
    if no_release.is_some() {
        ops.push(crate::record::patch::PatchOp::ClearFrontMatter(
            "release".into(),
        ));
        if doc.sections.iter().any(|s| s.heading == "## Release") {
            ops.push(crate::record::patch::PatchOp::ClearSection(
                "## Release".into(),
            ));
        }
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
        if doc.sections.iter().any(|s| s.heading == "## Labels") {
            let body = fm_labels
                .iter()
                .map(|l| format!("- {}", l))
                .collect::<Vec<_>>()
                .join("\n");
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Labels".into(),
                body,
            ));
        }
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
    })
}

pub fn archive(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_milestone_path(ctx, &cfg, &id)?;
    reject_milestone_write_conflict(ctx, &path)?;
    let path = crate::record::migrate::ensure_front_matter_canonical(ctx, &path)?;
    let archived = archived_dir(ctx, &cfg)?;
    ensure_dir(&archived, &ctx.project_dir)?;

    let archived_path = archived.join(
        path.file_name()
            .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?,
    );
    let archived_safe = Context::ensure_within(&ctx.project_dir, &archived_path)?;
    if archived_safe == path {
        return Err(AppError::Validation(format!("already archived: {}", id)));
    }
    if archived_safe.exists() {
        return Err(AppError::Validation(format!("already archived: {}", id)));
    }

    fs::rename(&path, &archived_safe).map_err(|e| AppError::Runtime(e.to_string()))?;

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(crate::CommandOutput {
        text: String::new(),
        json,
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
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        }
    }

    #[test]
    fn view_succeeds_via_canonical_frontmatter_id_on_mismatch() {
        let dir = tmpdir("mismatch-canonical-view");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: m-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/m-2 - MileTwo.md"), content).unwrap();
        // Canonical front-matter id m-99 -> display MS-100.
        let out = view(&c, "MS-100").unwrap_or_else(|e| panic!("canonical view failed: {}", e));
        assert!(out.text.contains("id: m-99"), "got: {}", out.text);
        let out2 = view(&c, "MS-003").unwrap_or_else(|e| panic!("filename view failed: {}", e));
        assert!(out2.text.contains("id: m-99"), "got: {}", out2.text);
    }

    #[test]
    fn edit_refuses_mismatch_via_canonical_id() {
        let dir = tmpdir("mismatch-canonical-edit");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: m-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/m-2 - MileTwo.md"), content).unwrap();
        let err = edit(&c, "MS-100", Some("NewMile"), None, None, None, &[], &[])
            .err()
            .expect("expected error");
        let msg = err.to_string();
        assert!(
            msg.contains("migration conflict"),
            "expected conflict, got: {}",
            msg
        );
        assert_eq!(
            fs::read_to_string(dir.join("backlog/milestones/m-2 - MileTwo.md")).unwrap(),
            content
        );
    }

    #[test]
    fn archive_refuses_mismatch() {
        let dir = tmpdir("mismatch-archive");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let content = "---\nid: m-99\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n\n## Description\n\nMile body\n";
        fs::write(dir.join("backlog/milestones/m-2 - MileTwo.md"), content).unwrap();
        let err = archive(&c, "MS-003").err().expect("expected error");
        assert!(
            err.to_string().contains("migration conflict"),
            "got: {}",
            err
        );
        assert!(dir.join("backlog/milestones/m-2 - MileTwo.md").exists());
    }

    #[test]
    fn backlog_md_dropped_keys_are_readded_on_view() {
        let dir = tmpdir("interop-readd");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let body = "\n## Description\n\nMile body\n";
        let content = format!("---\nid: m-2\ntitle: MileTwo\n---\n{}", body);
        let path = dir.join("backlog/milestones/m-2 - MileTwo.md");
        fs::write(&path, &content).unwrap();
        let out = view(&c, "MS-003").unwrap();
        for key in [
            "created_date:",
            "updated_date:",
            "labels:",
            "references:",
            "release:",
        ] {
            assert!(
                out.text.contains(key),
                "missing {} after re-add: {}",
                key,
                out.text
            );
        }
        let (_, out_body) = crate::record::frontmatter::split(&out.text).unwrap();
        assert_eq!(out_body, body);
        let on_disk = fs::read_to_string(&path).unwrap();
        assert!(on_disk.contains("release:"), "got: {}", on_disk);
    }

    #[test]
    fn title_edit_preserves_body_bytes() {
        let dir = tmpdir("title-edit-preserve");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/milestones")).unwrap();
        let body = "\n## Description\n\nMile body UNIQUE-M\n";
        let content = format!(
            "---\nid: m-2\ntitle: MileTwo\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nrelease: null\n---\n{}",
            body
        );
        let path = dir.join("backlog/milestones/m-2 - MileTwo.md");
        fs::write(&path, &content).unwrap();
        edit(&c, "MS-003", Some("NewMile"), None, None, None, &[], &[]).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("title: NewMile"), "got: {}", after);
        let (_, after_body) = crate::record::frontmatter::split(&after).unwrap();
        assert_eq!(after_body, body);
    }
}
