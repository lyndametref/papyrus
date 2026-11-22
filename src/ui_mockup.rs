#![allow(clippy::too_many_arguments)]
use crate::CommandOutput;
use crate::context::Context;
use crate::document::{DocumentArgs, DocumentScope, DocumentVerb};
use crate::error::AppError;
use crate::sanitize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn handle(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
    verb: DocumentVerb,
    args: &DocumentArgs,
) -> Result<CommandOutput, AppError> {
    match verb {
        DocumentVerb::Create => {
            let title = crate::document::required_title(args)?;
            let id = create(
                ctx,
                scope,
                registry,
                &title,
                args.require("task")?,
                args.get_all("spec"),
                args.get_all("ref"),
            )?;
            Ok(crate::document::typed_created(ctx, id))
        }
        DocumentVerb::List => list(ctx, scope),
        DocumentVerb::View => view(ctx, scope, args.require("id")?),
        DocumentVerb::Edit => {
            let title = crate::document::optional_title(args)?;
            edit(
                ctx,
                scope,
                registry,
                args.require("id")?,
                title.as_deref(),
                args.get("task"),
                args.flag("spec").then(|| args.get_all("spec")),
                args.flag("ref").then(|| args.get_all("ref")),
                args.get("status"),
                args.get("superseded_by"),
                args.flag("editor"),
            )
        }
        DocumentVerb::Archive => archive(ctx, scope, registry, args.require("id")?),
        DocumentVerb::Delete => delete(
            ctx,
            scope,
            registry,
            args.require("id")?,
            args.flag("force"),
        ),
        DocumentVerb::Supersede => supersede(
            ctx,
            scope,
            registry,
            args.require("old")?,
            args.require("new")?,
        ),
        other => Err(crate::document::unsupported_verb(other)),
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

fn ensure_dirs(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

fn is_record_path(path: &Path, prefix: &str) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("html")
        && path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|stem| crate::record::identity::is_record_stem(stem, prefix))
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
    archived: &Path,
    id: &str,
) -> Result<PathBuf, AppError> {
    if let Some(path) = find_record(ctx, active, id)? {
        return Ok(path);
    }
    if let Some(path) = find_record(ctx, archived, id)? {
        return Ok(path);
    }
    Err(AppError::NotFound(format!("not found: {}", id)))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Mockup {
    pub(crate) id: String,
    pub(crate) title: String,
    /// Raw (already-encoded) display title from the `<title>` element, kept
    /// verbatim on an unrelated edit.
    pub(crate) raw_title: String,
    pub(crate) task: String,
    pub(crate) specs: Vec<String>,
    pub(crate) refs: Vec<String>,
    pub(crate) status: String,
    pub(crate) superseded_by: Option<String>,
}

/// The fixed comment block: the first contiguous run of single-line
/// `<!-- ... -->` comments, preferring the run that starts with `mockup-id`.
///
/// Metadata is read from this block only, so a value shown in a visible
/// paragraph can never set record metadata.
fn comment_block(content: &str) -> Option<&str> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut current: Option<(usize, usize)> = None;
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<!--") {
        let start = search + rel;
        let Some(close_rel) = content[start + 4..].find("-->") else {
            break;
        };
        let end = start + 4 + close_rel + 3;
        if let Some((run_start, run_end)) = current {
            if content[run_end..start].trim().is_empty() {
                current = Some((run_start, end));
            } else {
                runs.push((run_start, run_end));
                current = Some((start, end));
            }
        } else {
            current = Some((start, end));
        }
        search = end;
    }
    if let Some(run) = current {
        runs.push(run);
    }
    let chosen = runs
        .iter()
        .find(|(start, end)| content[*start..*end].contains("<!-- mockup-id:"))
        .or_else(|| runs.first())?;
    Some(&content[chosen.0..chosen.1])
}

