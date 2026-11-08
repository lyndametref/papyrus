#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::manual_flatten
)]
use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::sanitize;
use clap::Subcommand;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

#[derive(Subcommand, Debug)]
pub enum TaskCmd {
    Create {
        #[arg(value_name = "title")]
        title: String,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(long = "ac")]
        ac: Vec<String>,
        #[arg(long = "depends-on")]
        depends_on: Vec<String>,
        #[arg(long = "milestone")]
        milestone: Option<String>,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
        #[arg(long = "documentation")]
        documentation: Vec<String>,
        #[arg(long = "type")]
        r#type: Option<String>,
        #[arg(long = "ordinal")]
        ordinal: Option<String>,
        #[arg(long = "assignee", num_args = 1..)]
        assignee: Vec<String>,
    },
    List {
        #[arg(short = 's', long = "status")]
        status: Option<String>,
        #[arg(long = "milestone")]
        milestone: Option<String>,
        #[arg(long = "release")]
        release: Option<String>,
        #[arg(long = "label")]
        label: Option<String>,
        #[arg(long = "assignee")]
        assignee: Option<String>,
        #[arg(long = "search")]
        search: Option<String>,
        #[arg(long = "limit", value_parser = crate::parse_limit)]
        limit: Option<usize>,
        #[arg(long = "sort")]
        sort: Option<String>,
        #[arg(long = "watch")]
        watch: bool,
    },
    View {
        id: String,
    },
    Edit {
        id: String,
        #[arg(long = "title")]
        title: Option<String>,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(long = "ac")]
        ac: Vec<String>,
        #[arg(long = "comment")]
        comment: Vec<String>,
        #[arg(long = "comment-author")]
        comment_author: Option<String>,
        #[arg(long = "status")]
        status: Option<String>,
        #[arg(long = "depends-on")]
        depends_on: Vec<String>,
        #[arg(long = "milestone")]
        milestone: Option<String>,
        #[arg(long = "no-milestone", num_args = 0..=1)]
        no_milestone: Option<Option<String>>,
        #[arg(long = "label")]
        label: Vec<String>,
        #[arg(long = "reference")]
        reference: Vec<String>,
        #[arg(long = "documentation")]
        documentation: Vec<String>,
        #[arg(long = "type")]
        r#type: Option<String>,
        #[arg(long = "ordinal")]
        ordinal: Option<String>,
        #[arg(long = "assignee", num_args = 1..)]
        assignee: Vec<String>,
        #[arg(long = "editor")]
        editor: bool,
    },
    Archive {
        id: String,
    },
    Restore {
        id: String,
    },
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
}

pub fn run(ctx: &Context, cmd: TaskCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        TaskCmd::Create {
            title,
            description,
            ac,
            depends_on,
            milestone,
            label,
            reference,
            documentation,
            r#type,
            ordinal,
            assignee,
        } => {
            let id = create(
                ctx,
                &title,
                description.as_deref(),
                &ac,
                &depends_on,
                milestone.as_deref(),
                &label,
                &reference,
                &documentation,
                r#type.as_deref(),
                ordinal.as_deref(),
                &assignee,
            )?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        TaskCmd::List {
            status,
            milestone,
            release,
            label,
            assignee,
            search,
            limit,
            sort,
            watch,
        } => list(
            ctx,
            status.as_deref(),
            milestone.as_deref(),
            release.as_deref(),
            label.as_deref(),
            assignee.as_deref(),
            search.as_deref(),
            limit,
            sort.as_deref(),
            watch,
        ),
        TaskCmd::View { id } => view(ctx, &id),
        TaskCmd::Edit {
            id,
            title,
            description,
            ac,
            comment,
            comment_author,
            status,
            depends_on,
            milestone,
            no_milestone,
            label,
            reference,
            documentation,
            r#type,
            ordinal,
            assignee,
            editor,
        } => edit(
            ctx,
            &id,
            title.as_deref(),
            description.as_deref(),
            &ac,
            &comment,
            comment_author.as_deref(),
            status.as_deref(),
            &depends_on,
            milestone.as_deref(),
            no_milestone,
            &label,
            &reference,
            &documentation,
            r#type.as_deref(),
            ordinal.as_deref(),
            &assignee,
            editor,
        ),
        TaskCmd::Archive { id } => archive(ctx, &id),
        TaskCmd::Restore { id } => restore(ctx, &id),
        TaskCmd::Delete { id, force } => delete(ctx, &id, force),
    }
}

fn sanitize_text(input: &str) -> Result<String, AppError> {
    sanitize::sanitize_name(input)
        .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))
}

fn scan_max_ordinal(active: &std::path::Path, archived: &std::path::Path) -> i64 {
    let mut max: Option<i64> = None;
    for dir in [active, archived] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Some(fm) = crate::record::frontmatter::parse(&content).ok().flatten()
                    && let Some(v) =
                        crate::record::frontmatter::current_value_string(&fm.raw, "ordinal")
                    && let Ok(n) = v.parse::<i64>()
                {
                    max = Some(max.map_or(n, |m: i64| m.max(n)));
                }
            }
        }
    }
    match max {
        Some(m) => m + 1,
        None => 0,
    }
}

