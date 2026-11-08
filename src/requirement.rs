use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Req,
    Nfr,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Req => "req",
            Kind::Nfr => "nfr",
        }
    }

    pub fn supports_story(self) -> bool {
        matches!(self, Kind::Req)
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDoc {
    pub id: String,
    pub title: String,
    pub story: Option<String>,
    pub group_id: Option<String>,
    pub group_title: Option<String>,
    pub description: Option<String>,
}

pub fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

pub fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

pub fn type_entry(cfg: &Config, key: &str) -> Result<crate::config::DocTypeEntry, AppError> {
    let entry = cfg
        .document_types
        .get(key)
        .ok_or_else(|| AppError::NotFound(format!("document type not configured: {}", key)))?;
    if !entry.enabled {
        return Err(AppError::Validation(format!(
            "document type disabled: {}",
            key
        )));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(format!(
            "document type prefix empty: {}",
            key
        )));
    }
    Ok(entry.clone())
}

pub fn active_dir(ctx: &Context, cfg: &Config, key: &str) -> Result<PathBuf, AppError> {
    let entry = type_entry(cfg, key)?;
    let raw = ctx
        .project_dir
        .join(&cfg.doc_directory)
        .join(&entry.directory);
    Context::ensure_within(&ctx.project_dir, &raw)
}

pub fn deprecated_dir(ctx: &Context, cfg: &Config, key: &str) -> Result<PathBuf, AppError> {
    Ok(active_dir(ctx, cfg, key)?.join("deprecated"))
}

