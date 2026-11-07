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
        } => {
            let id = create(
                ctx,
                &title,
                &description,
                &rationale,
                &applicability,
                &requirement,
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
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            description.as_deref(),
            &requirement,
            no_requirement,
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
pub(crate) struct RuleRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) rationale: String,
    pub(crate) applicability: String,
    pub(crate) requirements: Vec<String>,
}

pub(crate) fn parse_heading(content: &str) -> (String, String) {
    let first = content.lines().next().unwrap_or("");
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

fn parse_requirements(content: &str) -> Vec<String> {
    let Some(body) = section_text(content, "Requirements") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let item = if let Some(rest) = t.strip_prefix("- ") {
            rest.trim()
        } else if let Some(rest) = t.strip_prefix('-') {
            rest.trim()
        } else {
            t
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

pub(crate) fn parse_record(content: &str, fallback_id: &str) -> RuleRecord {
    let (id, title) = parse_heading(content);
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
        description: section_text(content, "Description").unwrap_or_default(),
        rationale: section_text(content, "Rationale").unwrap_or_default(),
        applicability: section_text(content, "Applicability").unwrap_or_default(),
        requirements: parse_requirements(content),
    }
}

pub(crate) fn render_record(record: &RuleRecord) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", record.id, record.title));
    parts.push(format!("## Description\n{}", record.description));
    parts.push(format!("## Rationale\n{}", record.rationale));
    parts.push(format!("## Applicability\n{}", record.applicability));
    if !record.requirements.is_empty() {
        let bullets = record
            .requirements
            .iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Requirements\n{}", bullets));
    }
    parts.join("\n\n") + "\n"
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

pub fn create(
    ctx: &Context,
    title_raw: &str,
    description_raw: &str,
    rationale_raw: &str,
    applicability_raw: &str,
    requirements_raw: &[String],
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

    let record = RuleRecord {
        id: id.clone(),
        title,
        description,
        rationale,
        applicability,
        requirements,
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
    description_raw: Option<&str>,
    requirements_add: &[String],
    no_requirement: Option<Option<String>>,
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
    if let Some(no) = no_requirement {
        match no {
            Some(rid) => {
                let rid = sanitize_field(&rid)?;
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
