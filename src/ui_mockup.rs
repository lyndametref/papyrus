#![allow(clippy::too_many_arguments)]
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
pub enum UiMockupCmd {
    Create {
        title: String,
        #[arg(long = "task")]
        task: String,
        #[arg(long = "spec", required = true)]
        spec: Vec<String>,
        #[arg(long = "ref")]
        r#ref: Vec<String>,
    },
    List,
    View {
        id: String,
    },
    Edit {
        id: String,
        #[arg(long = "title")]
        title: Option<String>,
        #[arg(long = "task")]
        task: Option<String>,
        #[arg(long = "spec")]
        spec: Option<Vec<String>>,
        #[arg(long = "ref")]
        r#ref: Option<Vec<String>>,
        #[arg(long = "status", conflicts_with = "superseded_by")]
        status: Option<String>,
        #[arg(long = "superseded-by", conflicts_with = "status")]
        superseded_by: Option<String>,
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

pub fn run(ctx: &Context, cmd: UiMockupCmd) -> Result<CommandOutput, AppError> {
    match cmd {
        UiMockupCmd::Create {
            title,
            task,
            spec,
            r#ref,
        } => {
            let id = create(ctx, &title, &task, &spec, &r#ref)?;
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(CommandOutput { text: id, json })
        }
        UiMockupCmd::List => list(ctx),
        UiMockupCmd::View { id } => view(ctx, &id),
        UiMockupCmd::Edit {
            id,
            title,
            task,
            spec,
            r#ref,
            status,
            superseded_by,
            editor,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            task.as_deref(),
            spec.as_deref(),
            r#ref.as_deref(),
            status.as_deref(),
            superseded_by.as_deref(),
            editor,
        ),
        UiMockupCmd::Deprecate { id } => deprecate(ctx, &id),
        UiMockupCmd::Restore { id } => restore(ctx, &id),
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

fn sanitize_list(values: &[String]) -> Result<Vec<String>, AppError> {
    values.iter().map(|v| sanitize_field(v)).collect()
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn type_entry(cfg: &Config) -> Result<DocTypeEntry, AppError> {
    let entry = cfg
        .document_types
        .get("ui-mockup")
        .ok_or_else(|| AppError::NotFound("document type not configured: ui-mockup".to_string()))?;
    if !entry.enabled {
        return Err(AppError::Validation(
            "document type disabled: ui-mockup".to_string(),
        ));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(
            "document type prefix empty: ui-mockup".to_string(),
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
    path.extension().and_then(|s| s.to_str()) == Some("html")
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
            if path.extension().and_then(|s| s.to_str()) != Some("html") {
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
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
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

fn resolve_record(
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
struct Mockup {
    id: String,
    title: String,
    task: String,
    specs: Vec<String>,
    refs: Vec<String>,
    status: String,
    superseded_by: Option<String>,
}

fn parse_field_comment(content: &str, key: &str) -> Option<String> {
    let mut start = 0usize;
    while let Some(i) = content[start..].find("<!--") {
        let i = start + i;
        if let Some(j) = content[i..].find("-->") {
            let inner = content[i + 4..i + j].trim();
            if let Some(rest) = inner.strip_prefix(&format!("{}:", key)) {
                return Some(rest.trim().to_string());
            }
            start = i + j + 3;
        } else {
            break;
        }
    }
    None
}

fn parse_visible(content: &str, prefix: &str) -> Option<String> {
    if let Some(i) = content.find(prefix) {
        let rest = &content[i + prefix.len()..];
        // trim until closing tag if present
        let val = if let Some(j) = rest.find("</") {
            rest[..j].trim()
        } else {
            rest.trim()
        };
        // strip trailing '>'? no
        return Some(val.to_string());
    }
    None
}

fn split_list(s: &str) -> Vec<String> {
    s.split_whitespace().map(|x| x.to_string()).collect()
}

fn extract_between(content: &str, start: &str, end: &str) -> Option<String> {
    let i = content.find(start)?;
    let rest = &content[i + start.len()..];
    let j = rest.find(end)?;
    Some(rest[..j].trim().to_string())
}

fn parse_record(content: &str, fallback_id: &str) -> Mockup {
    let mut id = String::new();
    let mut title = String::new();
    if let Some(val) = extract_between(content, "<h1>", "</h1>") {
        let mut parts = val.splitn(2, char::is_whitespace);
        id = parts.next().unwrap_or("").trim().to_string();
        title = parts.next().unwrap_or("").trim().to_string();
    }
    if id.is_empty() {
        id = parse_field_comment(content, "mockup-id").unwrap_or_default();
    }
    if id.is_empty() {
        id = fallback_id.to_string();
    }
    if title.is_empty()
        && let Some(rest) = extract_between(content, "<title>", "</title>")
    {
        let rest = rest.trim();
        if let Some(idx) = rest.find('—') {
            title = rest[idx + '—'.len_utf8()..].trim().to_string();
        } else {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let _ = parts.next();
            title = parts.next().unwrap_or("").trim().to_string();
        }
    }
    if title.is_empty() {
        title = id.clone();
    }

    let status_comment = parse_field_comment(content, "status");
    let status_visible = parse_visible(content, "<p>Status:");
    let mut status = status_visible
        .or(status_comment)
        .unwrap_or_else(|| "pending".to_string());
    if status.is_empty() {
        status = "pending".to_string();
    }

    let task_comment = parse_field_comment(content, "task");
    let task_visible = parse_visible(content, "<p>Task:");
    let task = task_visible.or(task_comment).unwrap_or_default();

    let specs_comment = parse_field_comment(content, "specs");
    let specs_visible = parse_visible(content, "<p>UI specifications:");
    let specs_str = specs_visible.or(specs_comment).unwrap_or_default();
    let specs = split_list(&specs_str);

    let refs_comment = parse_field_comment(content, "refs");
    let refs_visible = parse_visible(content, "<p>References:");
    let refs_str = refs_visible.or(refs_comment).unwrap_or_default();
    let refs = split_list(&refs_str);

    let superseded_visible = parse_visible(content, "<p>Superseded by:");
    let superseded_comment = parse_field_comment(content, "superseded-by");
    let superseded_str = superseded_visible
        .or(superseded_comment)
        .unwrap_or_default();
    let superseded_by = if superseded_str.trim().is_empty() {
        None
    } else {
        Some(superseded_str.trim().to_string())
    };

    Mockup {
        id,
        title,
        task,
        specs,
        refs,
        status,
        superseded_by,
    }
}

fn render_record(record: &Mockup) -> String {
    let specs_str = record.specs.join(" ");
    let refs_str = record.refs.join(" ");
    let superseded_str = record.superseded_by.clone().unwrap_or_default();
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{} — {}</title>\n</head>\n<body>\n<!-- mockup-id: {} -->\n<!-- status: {} -->\n<!-- task: {} -->\n<!-- specs: {} -->\n<!-- refs: {} -->\n<h1>{} {}</h1>\n<p>Status: {}</p>\n<p>Task: {}</p>\n<p>UI specifications: {}</p>\n<p>References: {}</p>\n<p>Superseded by: {}</p>\n</body>\n</html>\n",
        record.id,
        record.title,
        record.id,
        record.status,
        record.task,
        specs_str,
        refs_str,
        record.id,
        record.title,
        record.status,
        record.task,
        specs_str,
        refs_str,
        superseded_str
    )
}

const ALLOWED_STATUSES: &[&str] = &["pending", "approved", "deprecated"];

pub fn valid_transition(current: &str, next: &str) -> bool {
    if current == next {
        return true;
    }
    match current {
        "pending" => matches!(next, "approved" | "deprecated"),
        "approved" => matches!(next, "deprecated"),
        _ => false,
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

fn create(
    ctx: &Context,
    title_raw: &str,
    task_raw: &str,
    specs_raw: &[String],
    refs_raw: &[String],
) -> Result<String, AppError> {
    let title = sanitize_field(title_raw)?;
    let task = sanitize_field(task_raw)?;
    let specs = sanitize_list(specs_raw)?;
    if specs.is_empty() {
        return Err(AppError::Validation(
            "at least one spec is required".to_string(),
        ));
    }
    let refs = sanitize_list(refs_raw)?;

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

    let record = Mockup {
        id: id.clone(),
        title,
        task,
        specs,
        refs,
        status: "pending".to_string(),
        superseded_by: None,
    };
    let file_path = active.join(format!("{}.html", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    fs::write(&safe_path, render_record(&record).as_bytes())
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(id)
}

fn list(ctx: &Context) -> Result<CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg)?;
    let active = active_dir(ctx, &cfg)?;

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
            let record = parse_record(&content, id);
            let title = if record.title.is_empty() {
                id.to_string()
            } else {
                record.title
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

fn view(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let path = resolve_record(ctx, &active, &deprecated, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let json = if ctx.json {
        Some(serde_json::json!({"id": id, "path": path.display().to_string(), "content": content}))
    } else {
        None
    };
    let text = content.trim_end_matches('\n').to_string();
    Ok(CommandOutput { text, json })
}

fn edit(
    ctx: &Context,
    id_raw: &str,
    title_raw: Option<&str>,
    task_raw: Option<&str>,
    specs_raw: Option<&[String]>,
    refs_raw: Option<&[String]>,
    status_raw: Option<&str>,
    superseded_by_raw: Option<&str>,
    editor: bool,
) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let title = title_raw.map(sanitize_field).transpose()?;
    let task = task_raw.map(sanitize_field).transpose()?;
    let specs = specs_raw.map(sanitize_list).transpose()?;
    let refs = refs_raw.map(sanitize_list).transpose()?;
    let status = status_raw.map(sanitize_field).transpose()?;
    let superseded_by = superseded_by_raw.map(sanitize_field).transpose()?;

    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let path = resolve_record(ctx, &active, &deprecated, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut record = parse_record(&content, &id);
    if record.id.is_empty() {
        record.id = id.clone();
    }
    if record.title.is_empty() {
        record.title = id.clone();
    }
    if record.status.is_empty() {
        record.status = "pending".to_string();
    }

    if let Some(t) = title {
        record.title = t;
    }
    if let Some(t) = task {
        record.task = t;
    }
    if let Some(s) = specs {
        record.specs = s;
    }
    if let Some(r) = refs {
        record.refs = r;
    }

    let editor_cmd = if editor {
        Some(
            cfg.effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    if let Some(st) = status {
        if !ALLOWED_STATUSES.contains(&st.as_str()) {
            return Err(AppError::Validation(format!("unknown status: {}", st)));
        }
        if record.superseded_by.is_some() {
            return Err(AppError::Validation(format!(
                "status transitions blocked for superseded mockup: {}",
                id
            )));
        }
        if !valid_transition(&record.status, &st) {
            return Err(AppError::Validation(format!(
                "invalid status transition: {} -> {}",
                record.status, st
            )));
        }
        record.status = st;
    }

    let mut move_to_deprecated = false;
    if let Some(sup) = superseded_by {
        if sup == id {
            return Err(AppError::Validation(format!(
                "mockup cannot supersede itself: {}",
                id
            )));
        }
        let _ = resolve_record(ctx, &active, &deprecated, &sup)?;
        record.status = "deprecated".to_string();
        record.superseded_by = Some(sup);
        move_to_deprecated = true;
    }

    let destination = if move_to_deprecated {
        ensure_dirs(&deprecated)?;
        deprecated.join(format!("{}.html", id))
    } else {
        path.clone()
    };
    let safe_dest = Context::ensure_within(&ctx.project_dir, &destination)?;

    if move_to_deprecated && path != safe_dest {
        fs::rename(&path, &safe_dest).map_err(|e| AppError::Runtime(e.to_string()))?;
    }

    let new_content = render_record(&record);
    fs::write(&safe_dest, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

    if let Some(command) = editor_cmd {
        run_editor(&command, &safe_dest)?;
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

fn deprecate(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;

    let Some(path) = find_record(ctx, &active, &id)? else {
        if find_record(ctx, &deprecated, &id)?.is_some() {
            return Err(AppError::Validation(format!("already deprecated: {}", id)));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let record = parse_record(&content, &id);
    if record.superseded_by.is_some() {
        return Err(AppError::Validation(format!(
            "status transitions blocked for superseded mockup: {}",
            id
        )));
    }

    ensure_dirs(&deprecated)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?;
    let destination = deprecated.join(file_name);
    let safe_dest = Context::ensure_within(&ctx.project_dir, &destination)?;

    fs::rename(&path, &safe_dest).map_err(|e| AppError::Runtime(e.to_string()))?;

    let content = fs::read_to_string(&safe_dest).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut record = parse_record(&content, &id);
    if record.id.is_empty() {
        record.id = id.clone();
    }
    record.status = "deprecated".to_string();
    let new_content = render_record(&record);
    fs::write(&safe_dest, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

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

fn restore(ctx: &Context, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;

    let Some(path) = find_record(ctx, &deprecated, &id)? else {
        if find_record(ctx, &active, &id)?.is_some() {
            return Err(AppError::Validation(format!("not deprecated: {}", id)));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let record = parse_record(&content, &id);
    if record.superseded_by.is_some() {
        return Err(AppError::Validation(format!(
            "status transitions blocked for superseded mockup: {}",
            id
        )));
    }

    ensure_dirs(&active)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?;
    let destination = active.join(file_name);
    let safe_dest = Context::ensure_within(&ctx.project_dir, &destination)?;

    fs::rename(&path, &safe_dest).map_err(|e| AppError::Runtime(e.to_string()))?;

    let content = fs::read_to_string(&safe_dest).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut record = parse_record(&content, &id);
    if record.id.is_empty() {
        record.id = id.clone();
    }
    record.status = "pending".to_string();
    let new_content = render_record(&record);
    fs::write(&safe_dest, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_transition_table() {
        assert!(valid_transition("pending", "pending"));
        assert!(valid_transition("pending", "approved"));
        assert!(valid_transition("pending", "deprecated"));
        assert!(valid_transition("approved", "approved"));
        assert!(valid_transition("approved", "deprecated"));
        assert!(valid_transition("deprecated", "deprecated"));
        assert!(!valid_transition("approved", "pending"));
        assert!(!valid_transition("deprecated", "pending"));
        assert!(!valid_transition("deprecated", "approved"));
    }
}
