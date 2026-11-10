use crate::CommandOutput;
use crate::commands::load_effective_config;
use crate::config::{Config, DocTypeEntry};
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug)]
pub enum RuleCmd {
    /// Create a business rule.
    Create {
        /// Business rule title.
        title: String,
        /// State the business rule precisely.
        #[arg(long = "description")]
        description: String,
        /// Explain why the rule exists.
        #[arg(long = "rationale")]
        rationale: String,
        /// Describe when and where the rule applies.
        #[arg(long = "applicability")]
        applicability: String,
        /// Reference a business requirement. Repeatable.
        #[arg(long = "requirement")]
        requirement: Vec<String>,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// List business rules.
    List,
    /// View a business rule.
    View { id: String },
    /// Edit a business rule.
    Edit {
        id: String,
        /// Update the title.
        #[arg(long = "title")]
        title: Option<String>,
        /// Update the description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Add a business requirement. Repeatable.
        #[arg(long = "requirement")]
        requirement: Vec<String>,
        /// Remove a business requirement; without an ID removes all.
        #[arg(long = "no-requirement", num_args = 0..=1)]
        no_requirement: Option<Option<String>>,
        /// Open the document body in $VISUAL or $EDITOR.
        #[arg(long = "editor")]
        editor: bool,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// Deprecate a business rule.
    Deprecate { id: String },
    /// Restore a deprecated business rule.
    Restore { id: String },
}

pub fn run(ctx: &Context, cmd: RuleCmd) -> Result<CommandOutput, AppError> {
    match cmd {
        RuleCmd::Create {
            title,
            description,
            rationale,
            applicability,
            requirement,
            label,
            reference,
        } => {
            let id = create(
                ctx,
                &title,
                &description,
                &rationale,
                &applicability,
                &requirement,
                &label,
                &reference,
            )?;
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(CommandOutput { text: id, json })
        }
        RuleCmd::List => list(ctx),
        RuleCmd::View { id } => view(ctx, &id),
        RuleCmd::Edit {
            id,
            title,
            description,
            requirement,
            no_requirement,
            editor,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            description.as_deref(),
            &requirement,
            no_requirement,
            &label,
            &reference,
            editor,
        ),
        RuleCmd::Deprecate { id } => deprecate(ctx, &id),
        RuleCmd::Restore { id } => restore(ctx, &id),
    }
}

fn sanitize_field(raw: &str) -> Result<String, AppError> {
    let value = sanitize::sanitize_name(raw)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
    if value.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "input cannot be normalized".to_string(),
        ));
    }
    Ok(value)
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn type_entry(cfg: &Config) -> Result<DocTypeEntry, AppError> {
    let entry = cfg
        .document_types
        .get("rule")
        .ok_or_else(|| AppError::NotFound("document type not configured: rule".to_string()))?;
    if !entry.enabled {
        return Err(AppError::Validation(
            "document type disabled: rule".to_string(),
        ));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(
            "document type prefix empty: rule".to_string(),
        ));
    }
    Ok(entry.clone())
}

fn active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let entry = type_entry(cfg)?;
    let raw = ctx
        .project_dir
        .join(&cfg.doc_directory)
        .join(&entry.directory);
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn deprecated_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    Ok(active_dir(ctx, cfg)?.join("deprecated"))
}

fn ensure_dirs(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

fn is_record_path(path: &Path, prefix: &str) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("md")
        && path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|stem| crate::record::identity::is_record_stem(stem, prefix))
}

fn find_record(ctx: &Context, dir: &Path, id: &str) -> Result<Option<PathBuf>, AppError> {
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::Runtime(e.to_string())),
    };
    for entry in rd {
        let entry = entry.map_err(|e| AppError::Runtime(e.to_string()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) != Some(id) {
            continue;
        }
        let safe = Context::ensure_within(&ctx.project_dir, &path)?;
        return Ok(Some(safe));
    }
    Ok(None)
}

