use crate::CommandOutput;
use crate::context::Context;
use crate::document::{DocumentArgs, DocumentScope, DocumentVerb};
use crate::error::AppError;
use crate::sanitize;
use std::fs;

pub fn handle(
    ctx: &Context,
    scope: &DocumentScope,
    verb: DocumentVerb,
    args: &DocumentArgs,
) -> Result<CommandOutput, AppError> {
    match verb {
        DocumentVerb::Create => {
            let id = create(
                ctx,
                scope,
                args.require("term")?,
                args.require("definition")?,
                args.get_all("label"),
                args.get_all("reference"),
            )?;
            Ok(crate::document::typed_created(ctx, id))
        }
        DocumentVerb::List => list(ctx, scope),
        DocumentVerb::View => view(ctx, scope, args.require("id")?),
        DocumentVerb::Edit => edit(
            ctx,
            scope,
            args.require("id")?,
            args.get("term"),
            args.get("definition"),
            args.get_all("label"),
            args.get_all("reference"),
            args.flag("editor"),
        ),
        other => Err(crate::document::unsupported_verb(other)),
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

fn glossary_dotted() -> Result<bool, AppError> {
    let descriptor = crate::record::descriptor::descriptor_for_doc_type("glossary")
        .ok_or_else(|| AppError::Runtime("record descriptor is unavailable".to_string()))?;
    Ok(descriptor.is_dotted_id())
}

fn resolve_glossary(
    ctx: &Context,
    scope: &DocumentScope,
    id: &str,
) -> Result<crate::record::resolve::ResolvedRecord, AppError> {
    let dotted = glossary_dotted()?;
    crate::record::resolve::resolve_in_dirs(
        ctx,
        &[scope.active.clone(), scope.archived.clone()],
        id,
        std::slice::from_ref(&scope.prefix),
        dotted,
    )?
    .ok_or_else(|| AppError::NotFound(format!("not found: {}", id)))
}

fn ensure_dirs(dir: &std::path::Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

#[derive(Clone, Debug, Default)]
struct GlossaryData {
    id: String,
    term: String,
    definition: String,
}

fn record_from_document(doc: &crate::record::document::Document) -> GlossaryData {
    let (id, term) = crate::record::document::heading_id_title(doc);
    let definition = crate::record::document::section_text(doc, "Definition").unwrap_or_default();
    GlossaryData {
        id,
        term,
        definition,
    }
}

fn body_document(id: &str, term: &str, definition: &str) -> crate::record::document::Document {
    crate::record::document::Document {
        source: String::new(),
        front_matter: None,
        frontmatter: None,
        heading: Some(format!("# {id} {term}")),
        preamble: vec![String::new()],
        sections: vec![crate::record::document::Section {
            heading: "## Definition".to_string(),
            body: vec![definition.to_string()],
        }],
    }
}

pub fn create(
    ctx: &Context,
    scope: &DocumentScope,
    term_raw: &str,
    definition_raw: &str,
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let term = sanitize_text(term_raw)?;
    let definition = sanitize_text(definition_raw)?;

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

    let active = scope.active.clone();
    let archived = scope.archived.clone();
    ensure_dirs(&active)?;

    let next = crate::record::identity::next_number(&active, &archived, &scope.prefix);
    let id = crate::record::identity::format_id(&scope.prefix, next as i64, scope.id_padding);

    let body_doc = body_document(&id, &term, &definition);
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
            crate::record::frontmatter::FrontMatterValue::String(id.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "title".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(term.clone()),
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
    let file_path = active.join(format!("{}.md", id));
    let safe_path = Context::ensure_within(&ctx.project_dir, &file_path)?;
    crate::record::store::write_atomic(ctx, &safe_path, &file_content)?;
    Ok(id)
}

pub fn list(ctx: &Context, scope: &DocumentScope) -> Result<crate::CommandOutput, AppError> {
    let dotted = glossary_dotted()?;
    let active = scope.active.clone();
    let _ = ensure_dirs(&active);

    let records = crate::record::resolve::list_in_dir(
        ctx,
        &active,
        std::slice::from_ref(&scope.prefix),
        dotted,
    )?;
    let mut items: Vec<(String, String, String)> = Vec::new();
    for record in records {
        let content =
            fs::read_to_string(&record.path).map_err(|e| AppError::Runtime(e.to_string()))?;
        let term = crate::record::document::parse(&content)
            .ok()
            .map(|d| crate::record::document::heading_id_title(&d).1)
            .unwrap_or_default();
        let term = if term.is_empty() {
            record.id.clone()
        } else {
            term
        };
        items.push((record.id, term, content));
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let text = items
        .iter()
        .map(|(id, term, content)| {
            crate::record::view::RecordView::lenient(content, id).list_line(term)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let json = if ctx.json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, term, content)| {
                serde_json::json!({
                    "id": id,
                    "term": term,
                    "title": term,
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

pub fn view(
    ctx: &Context,
    scope: &DocumentScope,
    id_raw: &str,
) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let record = resolve_glossary(ctx, scope, &requested)?;
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

fn frontmatter_list_glo(content: &str, key: &str) -> Vec<String> {
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
    id_raw: &str,
    term_raw: Option<&str>,
    definition_raw: Option<&str>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<crate::CommandOutput, AppError> {
    let requested = sanitize_text(id_raw)?;
    let resolved = resolve_glossary(ctx, scope, &requested)?;
    resolved.ensure_writable()?;
    let id = resolved.id.clone();
    let path = resolved.path.clone();

    let (doc, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();
    let mut gd = record_from_document(&doc);
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
            scope
                .config
                .effective_editor()
                .ok_or_else(|| AppError::Runtime("no editor configured".to_string()))?,
        )
    } else {
        None
    };

    let mut ops: Vec<crate::record::patch::PatchOp> = Vec::new();
    if term_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            gd.term.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            gd.term.clone(),
        ));
    }
    if definition_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Definition".into(),
            gd.definition.clone(),
        ));
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_glo(&content, "labels");
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
        let mut fm_refs = frontmatter_list_glo(&content, "references");
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