fn load_config(ctx: &Context) -> Result<Config, AppError> {
    load_effective_config(&ctx.config_path, &ctx.project_dir)
}

fn backlog_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let raw = ctx.project_dir.join(&cfg.backlog_directory);
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("tasks");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn archived_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = active_dir(ctx, cfg)?;
    let raw = base.join("archived");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn backlog_archive_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("archive/tasks");
    fs::create_dir_all(&raw).map_err(|e| AppError::Runtime(e.to_string()))?;
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn resolve_task_path(ctx: &Context, cfg: &Config, id: &str) -> Result<PathBuf, AppError> {
    let storage_id =
        crate::record::identity::task_to_storage_id(id).unwrap_or_else(|_| id.to_lowercase());
    for dir in [
        active_dir(ctx, cfg),
        archived_dir(ctx, cfg),
        backlog_archive_dir(ctx, cfg),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(path) = find_task_file(&dir, &storage_id) {
            return Ok(path);
        }
    }
    Err(AppError::NotFound(format!("not found: {}", id)))
}

fn resolve_task_path_migrated(ctx: &Context, cfg: &Config, id: &str) -> Result<PathBuf, AppError> {
    let path = resolve_task_path(ctx, cfg, id)?;
    crate::record::migrate::ensure_front_matter_canonical(ctx, &path)
}

fn find_task_file(dir: &std::path::Path, storage_id: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    let rd = fs::read_dir(dir).ok()?;
    for e in rd.flatten() {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str())?;
        let base = stem.split(" - ").next().unwrap_or(stem).trim();
        if base.eq_ignore_ascii_case(storage_id) {
            return Some(path);
        }
        if let Ok(sid) = crate::record::identity::task_to_storage_id(base)
            && sid.eq_ignore_ascii_case(storage_id)
        {
            return Some(path);
        }
    }
    None
}

fn milestones_active_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("milestones");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn milestones_archived_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = milestones_active_dir(ctx, cfg)?;
    let raw = base.join("archived");
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn milestones_backlog_archive_dir(ctx: &Context, cfg: &Config) -> Result<PathBuf, AppError> {
    let base = backlog_dir(ctx, cfg)?;
    let raw = base.join("archive/milestones");
    fs::create_dir_all(&raw).map_err(|e| AppError::Runtime(e.to_string()))?;
    Context::ensure_within(&ctx.project_dir, &raw)
}

fn ensure_dir(dir: &std::path::Path, project_dir: &std::path::Path) -> Result<(), AppError> {
    let safe = Context::ensure_within(project_dir, dir)?;
    fs::create_dir_all(&safe).map_err(|e| AppError::Runtime(e.to_string()))
}

fn task_number_from_stem(stem: &str) -> Option<u32> {
    let base = stem.split(" - ").next()?.trim();
    let storage_id =
        crate::record::identity::task_to_storage_id(base).unwrap_or_else(|_| base.to_lowercase());
    let num = storage_id.strip_prefix("task-")?;
    num.parse::<u32>().ok()
}

fn scan_max(
    active: &std::path::Path,
    archived: &std::path::Path,
    archive: &std::path::Path,
) -> u32 {
    let mut max = 0u32;
    for dir in [active, archived, archive] {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(n) = task_number_from_stem(stem)
                    && n > max
                {
                    max = n;
                }
            }
        }
    }
    max
}

fn milestone_exists(ctx: &Context, cfg: &Config, id: &str) -> bool {
    let storage_id =
        crate::record::identity::milestone_to_storage_id(id).unwrap_or_else(|_| id.to_lowercase());
    for dir in [
        milestones_active_dir(ctx, cfg),
        milestones_archived_dir(ctx, cfg),
        milestones_backlog_archive_dir(ctx, cfg),
    ] {
        if let Ok(d) = dir
            && crate::milestone::find_milestone_file(&d, &storage_id).is_some()
        {
            return true;
        }
    }
    false
}

fn task_exists(ctx: &Context, cfg: &Config, id: &str) -> bool {
    let storage_id =
        crate::record::identity::task_to_storage_id(id).unwrap_or_else(|_| id.to_lowercase());
    for dir in [
        active_dir(ctx, cfg),
        archived_dir(ctx, cfg),
        backlog_archive_dir(ctx, cfg),
    ] {
        if let Ok(d) = dir
            && find_task_file(&d, &storage_id).is_some()
        {
            return true;
        }
    }
    false
}

