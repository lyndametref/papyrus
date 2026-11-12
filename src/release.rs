use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ReleaseCmd {
    /// Create a release.
    Create {
        /// Release title.
        #[arg(value_name = "title")]
        title: String,
        /// Release description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// List releases.
    List,
    /// View a release.
    View { id: String },
    /// Edit a release.
    Edit {
        id: String,
        /// Update the title.
        #[arg(long = "title")]
        title: Option<String>,
        /// Update the description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
}

pub fn run(ctx: &Context, cmd: ReleaseCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        ReleaseCmd::Create {
            title,
            description,
            label,
            reference,
        } => {
            let id = create(ctx, &title, description.as_deref(), &label, &reference)?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        ReleaseCmd::List => list(ctx),
        ReleaseCmd::View { id } => view(ctx, &id),
        ReleaseCmd::Edit {
            id,
            title,
            description,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            description.as_deref(),
            &label,
            &reference,
        ),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn releases_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let desc = crate::record::descriptor::descriptor_for("release")
        .ok_or_else(|| AppError::Runtime("release descriptor is unavailable".to_string()))?;
    desc.resolve_directory(cfg, &ctx.project_dir)
}

fn release_prefixes(cfg: &Config) -> Vec<String> {
    vec![cfg.release_prefix.clone()]
}

fn resolve_release(
    ctx: &Context,
    cfg: &Config,
    id: &str,
) -> Result<Option<crate::record::resolve::ResolvedRecord>, AppError> {
    let dir = releases_dir(ctx, cfg)?;
    crate::record::resolve::resolve_in_dirs(ctx, &[dir], id, &release_prefixes(cfg), false)
}

fn ensure_dir(dir: &std::path::Path, project_dir: &std::path::Path) -> Result<(), AppError> {
    let safe = Context::ensure_within(project_dir, dir)?;
    fs::create_dir_all(&safe).map_err(|e| AppError::Runtime(e.to_string()))
}

fn frontmatter_list_rel(content: &str, key: &str) -> Vec<String> {
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

fn body_document(id: &str, title: &str, desc: Option<&str>) -> crate::record::document::Document {
    let mut sections = Vec::new();
    if let Some(d) = desc {
        sections.push(crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![d.to_string()],
        });
    }
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {id} {title}")),
        preamble: vec![String::new()],
        sections,
    }
}

fn release_from_document(
    doc: &crate::record::document::Document,
) -> (String, String, Option<String>) {
    let (id, title) = crate::record::document::heading_id_title(doc);
    let desc = crate::record::document::section_text(doc, "Description");
    (id, title, desc)
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    desc_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };

    let mut labels: Vec<String> = Vec::new();
    for l in labels_raw {
        if !labels.contains(l) {
            labels.push(sanitize_text(l)?);
        }
    }
    let mut references: Vec<String> = Vec::new();
    for r in references_raw {
        if !references.contains(r) {
            references.push(
                sanitize::sanitize_path_string(r).ok_or_else(|| {
                    AppError::Validation("input cannot be normalized".to_string())
                })?,
            );
        }
    }

    let dir = releases_dir(ctx, &cfg)?;
    ensure_dir(&dir, &ctx.project_dir)?;

    let next = crate::record::identity::next_number_in_dirs(&[&dir], &cfg.release_prefix);
    let id = crate::record::identity::format_id(
        &cfg.release_prefix,
        next as i64,
        cfg.id_padding as usize,
    );

    let body_doc = body_document(&id, &title, desc.as_deref());
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
    ];
    let file_content = crate::record::patch::apply_patch(&parsed, &ops)?;
    let file_path = dir.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &file_content)?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);

    let records = crate::record::resolve::list_in_dir(ctx, &dir, &release_prefixes(&cfg), false)?;
    let mut items: Vec<(String, String, String)> = Vec::new();
    for record in records {
        let content = fs::read_to_string(&record.path).unwrap_or_default();
        let title = crate::record::document::parse(&content)
            .ok()
            .map(|d| crate::record::document::heading_id_title(&d).1)
            .unwrap_or_default();
        let title = if title.is_empty() {
            record.id.clone()
        } else {
            title
        };
        items.push((record.id, title, content));
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
    let requested = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);
    let record = resolve_release(ctx, &cfg, &requested)?
        .ok_or_else(|| AppError::NotFound(format!("not found: {}", requested)))?;
    let id = record.id.clone();
    let content = crate::record::migrate::ensure_front_matter(ctx, &record.path)?;

    let text = content.clone();
    let json = if ctx.json {
        Some(serde_json::json!({
            "id": id,
            "path": record.path.display().to_string(),
            "content": content,
            "metadata": crate::record::view::RecordView::lenient(&content, &id).metadata_json(),
        }))
    } else {
        None
    };
    Ok(crate::CommandOutput { text, json })
}