fn parse_field_comment(block: &str, key: &str) -> Option<String> {
    let mut start = 0usize;
    while let Some(i) = block[start..].find("<!--") {
        let i = start + i;
        if let Some(j) = block[i..].find("-->") {
            let inner = block[i + 4..i + j].trim();
            if let Some(rest) = inner.strip_prefix(&format!("{}:", key)) {
                return Some(decode_entities(rest.trim()));
            }
            start = i + j + 3;
        } else {
            break;
        }
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

/// Decode the standard HTML character references named in the browser
/// specification section 3.3. `&amp;` is decoded last so that an encoded
/// `&amp;mdash;` stays a literal `&mdash;` instead of double-decoding.
pub(crate) fn decode_entities(s: &str) -> String {
    s.replace("&mdash;", "—")
        .replace("&#45;", "-")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Encode one value for HTML text and attribute context.
pub(crate) fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Encode one value for the fixed HTML comment block.
///
/// `--` is broken so a value can never terminate or invalidate the comment.
fn escape_comment(value: &str) -> String {
    escape_text(value).replace("--", "-&#45;")
}

/// Raw inner text of the first `<title>` element, or `None` when absent.
fn raw_title_text(content: &str) -> Option<String> {
    let mut search = 0usize;
    while let Some(rel) = content[search..].find("<title") {
        let open_start = search + rel;
        let after_open = open_start + "<title".len();
        let after = &content[after_open..];
        // The tag name must end at `>` or whitespace (attributes allowed).
        let next = after.chars().next();
        if next != Some('>') && !next.is_some_and(char::is_whitespace) {
            search = open_start + "<title".len();
            continue;
        }
        let gt = content[open_start..].find('>')?;
        let inner_start = open_start + gt + 1;
        let close_rel = content[inner_start..].find("</title>")?;
        return Some(content[inner_start..inner_start + close_rel].to_string());
    }
    None
}

/// Derive the display title from raw `<title>` text (sections 3.2-3.3).
fn title_from_raw(raw: &str, id: &str) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return id.to_string();
    }
    let dash = text.find('—');
    let entity = text.find("&mdash;");
    let separator = match (dash, entity) {
        (Some(d), Some(e)) => {
            if d <= e {
                Some((d, '—'.len_utf8()))
            } else {
                Some((e, "&mdash;".len()))
            }
        }
        (Some(d), None) => Some((d, '—'.len_utf8())),
        (None, Some(e)) => Some((e, "&mdash;".len())),
        (None, None) => None,
    };
    if let Some((at, width)) = separator {
        let rest = text[at + width..].trim();
        if rest.is_empty() {
            return id.to_string();
        }
        let decoded = decode_entities(rest);
        return if decoded.trim().is_empty() {
            id.to_string()
        } else {
            decoded
        };
    }
    let without_id = text.strip_prefix(id).and_then(|rest| {
        if rest.is_empty() || !rest.chars().next().is_some_and(char::is_whitespace) {
            None
        } else {
            Some(rest.trim())
        }
    });
    let chosen = without_id.unwrap_or(text);
    if chosen.is_empty() {
        return id.to_string();
    }
    let decoded = decode_entities(chosen);
    if decoded.trim().is_empty() {
        id.to_string()
    } else {
        decoded
    }
}

/// Strict browser title: from `<title>` only, falling back to the record ID.
pub(crate) fn extract_title(content: &str, id: &str) -> String {
    match raw_title_text(content) {
        Some(raw) if !raw.trim().is_empty() => title_from_raw(&raw, id),
        _ => id.to_string(),
    }
}

/// Raw display title from `<title>`, before entity decoding, so an unrelated
/// edit can preserve the exact encoded bytes.
fn raw_display_title(content: &str, id: &str) -> Option<String> {
    let raw = raw_title_text(content)?;
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    let dash = text.find('—');
    let entity = text.find("&mdash;");
    let separator = match (dash, entity) {
        (Some(d), Some(e)) => {
            if d <= e {
                Some((d, '—'.len_utf8()))
            } else {
                Some((e, "&mdash;".len()))
            }
        }
        (Some(d), None) => Some((d, '—'.len_utf8())),
        (None, Some(e)) => Some((e, "&mdash;".len())),
        (None, None) => None,
    };
    if let Some((at, width)) = separator {
        let rest = text[at + width..].trim();
        return (!rest.is_empty()).then(|| rest.to_string());
    }
    let without_id = text
        .strip_prefix(id)
        .and_then(|rest| {
            if rest.is_empty() || !rest.chars().next().is_some_and(char::is_whitespace) {
                None
            } else {
                Some(rest.trim())
            }
        })
        .unwrap_or(text);
    (!without_id.is_empty()).then(|| without_id.to_string())
}

/// Split a `specs`/`refs` comment value on Unicode whitespace (section 3.4).
fn comment_list_in(block: &str, key: &str) -> Vec<String> {
    match parse_field_comment(block, key) {
        Some(raw) => split_list(&raw),
        None => Vec::new(),
    }
}

#[cfg(test)]
pub(crate) fn comment_list(content: &str, key: &str) -> Vec<String> {
    comment_list_in(comment_block(content).unwrap_or_default(), key)
}

/// Browser metadata: file-stem ID, `<title>` title, comment-sourced lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BrowserMetadata {
    pub title: String,
    pub specs: Vec<String>,
    pub refs: Vec<String>,
    pub task: String,
    pub status: String,
    pub superseded_by: Option<String>,
}