fn milestone_release(ctx: &Context, cfg: &Config, milestone_id: &str) -> Option<String> {
    for dir in [
        milestones_active_dir(ctx, cfg),
        milestones_archived_dir(ctx, cfg),
    ] {
        if let Ok(d) = dir {
            let path = d.join(format!("{}.md", milestone_id));
            if let Ok(safe) = Context::ensure_within(&ctx.project_dir, &path)
                && safe.exists()
            {
                let content = fs::read_to_string(&safe).unwrap_or_default();
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0usize;
                while i < lines.len() {
                    if lines[i].trim() == "## Release" {
                        i += 1;
                        let mut buf = Vec::new();
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
                        let text = buf.join("\n").trim().to_string();
                        if !text.is_empty() {
                            return Some(text);
                        }
                        return None;
                    }
                    i += 1;
                }
                return None;
            }
        }
    }
    None
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TaskData {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) status: String,
    pub(crate) milestone: Option<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) labels: Vec<String>,
    pub(crate) deps: Vec<String>,
    pub(crate) acs: Vec<String>,
    pub(crate) comments: Vec<(String, String)>,
}

fn parse_heading(content: &str) -> (String, String) {
    let body = if let Some((_, _, bs)) = crate::record::frontmatter::find_block(content) {
        &content[bs..]
    } else {
        content
    };
    let first = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let rest = first.trim_start_matches('#').trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("").trim().to_string();
    let title = parts.next().unwrap_or("").trim().to_string();
    // Prefer the canonical title from front matter when present.
    let title = if let Ok(Some(fm)) = crate::record::frontmatter::parse(content) {
        crate::record::frontmatter::current_value_string(&fm.raw, "title")
            .filter(|t| !t.is_empty())
            .unwrap_or(title)
    } else {
        title
    };
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

fn section_lines(content: &str, name: &str) -> Vec<String> {
    match section_text(content, name) {
        Some(t) => t.lines().map(|s| s.to_string()).collect(),
        None => Vec::new(),
    }
}

pub(crate) fn parse_task(content: &str) -> TaskData {
    let (id, title) = parse_heading(content);
    let desc = section_text(content, "Description");
    let status = section_text(content, "Status")
        .unwrap_or_default()
        .trim()
        .to_string();
    let milestone = section_text(content, "Milestone").map(|s| s.trim().to_string());
    let assignee = section_text(content, "Assignee").map(|s| s.trim().to_string());

    let mut labels = Vec::new();
    for l in section_lines(content, "Labels") {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let item = t.strip_prefix("- ").unwrap_or(t).trim().to_string();
        if !item.is_empty() {
            labels.push(item);
        }
    }
    let mut deps = Vec::new();
    for l in section_lines(content, "Dependencies") {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let item = t.strip_prefix("- ").unwrap_or(t).trim().to_string();
        if !item.is_empty() {
            deps.push(item);
        }
    }
    let mut acs = Vec::new();
    for l in section_lines(content, "Acceptance Criteria") {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let mut item = t.to_string();
        if let Some(rest) = item.strip_prefix("- ") {
            item = rest.trim().to_string();
        }
        for pref in ["[ ] ", "[x] ", "[X] "] {
            if let Some(rest) = item.strip_prefix(pref) {
                item = rest.trim().to_string();
                break;
            }
        }
        if !item.is_empty() {
            acs.push(item);
        }
    }
    let mut comments = Vec::new();
    for l in section_lines(content, "Comments") {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        let mut item = t.to_string();
        if let Some(rest) = item.strip_prefix("- ") {
            item = rest.trim().to_string();
        }
        if let Some(inner) = item.strip_prefix("**")
            && let Some(end) = inner.find("**")
        {
            let author = inner[..end].trim().to_string();
            let after = inner[end + 2..].trim();
            let text = after.trim_start_matches(':').trim().to_string();
            comments.push((author, text));
            continue;
        }
        comments.push(("unknown".to_string(), item));
    }

    TaskData {
        id,
        title,
        description: desc,
        status,
        milestone,
        assignee,
        labels,
        deps,
        acs,
        comments,
    }
}

pub(crate) fn render_task(td: &TaskData) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("# {} {}", td.id, td.title));
    if let Some(d) = &td.description
        && !d.trim().is_empty()
    {
        parts.push(format!("## Description\n{}", d));
    }
    parts.push(format!("## Status\n{}", td.status));
    if let Some(m) = &td.milestone
        && !m.trim().is_empty()
    {
        parts.push(format!("## Milestone\n{}", m));
    }
    if let Some(a) = &td.assignee
        && !a.trim().is_empty()
    {
        parts.push(format!("## Assignee\n{}", a));
    }
    if !td.labels.is_empty() {
        let body = td
            .labels
            .iter()
            .map(|l| format!("- {}", l))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Labels\n{}", body));
    }
    if !td.deps.is_empty() {
        let body = td
            .deps
            .iter()
            .map(|d| format!("- {}", d))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Dependencies\n{}", body));
    }
    if !td.acs.is_empty() {
        let body = td
            .acs
            .iter()
            .map(|a| format!("- [ ] {}", a))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Acceptance Criteria\n{}", body));
    }
    if !td.comments.is_empty() {
        let body = td
            .comments
            .iter()
            .map(|(a, t)| format!("- **{}**: {}", a, t))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Comments\n{}", body));
    }
    parts.join("\n\n") + "\n"
}