pub fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    desc_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);
    let record = resolve_release(ctx, &cfg, &requested)?
        .ok_or_else(|| AppError::NotFound(format!("not found: {}", requested)))?;
    record.ensure_writable()?;
    let id = record.id.clone();
    let path = record.path.clone();

    let (doc, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();
    let (parsed_id, parsed_title, parsed_desc) = release_from_document(&doc);
    let mut title_val = if parsed_title.is_empty() {
        id.clone()
    } else {
        parsed_title
    };
    let mut desc_val = parsed_desc;

    if let Some(t) = title_raw {
        title_val = sanitize_text(t)?;
    }
    if let Some(d) = desc_raw {
        desc_val = Some(sanitize_text(d)?);
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
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_rel(&content, "labels");
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
        let mut fm_refs = frontmatter_list_rel(&content, "references");
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

    let _safe_write = Context::ensure_within(&ctx.project_dir, &path)?;

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

#[cfg(test)]
mod native_layout_tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("release-native-{}", name));
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
    fn release_reads_resolve_by_canonical_id_with_tolerant_case_and_padding() {
        let dir = tmpdir("tolerant-read");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog").join("releases")).unwrap();
        fs::write(
            dir.join("backlog/releases/REL-001.md"),
            "---\nid: REL-001\ntitle: One\n---\n\n# REL-001 One\n",
        )
        .unwrap();

        let cfg = Config::default();
        let record = resolve_release(&c, &cfg, "REL-1").unwrap().unwrap();
        assert_eq!(record.id, "REL-001");
        assert_eq!(
            record.path.file_name().and_then(|s| s.to_str()),
            Some("REL-001.md")
        );

        // A Backlog-shaped unpadded lowercase stem is not a Papyrus record.
        fs::write(
            dir.join("backlog/releases/rel-2.md"),
            "---\nid: REL-002\ntitle: Hidden\n---\n\n# REL-002 Hidden\n",
        )
        .unwrap();
        assert!(resolve_release(&c, &cfg, "REL-002").unwrap().is_none());
    }

    #[test]
    fn release_allocation_ignores_the_archive_directory() {
        let dir = tmpdir("allocation-active-only");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog").join("releases")).unwrap();
        let archive = dir.join("backlog").join("archive").join("releases");
        fs::create_dir_all(&archive).unwrap();
        fs::write(
            dir.join("backlog/releases/REL-001.md"),
            "---\nid: REL-001\ntitle: One\n---\n\n# REL-001 One\n",
        )
        .unwrap();
        fs::write(
            archive.join("REL-009.md"),
            "---\nid: REL-009\ntitle: Old\n---\n\n# REL-009 Old\n",
        )
        .unwrap();

        let id = create(&c, "Two", None, &[], &[]).unwrap();
        assert_eq!(id, "REL-002", "releases allocate from the active directory");
        assert!(!dir.join("backlog/releases/REL-010.md").exists());
        assert!(!archive.join("REL-002.md").exists());
        let cfg = Config::default();
        let descriptor = crate::record::descriptor::descriptor_for("release").unwrap();
        assert!(
            !descriptor
                .resolve_read_directories(&cfg, &dir)
                .iter()
                .any(|path| path.ends_with(std::path::Path::new("archive").join("releases"))),
            "release archive directories are never read"
        );
    }
}
