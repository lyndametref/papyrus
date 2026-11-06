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
    },
    /// Deprecate a glossary term.
    Deprecate { id: String },
    /// Restore a deprecated glossary term.
    Restore { id: String },
}

pub fn run(ctx: &Context, cmd: GlossaryCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        GlossaryCmd::Create { term, definition } => {
            let id = create(ctx, &term, &definition)?;
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
        } => edit(ctx, &id, term.as_deref(), definition.as_deref(), editor),
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

fn scan_max(active: &std::path::Path, deprecated: &std::path::Path, prefix: &str) -> u32 {
    let mut max = 0u32;
    for dir in [active, deprecated] {
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
    }
    max
}

#[derive(Clone, Debug, Default)]
struct GlossaryData {
    id: String,
    term: String,
    definition: String,
}

fn parse_heading(content: &str) -> (String, String) {
    let first = content.lines().next().unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let term = parts.next().unwrap_or("").trim().to_string();
    (id, term)
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

fn parse_glossary(content: &str) -> GlossaryData {
    let (id, term) = parse_heading(content);
    let definition = section_text(content, "Definition").unwrap_or_default();
    GlossaryData {
        id,
        term,
        definition,
    }
}

fn render_glossary(gd: &GlossaryData) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", gd.id, gd.term));
    parts.push(format!("## Definition\n{}", gd.definition));
    parts.join("\n\n") + "\n"
}

pub fn create(ctx: &Context, term_raw: &str, definition_raw: &str) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let term = sanitize_text(term_raw)?;
    let definition = sanitize_text(definition_raw)?;
    let entry = type_entry(&cfg)?;

    let active = active_dir(ctx, &cfg)?;
    let deprecated = deprecated_dir(ctx, &cfg)?;
    ensure_dirs(&active)?;
    ensure_dirs(&deprecated)?;

    let max = scan_max(&active, &deprecated, &entry.prefix);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        entry.prefix,
        next,
        width = cfg.id_padding as usize
    );

    let gd = GlossaryData {
        id: id.clone(),
        term,
        definition,
    };
    let content = render_glossary(&gd);
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    fs::write(&safe_path, content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(id)
}

pub fn list(ctx: &Context) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dirs(&active);
    let deprecated = deprecated_dir(ctx, &cfg)?;
    let _ = deprecated;

    let mut items: Vec<(String, String)> = Vec::new();
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
            let (_, term) = parse_heading(&content);
            let term = if term.is_empty() { id.clone() } else { term };
            items.push((id, term));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, term)| format!("{} {}", id, term))
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, term)| serde_json::json!({"id": id, "term": term, "title": term}))
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
    term_raw: Option<&str>,
    definition_raw: Option<&str>,
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

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut gd = parse_glossary(&content);
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

    let new_content = render_glossary(&gd);
    let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
    fs::write(&safe_write, new_content.as_bytes()).map_err(|e| AppError::Runtime(e.to_string()))?;

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