fn frontmatter_list(content: &str, key: &str) -> Vec<String> {
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

fn unique_push(vec: &mut Vec<String>, val: String) {
    if !vec.contains(&val) {
        vec.push(val);
    }
}

pub fn create(
    ctx: &Context,
    title_raw: &str,
    desc_raw: Option<&str>,
    ac_raw: &[String],
    deps_raw: &[String],
    milestone_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
    documentation_raw: &[String],
    type_raw: Option<&str>,
    ordinal_raw: Option<&str>,
    assignee_raw: &[String],
) -> Result<String, AppError> {
    let cfg = load_config(ctx)?;
    let title = sanitize_text(title_raw)?;
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let mut acs: Vec<String> = Vec::new();
    for a in ac_raw {
        unique_push(&mut acs, sanitize_text(a)?);
    }
    let mut deps: Vec<String> = Vec::new();
    for d in deps_raw {
        let did = sanitize_text(d)?;
        if !task_exists(ctx, &cfg, &did) {
            return Err(AppError::NotFound(format!("task not found: {}", did)));
        }
        unique_push(&mut deps, did);
    }
    let milestone = match milestone_raw {
        Some(s) => {
            let mid = sanitize_text(s)?;
            if !milestone_exists(ctx, &cfg, &mid) {
                return Err(AppError::NotFound(format!("milestone not found: {}", mid)));
            }
            Some(mid)
        }
        None => None,
    };
    let mut labels: Vec<String> = Vec::new();
    for l in labels_raw {
        unique_push(&mut labels, sanitize_text(l)?);
    }
    let mut references: Vec<String> = Vec::new();
    for r in references_raw {
        unique_push(
            &mut references,
            sanitize::sanitize_path_string(r)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?,
        );
    }
    let mut documentation: Vec<String> = Vec::new();
    for d in documentation_raw {
        unique_push(
            &mut documentation,
            sanitize::sanitize_path_string(d)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?,
        );
    }
    let type_val = match type_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let ordinal_val: Option<i64> = match ordinal_raw {
        Some(s) => {
            let n: i64 = s
                .trim()
                .parse()
                .map_err(|_| AppError::Validation("invalid ordinal".to_string()))?;
            if n < 0 {
                return Err(AppError::Validation("invalid ordinal".to_string()));
            }
            Some(n)
        }
        None => None,
    };

    let mut assignee_list: Vec<String> = Vec::new();
    for a in assignee_raw {
        unique_push(&mut assignee_list, sanitize_text(a)?);
    }
    let assignee = if assignee_list.is_empty() {
        None
    } else {
        Some(assignee_list)
    };

    let active = active_dir(ctx, &cfg)?;
    let archived = archived_dir(ctx, &cfg)?;
    let archive = backlog_archive_dir(ctx, &cfg)?;
    ensure_dir(&active, &ctx.project_dir)?;
    let _ = ensure_dir(&archived, &ctx.project_dir);

    let max = scan_max(&active, &archived, &archive);
    let next = max + 1;
    let id = format!(
        "{}-{:0width$}",
        cfg.task_prefix,
        next,
        width = cfg.id_padding as usize
    );

    let td = TaskData {
        id: id.clone(),
        title: title.clone(),
        description: desc.clone(),
        status: cfg.default_status.clone(),
        milestone: milestone.clone(),
        assignee: assignee.clone().and_then(|v| v.into_iter().next()),
        labels: labels.clone(),
        deps: deps.clone(),
        acs: acs.clone(),
        comments: Vec::new(),
    };

    // ordinal default
    let ordinal_final = match ordinal_val {
        Some(n) => n,
        None => scan_max_ordinal(&active, &archived),
    };

    let now = crate::record::frontmatter::utc_now();
    let assignee_yaml = match &assignee {
        Some(v) => v.clone(),
        None => Vec::new(),
    };
    let mut fm = String::new();
    fm.push_str(&format!("id: {}\n", id));
    fm.push_str(&format!(
        "title: {}\n",
        crate::record::frontmatter::format_scalar_pub(&title)
    ));
    fm.push_str(&format!("created_date: '{}'\n", now));
    fm.push_str(&format!("updated_date: '{}'\n", now));
    // labels
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
    // references
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
    fm.push_str(&format!(
        "status: {}\n",
        crate::record::frontmatter::format_scalar_pub(&cfg.default_status)
    ));
    // assignee list
    if assignee_yaml.is_empty() {
        fm.push_str("assignee: []\n");
    } else {
        fm.push_str("assignee:\n");
        for a in &assignee_yaml {
            fm.push_str(&format!(
                "  - {}\n",
                crate::record::frontmatter::format_scalar_pub(a)
            ));
        }
    }
    if let Some(m) = &milestone {
        fm.push_str(&format!(
            "milestone: {}\n",
            crate::record::frontmatter::format_scalar_pub(m)
        ));
    } else {
        fm.push_str("milestone: null\n");
    }
    if deps.is_empty() {
        fm.push_str("dependencies: []\n");
    } else {
        fm.push_str("dependencies:\n");
        for d in &deps {
            fm.push_str(&format!(
                "  - {}\n",
                crate::record::frontmatter::format_scalar_pub(d)
            ));
        }
    }
    fm.push_str(&format!("ordinal: {}\n", ordinal_final));
    if documentation.is_empty() {
        fm.push_str("documentation: []\n");
    } else {
        fm.push_str("documentation:\n");
        for d in &documentation {
            fm.push_str(&format!(
                "  - {}\n",
                crate::record::frontmatter::format_scalar_pub(d)
            ));
        }
    }
    if let Some(t) = &type_val {
        fm.push_str(&format!(
            "type: {}\n",
            crate::record::frontmatter::format_scalar_pub(t)
        ));
    } else {
        fm.push_str("type: null\n");
    }

    let body = render_task(&td);
    let content = crate::record::frontmatter::wrap(&fm, &body);

    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
    Ok(id)
}

pub fn view(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_task_path_migrated(ctx, &cfg, &id)?;

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
    ac_raw: &[String],
    comments_raw: &[String],
    comment_author_raw: Option<&str>,
    status_raw: Option<&str>,
    deps_raw: &[String],
    milestone_raw: Option<&str>,
    no_milestone: Option<Option<String>>,
    labels_raw: &[String],
    references_raw: &[String],
    documentation_raw: &[String],
    type_raw: Option<&str>,
    ordinal_raw: Option<&str>,
    assignee_raw: &[String],
    editor: bool,
) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    if milestone_raw.is_some() && no_milestone.is_some() {
        return Err(AppError::Validation(
            "mutually exclusive flags: --milestone and --no-milestone".to_string(),
        ));
    }
    let cfg = load_config(ctx)?;
    let path = resolve_task_path_migrated(ctx, &cfg, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut td = parse_task(&content);
    if td.id.is_empty() {
        td.id = id.clone();
    }
    if td.title.is_empty() {
        td.title = id.clone();
    }
    if td.status.is_empty() {
        td.status = cfg.default_status.clone();
    }

    if let Some(t) = title_raw {
        td.title = sanitize_text(t)?;
    }
    if let Some(d) = desc_raw {
        td.description = Some(sanitize_text(d)?);
    }
    if let Some(s) = status_raw {
        let st = sanitize_text(s)?;
        if !cfg.statuses.contains(&st) {
            return Err(AppError::Validation(format!("unknown status: {}", st)));
        }
        td.status = st;
    }
    if let Some(m) = milestone_raw {
        let mid = sanitize_text(m)?;
        if !milestone_exists(ctx, &cfg, &mid) {
            return Err(AppError::NotFound(format!("milestone not found: {}", mid)));
        }
        td.milestone = Some(mid);
    }
    if let Some(no) = &no_milestone {
        match no {
            Some(rid_raw) => {
                let rid = sanitize_text(rid_raw)?;
                if td.milestone.as_deref() != Some(rid.as_str()) {
                    return Err(AppError::Validation(format!(
                        "task is not assigned to milestone: {}",
                        rid
                    )));
                }
                td.milestone = None;
            }
            None => {
                td.milestone = None;
            }
        }
    }
    for d in deps_raw {
        let did = sanitize_text(d)?;
        if did == id {
            return Err(AppError::Validation(
                "task cannot depend on itself".to_string(),
            ));
        }
        if !task_exists(ctx, &cfg, &did) {
            return Err(AppError::NotFound(format!("task not found: {}", did)));
        }
        unique_push(&mut td.deps, did);
    }
    for a in ac_raw {
        unique_push(&mut td.acs, sanitize_text(a)?);
    }
    for l in labels_raw {
        unique_push(&mut td.labels, sanitize_text(l)?);
    }
    // references/documentation/type/ordinal handled via front-matter patch below

    if !assignee_raw.is_empty() {
        let mut cleaned = Vec::new();
        for a in assignee_raw {
            cleaned.push(sanitize_text(a)?);
        }
        td.assignee = cleaned.first().cloned();
    }
    if !comments_raw.is_empty() {
        let author = match comment_author_raw {
            Some(a) => sanitize_text(a)?,
            None => "unknown".to_string(),
        };
        for c in comments_raw {
            td.comments.push((author.clone(), sanitize_text(c)?));
        }
    }

    // Build patch ops instead of re-rendering.
    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();

    if let Some(t) = title_raw {
        let tv = sanitize_text(t)?;
        td.title = tv.clone();
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            tv.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(tv));
    }
    if let Some(d) = desc_raw {
        let dv = sanitize::sanitize_path_string(d)
            .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
        td.description = Some(dv.clone());
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Description".into(),
            dv,
        ));
    }
    if let Some(s) = status_raw {
        let st = sanitize_text(s)?;
        if !cfg.statuses.contains(&st) {
            return Err(AppError::Validation(format!("unknown status: {}", st)));
        }
        td.status = st.clone();
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "status".into(),
            st.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Status".into(),
            st,
        ));
    }
    if let Some(m) = milestone_raw {
        let mid = sanitize_text(m)?;
        if !milestone_exists(ctx, &cfg, &mid) {
            return Err(AppError::NotFound(format!("milestone not found: {}", mid)));
        }
        td.milestone = Some(mid.clone());
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "milestone".into(),
            mid.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Milestone".into(),
            mid,
        ));
    }
    if let Some(no) = &no_milestone {
        match no {
            Some(_rid_raw) => {
                ops.push(crate::record::patch::PatchOp::ClearFrontMatter(
                    "milestone".into(),
                ));
                ops.push(crate::record::patch::PatchOp::ClearSection(
                    "## Milestone".into(),
                ));
            }
            None => {
                td.milestone = None;
                ops.push(crate::record::patch::PatchOp::ClearFrontMatter(
                    "milestone".into(),
                ));
                ops.push(crate::record::patch::PatchOp::ClearSection(
                    "## Milestone".into(),
                ));
            }
        }
    }
    if !deps_raw.is_empty() {
        let mut new_deps = td.deps.clone();
        for d in deps_raw {
            let did = sanitize_text(d)?;
            if did == id {
                return Err(AppError::Validation(
                    "task cannot depend on itself".to_string(),
                ));
            }
            if !task_exists(ctx, &cfg, &did) {
                return Err(AppError::NotFound(format!("task not found: {}", did)));
            }
            unique_push(&mut new_deps, did);
        }
        td.deps = new_deps.clone();
        let val = new_deps.join("\n");
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "dependencies".into(),
            val,
        ));
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Dependencies".into(),
            td.deps
                .iter()
                .map(|d| format!("- {}", d))
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    if !ac_raw.is_empty() {
        for a in ac_raw {
            unique_push(&mut td.acs, sanitize_text(a)?);
        }
        let body = td
            .acs
            .iter()
            .map(|a| format!("- [ ] {}", a))
            .collect::<Vec<_>>()
            .join("\n");
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Acceptance Criteria".into(),
            body,
        ));
    }
    if !labels_raw.is_empty() {
        for l in labels_raw {
            unique_push(&mut td.labels, sanitize_text(l)?);
        }
        // front-matter labels append unique
        let mut fm_labels = frontmatter_list(&content, "labels");
        for l in labels_raw {
            let lv = sanitize_text(l)?;
            if !fm_labels.contains(&lv) {
                fm_labels.push(lv);
            }
        }
        let val = fm_labels.join("\n");
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "labels".into(),
            val,
        ));
        let body = td
            .labels
            .iter()
            .map(|l| format!("- {}", l))
            .collect::<Vec<_>>()
            .join("\n");
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Labels".into(),
            body,
        ));
    }
    // references append
    if !references_raw.is_empty() {
        let mut fm_refs = frontmatter_list(&content, "references");
        for r in references_raw {
            let rv = sanitize::sanitize_path_string(r)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
            if !fm_refs.contains(&rv) {
                fm_refs.push(rv);
            }
        }
        let val = fm_refs.join("\n");
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "references".into(),
            val,
        ));
    }
    // documentation append
    if !documentation_raw.is_empty() {
        let mut fm_docs = frontmatter_list(&content, "documentation");
        for d in documentation_raw {
            let dv = sanitize::sanitize_path_string(d)
                .ok_or_else(|| AppError::Validation("input cannot be normalized".to_string()))?;
            if !fm_docs.contains(&dv) {
                fm_docs.push(dv);
            }
        }
        let val = fm_docs.join("\n");
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "documentation".into(),
            val,
        ));
    }
    // type
    if let Some(t) = type_raw {
        let tv = sanitize_text(t)?;
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "type".into(),
            tv,
        ));
    }
    // ordinal
    if let Some(o) = ordinal_raw {
        let n: i64 = o
            .trim()
            .parse()
            .map_err(|_| AppError::Validation("invalid ordinal".to_string()))?;
        if n < 0 {
            return Err(AppError::Validation("invalid ordinal".to_string()));
        }
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "ordinal".into(),
            n.to_string(),
        ));
    }
    // assignee replace
    if !assignee_raw.is_empty() {
        let mut cleaned = Vec::new();
        for a in assignee_raw {
            cleaned.push(sanitize_text(a)?);
        }
        td.assignee = cleaned.first().cloned();
        let val = cleaned.join("\n");
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "assignee".into(),
            val,
        ));
        if let Some(a) = &td.assignee {
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Assignee".into(),
                a.clone(),
            ));
        }
    }
    if !comments_raw.is_empty() {
        let author = match comment_author_raw {
            Some(a) => sanitize_text(a)?,
            None => "unknown".to_string(),
        };
        for c in comments_raw {
            td.comments.push((author.clone(), sanitize_text(c)?));
        }
        let body = td
            .comments
            .iter()
            .map(|(a, t)| format!("- **{}**: {}", a, t))
            .collect::<Vec<_>>()
            .join("\n");
        // append to existing comments? Current render all comments. SetSection replaces.
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Comments".into(),
            body,
        ));
    }

    // parse doc
    let doc = crate::record::document::parse(&content)?;
    let base_new = crate::record::patch::apply_patch(&doc, &ops)?;
    let new_content = if ops.is_empty() || base_new == content {
        content.clone()
    } else {
        // refresh updated_date
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
        if let Some(ed) = cfg.effective_editor() {
            let mut parts: Vec<&str> = ed.split_whitespace().collect();
            if parts.is_empty() {
                return Err(AppError::Runtime("editor not configured".to_string()));
            }
            let prog = parts.remove(0);
            let st = std::process::Command::new(prog)
                .args(&parts)
                .arg(&safe_write)
                .status()
                .map_err(|e| AppError::Runtime(e.to_string()))?;
            if !st.success() {
                return Err(AppError::Runtime(
                    "editor exited with non-zero status".to_string(),
                ));
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

static WATCH_STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn sigint_handler(_sig: i32) {
    WATCH_STOP.store(true, Ordering::SeqCst);
}

fn install_sigint() {
    #[link(name = "c")]
    unsafe extern "C" {
        fn signal(sig: i32, handler: Option<extern "C" fn(i32)>) -> usize;
    }
    unsafe {
        signal(2, Some(sigint_handler));
    }
}

fn fingerprint_dir(dir: &std::path::Path) -> String {
    let mut entries: Vec<String> = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let meta = fs::metadata(&path).ok();
            let mtime = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    t.duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_nanos())
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let len = meta.map(|m| m.len()).unwrap_or(0);
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            entries.push(format!("{}:{}:{}", name, mtime, len));
        }
    }
    entries.sort();
    entries.join("|")
}