fn active_or_deprecated(
    ctx: &Context,
    active: &Path,
    deprecated: &Path,
    id: &str,
) -> Result<PathBuf, AppError> {
    if let Some(path) = find_record(ctx, active, id)? {
        return Ok(path);
    }
    if let Some(path) = find_record(ctx, deprecated, id)? {
        return Ok(path);
    }
    Err(AppError::NotFound(format!("not found: {}", id)))
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct RuleRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) rationale: String,
    pub(crate) applicability: String,
    pub(crate) requirements: Vec<String>,
}

fn requirements_from_document(doc: &crate::record::document::Document) -> Vec<String> {
    let Some(body) = crate::record::document::section_text(doc, "Requirements") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item = if let Some(rest) = trimmed.strip_prefix("- ") {
            rest.trim()
        } else if let Some(rest) = trimmed.strip_prefix('-') {
            rest.trim()
        } else {
            trimmed
        };
        if item.is_empty() {
            continue;
        }
        if !out.contains(&item.to_string()) {
            out.push(item.to_string());
        }
    }
    out
}

fn record_from_document(doc: &crate::record::document::Document, fallback_id: &str) -> RuleRecord {
    let (id, title) = crate::record::document::heading_id_title(doc);
    RuleRecord {
        id: if id.is_empty() {
            fallback_id.to_string()
        } else {
            id
        },
        title: if title.is_empty() {
            fallback_id.to_string()
        } else {
            title
        },
        description: crate::record::document::section_text(doc, "Description").unwrap_or_default(),
        rationale: crate::record::document::section_text(doc, "Rationale").unwrap_or_default(),
        applicability: crate::record::document::section_text(doc, "Applicability")
            .unwrap_or_default(),
        requirements: requirements_from_document(doc),
    }
}

