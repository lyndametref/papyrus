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
pub enum AuditCmd {
    Create {
        #[arg(long = "title")]
        title: String,
        #[arg(long = "content")]
        content: String,
        #[arg(long = "audit-type")]
        audit_type: Option<String>,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    List,
    View {
        id: String,
    },
    Edit {
        id: String,
        #[arg(long = "title")]
        title: Option<String>,
        #[arg(long = "content")]
        content: Option<String>,
        #[arg(long = "audit-type")]
        audit_type: Option<String>,
        #[arg(long = "editor")]
        editor: bool,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    Deprecate {
        id: String,
    },
    Restore {
        id: String,
    },
}

pub fn run(ctx: &Context, cmd: AuditCmd) -> Result<CommandOutput, AppError> {
    match cmd {
        AuditCmd::Create {
            title,
            content,
            audit_type,
            label,
            reference,
        } => {
            let id = create(
                ctx,
                &title,
                &content,
                audit_type.as_deref(),
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
        AuditCmd::List => list(ctx),
        AuditCmd::View { id } => view(ctx, &id),
        AuditCmd::Edit {
            id,
            title,
            content,
            audit_type,
            editor,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            content.as_deref(),
            audit_type.as_deref(),
            &label,
            &reference,
            editor,
        ),
        AuditCmd::Deprecate { id } => deprecate(ctx, &id),
        AuditCmd::Restore { id } => restore(ctx, &id),
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
        .get("audit")
        .ok_or_else(|| AppError::NotFound("document type not configured: audit".to_string()))?;
    if !entry.enabled {
        return Err(AppError::Validation(
            "document type disabled: audit".to_string(),
        ));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(
            "document type prefix empty: audit".to_string(),
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

fn id_number(stem: &str, prefix: &str) -> Option<u32> {
    let rest = stem.strip_prefix(prefix)?.strip_prefix('-')?;
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u32>().ok()
}

fn is_record_path(path: &Path, prefix: &str) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("md")
        && path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|stem| id_number(stem, prefix))
            .is_some()
}

fn scan_max(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
    let mut max = 0u32;
    for dir in [active, deprecated] {
        let Ok(rd) = fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Some(n) = id_number(stem, prefix)
                && n > max
            {
                max = n;
            }
        }
    }
    max
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
pub(crate) struct AuditData {
    id: String,
    title: String,
    audit_type: String,
    content: String,
}

pub(crate) fn parse_heading(content: &str) -> (String, String) {
    let body = if let Some((_, _, bs)) = crate::record::frontmatter::find_block(content) {
        &content[bs..]
    } else {
        content
    };
    let first = body
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .next()
        .unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let title = parts.next().unwrap_or("").trim().to_string();
    (id, title)
}

pub(crate) fn section_text(content: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let header = format!("## {}", name);
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].trim() == header {
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
            if text.trim().is_empty() {
                return None;
            }
            return Some(text);
        }
        i += 1;
    }
    None
}

pub(crate) fn parse_record(content: &str, fallback_id: &str) -> AuditData {
    let (id, title) = parse_heading(content);
    AuditData {
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
        audit_type: section_text(content, "Audit Type").unwrap_or_default(),
        content: section_text(content, "Content").unwrap_or_default(),
    }
}

fn render_record(record: &AuditData) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", record.id, record.title));
    parts.push(format!("## Audit Type\n{}", record.audit_type));
    parts.push(format!("## Content\n{}", record.content));
    parts.join("\n\n") + "\n"
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    content_raw: &str,
    audit_type_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let title = sanitize_field(title_raw)?;
    let content = sanitize_field(content_raw)?;
    let audit_type = match audit_type_raw {
        Some(v) => sanitize_field(v)?,
        None => "security".to_string(),
    };

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

    let next = scan_max(&active, &deprecated, &entry.prefix) + 1;
    let id = format!(
        "{}-{:0width$}",
        entry.prefix,
        next,
        width = cfg.id_padding as usize
    );

    let record = AuditData {
        id: id.clone(),
        title: title.clone(),
        audit_type: audit_type.clone(),
        content,
    };
    let body = render_record(&record);
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
    // audit type front-matter key "type"
    fm.push_str(&format!(
        "type: {}\n",
        crate::record::frontmatter::format_scalar_pub(&audit_type)
    ));
    let content = crate::record::frontmatter::wrap(&fm, &body);
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
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
            let (_, title) = parse_heading(&content);
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

fn frontmatter_list_audit(content: &str, key: &str) -> Vec<String> {
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
    title_raw: Option<&str>,
    content_raw: Option<&str>,
    audit_type_raw: Option<&str>,
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

    let editor_cmd = if editor {
        Some(
            cfg.effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    let content = crate::record::migrate::ensure_front_matter(ctx, &path)?;
    let mut record = parse_record(&content, &id);

    if let Some(t) = title_raw {
        record.title = sanitize_field(t)?;
    }
    if let Some(c) = content_raw {
        record.content = sanitize_field(c)?;
    }
    if let Some(a) = audit_type_raw {
        record.audit_type = sanitize_field(a)?;
    }

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
    if content_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Content".into(),
            record.content.clone(),
        ));
    }
    if audit_type_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "type".into(),
            record.audit_type.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Audit Type".into(),
            record.audit_type.clone(),
        ));
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_audit(&content, "labels");
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
        let mut fm_refs = frontmatter_list_audit(&content, "references");
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

    let doc = crate::record::document::parse(&content)?;
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