pub fn list(
    ctx: &Context,
    status_filter: Option<&str>,
    milestone_filter: Option<&str>,
    release_filter: Option<&str>,
    label_filter: Option<&str>,
    assignee_filter: Option<&str>,
    search_filter: Option<&str>,
    limit: Option<usize>,
    sort_field: Option<&str>,
    watch: bool,
) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_config(ctx)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let status_f = match status_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let milestone_f = match milestone_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let release_f = match release_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let label_f = match label_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let assignee_f = match assignee_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let search_f = match search_filter {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };

    let sort = sort_field.unwrap_or("id");
    let allowed = ["id", "title", "status", "milestone", "assignee", "created"];
    if !allowed.contains(&sort) {
        return Err(AppError::Validation(format!(
            "unknown sort field: {}",
            sort
        )));
    }

    let render = |ctx: &Context,
                  cfg: &Config,
                  active: &std::path::Path|
     -> (String, Option<serde_json::Value>) {
        let mut items: Vec<(
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            SystemTime,
        )> = Vec::new();
        if let Ok(rd) = fs::read_dir(active) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let id = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let safe = match Context::ensure_within(&ctx.project_dir, &path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let content = fs::read_to_string(&safe).unwrap_or_default();
                let td = parse_task(&content);

                if let Some(sf) = &status_f
                    && td.status != *sf
                {
                    continue;
                }
                if let Some(mf) = &milestone_f
                    && td.milestone.as_deref() != Some(mf.as_str())
                {
                    continue;
                }
                if let Some(rf) = &release_f {
                    match &td.milestone {
                        Some(mid) => {
                            let mr = milestone_release(ctx, cfg, mid);
                            if mr.as_deref() != Some(rf.as_str()) {
                                continue;
                            }
                        }
                        None => continue,
                    }
                }
                if let Some(lf) = &label_f
                    && !td.labels.iter().any(|l| l == lf)
                {
                    continue;
                }
                if let Some(af) = &assignee_f
                    && td.assignee.as_deref() != Some(af.as_str())
                {
                    continue;
                }
                if let Some(sf) = &search_f {
                    let needle = sf.to_lowercase();
                    if !content.to_lowercase().contains(&needle) {
                        continue;
                    }
                }

                let created = fs::metadata(&safe)
                    .ok()
                    .and_then(|m| m.created().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let title = if td.title.is_empty() {
                    id.clone()
                } else {
                    td.title
                };
                items.push((id, title, td.status, td.milestone, td.assignee, created));
            }
        }

        items.sort_by(|a, b| match sort {
            "title" => a.1.cmp(&b.1),
            "status" => a.2.cmp(&b.2),
            "milestone" => a.3.cmp(&b.3),
            "assignee" => a.4.cmp(&b.4),
            "created" => a.5.cmp(&b.5).then(a.0.cmp(&b.0)),
            _ => a.0.cmp(&b.0),
        });

        if let Some(n) = limit
            && items.len() > n
        {
            items.truncate(n);
        }

        let text = items
            .iter()
            .map(|(id, title, ..)| format!("{} {}", id, title))
            .collect::<Vec<_>>()
            .join("\n");
        let json = if ctx.json {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .map(|(id, title, ..)| serde_json::json!({"id": id, "title": title}))
                .collect();
            Some(serde_json::Value::Array(arr))
        } else {
            None
        };
        (text, json)
    };

    if !watch {
        let (text, json) = render(ctx, &cfg, &active);
        return Ok(crate::CommandOutput { text, json });
    }

    install_sigint();
    WATCH_STOP.store(false, Ordering::SeqCst);

    let (text, json) = render(ctx, &cfg, &active);
    if ctx.json {
        if let Some(v) = &json {
            println!("{}", v);
        }
    } else if !text.is_empty() {
        println!("{}", text);
    }
    let _ = io::stdout().flush();

    let mut last_fp = fingerprint_dir(&active);
    while !WATCH_STOP.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(200));
        if WATCH_STOP.load(Ordering::SeqCst) {
            break;
        }
        let new_fp = fingerprint_dir(&active);
        if new_fp != last_fp {
            last_fp = new_fp;
            let (text, json) = render(ctx, &cfg, &active);
            if ctx.json {
                if let Some(v) = &json {
                    println!("{}", v);
                }
            } else if !text.is_empty() {
                println!("{}", text);
            }
            let _ = io::stdout().flush();
        }
    }

    Ok(crate::CommandOutput {
        text: String::new(),
        json: None,
    })
}

