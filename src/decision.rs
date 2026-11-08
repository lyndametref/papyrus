#![allow(clippy::too_many_arguments)]
use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum DecisionCmd {
    Create {
        #[arg(value_name = "title")]
        title: String,
        #[arg(short = 't', long = "decision-type")]
        decision_type: Option<String>,
        #[arg(short = 'd', long = "decision")]
        decision: Option<String>,
        #[arg(long = "context")]
        context: Option<String>,
        #[arg(long = "consequences")]
        consequences: Option<String>,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    List {
        #[arg(short = 't', long = "decision-type")]
        decision_type: Option<String>,
    },
    View {
        id: String,
    },
    Edit {
        id: String,
        #[arg(long = "title")]
        title: Option<String>,
        #[arg(short = 'd', long = "decision")]
        decision: Option<String>,
        #[arg(long = "status", conflicts_with = "superseded_by")]
        status: Option<String>,
        #[arg(long = "context")]
        context: Option<String>,
        #[arg(long = "consequences")]
        consequences: Option<String>,
        #[arg(long = "superseded-by", conflicts_with = "status")]
        superseded_by: Option<String>,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    Archive {
        id: String,
    },
}

pub fn run(ctx: &Context, cmd: DecisionCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        DecisionCmd::Create {
            title,
            decision_type,
            decision,
            context,
            consequences,
            label,
            reference,
        } => {
            let id = create(
                ctx,
                &title,
                decision_type.as_deref(),
                decision.as_deref(),
                context.as_deref(),
                consequences.as_deref(),
                &label,
                &reference,
            )?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        DecisionCmd::List { decision_type } => list(ctx, decision_type.as_deref()),
        DecisionCmd::View { id } => view(ctx, &id),
        DecisionCmd::Edit {
            id,
            title,
            decision,
            status,
            context,
            consequences,
            superseded_by,
            label,
            reference,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            decision.as_deref(),
            status.as_deref(),
            context.as_deref(),
            consequences.as_deref(),
            superseded_by.as_deref(),
            &label,
            &reference,
        ),
        DecisionCmd::Archive { id } => archive(ctx, &id),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let raw = ctx.project_dir.join(&cfg.decision_directory);
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn archived_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = active_dir(ctx, cfg)?;
    let raw = base.join("archived");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn ensure_dir(dir: &std::path::Path, project_dir: &std::path::Path) -> Result<(), AppError> {
    let safe = Context::ensure_within(project_dir, dir)?;
    fs::create_dir_all(&safe).map_err(|e| AppError::Runtime(e.to_string()))
}

fn type_entry(cfg: &Config, key: &str) -> Result<crate::config::TypeEntry, AppError> {
    let key = sanitize_text(key)?;
    let entry = cfg
        .decision_types
        .get(&key)
        .ok_or_else(|| AppError::Validation(format!("unknown decision type: {}", key)))?;
    if !entry.enabled {
        return Err(AppError::Validation(format!(
            "decision type disabled: {}",
            key
        )));
    }
    if entry.prefix.is_empty() {
        return Err(AppError::Validation(format!(
            "decision type prefix empty: {}",
            key
        )));
    }
    Ok(entry.clone())
}

pub fn type_key_for_id(cfg: &Config, id: &str) -> Option<String> {
    for (key, entry) in &cfg.decision_types {
        if !entry.enabled {
            continue;
        }
        if entry.prefix.is_empty() {
            continue;
        }
        if id.starts_with(&format!("{}-", entry.prefix)) {
            return Some(key.clone());
        }
    }
    None
}

fn scan_max(active: &std::path::Path, archived: &std::path::Path, prefix: &str) -> u32 {
    let mut max = 0u32;
    for dir in [active, archived] {
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
pub(crate) struct DecisionData {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) decision: Option<String>,
    pub(crate) context: Option<String>,
    pub(crate) consequences: Option<String>,
}

fn frontmatter_list_dec(content: &str, key: &str) -> Vec<String> {
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

fn parse_heading(content: &str) -> (String, String) {
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

pub(crate) fn parse_decision(content: &str) -> DecisionData {
    let (id, title) = parse_heading(content);
    let status = section_text(content, "Status")
        .unwrap_or_default()
        .trim()
        .to_string();
    let decision = section_text(content, "Decision");
    let context = section_text(content, "Context");
    let consequences = section_text(content, "Consequences");
    DecisionData {
        id,
        title,
        status,
        decision,
        context,
        consequences,
    }
}

fn render_decision(dd: &DecisionData) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", dd.id, dd.title));
    let status = if dd.status.trim().is_empty() {
        "proposed".to_string()
    } else {
        dd.status.clone()
    };
    parts.push(format!("## Status\n{}", status));
    if let Some(d) = &dd.decision
        && !d.trim().is_empty()
    {
        parts.push(format!("## Decision\n{}", d));
    }
    if let Some(c) = &dd.context
        && !c.trim().is_empty()
    {
        parts.push(format!("## Context\n{}", c));
    }
    if let Some(c) = &dd.consequences
        && !c.trim().is_empty()
    {
        parts.push(format!("## Consequences\n{}", c));
    }
    parts.join("\n\n") + "\n"
}

pub const ALLOWED_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
];

pub fn valid_transition(current: &str, next: &str) -> bool {
    if current == next {
        return true;
    }
    match current {
        "proposed" => matches!(next, "accepted" | "rejected"),
        "accepted" => matches!(next, "deprecated" | "superseded"),
        _ => false,
    }
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    type_raw: Option<&str>,
    decision_raw: Option<&str>,
    context_raw: Option<&str>,
    consequences_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let type_key = match type_raw {
        Some(s) => sanitize_text(s)?,
        None => "adr".to_string(),
    };
    let entry = type_entry(&cfg, &type_key)?;
    let decision = match decision_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let context = match context_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let consequences = match consequences_raw {
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

    let active = active_dir(ctx, &cfg)?;
    let archived = archived_dir(ctx, &cfg)?;
    ensure_dir(&active, &ctx.project_dir)?;

    let max = scan_max(&active, &archived, &entry.prefix);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        entry.prefix,
        next,
        width = cfg.id_padding as usize
    );

    let dd = DecisionData {
        id: id.clone(),
        title,
        status: "proposed".to_string(),
        decision,
        context,
        consequences,
    };
    let now = crate::record::frontmatter::utc_now();
    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", id));
    fm.push_str(&format!(
        "title: {}\n",
        crate::record::frontmatter::format_scalar_pub(&dd.title)
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
    // superseded_by optional; default null
    fm.push_str("superseded_by: null\n");
    // type? decision has no type front matter (type is adr/bdr derived). general only.

    let body = render_decision(&dd);
    let content = crate::record::frontmatter::wrap(&fm, &body);

    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
    Ok(id)
}

pub fn list(
    ctx: &Context,
    type_filter_raw: Option<&str>,
) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let type_filter = match type_filter_raw {
        Some(s) => {
            let key = sanitize_text(s)?;
            let entry = type_entry(&cfg, &key)?;
            Some((key, entry.prefix))
        }
        None => None,
    };

    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

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
            let included = if let Some((_, prefix)) = &type_filter {
                id.starts_with(&format!("{}-", prefix))
            } else {
                type_key_for_id(&cfg, &id).is_some()
            };
            if !included {
                continue;
            }
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
    let active = active_dir(ctx, &cfg)?;
    let archived = archived_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let active_path = active.join(format!("{}.md", id));
    let archived_path = archived.join(format!("{}.md", id));
    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let archived_safe = if archived.exists() {
        Context::ensure_within(&ctx.project_dir, &archived_path).ok()
    } else {
        None
    };

    let path = if active_safe.exists() {
        active_safe
    } else if archived_safe.as_ref().map(|p| p.exists()).unwrap_or(false) {
        archived_safe.unwrap()
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
    title_raw: Option<&str>,
    decision_raw: Option<&str>,
    status_raw: Option<&str>,
    context_raw: Option<&str>,
    consequences_raw: Option<&str>,
    superseded_by_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let archived = archived_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let active_path = active.join(format!("{}.md", id));
    let archived_path = archived.join(format!("{}.md", id));
    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let archived_safe = if archived.exists() {
        Context::ensure_within(&ctx.project_dir, &archived_path).ok()
    } else {
        None
    };

    let path = if active_safe.exists() {
        active_safe
    } else if archived_safe.as_ref().map(|p| p.exists()).unwrap_or(false) {
        archived_safe.unwrap()
    } else {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut dd = parse_decision(&content);
    if dd.id.is_empty() {
        dd.id = id.clone();
    }
    if dd.title.is_empty() {
        dd.title = id.clone();
    }
    if dd.status.is_empty() {
        dd.status = "proposed".to_string();
    }

    if let Some(t) = title_raw {
        dd.title = sanitize_text(t)?;
    }
    if let Some(d) = decision_raw {
        dd.decision = Some(sanitize_text(d)?);
    }
    if let Some(c) = context_raw {
        dd.context = Some(sanitize_text(c)?);
    }
    if let Some(c) = consequences_raw {
        dd.consequences = Some(sanitize_text(c)?);
    }
    if let Some(s) = status_raw {
        let st = sanitize_text(s)?;
        if !ALLOWED_STATUSES.contains(&st.as_str()) {
            return Err(AppError::Validation(format!("unknown status: {}", st)));
        }
        if !valid_transition(&dd.status, &st) {
            return Err(AppError::Validation(format!(
                "invalid status transition: {} -> {}",
                dd.status, st
            )));
        }
        dd.status = st;
    }
    if let Some(sup) = superseded_by_raw {
        let sup_id = sanitize_text(sup)?;
        let sup_active_path = active.join(format!("{}.md", sup_id));
        let sup_archived_path = archived.join(format!("{}.md", sup_id));
        let sup_active_safe = Context::ensure_within(&ctx.project_dir, &sup_active_path)?;
        let sup_archived_safe = if archived.exists() {
            Context::ensure_within(&ctx.project_dir, &sup_archived_path).ok()
        } else {
            None
        };
        if !sup_active_safe.exists()
            && !sup_archived_safe
                .as_ref()
                .map(|p| p.exists())
                .unwrap_or(false)
        {
            return Err(AppError::NotFound(format!("not found: {}", sup_id)));
        }
        if sup_id == id {
            return Err(AppError::Validation(format!(
                "decision cannot supersede itself: {}",
                id
            )));
        }
        let current_type = type_key_for_id(&cfg, &id)
            .ok_or_else(|| AppError::Validation(format!("unknown decision type for: {}", id)))?;
        let sup_type = type_key_for_id(&cfg, &sup_id).ok_or_else(|| {
            AppError::Validation(format!("unknown decision type for: {}", sup_id))
        })?;
        if current_type != sup_type {
            return Err(AppError::Validation(format!(
                "decision type mismatch: {} vs {}",
                current_type, sup_type
            )));
        }
        dd.status = format!("Superseded by {}", sup_id);
    }

    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            dd.title.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            dd.title.clone(),
        ));
    }
    if decision_raw.is_some() {
        if let Some(d) = &dd.decision {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Decision".into(),
                d.clone(),
            ));
        }
    }
    if context_raw.is_some() {
        if let Some(c) = &dd.context {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Context".into(),
                c.clone(),
            ));
        }
    }
    if consequences_raw.is_some() {
        if let Some(c) = &dd.consequences {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Consequences".into(),
                c.clone(),
            ));
        }
    }
    if status_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Status".into(),
            dd.status.clone(),
        ));
    }
    if superseded_by_raw.is_some() {
        // superseded_by id stored in body? set front matter key superseded_by
        let sup = superseded_by_raw
            .as_ref()
            .map(|s| sanitize_text(s))
            .transpose()?;
        if let Some(s) = sup {
            ops.push(crate::record::patch::PatchOp::SetFrontMatter(
                "superseded_by".into(),
                s,
            ));
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Status".into(),
                dd.status.clone(),
            ));
        }
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_dec(&content, "labels");
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
        let mut fm_refs = frontmatter_list_dec(&content, "references");
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

