use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum GlossaryCmd {
    /// Create a glossary term.
    Create {
        /// Glossary term.
        #[arg(long = "term")]
        term: String,
        /// Definition of the term.
        #[arg(long = "definition")]
        definition: String,
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// List glossary terms.
    List,
    /// View a glossary term.
    View { id: String },
    /// Edit a glossary term.
    Edit {
        id: String,
        /// Update the glossary term.
        #[arg(long = "term")]
        term: Option<String>,
        /// Update the definition.
        #[arg(long = "definition")]
        definition: Option<String>,
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
    /// Deprecate a glossary term.
    Deprecate { id: String },
    /// Restore a deprecated glossary term.
    Restore { id: String },
}

pub fn run(ctx: &Context, cmd: GlossaryCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        GlossaryCmd::Create {
            term,
            definition,
            label,
            reference,
        } => {
            let id = create(ctx, &term, &definition, &label, &reference)?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        GlossaryCmd::List => list(ctx),
        GlossaryCmd::View { id } => view(ctx, &id),
        GlossaryCmd::Edit {
            id,
            term,
            definition,
            editor,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            term.as_deref(),
            definition.as_deref(),
            &label,
            &reference,
            editor,
        ),
        GlossaryCmd::Deprecate { id } => deprecate(ctx, &id),
        GlossaryCmd::Restore { id } => restore(ctx, &id),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    let value = sanitize::sanitize_name(input)
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

fn type_entry(cfg: &Config) -> Result<crate::config::DocTypeEntry, AppError> {
    let entry = cfg
        .document_types
        .get("glossary")
        .ok_or_else(|| AppError::NotFound("document type not configured: glossary".to_string()))?;
    if !entry.enabled {
        return Err(AppError::Validation(
            "document type disabled: glossary".to_string(),
        ));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(
            "document type prefix empty: glossary".to_string(),
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

fn ensure_dirs(dir: &std::path::Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

#[derive(Clone, Debug, Default)]
struct GlossaryData {
    id: String,
    term: String,
    definition: String,
}

fn record_from_document(doc: &crate::record::document::Document) -> GlossaryData {
    let (id, term) = crate::record::document::heading_id_title(doc);
    let definition = crate::record::document::section_text(doc, "Definition").unwrap_or_default();
    GlossaryData {
        id,
        term,
        definition,
    }
}

fn body_document(id: &str, term: &str, definition: &str) -> crate::record::document::Document {
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {id} {term}")),
        preamble: vec![String::new()],
        sections: vec![crate::record::document::Section {
            heading: "## Definition".to_string(),
            body: vec![definition.to_string()],
        }],
    }
}

pub fn create(
    ctx: &Context,
    term_raw: &str,
    definition_raw: &str,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let term = sanitize_text(term_raw)?;
    let definition = sanitize_text(definition_raw)?;
    let entry = type_entry(&cfg)?;

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

    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    ensure_dirs(&active)?;
    ensure_dirs(&deprecated)?;

    let next = crate::record::identity::next_number(&active, &deprecated, &entry.prefix);
    let id =
        crate::record::identity::format_id(&entry.prefix, next as i64, cfg.id_padding as usize);

    let body_doc = body_document(&id, &term, &definition);
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
            crate::record::frontmatter::FrontMatterValue::String(term.clone()),
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

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = deprecated;

    let mut items: Vec<(String, String, String)> = Vec::new();
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
            let term = crate::record::document::parse(&content)
                .ok()
                .map(|d| crate::record::document::heading_id_title(&d).1)
                .unwrap_or_default();
            let term = if term.is_empty() { id.clone() } else { term };
            items.push((id, term, content));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, term, content)| {
            crate::record::view::RecordView::lenient(content, id).list_line(term)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, term, content)| {
                serde_json::json!({
                    "id": id,
                    "term": term,
                    "title": term,
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
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);

    let active_path = active.join(format!("{}.md", id));
    let deprecated_path = deprecated.join(format!("{}.md", id));

    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let deprecated_safe = Context::ensure_within(&ctx.project_dir, &deprecated_path)?;

    let path = if active_safe.exists() {
        active_safe
    } else if deprecated_safe.exists() {
        deprecated_safe
    } else {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let content = crate::record::migrate::ensure_front_matter(ctx, &path)?;
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

fn frontmatter_list_glo(content: &str, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(Some(fm)) = crate::record::frontmatter::parse(content) {
        if let Some(yaml_serde::Value::Sequence(seq)) =
            fm.data.get(yaml_serde::Value::String(key.to_string()))
        {
            for v in seq {
                if let yaml_serde::Value::String(s) = v {
                    out.push(s.clone());
                }
            }
        }
    }
    out
}

pub fn edit(
    ctx: &Context,
    id_raw: &str,
    term_raw: Option<&str>,
    definition_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);

    let active_path = active.join(format!("{}.md", id));
    let deprecated_path = deprecated.join(format!("{}.md", id));

    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let deprecated_safe = Context::ensure_within(&ctx.project_dir, &deprecated_path)?;

    let path = if active_safe.exists() {
        active_safe
    } else if deprecated_safe.exists() {
        deprecated_safe
    } else {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let (doc, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();
    let mut gd = record_from_document(&doc);
    if gd.id.is_empty() {
        gd.id = id.clone();
    }
    if gd.term.is_empty() {
        gd.term = id.clone();
    }

    if let Some(t) = term_raw {
        gd.term = sanitize_text(t)?;
    }
    if let Some(d) = definition_raw {
        gd.definition = sanitize_text(d)?;
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
    if term_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            gd.term.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            gd.term.clone(),
        ));
    }
    if definition_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Definition".into(),
            gd.definition.clone(),
        ));
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_glo(&content, "labels");
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
        let mut fm_refs = frontmatter_list_glo(&content, "references");
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

    let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;

    if let Some(command) = editor_cmd {
        let mut parts = command.split_whitespace();
        let prog = parts.next().unwrap_or("");
        if prog.is_empty() {
            return Err(AppError::Runtime("no editor configured".to_string()));
        }
        let status = std::process::Command::new(prog)
            .args(parts)
            .arg(&safe_write)
            .status()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        if !status.success() {
            return Err(AppError::Runtime(format!("editor exited with {}", status)));
        }
    }

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

pub fn deprecate(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    ensure_dirs(&deprecated)?;

    let active_path = active.join(format!("{}.md", id));
    let deprecated_path = deprecated.join(format!("{}.md", id));

    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let deprecated_safe = Context::ensure_within(&ctx.project_dir, &deprecated_path)?;

    if deprecated_safe.exists() && !active_safe.exists() {
        return Err(AppError::Validation(format!("already deprecated: {}", id)));
    }
    if !active_safe.exists() {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }

    fs::rename(&active_safe, &deprecated_safe).map_err(|e| AppError::Runtime(e.to_string()))?;

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

pub fn restore(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);

    let active_path = active.join(format!("{}.md", id));
    let deprecated_path = deprecated.join(format!("{}.md", id));

    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let deprecated_safe = Context::ensure_within(&ctx.project_dir, &deprecated_path)?;

    if active_safe.exists() && !deprecated_safe.exists() {
        return Err(AppError::Validation(format!("not deprecated: {}", id)));
    }
    if !deprecated_safe.exists() {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }

    fs::rename(&deprecated_safe, &active_safe).map_err(|e| AppError::Runtime(e.to_string()))?;

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
