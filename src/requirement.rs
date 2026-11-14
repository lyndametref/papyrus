use crate::CommandOutput;
use crate::context::Context;
use crate::document::{DocumentArgs, DocumentScope, DocumentVerb};
use crate::error::AppError;
use crate::registry::ArtifactRegistry;
use crate::sanitize;
use std::fs;
use std::path::Path;

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

pub fn handle(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &ArtifactRegistry,
    kind: Kind,
    verb: DocumentVerb,
    args: &DocumentArgs,
) -> Result<CommandOutput, AppError> {
    match verb {
        DocumentVerb::Create => {
            let id = create(
                ctx,
                scope,
                registry,
                kind,
                args.get("title"),
                args.get("title_flag"),
                args.get("story"),
                args.get("description"),
                args.require("group")?,
                args.get_all("label"),
                args.get_all("reference"),
            )?;
            Ok(crate::document::typed_created(ctx, id))
        }
        DocumentVerb::CreateGroup => {
            let id = create_group(
                ctx,
                scope,
                kind,
                args.require("title")?,
                args.get("story"),
                args.get("description"),
            )?;
            Ok(crate::document::typed_created(ctx, id))
        }
        DocumentVerb::List => list(ctx, scope, kind),
        DocumentVerb::View => view(ctx, scope, kind, args.require("id")?),
        DocumentVerb::Edit => edit(
            ctx,
            scope,
            kind,
            args.require("id")?,
            args.get("title"),
            args.get("story"),
            args.get("description"),
            args.get_all("label"),
            args.get_all("reference"),
            args.flag("editor"),
        ),
        other => Err(crate::document::unsupported_verb(other)),
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

fn kind_dotted(kind: Kind) -> Result<bool, AppError> {
    let descriptor = crate::record::descriptor::descriptor_for_doc_type(kind.key())
        .ok_or_else(|| AppError::Runtime("record descriptor is unavailable".to_string()))?;
    Ok(descriptor.is_dotted_id())
}

fn resolve_kind_record(
    ctx: &Context,
    scope: &DocumentScope,
    kind: Kind,
    id: &str,
) -> Result<crate::record::resolve::ResolvedRecord, AppError> {
    let dotted = kind_dotted(kind)?;
    let dirs = [scope.active.clone(), scope.archived.clone()];
    if let Some(record) = crate::record::resolve::resolve_in_dirs(
        ctx,
        &dirs,
        id,
        std::slice::from_ref(&scope.prefix),
        dotted,
    )? {
        return Ok(record);
    }
    // A group record carries the simple `<ID>` shape, not a dotted ID.
    if dotted
        && let Some(record) = crate::record::resolve::resolve_in_dirs(
            ctx,
            &dirs,
            id,
            std::slice::from_ref(&scope.prefix),
            false,
        )?
    {
        return Ok(record);
    }
    Err(AppError::NotFound(format!("not found: {}", id)))
}

fn ensure_dirs(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

pub fn create_group(
    ctx: &Context,
    scope: &DocumentScope,
    kind: Kind,
    title_raw: &str,
    story_raw: Option<&str>,
    desc_raw: Option<&str>,
) -> Result<String, AppError> {
    let title = sanitize_text(title_raw)?;
    let story = match story_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };
    let desc = match desc_raw {
        Some(s) => Some(sanitize_text(s)?),
        None => None,
    };

    let active = scope.active.clone();
    ensure_dirs(&active)?;

    let next = crate::record::identity::next_number(&active, &scope.archived, &scope.prefix);
    let id = crate::record::identity::format_id(&scope.prefix, next as i64, scope.id_padding);

    let body_doc = body_document_group(kind, &id, &title, story.as_deref(), desc.as_deref());
    let content = crate::record::document::render(&body_doc);

    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &content)?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    ctx: &Context,
    scope: &DocumentScope,
    registry: &ArtifactRegistry,
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

    if !crate::record::identity::is_group_id(&group_id, &scope.prefix) {
        let other_kind = match kind {
            Kind::Req => Kind::Nfr,
            Kind::Nfr => Kind::Req,
        };
        let mismatch = registry
            .descriptor(other_kind.key())
            .filter(|other| crate::record::identity::is_group_id(&group_id, &other.prefix))
            .map(|other| {
                DocumentScope::resolve(ctx, other)
                    .map(|other_scope| {
                        other_scope.active.join(format!("{}.md", group_id)).exists()
                            || other_scope
                                .archived
                                .join(format!("{}.md", group_id))
                                .exists()
                    })
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

    let active = scope.active.clone();
    let deprecated = scope.archived.clone();
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

    let (group_doc, _token) = crate::record::store::read_document(ctx, &group_active_safe)?;
    let group_parsed = parsed_from_document(&group_doc);
    let group_title = group_parsed.title.clone();
    if group_title.is_empty() {
        return Err(AppError::Runtime(format!(
            "group has no title: {}",
            group_id
        )));
    }

    let next_seq = crate::record::identity::next_seq_number(&active, &scope.archived, &group_id);
    let req_id =
        crate::record::identity::format_seq_id(&group_id, next_seq as i64, scope.id_padding);

    let body_doc = body_document_requirement(
        kind,
        &req_id,
        &title_string,
        story.as_deref(),
        desc.as_deref(),
        &group_id,
        &group_title,
    );
    let body = crate::record::document::render(&body_doc);
    let parsed = crate::record::document::parse(&body)?;
    let now = crate::record::frontmatter::utc_now();
    let labels_value = if labels.is_empty() {
        crate::record::frontmatter::FrontMatterValue::String("[]".to_string())
    } else {
        crate::record::frontmatter::FrontMatterValue::String(labels.join("\n"))
    };
    let refs_value = if references.is_empty() {
        crate::record::frontmatter::FrontMatterValue::String("[]".to_string())
    } else {
        crate::record::frontmatter::FrontMatterValue::String(references.join("\n"))
    };
    let ops = vec![
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "id".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(req_id.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "title".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(title_string.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "created_date".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(now.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "updated_date".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(now),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue("labels".to_string(), labels_value),
        crate::record::patch::PatchOp::SetFrontMatterValue("references".to_string(), refs_value),
    ];
    let file_content = crate::record::patch::apply_patch(&parsed, &ops)?;
    let file_path = active.join(format!("{}.md", req_id));
    let safe_file = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_file, &file_content)?;
    Ok(req_id)
}

pub fn list(
    ctx: &Context,
    scope: &DocumentScope,
    kind: Kind,
) -> Result<crate::CommandOutput, AppError> {
    let dotted = kind_dotted(kind)?;
    let active = scope.active.clone();
    ensure_dirs(&active).ok();

    let mut records = crate::record::resolve::list_in_dir(
        ctx,
        &active,
        std::slice::from_ref(&scope.prefix),
        dotted,
    )?;
    if dotted {
        // Groups keep the simple `<ID>` shape and are listed alongside the
        // dotted requirement records.
        for group in crate::record::resolve::list_in_dir(
            ctx,
            &active,
            std::slice::from_ref(&scope.prefix),
            false,
        )? {
            if !group.id.contains('.') {
                records.push(group);
            }
        }
        records.sort_by(|a, b| a.id.cmp(&b.id));
    }
    let mut items: Vec<(String, String, String)> = Vec::new();
    for record in records {
        let content =
            fs::read_to_string(&record.path).map_err(|e| AppError::Runtime(e.to_string()))?;
        let title = crate::record::document::parse(&content)
            .ok()
            .map(|d| crate::record::document::heading_id_title(&d).1)
            .unwrap_or_default();
        let title = if title.is_empty() {
            record.id.clone()
        } else {
            title
        };
        items.push((record.id, title, content));
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

    Ok(crate::CommandOutput { text, json })
}

pub fn parsed_from_document(doc: &crate::record::document::Document) -> ParsedDoc {
    let (id, title) = crate::record::document::heading_id_title(doc);
    let story = crate::record::document::section_text(doc, "User Story");
    let mut group_id: Option<String> = None;
    let mut group_title: Option<String> = None;
    if let Some(text) = crate::record::document::section_text(doc, "Group") {
        let mut parts = text.splitn(2, char::is_whitespace);
        group_id = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        group_title = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    let description = crate::record::document::section_text(doc, "Description");
    ParsedDoc {
        id,
        title,
        story,
        group_id,
        group_title,
        description,
    }
}

fn body_document_group(
    kind: Kind,
    id: &str,
    title: &str,
    story: Option<&str>,
    desc: Option<&str>,
) -> crate::record::document::Document {
    let mut sections = Vec::new();
    if kind.supports_story()
        && let Some(s) = story
    {
        sections.push(crate::record::document::Section {
            heading: "## User Story".to_string(),
            body: vec![s.to_string(), String::new()],
        });
    }
    if let Some(d) = desc {
        sections.push(crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![d.to_string()],
        });
    }
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {id} {title}")),
        preamble: vec![String::new()],
        sections,
    }
}

fn body_document_requirement(
    kind: Kind,
    id: &str,
    title: &str,
    story: Option<&str>,
    desc: Option<&str>,
    group_id: &str,
    group_title: &str,
) -> crate::record::document::Document {
    let mut sections = Vec::new();
    if kind.supports_story()
        && let Some(s) = story
    {
        sections.push(crate::record::document::Section {
            heading: "## User Story".to_string(),
            body: vec![s.to_string(), String::new()],
        });
    }
    sections.push(crate::record::document::Section {
        heading: "## Group".to_string(),
        body: vec![group_id.to_string() + " " + group_title, String::new()],
    });
    if let Some(d) = desc {
        sections.push(crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![d.to_string()],
        });
    }
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {id} {title}")),
        preamble: vec![String::new()],
        sections,
    }
}

pub fn view(
    ctx: &Context,
    scope: &DocumentScope,
    kind: Kind,
    id_raw: &str,
) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let record = resolve_kind_record(ctx, scope, kind, &requested)?;
    let id = record.id.clone();
    let path = record.path;

    let content = crate::record::migrate::ensure_front_matter(ctx, &path)?;

    let text = content.clone();
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
    Ok(crate::CommandOutput { text, json })
}

fn frontmatter_list_req(content: &str, key: &str) -> Vec<String> {
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

#[allow(clippy::too_many_arguments)]
pub fn edit(
    ctx: &Context,
    scope: &DocumentScope,
    kind: Kind,
    id_raw: &str,
    title_raw: Option<&str>,
    story_raw: Option<&str>,
    desc_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let resolved = resolve_kind_record(ctx, scope, kind, &requested)?;
    resolved.ensure_writable()?;
    let id = resolved.id.clone();
    let path = resolved.path.clone();

    let (doc_read, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc_read.source.clone();
    let mut parsed = parsed_from_document(&doc_read);

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
    if kind.supports_story()
        && story_raw.is_some()
        && let Some(s) = &parsed.story
    {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## User Story".into(),
            s.clone(),
        ));
    }
    if desc_raw.is_some()
        && let Some(d) = &parsed.description
    {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Description".into(),
            d.clone(),
        ));
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

    let doc = doc_read;
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
        let editor_cmd = scope.config.effective_editor();
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

#[cfg(test)]
mod native_layout_tests {
    use super::*;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("requirement-native-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn ctx(dir: &std::path::Path) -> Context {
        Context {
            project_dir: dir.to_path_buf(),
            config_path: {
                let p = dir.join("papyrus.config.yml");
                if !p.exists() {
                    let _ = std::fs::write(&p, "project_name: Test\n");
                }
                p
            },
            json: true,
            no_color: true,
        }
    }

    fn req_scope(c: &Context) -> DocumentScope {
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir).unwrap();
        let registry = crate::registry::ArtifactRegistry::build(&cfg).unwrap();
        DocumentScope::resolve(c, registry.descriptor("req").unwrap()).unwrap()
    }

    #[test]
    fn dotted_ids_stay_distinct_during_resolution_and_listing() {
        let dir = tmpdir("dotted");
        let c = ctx(&dir);
        let scope = req_scope(&c);
        let active = dir.join("docs/requirements/business");
        fs::create_dir_all(&active).unwrap();
        fs::write(
            active.join("REQ-01.001.md"),
            "# REQ-01.001 One\n\n## Description\nA\n",
        )
        .unwrap();
        fs::write(
            active.join("REQ-01.002.md"),
            "# REQ-01.002 Two\n\n## Description\nB\n",
        )
        .unwrap();

        let one = view(&c, &scope, Kind::Req, "REQ-01.001").unwrap();
        assert_eq!(one.json.unwrap()["id"], "REQ-01.001");
        let two = view(&c, &scope, Kind::Req, "REQ-1.2").unwrap();
        assert_eq!(two.json.unwrap()["id"], "REQ-01.002");

        let listed = list(&c, &scope, Kind::Req).unwrap();
        assert!(listed.text.contains("REQ-01.001"));
        assert!(listed.text.contains("REQ-01.002"));
    }
}
