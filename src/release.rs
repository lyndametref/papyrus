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
    },
}

pub fn run(ctx: &Context, cmd: ReleaseCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        ReleaseCmd::Create { title, description } => {
            let id = create(ctx, &title, description.as_deref())?;
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
        } => edit(ctx, &id, title.as_deref(), description.as_deref()),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn backlog_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let raw = ctx.project_dir.join(&cfg.backlog_directory);
    Context::ensure_within(&ctx.project_dir, &raw)
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

fn scan_max(dir: &std::path::Path, prefix: &str) -> u32 {
    let mut max = 0u32;
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Some(rest) = stem.strip_prefix(&format!("{}-", prefix))
                && !rest.is_empty()
                && rest.chars().all(|c| c.is_ascii_digit())
                && let Ok(n) = rest.parse::<u32>()
                && n > max
            {
                max = n;
            }
        }
    }
    max
}

fn parse_heading(content: &str) -> (String, String) {
    let first = content.lines().next().unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let title = parts.next().unwrap_or("").trim().to_string();
    (id, title)
}

pub(crate) fn parse_release(content: &str) -> (String, String, Option<String>) {
    let (id, title) = parse_heading(content);
    let mut desc: Option<String> = None;
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
            if section == "## Description" {
                desc = val;
            }
        } else {
            i += 1;
        }
    }
    (id, title, desc)
}

pub(crate) fn render_release(id: &str, title: &str, desc: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", id, title));
    if let Some(d) = desc {
        parts.push(format!("## Description\n{}", d));
    }
    parts.join("\n\n") + "\n"
}

pub fn create(ctx: &Context, title_raw: &str, desc_raw: Option<&str>) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };

    let dir = releases_dir(ctx, &cfg)?;
    ensure_dir(&dir, &ctx.project_dir)?;

    let max = scan_max(&dir, &cfg.release_prefix);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        cfg.release_prefix,
        next,
        width = cfg.id_padding as usize
    );

    let content = render_release(&id, &title, desc.as_deref());

    let file_path = dir.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    fs::write(&safe_path, content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);

    let mut items: Vec<(String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
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
            let (_, title) = parse_heading(&content);
            let title = if title.is_empty() { id.clone() } else { title };
            items.push((id, title));
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

    Ok(crate::CommandOutput { text, json })
}

pub fn view(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);

    let path_raw = dir.join(format!("{}.md", id));
    let path = Context::ensure_within(&ctx.project_dir, &path_raw)?;
    if !path.exists() {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }
    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;

    let text = content.clone();
    let json = if ctx.json {
        Some(serde_json::json!({"id": id, "path": path.display().to_string(), "content": content}))
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
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let dir = releases_dir(ctx, &cfg)?;
    let _ = ensure_dir(&dir, &ctx.project_dir);

    let path_raw = dir.join(format!("{}.md", id));
    let path = Context::ensure_within(&ctx.project_dir, &path_raw)?;
    if !path.exists() {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let (parsed_id, parsed_title, parsed_desc) = parse_release(&content);
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
    let new_content = render_release(&new_id, &title_val, desc_val.as_deref());

    let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
    fs::write(&safe_write, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

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