pub fn archive(ctx: &Context, id_raw: &str) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_task_path(ctx, &cfg, &id)?;
    let archived = archived_dir(ctx, &cfg)?;
    ensure_dir(&archived, &ctx.project_dir)?;

    let archived_path = archived.join(
        path.file_name()
            .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?,
    );
    let archived_safe = Context::ensure_within(&ctx.project_dir, &archived_path)?;
    if archived_safe == path {
        return Err(AppError::Validation(format!("already archived: {}", id)));
    }
    if archived_safe.exists() {
        return Err(AppError::Validation(format!("already archived: {}", id)));
    }

    fs::rename(&path, &archived_safe).map_err(|e| AppError::Runtime(e.to_string()))?;
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
    let path = resolve_task_path(ctx, &cfg, &id)?;
    let active = active_dir(ctx, &cfg)?;
    let _ = ensure_dir(&active, &ctx.project_dir);

    let active_path = active.join(
        path.file_name()
            .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?,
    );
    let active_safe = Context::ensure_within(&ctx.project_dir, &active_path)?;
    if active_safe == path {
        return Err(AppError::Validation(format!("already active: {}", id)));
    }
    if active_safe.exists() {
        return Err(AppError::Validation(format!("already active: {}", id)));
    }

    fs::rename(&path, &active_safe).map_err(|e| AppError::Runtime(e.to_string()))?;
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