pub fn archive(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let archived = archived_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);
    ensure_dir(&archived, &ctx.project_dir)?;

    let active_path = active.join(format!("{}.md", id));
    let archived_path = archived.join(format!("{}.md", id));
    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    let archived_safe = Context::ensure_within(&ctx.project_dir, &archived_path)?;

    if !active_safe.exists() {
        if archived_safe.exists() {
            return Err(AppError::Validation(format!("already archived: {}", id)));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }
    if archived_safe.exists() {
        return Err(AppError::Validation(format!("already archived: {}", id)));
    }

    fs::rename(&active_safe, &archived_safe).map_err(|e| AppError::Runtime(e.to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_render_roundtrip() {
        let dd = DecisionData {
            id: "ADR-001".to_string(),
            title: "T".to_string(),
            status: "proposed".to_string(),
            decision: Some("d".to_string()),
            context: None,
            consequences: Some("c".to_string()),
        };
        let content = render_decision(&dd);
        let parsed = parse_decision(&content);
        assert_eq!(parsed.id, "ADR-001");
        assert_eq!(parsed.title, "T");
        assert_eq!(parsed.status, "proposed");
        assert_eq!(parsed.decision.as_deref(), Some("d"));
        assert!(parsed.context.is_none());
        assert_eq!(parsed.consequences.as_deref(), Some("c"));
    }

    #[test]
    fn transitions() {
        assert!(valid_transition("proposed", "proposed"));
        assert!(valid_transition("proposed", "accepted"));
        assert!(valid_transition("proposed", "rejected"));
        assert!(!valid_transition("proposed", "deprecated"));
        assert!(valid_transition("accepted", "deprecated"));
        assert!(valid_transition("accepted", "superseded"));
        assert!(!valid_transition("accepted", "proposed"));
        assert!(!valid_transition("rejected", "accepted"));
    }
}