pub(crate) fn parse_browser_metadata(content: &str, id: &str) -> BrowserMetadata {
    let title = extract_title(content, id);
    let block = comment_block(content).unwrap_or_default();
    let specs = comment_list_in(block, "specs");
    let refs = comment_list_in(block, "refs");
    let task = parse_field_comment(block, "task")
        .map(|v| v.trim().to_string())
        .unwrap_or_default();
    let status = parse_field_comment(block, "status")
        .map(|v| v.trim().to_string())
        .unwrap_or_default();
    let superseded_by = parse_field_comment(block, "superseded-by")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    BrowserMetadata {
        title,
        specs,
        refs,
        task,
        status,
        superseded_by,
    }
}

pub(crate) fn parse_record(content: &str, fallback_id: &str) -> Mockup {
    // The record ID is always the file stem; the first <h1> of a review
    // package is the landing heading, never the record identity.
    let id = fallback_id.to_string();
    // Title: prefer the <title> element (browser rule); only when it is
    // absent do CLI files fall back to the stub <h1>.
    let mut title = String::new();
    let has_title = raw_title_text(content).is_some_and(|raw| !raw.trim().is_empty());
    if has_title {
        title = extract_title(content, &id);
    } else if let Some(val) = extract_between(content, "<h1>", "</h1>") {
        let mut parts = val.splitn(2, char::is_whitespace);
        let _first = parts.next().unwrap_or("").trim();
        let rest = parts.next().unwrap_or("").trim();
        if !rest.is_empty() {
            title = decode_entities(rest);
        }
    }
    if title.is_empty() {
        title = id.clone();
    }
    let raw_title = raw_display_title(content, &id).unwrap_or_else(|| escape_text(&title));

    // Metadata comes only from the fixed comment block. A visible paragraph
    // or a title is never a metadata source.
    let block = comment_block(content).unwrap_or_default();
    let mut status = parse_field_comment(block, "status").unwrap_or_default();
    if status.is_empty() {
        status = "pending".to_string();
    }
    let task = parse_field_comment(block, "task").unwrap_or_default();
    let specs = split_list(&parse_field_comment(block, "specs").unwrap_or_default());
    let refs = split_list(&parse_field_comment(block, "refs").unwrap_or_default());
    let superseded_str = parse_field_comment(block, "superseded-by").unwrap_or_default();
    let superseded_by = if superseded_str.trim().is_empty() {
        None
    } else {
        Some(superseded_str.trim().to_string())
    };

    Mockup {
        id,
        title,
        raw_title,
        task,
        specs,
        refs,
        status,
        superseded_by,
    }
}

pub(crate) fn render_record(record: &Mockup) -> String {
    let id = escape_text(&record.id);
    let title_html = escape_text(&record.title);
    let title_raw = if record.raw_title.is_empty() {
        if record.title.is_empty() {
            id.clone()
        } else {
            title_html.clone()
        }
    } else {
        record.raw_title.clone()
    };
    let status = escape_text(&record.status);
    let status_comment = escape_comment(&record.status);
    let task = escape_text(&record.task);
    let task_comment = escape_comment(&record.task);
    let specs_str = record.specs.join(" ");
    let specs_comment = escape_comment(&specs_str);
    let specs_text = escape_text(&specs_str);
    let refs_str = record.refs.join(" ");
    let refs_comment = escape_comment(&refs_str);
    let refs_text = escape_text(&refs_str);
    let superseded = record.superseded_by.clone().unwrap_or_default();
    let superseded_text = escape_text(&superseded);
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{id} — {title_raw}</title>\n</head>\n<body>\n<!-- mockup-id: {id} -->\n<!-- status: {status_comment} -->\n<!-- task: {task_comment} -->\n<!-- specs: {specs_comment} -->\n<!-- refs: {refs_comment} -->\n<h1>{id} {title_html}</h1>\n<p>Status: {status}</p>\n<p>Task: {task}</p>\n<p>UI specifications: {specs_text}</p>\n<p>References: {refs_text}</p>\n<p>Superseded by: {superseded_text}</p>\n</body>\n</html>\n"
    )
}

