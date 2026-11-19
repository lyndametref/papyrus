use crate::CommandOutput;
use crate::context::Context;
use crate::document::{DocumentArgs, DocumentScope, DocumentVerb};
use crate::error::AppError;
use crate::sanitize;
use std::fs;
use std::path::Path;

pub fn handle(
    ctx: &Context,
    scope: &DocumentScope,
    record: &crate::record::descriptor::RecordDescriptor,
    verb: DocumentVerb,
    args: &DocumentArgs,
) -> Result<CommandOutput, AppError> {
    match verb {
        DocumentVerb::Create => {
            let id = create(
                ctx,
                scope,
                args.require("title")?,
                args.require_text("description")?,
                args.require("rationale")?,
                args.require("applicability")?,
                args.get_all("requirement"),
                args.get_all("label"),
                args.get_all("reference"),
            )?;
            Ok(crate::document::typed_created(ctx, id))
        }
        DocumentVerb::List => list(ctx, scope, record),
        DocumentVerb::View => view(ctx, scope, record, args.require("id")?),
        DocumentVerb::Edit => edit(
            ctx,
            scope,
            record,
            args.require("id")?,
            args.get("title"),
            args.text("description"),
            args.get_all("requirement"),
            args.optional("no_requirement")
                .map(|value| value.map(str::to_string)),
            args.get_all("label"),
            args.get_all("reference"),
            args.flag("editor"),
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

fn ensure_dirs(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| AppError::Runtime(e.to_string()))
}

fn active_or_archived(
    ctx: &Context,
    active: &Path,
    archived: &Path,
    id: &str,
    prefix: &str,
    dotted: bool,
) -> Result<crate::record::resolve::ResolvedRecord, AppError> {
    crate::record::resolve::resolve_in_dirs(
        ctx,
        &[active.to_path_buf(), archived.to_path_buf()],
        id,
        &[prefix.to_string()],
        dotted,
    )?
    .ok_or_else(|| AppError::NotFound(format!("not found: {}", id)))
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct RuleRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) rationale: String,
    pub(crate) applicability: String,
    pub(crate) requirements: Vec<String>,
}

fn requirements_from_document(doc: &crate::record::document::Document) -> Vec<String> {
    let Some(body) = crate::record::document::section_text(doc, "Requirements") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item = if let Some(rest) = trimmed.strip_prefix("- ") {
            rest.trim()
        } else if let Some(rest) = trimmed.strip_prefix('-') {
            rest.trim()
        } else {
            trimmed
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

fn record_from_document(doc: &crate::record::document::Document, fallback_id: &str) -> RuleRecord {
    let (id, title) = crate::record::document::heading_id_title(doc);
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
        description: crate::record::document::section_text(doc, "Description").unwrap_or_default(),
        rationale: crate::record::document::section_text(doc, "Rationale").unwrap_or_default(),
        applicability: crate::record::document::section_text(doc, "Applicability")
            .unwrap_or_default(),
        requirements: requirements_from_document(doc),
    }
}

fn body_document(
    id: &str,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
    requirements: &[String],
) -> crate::record::document::Document {
    let mut sections = vec![
        crate::record::document::Section {
            heading: "## Description".to_string(),
            body: vec![description.to_string(), String::new()],
        },
        crate::record::document::Section {
            heading: "## Rationale".to_string(),
            body: vec![rationale.to_string(), String::new()],
        },
        crate::record::document::Section {
            heading: "## Applicability".to_string(),
            body: vec![applicability.to_string()],
        },
    ];
    if !requirements.is_empty() {
        sections.push(crate::record::document::Section {
            heading: "## Requirements".to_string(),
            body: requirements.iter().map(|r| format!("- {r}")).collect(),
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

#[allow(clippy::too_many_arguments)]
pub fn create(
    ctx: &Context,
    scope: &DocumentScope,
    title_raw: &str,
    description_raw: sanitize::Text,
    rationale_raw: &str,
    applicability_raw: &str,
    requirements_raw: &[String],
    labels_raw: &[String],
    references_raw: &[String],
) -> Result<String, AppError> {
    let title = sanitize_field(title_raw)?;
    let description = description_raw.resolve()?;
    let rationale = sanitize_field(rationale_raw)?;
    let applicability = sanitize_field(applicability_raw)?;

    let mut requirements = Vec::new();
    for r in requirements_raw {
        requirements.push(sanitize_field(r)?);
    }
    let requirements = dedup(requirements);

    let mut labels: Vec<String> = Vec::new();
    for l in labels_raw {
        if !labels.contains(l) {
            labels.push(sanitize_field(l)?);
        }
    }
    let mut references: Vec<String> = Vec::new();
    for r in references_raw {
        if !references.contains(r) {
            references.push(sanitize_field(r)?);
        }
    }

    let active = scope.active.clone();
    let archived = scope.archived.clone();
    ensure_dirs(&active)?;

    let next = crate::record::identity::next_number(&active, &archived, &scope.prefix);
    let id = crate::record::identity::format_id(&scope.prefix, next as i64, scope.id_padding);

    let body_doc = body_document(
        &id,
        &title,
        &description,
        &rationale,
        &applicability,
        &requirements,
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
            crate::record::frontmatter::FrontMatterValue::String(id.clone()),
        ),
        crate::record::patch::PatchOp::SetFrontMatterValue(
            "title".to_string(),
            crate::record::frontmatter::FrontMatterValue::String(title.clone()),
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

pub fn list(
    ctx: &Context,
    scope: &DocumentScope,
    record: &crate::record::descriptor::RecordDescriptor,
) -> Result<CommandOutput, AppError> {
    let dotted = record.is_dotted_id();
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
    Ok(CommandOutput {
        text,
        json,
        ..Default::default()
    })
}

pub fn view(
    ctx: &Context,
    scope: &DocumentScope,
    record: &crate::record::descriptor::RecordDescriptor,
    id_raw: &str,
) -> Result<CommandOutput, AppError> {
    let requested = sanitize_field(id_raw)?;
    let dotted = record.is_dotted_id();
    let active = scope.active.clone();
    let archived = scope.archived.clone();
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&archived);
    let record = active_or_archived(ctx, &active, &archived, &requested, &scope.prefix, dotted)?;
    let id = record.id.clone();
    let path = record.path;

    let content = crate::record::migrate::ensure_front_matter(ctx, &path)?;
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

fn frontmatter_list_rule(content: &str, key: &str) -> Vec<String> {
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
    record: &crate::record::descriptor::RecordDescriptor,
    id_raw: &str,
    title_raw: Option<&str>,
    description_raw: Option<sanitize::Text>,
    requirements_add: &[String],
    no_requirement: Option<Option<String>>,
    labels_raw: &[String],
    references_raw: &[String],
    editor: bool,
) -> Result<CommandOutput, AppError> {
    let requested = sanitize_field(id_raw)?;
    let dotted = record.is_dotted_id();
    let active = scope.active.clone();
    let archived = scope.archived.clone();
    let _ = ensure_dirs(&active);
    let _ = ensure_dirs(&archived);
    let resolved = active_or_archived(ctx, &active, &archived, &requested, &scope.prefix, dotted)?;
    resolved.ensure_writable()?;
    let id = resolved.id.clone();
    let path = resolved.path.clone();

    if editor && scope.config.effective_editor().is_none() {
        return Err(AppError::Runtime("no editor configured".to_string()));
    }

    let (doc, _token) = crate::record::store::read_document(ctx, &path)?;
    let content = doc.source.clone();
    let mut record = record_from_document(&doc, &id);

    if let Some(t) = title_raw {
        record.title = sanitize_field(t)?;
    }
    if let Some(d) = &description_raw {
        record.description = d.resolve()?;
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
    if let Some(no) = &no_requirement {
        match no {
            Some(rid) => {
                let rid = sanitize_field(rid)?;
                record.requirements.retain(|r| r != &rid);
            }
            None => record.requirements.clear(),
        }
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
    if title_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetFrontMatter(
            "title".into(),
            record.title.clone(),
        ));
        ops.push(crate::record::patch::PatchOp::SetHeadingTitle(
            record.title.clone(),
        ));
    }
    if description_raw.is_some() {
        ops.push(crate::record::patch::PatchOp::SetSection(
            "## Description".into(),
            record.description.clone(),
        ));
    }
    // requirements body section
    if !requirements_add.is_empty() || no_requirement.is_some() {
        if record.requirements.is_empty() {
            ops.push(crate::record::patch::PatchOp::ClearSection(
                "## Requirements".into(),
            ));
        } else {
            let body = record
                .requirements
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n");
            ops.push(crate::record::patch::PatchOp::SetSection(
                "## Requirements".into(),
                body,
            ));
        }
    }
    if !labels_raw.is_empty() {
        let mut fm_labels = frontmatter_list_rule(&content, "labels");
        for l in labels_raw {
            let lv = sanitize_field(l)?;
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
        let mut fm_refs = frontmatter_list_rule(&content, "references");
        for r in references_raw {
            let rv = sanitize_field(r)?;
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
        let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;
        crate::record::store::write_atomic(ctx, &safe_path, &new_content)?;
    }

    let safe_path = Context::ensure_within(&ctx.project_dir, &path)?;

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
        ..Default::default()
    })
}

#[cfg(test)]
mod native_layout_tests {
    use super::*;
    use std::path::PathBuf;

    fn scope(c: &Context) -> DocumentScope {
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir).unwrap();
        let registry = crate::registry::ArtifactRegistry::build(&cfg).unwrap();
        DocumentScope::resolve(c, registry.descriptor("rule").unwrap()).unwrap()
    }

    fn record(c: &Context) -> crate::record::descriptor::RecordDescriptor {
        let cfg = crate::commands::load_effective_config(&c.config_path, &c.project_dir).unwrap();
        let registry = crate::registry::ArtifactRegistry::build(&cfg).unwrap();
        registry.record_descriptor("rule").cloned().unwrap()
    }

    fn archive(c: &Context, id: &str) {
        let record = record(c);
        crate::document::archive_record(c, &record, Some(id.to_string())).unwrap();
    }

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("rule-native-{}", name));
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

    fn rule_record(id: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: {title}\ncreated_date: '2026-01-01T00:00:00Z'\nupdated_date: '2026-01-01T00:00:00Z'\nlabels: []\nreferences: []\n---\n\n# {id} {title}\n\n## Description\nBody\n"
        )
    }

    #[test]
    fn legacy_slugged_name_resolves_and_archive() {
        let dir = tmpdir("slugged");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("docs/business-rules")).unwrap();
        let active = dir.join("docs/business-rules/RUL-016-legacy-rule.md");
        fs::write(&active, rule_record("RUL-016", "Legacy")).unwrap();

        let scope = scope(&c);
        let record = record(&c);
        let viewed = view(&c, &scope, &record, "RUL-16").unwrap();
        assert_eq!(viewed.json.unwrap()["id"], "RUL-016");

        archive(&c, "RUL-016");
        let archived = dir.join("docs/business-rules/archived/RUL-016-legacy-rule.md");
        assert!(archived.exists(), "archive must keep the file name");
        assert!(!active.exists());
    }

    #[test]
    fn stem_and_frontmatter_mismatch_refuses_writes() {
        let dir = tmpdir("mismatch");
        let c = ctx(&dir);
        fs::create_dir_all(dir.join("docs/business-rules")).unwrap();
        let path = dir.join("docs/business-rules/RUL-020.md");
        let before = rule_record("RUL-021", "Mismatch");
        fs::write(&path, &before).unwrap();

        archive(&c, "RUL-020");
        assert!(dir.join("docs/business-rules/archived/RUL-020.md").exists());
    }
}