fn body_document(
    id: &str,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
    requirements: &[String],
) -> crate::record::document::Document {
    let mut sections = vec![
        crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![description.to_string(), String::new()],
        },
        crate::record::document::Section {
            heading: "## Rationale".to_string(),
            body: vec![rationale.to_string(), String::new()],
        },
        crate::record::document::Section {
            heading: "## Applicability".to_string(),
            body: vec![applicability.to_string()],
        },
    ];
    if !requirements.is_empty() {
        sections.push(crate::record::document::Section {
            heading: "## Requirements".to_string(),
            body: requirements.iter().map(|r| format!("- {r}")).collect(),
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

fn run_editor(command: &str, path: &Path) -> Result<(), AppError> {
    let mut parts = command.split_whitespace();
    let prog = parts.next().unwrap_or("");
    if prog.is_empty() {
        return Err(AppError::Runtime("no editor configured".to_string()));
    }
    let status = std::process::Command::new(prog)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    if !status.success() {
        return Err(AppError::Runtime(format!("editor exited with {}", status)));
    }
    Ok(())
}

fn dedup(mut values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for v in values.drain(..) {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    ctx: &Context,
    title_raw: &str,
    description_raw: &str,
    rationale_raw: &str,
    applicability_raw: &str,
    requirements_raw: &[String],
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let title = sanitize_field(title_raw)?;
    let description = sanitize_field(description_raw)?;
    let rationale = sanitize_field(rationale_raw)?;
    let applicability = sanitize_field(applicability_raw)?;

    let mut requirements = Vec::new();
    for r in requirements_raw {
        requirements.push(sanitize_field(r)?);
    }
    let requirements = dedup(requirements);

    let mut labels: Vec<String> = Vec::new();
    for l in labels_raw {
        if !labels.contains(l) {
            labels.push(sanitize_field(l)?);
        }
    }
    let mut references: Vec<String> = Vec::new();
    for r in references_raw {
        if !references.contains(r) {
            references.push(sanitize_field(r)?);
        }
    }

    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    ensure_dirs(&active)?;
    ensure_dirs(&deprecated)?;

    let next = crate::record::identity::next_number(&active, &deprecated, &entry.prefix);
    let id =
        crate::record::identity::format_id(&entry.prefix, next as i64, cfg.id_padding as usize);

    let body_doc = body_document(
        &id,
        &title,
        &description,
        &rationale,
        &applicability,
        &requirements,
    );
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
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &file_content)?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);

    let mut items: Vec<(String, String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&active) {
        for e in rd {
            let e = e.map_err(|err| AppError::Runtime(err.to_string()))?;
            let path = e.path();
            if !path.is_file() || !is_record_path(&path, &entry.prefix) {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let safe = Context::ensure_within(&ctx.project_dir, &path)?;
            let content =
                fs::read_to_string(&safe).map_err(|e| AppError::Runtime(e.to_string()))?;
            let title = crate::record::document::parse(&content)
                .ok()
                .map(|d| crate::record::document::heading_id_title(&d).1)
                .unwrap_or_default();
            let title = if title.is_empty() {
                id.to_string()
            } else {
                title
            };
            items.push((id.to_string(), title, content));
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
    Ok(CommandOutput { text, json })
}

pub fn view(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);
    let path = active_or_deprecated(ctx, &active, &deprecated, &id)?;

    let content = crate::record::migrate::ensure_front_matter(ctx, &path)?;
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
    let text = content.trim_end_matches('\n').to_string();
    Ok(CommandOutput { text, json })
}

fn frontmatter_list_rule(content: &str, key: &str) -> Vec<String> {
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

#[allow(clippy::too_many_arguments)]
pub fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    description_raw: Option<&str>,
    requirements_add: &[String],
    no_requirement: Option<Option<String>>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);
    let path = active_or_deprecated(ctx, &active, &deprecated, &id)?;

    if editor && cfg.effective_editor().is_none() {
        return Err(AppError::Runtime("no editor configured".to_string()));
    }

    let (doc, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();
    let mut record = record_from_document(&doc, &id);

    if let Some(t) = title_raw {
        record.title = sanitize_field(t)?;
    }
    if let Some(d) = description_raw {
        record.description = sanitize_field(d)?;
    }

    // additions before removals
    let mut additions = Vec::new();
    for r in requirements_add {
        additions.push(sanitize_field(r)?);
    }
    for a in additions {
        if !record.requirements.contains(&a) {
            record.requirements.push(a);
        }
    }
    if let Some(no) = &no_requirement {
        match no {
            Some(rid) => {
                let rid = sanitize_field(rid)?;
                record.requirements.retain(|r| r != &rid);
            }
            None => record.requirements.clear(),
        }
    }

    let editor_cmd = if editor {
        Some(
            cfg.effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            record.title.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            record.title.clone(),
        ));
    }
    if description_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Description".into(),
            record.description.clone(),
        ));
    }
    // requirements body section
    if !requirements_add.is_empty() || no_requirement.is_some() {
        if record.requirements.is_empty() {
            ops.push(crate::record::patch::PatchOp::ClearSection(
                "## Requirements".into(),
            ));
        } else {
            let body = record
                .requirements
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n");
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Requirements".into(),
                body,
            ));
        }
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_rule(&content, "labels");
        for l in labels_raw {
            let lv = sanitize_field(l)?;
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
        let mut fm_refs = frontmatter_list_rule(&content, "references");
        for r in references_raw {
            let rv = sanitize_field(r)?;
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
        let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
        crate::record::store::write_atomic(ctx, &safe_path, &new_content)?;
    }

    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;

    if let Some(command) = editor_cmd {
        run_editor(&command, &safe_path)?;
    }

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
    })
}

pub fn deprecate(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    ensure_dirs(&deprecated)?;

    let Some(path) = find_record(ctx, &active, &id)? else {
        if find_record(ctx, &deprecated, &id)?.is_some() {
            return Err(AppError::Validation(format!("already deprecated: {}", id)));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?;
    let destination = deprecated.join(file_name);
    let destination = Context::ensure_within(&ctx.project_dir, &destination)?;
    fs::rename(&path, &destination).map_err(|e| AppError::Runtime(e.to_string()))?;

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
    })
}

pub fn restore(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);

    let Some(path) = find_record(ctx, &deprecated, &id)? else {
        if find_record(ctx, &active, &id)?.is_some() {
            return Err(AppError::Validation(format!("not deprecated: {}", id)));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?;
    let destination = active.join(file_name);
    let destination = Context::ensure_within(&ctx.project_dir, &destination)?;
    fs::rename(&path, &destination).map_err(|e| AppError::Runtime(e.to_string()))?;

    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    Ok(CommandOutput {
        text: String::new(),
        json,
    })
}