pub fn delete(ctx: &Context, id_raw: &str, force: bool) -> Result<crate::CommandOutput, AppError> {
    let id = sanitize_text(id_raw)?;
    let cfg = load_config(ctx)?;
    let path = resolve_task_path_migrated(ctx, &cfg, &id)?;

    if !force {
        print!("Delete {}? [y/N] ", id);
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let ans = input.trim().to_lowercase();
        if ans != "y" && ans != "yes" {
            let json = if ctx.json {
                Some(serde_json::json!({"status": "aborted", "id": id}))
            } else {
                None
            };
            return Ok(crate::CommandOutput {
                text: String::new(),
                json,
            });
        }
    }

    fs::remove_file(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
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
    use crate::context::Context;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("task-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn ctx(dir: &std::path::Path) -> Context {
        Context {
            project_dir: dir.to_path_buf(),
            config_path: dir.join("papyrus.config.yml"),
            json: false,
            no_color: true,
        }
    }

    #[test]
    fn list_uses_front_matter_title() {
        let dir = tmpdir("list-title");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
        let content = "---\nid: TASK-5\ntitle: Front-Title\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: To Do\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 5\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody\n<!-- SECTION:DESCRIPTION:END -->\n";
        fs::write(dir.join("backlog/tasks/task-5 - File-Title.md"), content).unwrap();
        let out = list(&c, None, None, None, None, None, None, None, None, false).unwrap();
        assert!(
            out.text.contains("task-5 - File-Title Front-Title"),
            "got: {}",
            out.text
        );
    }

    #[test]
    fn create_does_not_reuse_storage_id_from_archive() {
        let dir = tmpdir("create-no-reuse");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("backlog/tasks")).unwrap();
        fs::create_dir_all(dir.join("backlog/tasks/archived")).unwrap();
        fs::create_dir_all(dir.join("backlog/archive/tasks")).unwrap();
        let archived = "---\nid: TASK-1\ntitle: Archived\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\nstatus: Done\nassignee: []\nmilestone: null\ndependencies: []\nordinal: 1\ndocumentation: []\ntype: null\n---\n\n## Description\n\n<!-- SECTION:DESCRIPTION:BEGIN -->\nBody\n<!-- SECTION:DESCRIPTION:END -->\n";
        fs::write(
            dir.join("backlog/archive/tasks/task-1 - Archived.md"),
            archived,
        )
        .unwrap();
        let id = create(
            &c,
            "New",
            None,
            &[],
            &[],
            None,
            &[],
            &[],
            &[],
            None,
            None,
            &[],
        )
        .unwrap_or_else(|e| panic!("create failed: {}", e));
        assert_eq!(id, "TASK-002", "got: {}", id);
    }
}