fn ensure_dirs(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

fn scan_max_group(active: &Path, deprecated: &Path, prefix: &str) -> u32 {
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

fn scan_max_seq(active: &Path, deprecated: &Path, group_id: &str) -> u32 {
    let mut max = 0u32;
    let prefix = format!("{}.", group_id);
    for dir in [active, deprecated] {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Some(rest) = stem.strip_prefix(&prefix)
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

fn is_group_id(id: &str, prefix: &str) -> bool {
    if let Some(rest) = id.strip_prefix(&format!("{}-", prefix)) {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

pub fn create_group(
    ctx: &Context,
    kind: Kind,
    title_raw: &str,
    story_raw: Option<&str>,
    desc_raw: Option<&str>,
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg, kind.key())?;
    let title = sanitize_text(title_raw)?;
    let story = match story_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };

    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;
    ensure_dirs(&active)?;
    ensure_dirs(&deprecated)?;

    let max = scan_max_group(&active, &deprecated, &entry.prefix);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        entry.prefix,
        next,
        width = cfg.id_padding as usize
    );

    let content = render_group(kind, &id, &title, story.as_deref(), desc.as_deref());

    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
    Ok(id)
}

pub fn create(
    ctx: &Context,
    kind: Kind,
    title_raw: Option<&str>,
    title_flag_raw: Option<&str>,
    story_raw: Option<&str>,
    desc_raw: Option<&str>,
    group_raw: &str,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let title_string = match (title_raw, title_flag_raw) {
        (Some(a), Some(b)) => {
            let sa = sanitize_text(a)?;
            let sb = sanitize_text(b)?;
            if sa != sb {
                return Err(AppError::Validation(
                    "conflicting title sources".to_string(),
                ));
            }
            sa
        }
        (Some(a), None) => sanitize_text(a)?,
        (None, Some(b)) => sanitize_text(b)?,
        (None, None) => {
            return Err(AppError::Validation("title is required".to_string()));
        }
    };

    let cfg = load_config(ctx)?;
    let entry = type_entry(&cfg, kind.key())?;

    let story = match story_raw {
        Some(s) if kind.supports_story() => Some(sanitize_text(s)?),
        Some(_) => None,
        None => None,
    };
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
    let group_id = sanitize_text(group_raw)?;

    if !is_group_id(&group_id, &entry.prefix) {
        let other_kind = match kind {
            Kind::Req => Kind::Nfr,
            Kind::Nfr => Kind::Req,
        };
        let mismatch = type_entry(&cfg, other_kind.key())
            .ok()
            .filter(|other_entry| is_group_id(&group_id, &other_entry.prefix))
            .map(|_| {
                let other_active = active_dir(ctx, &cfg, other_kind.key()).ok();
                let other_deprecated = deprecated_dir(ctx, &cfg, other_kind.key()).ok();
                other_active
                    .as_ref()
                    .map(|d| d.join(format!("{}.md", group_id)).exists())
                    .unwrap_or(false)
                    || other_deprecated
                        .as_ref()
                        .map(|d| d.join(format!("{}.md", group_id)).exists())
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if mismatch {
            return Err(AppError::Validation(format!(
                "group type mismatch: {}",
                group_id
            )));
        }
        return Err(AppError::NotFound(format!("group not found: {}", group_id)));
    }

    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;
    ensure_dirs(&active)?;
    ensure_dirs(&deprecated)?;

    let group_active_path = active.join(format!("{}.md", group_id));
    let group_deprecated_path = deprecated.join(format!("{}.md", group_id));
    let group_active_safe = Context::ensure_within(&ctx.project_dir, &group_active_path)?;
    let group_deprecated_safe = Context::ensure_within(&ctx.project_dir, &group_deprecated_path)?;
    let group_deprecated = if group_active_safe.exists() {
        false
    } else if group_deprecated_safe.exists() {
        true
    } else {
        return Err(AppError::NotFound(format!("group not found: {}", group_id)));
    };
    if group_deprecated {
        return Err(AppError::Validation(format!(
            "group is deprecated: {}",
            group_id
        )));
    }

    let group_content =
        fs::read_to_string(&group_active_safe).map_err(|e| AppError::Runtime(e.to_string()))?;
    let group_parsed = parse_document(&group_content);
    let group_title = group_parsed.title.clone();
    if group_title.is_empty() {
        return Err(AppError::Runtime(format!(
            "group has no title: {}",
            group_id
        )));
    }

    let max_seq = scan_max_seq(&active, &deprecated, &group_id);
    let next_seq = max_seq + 1;
    let req_id = format!(
        "{}.{:0width$}",
        group_id,
        next_seq,
        width = cfg.id_padding as usize
    );

    let body = render_requirement(
        kind,
        &req_id,
        &title_string,
        story.as_deref(),
        desc.as_deref(),
        &group_id,
        &group_title,
    );

    let now = crate::record::frontmatter::utc_now();
    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", req_id));
    fm.push_str(&format!(
        "title: {}\n",
        crate::record::frontmatter::format_scalar_pub(&title_string)
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

    let content = crate::record::frontmatter::wrap(&fm, &body);

    let file_path = active.join(format!("{}.md", req_id));
    let safe_file = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_file, &content)?;
    Ok(req_id)
}

pub fn list(ctx: &Context, kind: Kind) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg, kind.key())?;
    ensure_dirs(&active).ok();
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;

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
            let (_, title) = parse_heading(&content);
            let title = if title.is_empty() { id.clone() } else { title };
            items.push((id, title));
        }
    }
    let _ = deprecated;
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

pub fn parse_document(content: &str) -> ParsedDoc {
    let (id, title) = parse_heading(content);
    let mut story: Option<String> = None;
    let mut group_id: Option<String> = None;
    let mut group_title: Option<String> = None;
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
            match section.as_str() {
                "## User Story" => story = val,
                "## Group" => {
                    if let Some(t) = val {
                        let mut parts = t.splitn(2, char::is_whitespace);
                        group_id = parts
                            .next()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        group_title = parts
                            .next()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                    }
                }
                "## Description" => desc = val,
                _ => {}
            }
        } else {
            i += 1;
        }
    }
    ParsedDoc {
        id,
        title,
        story,
        group_id,
        group_title,
        description: desc,
    }
}

pub fn view(ctx: &Context, kind: Kind, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;

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

fn frontmatter_list_req(content: &str, key: &str) -> Vec<String> {
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
    kind: Kind,
    id_raw: &str,
    title_raw: Option<&str>,
    story_raw: Option<&str>,
    desc_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;

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
    let mut parsed = parse_document(&content);

    if let Some(t) = title_raw {
        parsed.title = sanitize_text(t)?;
    }
    if kind.supports_story()
        && let Some(s) = story_raw
    {
        parsed.story = Some(sanitize_text(s)?);
    }
    if let Some(d) = desc_raw {
        parsed.description = Some(sanitize_text(d)?);
    }

    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            parsed.title.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            parsed.title.clone(),
        ));
    }
    if kind.supports_story() && story_raw.is_some() {
        if let Some(s) = &parsed.story {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## User Story".into(),
                s.clone(),
            ));
        }
    }
    if desc_raw.is_some() {
        if let Some(d) = &parsed.description {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Description".into(),
                d.clone(),
            ));
        }
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_req(&content, "labels");
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
        let mut fm_refs = frontmatter_list_req(&content, "references");
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
        let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;
        crate::record::store::write_atomic(ctx, &safe_write, &new_content)?;
    }

    let safe_write = Context::ensure_within(&ctx.project_dir, &path)?;

    if editor {
        let editor_cmd = cfg.effective_editor();
        if let Some(cmd) = editor_cmd {
            let mut parts = cmd.split_whitespace();
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
        } else {
            return Err(AppError::Runtime("no editor configured".to_string()));
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

pub fn deprecate(
    ctx: &Context,
    kind: Kind,
    id_raw: &str,
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;
    ensure_dirs(&active).ok();
    ensure_dirs(&deprecated).ok();

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

pub fn restore(ctx: &Context, kind: Kind, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg, kind.key())?;
    let deprecated = deprecated_dir(ctx, &cfg, kind.key())?;
    ensure_dirs(&active).ok();
    ensure_dirs(&deprecated).ok();

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

fn render_group(
    kind: Kind,
    id: &str,
    title: &str,
    story: Option<&str>,
    desc: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", id, title));
    if kind.supports_story()
        && let Some(s) = story
    {
        parts.push(format!("## User Story\n{}", s));
    }
    if let Some(d) = desc {
        parts.push(format!("## Description\n{}", d));
    }
    parts.join("\n\n") + "\n"
}

fn render_requirement(
    kind: Kind,
    id: &str,
    title: &str,
    story: Option<&str>,
    desc: Option<&str>,
    group_id: &str,
    group_title: &str,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", id, title));
    if kind.supports_story()
        && let Some(s) = story
    {
        parts.push(format!("## User Story\n{}", s));
    }
    parts.push(format!("## Group\n{} {}", group_id, group_title));
    if let Some(d) = desc {
        parts.push(format!("## Description\n{}", d));
    }
    parts.join("\n\n") + "\n"
}
