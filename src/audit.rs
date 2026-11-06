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
        } => {
            let id = create(ctx, &title, &content, audit_type.as_deref())?;
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
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            content.as_deref(),
            audit_type.as_deref(),
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
struct AuditData {
    id: String,
    title: String,
    audit_type: String,
    content: String,
}

fn parse_heading(content: &str) -> (String, String) {
    let first = content.lines().next().unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let title = parts.next().unwrap_or("").trim().to_string();
    (id, title)
}

fn section_text(content: &str, name: &str) -> Option<String> {
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

fn parse_record(content: &str, fallback_id: &str) -> AuditData {
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
) -> Result<String, AppError> {
    let title = sanitize_field(title_raw)?;
    let content = sanitize_field(content_raw)?;
    let audit_type = match audit_type_raw {
        Some(v) => sanitize_field(v)?,
        None => "security".to_string(),
    };

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
        title,
        audit_type,
        content,
    };
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    fs::write(&safe_path, render_record(&record).as_bytes())
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);

    let mut items: Vec<(String, String)> = Vec::new();
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
            items.push((id.to_string(), title));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, title)| format!("{} {}", id, title))
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, title)| serde_json::json!({"id": id, "title": title}))
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

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let json = if ctx.json {
        Some(serde_json::json!({
            "id": id,
            "path": path.display().to_string(),
            "content": content,
        }))
    } else {
        None
    };
    let text = content.trim_end_matches('\n').to_string();
    Ok(CommandOutput { text, json })
}

pub fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    content_raw: Option<&str>,
    audit_type_raw: Option<&str>,
    editor: bool,
) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&deprecated);
    let path = active_or_deprecated(ctx, &active, &deprecated, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
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

    let editor_cmd = if editor {
        Some(
            cfg.effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    let new_content = render_record(&record);
    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
    fs::write(&safe_path, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

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