const ALLOWED_STATUSES: &[&str] = &["pending", "approved", "archived"];

pub fn valid_transition(current: &str, next: &str) -> bool {
    if current == next {
        return true;
    }
    match current {
        "pending" => matches!(next, "approved" | "archived"),
        "approved" => matches!(next, "archived"),
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
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
    title_raw: &str,
    task_raw: &str,
    specs_raw: &[String],
    refs_raw: &[String],
) -> Result<String, AppError> {
    // The title has already passed the shared title contract.
    let title = title_raw.trim().to_string();
    let task = sanitize_field(task_raw)?;
    let specs = sanitize_list(specs_raw)?;
    if specs.is_empty() {
        return Err(AppError::Validation(
            "at least one spec is required".to_string(),
        ));
    }
    let refs = sanitize_list(refs_raw)?;

    let active = scope.active.clone();
    ensure_dirs(&active)?;

    let reservation = crate::record::identity::reserve_number(
        &active,
        &scope.archived,
        &scope.prefix,
        scope.id_padding,
        "html",
    )
    .map_err(|e| AppError::Runtime(e.to_string()))?;
    let id = reservation.id.clone();

    let record = Mockup {
        id: id.clone(),
        raw_title: escape_text(&title),
        title,
        task,
        specs,
        refs,
        status: "pending".to_string(),
        superseded_by: None,
    };
    let safe_path = Context::ensure_within(&ctx.project_dir, &reservation.path)?;
    crate::record::store::write_atomic(
        ctx,
        &scope.config,
        registry,
        &safe_path,
        &render_record(&record),
    )
    .map_err(|e| AppError::Runtime(e.to_string()))?;
    reservation.commit();
    Ok(id)
}

fn list(ctx: &Context, scope: &DocumentScope) -> Result<CommandOutput, AppError> {
    let active = scope.active.clone();

    let mut items: Vec<(String, String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&active) {
        for e in rd {
            let e = e.map_err(|err| AppError::Runtime(err.to_string()))?;
            let path = e.path();
            if !path.is_file() || !is_record_path(&path, &scope.prefix) {
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
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn view(ctx: &Context, scope: &DocumentScope, id_raw: &str) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let active = scope.active.clone();
    let archived = scope.archived.clone();
    let path = resolve_record(ctx, &active, &archived, &id)?;

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
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
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

fn edit(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
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
    // The title has already passed the shared title contract.
    let title = title_raw.map(|value| value.trim().to_string());
    let task = task_raw.map(sanitize_field).transpose()?;
    let specs = specs_raw.map(sanitize_list).transpose()?;
    let refs = refs_raw.map(sanitize_list).transpose()?;
    let status = status_raw.map(sanitize_field).transpose()?;
    let superseded_by = superseded_by_raw.map(sanitize_field).transpose()?;

    let active = scope.active.clone();
    let archived = scope.archived.clone();
    let path = resolve_record(ctx, &active, &archived, &id)?;

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
        record.raw_title = escape_text(&t);
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
            scope
                .config
                .effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    if let Some(st) = status {
        if !ALLOWED_STATUSES.contains(&st.as_str()) {
            return Err(AppError::Validation(format!("unknown status: {}", st)));
        }
        // `archive` and `supersede` are the only writers of `archived`.
        if st == "archived" {
            return Err(AppError::Validation(format!(
                "status 'archived' is not allowed here; use archive: {}",
                id
            )));
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

    let mut move_to_archived = false;
    if let Some(sup) = superseded_by {
        if sup == id {
            return Err(AppError::Validation(format!(
                "mockup cannot supersede itself: {}",
                id
            )));
        }
        let _ = resolve_record(ctx, &active, &archived, &sup)?;
        record.status = "archived".to_string();
        record.superseded_by = Some(sup);
        move_to_archived = true;
    }

    let destination = if move_to_archived {
        archived.join(format!("{}.html", id))
    } else {
        path.clone()
    };
    let safe_dest = Context::ensure_within(&ctx.project_dir, &destination)?;
    write_archived(ctx, &scope.config, registry, &path, &safe_dest, &record)?;

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
        ..Default::default()
    })
}

fn ok_output(ctx: &Context, id: &str) -> CommandOutput {
    let json = if ctx.json {
        Some(serde_json::json!({"status": "ok", "id": id}))
    } else {
        None
    };
    CommandOutput {
        text: String::new(),
        json,
        ..Default::default()
    }
}

/// Render the archived form of a record, then write the destination and remove
/// the source. The destination is written first (ADR-0022 decision 2), so a
/// failed write leaves the source untouched and an interrupted run never loses
/// content. When source and destination coincide (already archived), a single
/// in-place write is used and nothing is unlinked.
fn write_archived(
    ctx: &Context,
    cfg: &crate::config::Config,
    registry: &crate::registry::ArtifactRegistry,
    source: &Path,
    destination: &Path,
    record: &Mockup,
) -> Result<(), AppError> {
    let rendered = render_record(record);
    let safe_dest = Context::ensure_within(&ctx.project_dir, destination)?;
    let source = Context::ensure_within(&ctx.project_dir, source)?;
    if source == safe_dest {
        crate::record::store::write_atomic(ctx, cfg, registry, &safe_dest, &rendered)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
    } else {
        crate::record::store::archive_content(ctx, cfg, registry, &source, &safe_dest, &rendered)?;
    }
    Ok(())
}

fn archive(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
    id_raw: &str,
) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let active = scope.active.clone();
    let archived = scope.archived.clone();

    let Some(path) = find_record(ctx, &active, &id)? else {
        // Already archived: no read, no write, no modification-time change.
        if find_record(ctx, &archived, &id)?.is_some() {
            return Ok(ok_output(ctx, &id));
        }
        return Err(AppError::NotFound(format!("not found: {}", id)));
    };

    let content = fs::read_to_string(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut record = parse_record(&content, &id);
    if record.superseded_by.is_some() {
        return Err(AppError::Validation(format!(
            "status transitions blocked for superseded mockup: {}",
            id
        )));
    }
    record.status = "archived".to_string();

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::Runtime("invalid record path".to_string()))?;
    let destination = archived.join(file_name);
    write_archived(ctx, &scope.config, registry, &path, &destination, &record)?;

    Ok(ok_output(ctx, &id))
}

pub fn delete(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
    id_raw: &str,
    force: bool,
) -> Result<CommandOutput, AppError> {
    let id = sanitize_field(id_raw)?;
    let active = scope.active.clone();
    let archived = scope.archived.clone();

    let is_active = find_record(ctx, &active, &id)?.is_some();
    if !is_active && find_record(ctx, &archived, &id)?.is_none() {
        return Err(AppError::NotFound(format!("not found: {}", id)));
    }

    // An active mockup is archived first; the active file is never unlinked
    // directly. A failed archive step removes nothing.
    if is_active {
        archive(ctx, scope, registry, &id)?;
    }

    if !force {
        use std::io::{self, Write};
        print!("Delete {}? [y/N] ", id);
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let ans = input.trim().to_lowercase();
        if ans != "y" && ans != "yes" {
            let json = if ctx.json {
                Some(serde_json::json!({"status":"aborted","id":id}))
            } else {
                None
            };
            return Ok(CommandOutput {
                text: String::new(),
                json,
                ..Default::default()
            });
        }
    }

    // Only a resolved archived path is ever removed.
    let path = find_record(ctx, &archived, &id)?.ok_or_else(|| {
        AppError::Runtime(format!("archived record not found after archive: {}", id))
    })?;
    fs::remove_file(&path).map_err(|e| AppError::Runtime(e.to_string()))?;
    Ok(ok_output(ctx, &id))
}

pub fn supersede(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &crate::registry::ArtifactRegistry,
    old_raw: &str,
    new_raw: &str,
) -> Result<CommandOutput, AppError> {
    let old = sanitize_field(old_raw)?;
    let new = sanitize_field(new_raw)?;
    if old == new {
        return Err(AppError::Validation(format!(
            "mockup cannot supersede itself: {}",
            old
        )));
    }
    let active = scope.active.clone();
    let archived = scope.archived.clone();
    let _ = resolve_record(ctx, &active, &archived, &new)?;
    // Archive old with the superseded_by back-reference already present in the
    // destination before the active source is unlinked.
    let old_path = resolve_record(ctx, &active, &archived, &old)?;
    let content = fs::read_to_string(&old_path).map_err(|e| AppError::Runtime(e.to_string()))?;
    let mut record = parse_record(&content, &old);
    if record.id.is_empty() {
        record.id = old.clone();
    }
    if record.title.is_empty() {
        record.title = old.clone();
    }
    if record.status.is_empty() {
        record.status = "pending".to_string();
    }
    record.status = "archived".to_string();
    record.superseded_by = Some(new.clone());
    let destination = archived.join(format!("{}.html", old));
    write_archived(
        ctx,
        &scope.config,
        registry,
        &old_path,
        &destination,
        &record,
    )?;
    Ok(ok_output(ctx, &old))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_transition_table() {
        assert!(valid_transition("pending", "pending"));
        assert!(valid_transition("pending", "approved"));
        assert!(valid_transition("pending", "archived"));
        assert!(valid_transition("approved", "approved"));
        assert!(valid_transition("approved", "archived"));
        assert!(valid_transition("archived", "archived"));
        assert!(!valid_transition("approved", "pending"));
        assert!(!valid_transition("archived", "pending"));
        assert!(!valid_transition("archived", "approved"));
    }

    #[test]
    fn title_from_em_dash() {
        let content = "<title>MOCK-001 — Login screen</title>";
        assert_eq!(extract_title(content, "MOCK-001"), "Login screen");
    }

    #[test]
    fn title_from_mdash_entity() {
        let content = "<title>MOCK-006 &mdash; Browser UI mockups review package</title>";
        assert_eq!(
            extract_title(content, "MOCK-006"),
            "Browser UI mockups review package"
        );
    }

    #[test]
    fn title_without_separator_uses_id_prefix_or_whole() {
        assert_eq!(
            extract_title("<title>MOCK-001 My Title</title>", "MOCK-001"),
            "My Title"
        );
        assert_eq!(
            extract_title("<title>Just Title</title>", "MOCK-001"),
            "Just Title"
        );
        assert_eq!(
            extract_title("<title>MOCK-001</title>", "MOCK-001"),
            "MOCK-001"
        );
    }

    #[test]
    fn title_absent_or_empty_falls_back_to_id() {
        assert_eq!(
            extract_title("<html><body>hi</body></html>", "MOCK-002"),
            "MOCK-002"
        );
        assert_eq!(extract_title("<title>   </title>", "MOCK-002"), "MOCK-002");
        assert_eq!(extract_title("<title></title>", "MOCK-002"), "MOCK-002");
    }

    #[test]
    fn title_decodes_standard_entities() {
        assert_eq!(
            extract_title(
                "<title>MOCK-001 — A &amp; B &lt;tag&gt;</title>",
                "MOCK-001"
            ),
            "A & B <tag>"
        );
        assert_eq!(
            extract_title(
                "<title>MOCK-001 — &quot;Hi&quot; &#39;x&#39;</title>",
                "MOCK-001"
            ),
            "\"Hi\" 'x'"
        );
        assert_eq!(decode_entities("&amp;mdash;"), "&mdash;");
    }

    #[test]
    fn title_first_separator_wins() {
        assert_eq!(
            extract_title("<title>MOCK-001 — First &mdash; Second</title>", "MOCK-001"),
            "First — Second"
        );
        assert_eq!(
            extract_title("<title>MOCK-001 &mdash; First — Second</title>", "MOCK-001"),
            "First — Second"
        );
    }

    #[test]
    fn comment_lists_preserve_order_and_drop_empty() {
        let content = "<!-- specs: UI-002 UI-001 -->\n<!-- refs: docs/a.md  docs/b.md -->";
        assert_eq!(comment_list(content, "specs"), vec!["UI-002", "UI-001"]);
        assert_eq!(
            comment_list(content, "refs"),
            vec!["docs/a.md", "docs/b.md"]
        );
        assert_eq!(comment_list("<html></html>", "specs"), Vec::<String>::new());
        assert_eq!(
            comment_list("<!-- specs:  -->", "specs"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn browser_metadata_reads_comments_only_and_stem_id() {
        let content = "<title>MOCK-006 &mdash; Review package</title><body><!-- specs: UI-001 UI-002 --><!-- refs: docs/a.md --><!-- task: TASK-1 --><!-- status: approved --><!-- superseded-by: MOCK-002 --><p>UI specifications: WRONG</p><h1>TASK-41 Landing heading</h1></body>";
        let meta = parse_browser_metadata(content, "MOCK-006");
        assert_eq!(meta.title, "Review package");
        assert_eq!(meta.specs, vec!["UI-001", "UI-002"]);
        assert_eq!(meta.refs, vec!["docs/a.md"]);
        assert_eq!(meta.task, "TASK-1");
        assert_eq!(meta.status, "approved");
        assert_eq!(meta.superseded_by, Some("MOCK-002".to_string()));
        let clean = "<!-- task: TASK-41 -->";
        let meta2 = parse_browser_metadata(clean, "MOCK-006");
        assert_eq!(meta2.task, "TASK-41");
        assert_eq!(meta2.status, "");
        assert_eq!(meta2.superseded_by, None);
        assert_eq!(meta2.title, "MOCK-006");
    }

    #[test]
    fn parse_record_uses_stem_and_title_tag_not_landing_h1() {
        let content = "<title>MOCK-006 &mdash; Browser UI mockups review package</title><body><h1>TASK-41 Landing heading</h1></body>";
        assert_eq!(
            extract_title(content, "MOCK-006"),
            "Browser UI mockups review package"
        );
        let record = parse_record(content, "MOCK-006");
        assert_eq!(record.id, "MOCK-006");
        assert_eq!(record.title, "Browser UI mockups review package");
    }

    #[test]
    fn injection_titles_do_not_change_metadata() {
        for title in [
            "Status: approved",
            "a --> b",
            "<script>alert(1)</script>",
            "-- status: approved",
        ] {
            let record = Mockup {
                id: "MOCK-001".to_string(),
                raw_title: escape_text(title),
                title: title.to_string(),
                task: "TASK-001".to_string(),
                specs: vec!["UI-001".to_string()],
                refs: vec!["R-1".to_string()],
                status: "pending".to_string(),
                superseded_by: None,
            };
            let rendered = render_record(&record);
            assert!(!rendered.contains("<script>"), "{rendered}");
            assert!(
                !rendered.contains("<!-- status: approved"),
                "title must not inject a comment: {rendered}"
            );
            let parsed = parse_record(&rendered, "MOCK-001");
            assert_eq!(parsed.status, "pending", "title {title:?}");
            assert_eq!(parsed.task, "TASK-001");
            assert_eq!(parsed.title, title);
        }
    }

    #[test]
    fn visible_paragraphs_are_not_metadata_sources() {
        let content = r#"<html><head><title>MOCK-001 — T</title></head><body>
<!-- mockup-id: MOCK-001 -->
<!-- status: pending -->
<!-- task: TASK-1 -->
<!-- specs: UI-001 -->
<!-- refs: R-1 -->
<h1>MOCK-001 T</h1>
<p>Status: approved</p>
<p>Task: TASK-999</p>
<p>UI specifications: UI-999</p>
<p>References: R-999</p>
</body></html>"#;
        let parsed = parse_record(content, "MOCK-001");
        assert_eq!(parsed.status, "pending");
        assert_eq!(parsed.task, "TASK-1");
        assert_eq!(parsed.specs, vec!["UI-001"]);
        assert_eq!(parsed.refs, vec!["R-1"]);
    }

    #[test]
    fn unrelated_render_preserves_the_raw_encoded_title() {
        let content = "<title>MOCK-001 — A &amp; B &lt;tag&gt;</title>\n<!-- mockup-id: MOCK-001 -->\n<!-- status: pending -->\n<!-- task: TASK-1 -->\n<h1>MOCK-001 A &amp; B &lt;tag&gt;</h1>\n";
        let parsed = parse_record(content, "MOCK-001");
        assert_eq!(parsed.title, "A & B <tag>");
        assert_eq!(parsed.raw_title, "A &amp; B &lt;tag&gt;");
        let rendered = render_record(&parsed);
        assert!(
            rendered.contains("<title>MOCK-001 — A &amp; B &lt;tag&gt;</title>"),
            "{rendered}"
        );
        assert!(!rendered.contains("&amp;amp;"), "{rendered}");
        assert_eq!(parse_record(&rendered, "MOCK-001").title, "A & B <tag>");
    }

    #[test]
    fn parse_record_falls_back_to_stub_h1_only_without_title() {
        let record = parse_record("<h1>MOCK-010 Later</h1>", "MOCK-010");
        assert_eq!(record.id, "MOCK-010");
        assert_eq!(record.title, "Later");
        let record = parse_record("<html><body>hi</body></html>", "MOCK-010");
        assert_eq!(record.title, "MOCK-010");
    }
}
