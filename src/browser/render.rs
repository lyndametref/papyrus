//! Server-rendered HTML for the local browser increment.
//!
//! Templates are embedded text files using a fixed `{{name}}` placeholder
//! syntax. Every user-controlled value is escaped at render time; the only raw
//! placeholders are internal fragments whose values are escaped as they are
//! built.

use crate::browser::records::Category;
use crate::browser::records::{FieldChange, Form, Index, Record};
use crate::config::Config;

pub const LIST_TEMPLATE: &str = include_str!("templates/list.html");
pub const VIEW_TEMPLATE: &str = include_str!("templates/view.html");
pub const EDIT_TEMPLATE: &str = include_str!("templates/edit.html");
pub const ERROR_TEMPLATE: &str = include_str!("templates/error.html");

/// Escape a value for HTML text and double-quoted attribute contexts.
pub fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn render_template(template: &str, escaped: &[(&str, &str)], raw: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 512);
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let name = &after[..end];
            if let Some((_, value)) = escaped.iter().find(|(key, _)| *key == name) {
                out.push_str(&escape(value));
            } else if let Some((_, value)) = raw.iter().find(|(key, _)| *key == name) {
                out.push_str(value);
            } else {
                out.push_str("{{");
                out.push_str(name);
                out.push_str("}}");
            }
            rest = &after[end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

/// Header and footer chrome shared by every page.
#[derive(Clone, Copy)]
pub struct Chrome<'a> {
    pub project: &'a str,
    pub bind_note: &'a str,
}

fn category_name(kind: Category) -> &'static str {
    match kind {
        Category::Req => "Business requirements",
        Category::Nfr => "Non-functional requirements",
        Category::Task => "Tasks",
        Category::Milestone => "Milestones",
        Category::Release => "Releases",
        Category::Decision => "Decisions",
        Category::Glossary => "Glossary",
        Category::UiSpec => "UI Specifications",
        Category::UiMockup => "UI Mockups",
        Category::BusinessRule => "Business Rules",
        Category::AuditRecord => "Audit Records",
        Category::TechSpec => "Technical Specifications",
    }
}

fn category_short(kind: Category) -> &'static str {
    match kind {
        Category::Req => "business requirements",
        Category::Nfr => "non-functional requirements",
        Category::Task => "tasks",
        Category::Milestone => "milestones",
        Category::Release => "releases",
        Category::Decision => "decisions",
        Category::Glossary => "glossary terms",
        Category::UiSpec => "UI specifications",
        Category::UiMockup => "UI mockups",
        Category::BusinessRule => "business rules",
        Category::AuditRecord => "audit records",
        Category::TechSpec => "technical specifications",
    }
}

fn first_paragraph(text: &str) -> String {
    let mut buf = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !buf.is_empty() {
                break;
            }
        } else {
            buf.push(line);
        }
    }
    buf.join("\n").trim().to_string()
}

fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {}", singular)
    } else {
        format!("{} {}", count, plural)
    }
}

/// Sidebar fragment. Only enabled document types are listed.
pub fn sidebar(selected: Option<Category>, cfg: &Config) -> String {
    let mut items = String::new();
    // Three sidebar entries: Business requirements, Non-functional requirements, Backlog.
    // The Backlog entry covers task, milestone, and release records.
    let backlog_selected = matches!(
        selected,
        Some(Category::Task | Category::Milestone | Category::Release)
    );
    // Business requirements
    let req_enabled = cfg
        .document_types
        .get(Category::Req.key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if req_enabled {
        let current = if selected == Some(Category::Req) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=req\"{}>{}</a></li>",
            current,
            escape(category_name(Category::Req))
        ));
    }
    // Business Rules (omitted when the document type is disabled)
    let rule_enabled = cfg
        .document_types
        .get(Category::BusinessRule.config_key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if rule_enabled {
        let current = if selected == Some(Category::BusinessRule) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=business-rule\"{}>Business Rules</a></li>",
            current
        ));
    }

    // Audit Records (omitted when the document type is disabled)
    let audit_enabled = cfg
        .document_types
        .get(Category::AuditRecord.config_key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if audit_enabled {
        let current = if selected == Some(Category::AuditRecord) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=audit-record\"{}>Audit Records</a></li>",
            current
        ));
    }
    // Technical Specifications (omitted when the document type is disabled)
    let tech_spec_enabled = cfg
        .document_types
        .get(Category::TechSpec.config_key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if tech_spec_enabled {
        let current = if selected == Some(Category::TechSpec) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=tech-spec\"{}>Technical Specifications</a></li>",
            current
        ));
    }
    // Non-functional requirements
    let nfr_enabled = cfg
        .document_types
        .get(Category::Nfr.key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if nfr_enabled {
        let current = if selected == Some(Category::Nfr) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=nfr\"{}>{}</a></li>",
            current,
            escape(category_name(Category::Nfr))
        ));
    }
    // Backlog (always available)
    let current = if backlog_selected {
        " aria-current=\"page\""
    } else {
        ""
    };
    items.push_str(&format!(
        "<li><a href=\"/?type=task\"{}>Backlog</a></li>",
        current
    ));
    // Decisions (always available)
    let current = if selected == Some(Category::Decision) {
        " aria-current=\"page\""
    } else {
        ""
    };
    items.push_str(&format!(
        "<li><a href=\"/?type=decision\"{}>Decisions</a></li>",
        current
    ));
    // Glossary (always available)
    let current = if selected == Some(Category::Glossary) {
        " aria-current=\"page\""
    } else {
        ""
    };
    items.push_str(&format!(
        "<li><a href=\"/?type=glossary\"{}>Glossary</a></li>",
        current
    ));
    // UI Specifications (always available)
    let current = if selected == Some(Category::UiSpec) {
        " aria-current=\"page\""
    } else {
        ""
    };
    items.push_str(&format!(
        "<li><a href=\"/?type=ui-spec\"{}>UI Specifications</a></li>",
        current
    ));
    // UI Mockups (omitted when the document type is disabled)
    let mockup_enabled = cfg
        .document_types
        .get(Category::UiMockup.key())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if mockup_enabled {
        let current = if selected == Some(Category::UiMockup) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type=ui-mockup\"{}>UI Mockups</a></li>",
            current
        ));
    }
    format!(
        "<aside class=\"sidebar\">\
<button class=\"sidebar-toggle\" type=\"button\" aria-expanded=\"true\" aria-controls=\"type-list\">\
<span>Document types</span> <span aria-hidden=\"true\">&#9662;</span></button>\
<nav aria-label=\"Document types\">\
<p class=\"sidebar-label\" aria-hidden=\"true\">Document types</p>\
<ul id=\"type-list\">{}</ul>\
</nav>\
</aside>",
        items
    )
}

fn breadcrumb(
    record: &Record,
    _resolved_group: Option<&Record>,
    _related_milestone: Option<&Record>,
    _related_release: Option<&Record>,
) -> String {
    let (label, type_key) = match record.kind {
        Category::Req | Category::Nfr => ("Requirements", record.kind.key()),
        Category::Task => ("Tasks", "task"),
        Category::Milestone => ("Milestones", "milestone"),
        Category::Release => ("Releases", "release"),
        Category::Decision => ("Decisions", "decision"),
        Category::Glossary => ("Glossary", "glossary"),
        Category::UiSpec => ("UI Specifications", "ui-spec"),
        Category::UiMockup => ("UI Mockups", "ui-mockup"),
        Category::BusinessRule => ("Business Rules", "business-rule"),
        Category::AuditRecord => ("Audit Records", "audit-record"),
        Category::TechSpec => ("Technical Specifications", "tech-spec"),
    };
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">{}</a></li>",
        type_key, label
    ));
    items.push_str(&format!(
        "<li><span aria-current=\"page\">{}</span></li>",
        escape(&record.id)
    ));
    format!(
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol>{}</ol></nav>",
        items
    )
}

fn edit_breadcrumb(record: &Record, _resolved_group: Option<&Record>) -> String {
    let (label, type_key) = match record.kind {
        Category::Req | Category::Nfr => ("Requirements", record.kind.key()),
        Category::Task => ("Tasks", "task"),
        Category::Milestone => ("Milestones", "milestone"),
        Category::Release => ("Releases", "release"),
        Category::Decision => ("Decisions", "decision"),
        Category::Glossary => ("Glossary", "glossary"),
        Category::UiSpec => ("UI Specifications", "ui-spec"),
        Category::UiMockup => ("UI Mockups", "ui-mockup"),
        Category::BusinessRule => ("Business Rules", "business-rule"),
        Category::AuditRecord => ("Audit Records", "audit-record"),
        Category::TechSpec => ("Technical Specifications", "tech-spec"),
    };
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">{}</a></li>",
        type_key, label
    ));
    items.push_str(&format!(
        "<li><a href=\"/doc/{}\">{}</a></li>",
        escape(&record.id),
        escape(&record.id)
    ));
    items.push_str("<li><span aria-current=\"page\">Edit</span></li>");
    format!(
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol>{}</ol></nav>",
        items
    )
}

fn owning_group_dd(record: &Record, resolved_group: Option<&Record>) -> String {
    match record.form {
        Form::Group => "<dd class=\"link-muted\">None (group record)</dd>".to_string(),
        Form::Requirement => match (record.group_id.as_deref(), resolved_group) {
            (Some(id), Some(group)) => format!(
                "<dd><a href=\"/doc/{}\">{} {}</a></dd>",
                escape(&group.id),
                escape(id),
                escape(&group.title)
            ),
            (Some(id), None) => {
                let stored = match &record.group_title {
                    Some(title) => format!("{} {}", id, title),
                    None => id.to_string(),
                };
                format!(
                    "<dd>{} <span class=\"unresolved\">(not an active group)</span></dd>",
                    escape(&stored)
                )
            }
            (None, _) => "<dd class=\"link-muted\">None</dd>".to_string(),
        },
        Form::Task
        | Form::Milestone
        | Form::Release
        | Form::Decision
        | Form::Glossary
        | Form::UiSpec
        | Form::UiMockup
        | Form::BusinessRule
        | Form::AuditRecord
        | Form::TechSpec => "<dd class=\"link-muted\">(none)</dd>".to_string(),
    }
}

fn record_view(record: &Record) -> Option<crate::record::view::RecordView> {
    crate::record::view::RecordView::from_content(&record.doc.source, &record.id, &record.version)
        .ok()
        .filter(|view| !view.metadata.is_empty())
}

fn metadata_dt(key: &str) -> String {
    match key {
        "title" => "Title".to_string(),
        "created_date" => "Created date".to_string(),
        "updated_date" => "Updated date".to_string(),
        "labels" => "Labels".to_string(),
        "references" => "References".to_string(),
        other => other.to_string(),
    }
}

/// Extra detail-page metadata rows rendered from the shared
/// [`crate::record::view::RecordView`] (TASK-67).
fn record_view_extra_rows(record: &Record, skip: &[&str]) -> String {
    let Some(view) = record_view(record) else {
        return String::new();
    };
    let mut rows = String::new();
    for (key, _) in &view.metadata {
        if key == "id" || skip.contains(&key.as_str()) {
            continue;
        }
        let value = view.value_text(key);
        let value = if value.is_empty() {
            "(none)"
        } else {
            value.as_str()
        };
        rows.push_str(&format!(
            "<div><dt>{}</dt><dd>{}</dd></div>",
            escape(&metadata_dt(key)),
            escape(value)
        ));
    }
    rows
}

fn with_extra_rows(block: String, record: &Record, skip: &[&str]) -> String {
    let extra = record_view_extra_rows(record, skip);
    if extra.is_empty() {
        return block;
    }
    block.replacen("</dl></div>", &format!("{}</dl></div>", extra), 1)
}

/// List-row metadata line rendered from the shared `RecordView` (TASK-67).
fn record_view_list_meta(record: &Record, skip: &[&str]) -> String {
    let Some(view) = record_view(record) else {
        return String::new();
    };
    let mut parts = vec![
        format!("created {}", view.value_text("created_date")),
        format!("updated {}", view.value_text("updated_date")),
        format!("labels {}", view.list_value("labels").join(",")),
        format!("references {}", view.list_value("references").join(",")),
    ];
    for (key, _) in &view.metadata {
        if [
            "id",
            "title",
            "created_date",
            "updated_date",
            "labels",
            "references",
        ]
        .contains(&key.as_str())
            || skip.contains(&key.as_str())
        {
            continue;
        }
        parts.push(format!("{}={}", key, view.value_text(key)));
    }
    format!(
        "<span class=\"row-meta\">{}</span>",
        escape(&parts.join(" \u{b7} "))
    )
}

/// Read-only `created_date` / `updated_date` fields for edit forms (TASK-67).
/// The inputs are disabled so browsers never submit them; browser saves
/// ignore these fields even when a client sends them.
fn read_only_timestamps(record: &Record) -> String {
    let view = record_view(record);
    let created = view
        .as_ref()
        .and_then(|v| v.str_value("created_date"))
        .unwrap_or_default();
    let updated = view
        .as_ref()
        .and_then(|v| v.str_value("updated_date"))
        .unwrap_or_default();
    format!(
        "<div class=\"field\"><label for=\"created_date\">Created date (read-only)</label><input class=\"input\" id=\"created_date\" name=\"created_date\" type=\"text\" value=\"{}\" disabled readonly><p class=\"help\">(read-only)</p></div><div class=\"field\"><label for=\"updated_date\">Updated date (read-only)</label><input class=\"input\" id=\"updated_date\" name=\"updated_date\" type=\"text\" value=\"{}\" disabled readonly><p class=\"help\">(read-only)</p></div>",
        escape(&created),
        escape(&updated)
    )
}

fn metadata_block(record: &Record, resolved_group: Option<&Record>, group_id_only: bool) -> String {
    let owning = if group_id_only {
        match record.form {
            Form::Group => "<dd class=\"link-muted\">None (group record)</dd>".to_string(),
            Form::Requirement => match (record.group_id.as_deref(), resolved_group) {
                (Some(id), Some(group)) => format!(
                    "<dd><a href=\"/doc/{}\">{}</a></dd>",
                    escape(&group.id),
                    escape(id)
                ),
                (Some(id), None) => format!(
                    "<dd>{} <span class=\"unresolved\">(not an active group)</span></dd>",
                    escape(id)
                ),
                (None, _) => "<dd class=\"link-muted\">None</dd>".to_string(),
            },
            Form::Task
            | Form::Milestone
            | Form::Release
            | Form::Decision
            | Form::Glossary
            | Form::UiSpec
            | Form::UiMockup
            | Form::BusinessRule
            | Form::AuditRecord
            | Form::TechSpec => "<dd class=\"link-muted\">(none)</dd>".to_string(),
        }
    } else {
        owning_group_dd(record, resolved_group)
    };
    let block = format!(
        "<div class=\"meta\"><dl class=\"dl\">\
<div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div>\
<div><dt>Record form</dt><dd>{}</dd></div>\
<div><dt>Document type</dt><dd>{}</dd></div>\
<div><dt>Owning group</dt>{}</div>\
</dl></div>",
        escape(&record.id),
        escape(record.form.label()),
        escape(&record.type_label()),
        owning
    );
    with_extra_rows(block, record, &[])
}

fn requirement_link(record: &Record) -> String {
    format!(
        "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a>{}</li>",
        escape(&record.id),
        escape(&record.id),
        escape(&record.title),
        record_view_list_meta(record, &[])
    )
}

fn member_list(members: &[Record]) -> String {
    if members.is_empty() {
        return "<div class=\"empty\"><p>No requirements in this group.</p></div>".to_string();
    }
    let mut items = String::new();
    for member in members {
        items.push_str(&requirement_link(member));
    }
    format!("<ul class=\"req-list\">{}</ul>", items)
}

fn list_section(kind: Category, index: &Index, other_has_records: bool) -> String {
    let total = index.records.len();
    let count = if total == 0 {
        "No groups".to_string()
    } else {
        format!(
            "{} \u{b7} {}",
            plural(index.groups.len(), "group", "groups"),
            plural(index.requirements.len(), "requirement", "requirements")
        )
    };

    let mut body = String::new();
    body.push_str(&format!(
        "<section class=\"section\" aria-label=\"{}\">\
<div class=\"section-head\"><h2 class=\"h2\">{}</h2><p class=\"count\">{}</p></div>",
        escape(category_name(kind)),
        escape(category_name(kind)),
        escape(&count)
    ));

    if total == 0 {
        let message = if other_has_records {
            format!("No {} in this project.", category_short(kind))
        } else {
            "No requirement records were found in the configured document directories.".to_string()
        };
        let other_name = match kind {
            Category::Req => "Non-functional requirements",
            Category::Nfr => "Business requirements",
            _ => "Other records",
        };
        let help = if other_has_records {
            format!(
                "{} are available in Document types. Records are created from the command line; this browser does not create records.",
                other_name
            )
        } else {
            "Records are created from the command line; this browser does not create records."
                .to_string()
        };
        body.push_str(&format!(
            "<div class=\"empty\"><p>{}</p><p class=\"help\">{}</p></div>",
            escape(&message),
            escape(&help)
        ));
        body.push_str("</section>");
        return body;
    }

    for (position, group) in index.groups.iter().enumerate() {
        let members: Vec<Record> = index
            .members
            .get(&group.id)
            .map(|positions| {
                positions
                    .iter()
                    .filter_map(|position| index.requirements.get(*position).cloned())
                    .collect()
            })
            .unwrap_or_default();
        let member_count = if members.is_empty() {
            "No requirements".to_string()
        } else {
            plural(members.len(), "requirement", "requirements")
        };
        body.push_str(&format!(
            "<article class=\"group\">\
<div class=\"group-head\"><h3 class=\"group-title\"><a href=\"/doc/{}\"><span class=\"group-id\">{}</span> {}</a></h3>\
<span class=\"group-count\">{}</span></div>",
            escape(&group.id),
            escape(&group.id),
            escape(&group.title),
            escape(&member_count)
        ));
        if members.is_empty() {
            body.push_str("<p class=\"empty-group\">No requirements in this group</p>");
        } else {
            let mut items = String::new();
            for member in &members {
                items.push_str(&requirement_link(member));
            }
            body.push_str(&format!("<ul class=\"req-list\">{}</ul>", items));
        }
        body.push_str("</article>");
        let _ = position;
    }

    if !index.other.is_empty() {
        body.push_str(&format!(
            "<section class=\"other\" aria-label=\"Other {}\">\
<h3 class=\"h3\">Other requirements</h3>\
<p class=\"other-note\">Group reference does not match an active group</p>\
<ul class=\"other-list\">",
            escape(category_name(kind))
        ));
        for position in &index.other {
            if let Some(requirement) = index.requirements.get(*position) {
                let reference = requirement
                    .group_id
                    .as_deref()
                    .map(|id| format!("Group: {} (not found)", id))
                    .unwrap_or_else(|| "Group: (none)".to_string());
                body.push_str(&format!(
                    "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a>\
<span class=\"other-ref\">{}</span></li>",
                    escape(&requirement.id),
                    escape(&requirement.id),
                    escape(&requirement.title),
                    escape(&reference)
                ));
            }
        }
        body.push_str("</ul></section>");
    }

    body.push_str("</section>");
    body
}

/// Render the category list page.
fn backlog_list_section(kind: Category, index: &Index) -> String {
    let total = index.records.len();
    // MOCK-0009's empty list still shows a count ("0 technical specifications"),
    // unlike the other categories' "No records" summary.
    let count = if total == 0 && kind != Category::TechSpec {
        "No records".to_string()
    } else {
        let singular = match kind {
            Category::Task => "task",
            Category::Milestone => "milestone",
            Category::Release => "release",
            Category::Decision => "decision",
            Category::Glossary => "glossary term",
            Category::UiSpec => "UI specification",
            Category::UiMockup => "UI mockup",
            Category::BusinessRule => "business rule",
            Category::AuditRecord => "audit record",
            Category::TechSpec => "technical specification",
            _ => "record",
        };
        let plural_word = format!("{}s", singular);
        plural(total, singular, &plural_word)
    };
    let mut body = String::new();
    body.push_str(&format!(
        "<section class=\"section\" aria-label=\"{}\"><div class=\"section-head\"><h2 class=\"h2\">{}</h2><p class=\"count\">{}</p></div>",
        escape(category_name(kind)),
        escape(category_name(kind)),
        escape(&count)
    ));
    if total == 0 {
        if kind == Category::TechSpec {
            body.push_str(
                "<div class=\"empty\"><p>No technical specifications were found in the configured directory.</p><p class=\"help\">Technical specifications are created from the command line; this browser does not create records. Deprecated technical specifications are hidden.</p></div></section>",
            );
            return body;
        }
        let message = format!("No {} in this project.", category_short(kind));
        body.push_str(&format!(
            "<div class=\"empty\"><p>{}</p><p class=\"help\">Records are created from the command line; this browser does not create records.</p></div>",
            escape(&message)
        ));
        body.push_str("</section>");
        return body;
    }
    body.push_str("<ul class=\"req-list\">");
    for record in &index.records {
        let meta = match kind {
            Category::Task => {
                let mut parts = Vec::new();
                if let Some(s) = &record.status {
                    parts.push(s.clone());
                }
                if let Some(m) = &record.milestone_id {
                    parts.push(m.clone());
                }
                if let Some(a) = &record.assignee {
                    parts.push(a.clone());
                }
                parts.join(" \u{b7} ")
            }
            Category::Milestone => record.release_id.clone().unwrap_or_default(),
            Category::Release => String::new(),
            Category::Decision => {
                let mut parts = Vec::new();
                if let Some(s) = &record.status {
                    parts.push(s.clone());
                }
                if let Some(t) = &record.decision_type {
                    parts.push(t.clone());
                }
                parts.join(" \u{b7} ")
            }
            Category::Glossary => String::new(),
            Category::UiSpec => String::new(),
            Category::UiMockup => String::new(),
            Category::BusinessRule => String::new(),
            Category::AuditRecord => String::new(),
            _ => String::new(),
        };
        if kind == Category::Glossary
            || kind == Category::UiSpec
            || kind == Category::BusinessRule
            || kind == Category::AuditRecord
            || kind == Category::TechSpec
        {
            let preview = if kind == Category::UiSpec
                || kind == Category::BusinessRule
                || kind == Category::AuditRecord
                || kind == Category::TechSpec
            {
                record
                    .description
                    .as_deref()
                    .map(first_paragraph)
                    .unwrap_or_default()
            } else {
                record
                    .definition
                    .as_deref()
                    .map(first_paragraph)
                    .unwrap_or_default()
            };
            if preview.is_empty() {
                body.push_str(&format!(
                    "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></li>",
                    escape(&record.id),
                    escape(&record.id),
                    escape(&record.title)
                ));
            } else {
                body.push_str(&format!(
                    "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a><p class=\"text\">{}</p></li>",
                    escape(&record.id),
                    escape(&record.id),
                    escape(&record.title),
                    escape(&preview)
                ));
            }
        } else if meta.is_empty() {
            body.push_str(&format!(
                "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></li>",
                escape(&record.id),
                escape(&record.id),
                escape(&record.title)
            ));
        } else {
            body.push_str(&format!(
                "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a> <span class=\"row-meta\">{}</span></li>",
                escape(&record.id),
                escape(&record.id),
                escape(&record.title),
                escape(&meta)
            ));
        }
        let record_meta = record_view_backlog_row_meta(record);
        if !record_meta.is_empty()
            && let Some(li_end) = body.rfind("</li>")
        {
            body.insert_str(li_end, &record_meta);
        }
    }
    body.push_str("</ul></section>");
    body
}

/// Backlog list-row metadata line from the shared `RecordView` (TASK-67).
fn record_view_backlog_row_meta(record: &Record) -> String {
    let skip: &[&str] = match record.kind {
        Category::Task => &["status", "milestone", "assignee"],
        Category::Milestone => &["release"],
        Category::Decision => &["status"],
        _ => &[],
    };
    record_view_list_meta(record, skip)
}

pub fn list_page(
    kind: Category,
    index: &Index,
    other_has_records: bool,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let section = if kind.is_requirement() {
        list_section(kind, index, other_has_records)
    } else {
        backlog_list_section(kind, index)
    };
    let heading = match kind {
        Category::Req | Category::Nfr => "Requirements",
        Category::Task => "Tasks",
        Category::Milestone => "Milestones",
        Category::Release => "Releases",
        Category::Decision => "Decisions",
        Category::Glossary => "Glossary",
        Category::UiSpec => "UI Specifications",
        Category::UiMockup => "UI Mockups",
        Category::BusinessRule => "Business Rules",
        Category::AuditRecord => "Audit Records",
        Category::TechSpec => "Technical Specifications",
    };
    let subheading = match kind {
        Category::Req => "Business requirement records in this project.",
        Category::Nfr => "Non-functional requirement records in this project.",
        Category::Task => "Task records in this project.",
        Category::Milestone => "Milestone records in this project.",
        Category::Release => "Release records in this project.",
        Category::Decision => "Decision records in this project.",
        Category::Glossary => "Glossary terms in this project.",
        Category::UiSpec => "UI specifications in this project.",
        Category::UiMockup => "UI mockups in this project.",
        Category::BusinessRule => "Business rules in this project.",
        Category::AuditRecord => "Audit records in this project.",
        Category::TechSpec => "Technical specifications in this project.",
    };
    let title = format!("Papyrus \u{2014} {}", category_name(kind));
    render_template(
        LIST_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
            ("heading", heading),
            ("subheading", subheading),
        ],
        &[
            ("sidebar", &sidebar(Some(kind), cfg)),
            ("section", &section),
        ],
    )
}

/// Render a record view page.
#[allow(clippy::too_many_arguments)]
pub fn view_page(
    record: &Record,
    resolved_group: Option<&Record>,
    members: &[Record],
    related_milestone: Option<&Record>,
    related_release: Option<&Record>,
    resolved_dependencies: &[Option<Record>],
    resolved_refs: &[Option<Record>],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut content = String::new();
    match record.kind {
        Category::Req | Category::Nfr => {
            if record.kind.supports_story()
                && let Some(story) = &record.story
            {
                content.push_str(&format!(
                    "<section class=\"record-section\"><h2>User story</h2><p class=\"text\">{}</p></section>",
                    escape(story)
                ));
            }
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">No description.</p>"),
            }
            content.push_str("</section>");
            if record.form == Form::Group {
                content.push_str(&format!(
                    "<section class=\"record-section\"><h2>Requirements in this group</h2>{}</section>",
                    member_list(members)
                ));
            }
        }
        Category::Task => {
            // Status
            content.push_str(&format!(
                "<section class=\"record-section\"><h2>Status</h2><p class=\"text\">{}</p></section>",
                escape(record.status.as_deref().unwrap_or("(none)"))
            ));
            // Milestone
            let ms_html = match (&record.milestone_id, related_milestone) {
                (Some(id), Some(rel)) => format!(
                    "<a href=\"/doc/{}\">{} {}</a>",
                    escape(&rel.id),
                    escape(id),
                    escape(&rel.title)
                ),
                (Some(id), None) => escape(id),
                _ => "(none)".to_string(),
            };
            content.push_str(&format!(
                "<section class=\"record-section\"><h2>Milestone</h2><p class=\"text\">{}</p></section>",
                ms_html
            ));
            // Assignee
            content.push_str(&format!(
                "<section class=\"record-section\"><h2>Assignee</h2><p class=\"text\">{}</p></section>",
                escape(record.assignee.as_deref().unwrap_or("(none)"))
            ));
            // Labels
            content.push_str("<section class=\"record-section\"><h2>Labels</h2>");
            if record.labels.is_empty() {
                content.push_str("<p class=\"text empty-text\">No labels.</p>");
            } else {
                content.push_str("<ul class=\"req-list\">");
                for l in &record.labels {
                    content.push_str(&format!("<li>{}</li>", escape(l)));
                }
                content.push_str("</ul>");
            }
            content.push_str("</section>");
            // Dependencies
            content.push_str("<section class=\"record-section\"><h2>Dependencies</h2>");
            if record.dependencies.is_empty() {
                content.push_str("<p class=\"text empty-text\">No dependencies.</p>");
            } else {
                content.push_str("<ul class=\"req-list\">");
                for (i, dep) in record.dependencies.iter().enumerate() {
                    let resolved = resolved_dependencies.get(i).and_then(|r| r.as_ref());
                    match resolved {
                        Some(rel) => content.push_str(&format!(
                            "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></li>",
                            escape(&rel.id),
                            escape(dep),
                            escape(&rel.title)
                        )),
                        None => content.push_str(&format!("<li>{}</li>", escape(dep))),
                    }
                }
                content.push_str("</ul>");
            }
            content.push_str("</section>");
            // Acceptance criteria
            content.push_str("<section class=\"record-section\"><h2>Acceptance criteria</h2>");
            if record.acceptance_criteria.is_empty() {
                content.push_str("<p class=\"text empty-text\">No acceptance criteria.</p>");
            } else {
                content.push_str("<ul class=\"req-list\">");
                for ac in &record.acceptance_criteria {
                    content.push_str(&format!("<li>{}</li>", escape(ac)));
                }
                content.push_str("</ul>");
            }
            content.push_str("</section>");
            // Comments
            content.push_str("<section class=\"record-section\"><h2>Comments</h2>");
            if record.comments.is_empty() {
                content.push_str("<p class=\"text empty-text\">No comments.</p>");
            } else {
                content.push_str("<ul class=\"req-list\">");
                for (author, text) in &record.comments {
                    content.push_str(&format!("<li>{}: {}</li>", escape(author), escape(text)));
                }
                content.push_str("</ul>");
            }
            content.push_str("</section>");
            // Description
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Category::Milestone => {
            let rel_html = match (&record.release_id, related_release) {
                (Some(id), Some(rel)) => format!(
                    "<a href=\"/doc/{}\">{} {}</a>",
                    escape(&rel.id),
                    escape(id),
                    escape(&rel.title)
                ),
                (Some(id), None) => escape(id),
                _ => "(none)".to_string(),
            };
            content.push_str(&format!(
                "<section class=\"record-section\"><h2>Release</h2><p class=\"text\">{}</p></section>",
                rel_html
            ));
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Category::Release => {
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Category::Decision => {
            // Decision text
            content.push_str("<section class=\"record-section\"><h2>Decision</h2>");
            match &record.decision {
                Some(t) if !t.trim().is_empty() => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(t)))
                }
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Context
            content.push_str("<section class=\"record-section\"><h2>Context</h2>");
            match &record.context {
                Some(t) if !t.trim().is_empty() => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(t)))
                }
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Consequences
            content.push_str("<section class=\"record-section\"><h2>Consequences</h2>");
            match &record.consequences {
                Some(t) if !t.trim().is_empty() => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(t)))
                }
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Supersession
            content.push_str("<section class=\"record-section\"><h2>Supersession</h2>");
            if let Some(sup_id) = &record.superseded_by {
                if let Some(title) = &record.superseded_title {
                    content.push_str(&format!(
                        "<p class=\"text\">Superseded by <a href=\"/doc/{}\">{} {}</a></p>",
                        escape(sup_id),
                        escape(sup_id),
                        escape(title)
                    ));
                } else {
                    content.push_str(&format!(
                        "<p class=\"text\">Superseded by {}</p>",
                        escape(sup_id)
                    ));
                }
            } else {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            }
            content.push_str("</section>");
        }
        Category::Glossary => {
            content.push_str("<section class=\"record-section\"><h2>Definition</h2>");
            match &record.definition {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Category::UiSpec => {
            content.push_str("<section class=\"record-section\"><h2>View</h2>");
            match &record.view {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Category::UiMockup => {
            // Shared parser values loaded with the record (section 3);
            // the stored package itself is never rendered or embedded.
            let specs = record.specs.join(" ");
            let refs = record.refs.join(" ");
            content.push_str("<section class=\"record-section\"><h2>UI specifications</h2>");
            if specs.is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(&specs)
                ));
            }
            content.push_str("</section>");
            content.push_str("<section class=\"record-section\"><h2>References</h2>");
            if refs.is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(&refs)
                ));
            }
            content.push_str("</section>");
        }
        Category::BusinessRule => {
            // Description
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Rationale
            content.push_str("<section class=\"record-section\"><h2>Rationale</h2>");
            match &record.rationale {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Applicability
            content.push_str("<section class=\"record-section\"><h2>Applicability</h2>");
            match &record.applicability {
                Some(t) if !t.trim().is_empty() => content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(t)
                )),
                _ => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
            // Requirements
            content.push_str("<section class=\"record-section\"><h2>Requirements</h2>");
            if record.requirements.is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str("<ul class=\"req-list\">");
                for (i, req) in record.requirements.iter().enumerate() {
                    let resolved = resolved_refs.get(i).and_then(|r| r.as_ref());
                    match resolved {
                        Some(rel) => content.push_str(&format!(
                            "<li><a href=\"/doc/{}\">{}</a></li>",
                            escape(&rel.id),
                            escape(req)
                        )),
                        None => content.push_str(&format!("<li>{}</li>", escape(req))),
                    }
                }
                content.push_str("</ul>");
            }
            content.push_str("</section>");
        }
        Category::AuditRecord => {
            let content_text = record.description.as_deref().unwrap_or("");
            if content_text.trim().is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(content_text)
                ));
            }
        }
        Category::TechSpec => {
            // The description is body text: no "Description" heading. The
            // shared `.text` class already applies `white-space: pre-wrap`.
            let content_text = record.description.as_deref().unwrap_or("");
            if content_text.trim().is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!("<p class=\"text\">{}</p>", escape(content_text)));
            }
        }
    }

    let page_head = if record.kind == Category::AuditRecord {
        let audit_type = record.audit_type.as_deref().unwrap_or("security");
        format!(
            "<div class=\"page-head\"><div><p class=\"id\">{} <span class=\"badge\">{}</span></p><h1 class=\"h1\">{}</h1></div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
            escape(&record.id),
            escape(audit_type),
            escape(&record.title),
            escape(&record.id)
        )
    } else {
        format!(
            "<div class=\"page-head\"><div><p class=\"id\">{}</p><h1 class=\"h1\">{}</h1></div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
            escape(&record.id),
            escape(&record.title),
            escape(&record.id)
        )
    };
    let title = format!("Papyrus \u{2014} {} {}", record.id, record.title);
    let breadcrumb_html = breadcrumb(record, resolved_group, related_milestone, related_release);
    let meta_html = if record.kind.is_requirement() {
        metadata_block(record, resolved_group, false)
    } else {
        metadata_block_backlog(record, related_milestone, related_release)
    };
    render_template(
        VIEW_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &breadcrumb_html),
            ("page_head", &page_head),
            ("meta", &meta_html),
            ("content", &content),
        ],
    )
}

fn metadata_block_backlog(
    record: &Record,
    related_milestone: Option<&Record>,
    related_release: Option<&Record>,
) -> String {
    let block = metadata_block_backlog_inner(record, related_milestone, related_release);
    let skip: &[&str] = match record.kind {
        Category::Task => &["status", "milestone", "assignee"],
        Category::Milestone => &["release"],
        Category::AuditRecord => &["type"],
        _ => &[],
    };
    with_extra_rows(block, record, skip)
}

fn metadata_block_backlog_inner(
    record: &Record,
    related_milestone: Option<&Record>,
    related_release: Option<&Record>,
) -> String {
    match record.kind {
        Category::Task => {
            let ms = match (&record.milestone_id, related_milestone) {
                (Some(id), Some(rel)) => format!(
                    "<a href=\"/doc/{}\">{} {}</a>",
                    escape(&rel.id),
                    escape(id),
                    escape(&rel.title)
                ),
                (Some(id), None) => escape(id),
                _ => "(none)".to_string(),
            };
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Document type</dt><dd>{}</dd></div><div><dt>Status</dt><dd>{}</dd></div><div><dt>Milestone</dt><dd>{}</dd></div><div><dt>Assignee</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape(&record.type_label()),
                escape(record.status.as_deref().unwrap_or("(none)")),
                ms,
                escape(record.assignee.as_deref().unwrap_or("(none)")),
            )
        }
        Category::Milestone => {
            let rel = match (&record.release_id, related_release) {
                (Some(id), Some(r)) => format!(
                    "<a href=\"/doc/{}\">{} {}</a>",
                    escape(&r.id),
                    escape(id),
                    escape(&r.title)
                ),
                (Some(id), None) => escape(id),
                _ => "(none)".to_string(),
            };
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Document type</dt><dd>{}</dd></div><div><dt>Release</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape(&record.type_label()),
                rel,
            )
        }
        Category::Release => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Document type</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape(&record.type_label()),
            )
        }
        Category::Glossary => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("Glossary"),
            )
        }
        Category::UiSpec => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("UI Specifications"),
            )
        }
        Category::UiMockup => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("UI Mockups"),
            )
        }
        Category::BusinessRule => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("Business Rules"),
            )
        }
        Category::AuditRecord => {
            let audit_type = record.audit_type.as_deref().unwrap_or("security");
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div><div><dt>Audit type</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("Audit Records"),
                escape(audit_type),
            )
        }
        Category::TechSpec => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape("Technical Specifications"),
            )
        }
        Category::Decision => {
            format!(
                "<div class=\"meta\"><dl class=\"dl\"><div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div><div><dt>Record form</dt><dd>{}</dd></div><div><dt>Category</dt><dd>{}</dd></div><div><dt>Decision type</dt><dd>{}</dd></div><div><dt>Status</dt><dd>{}</dd></div></dl></div>",
                escape(&record.id),
                escape(record.form.label()),
                escape(&record.type_label()),
                escape(record.decision_type.as_deref().unwrap_or("(none)")),
                escape(record.status.as_deref().unwrap_or("(none)")),
            )
        }
        _ => String::new(),
    }
}

/// Submitted values shown on the edit form (pristine or after a 422).
#[derive(Clone, Debug, Default)]
pub struct EditValues {
    pub title: String,
    pub story: String,
    pub description: String,
}

/// Field-level validation messages.
#[derive(Clone, Debug, Default)]
pub struct FieldErrors {
    pub title: Option<String>,
    pub story: Option<String>,
    pub description: Option<String>,
}

impl FieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.story.is_some() || self.description.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct TaskEditValues {
    pub title: String,
    pub description: String,
    pub status: String,
    pub milestone: String,
    pub assignee: String,
    pub labels: String,
    pub dependencies: String,
    pub acceptance_criteria: String,
}

#[derive(Clone, Debug, Default)]
pub struct TaskFieldErrors {
    pub title: Option<String>,
    pub status: Option<String>,
    pub milestone: Option<String>,
    pub labels: Option<String>,
    pub dependencies: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub description: Option<String>,
}

impl TaskFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some()
            || self.status.is_some()
            || self.milestone.is_some()
            || self.labels.is_some()
            || self.dependencies.is_some()
            || self.acceptance_criteria.is_some()
            || self.description.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct MilestoneEditValues {
    pub title: String,
    pub description: String,
    pub release: String,
}

#[derive(Clone, Debug, Default)]
pub struct MilestoneFieldErrors {
    pub title: Option<String>,
    pub release: Option<String>,
    pub description: Option<String>,
}

impl MilestoneFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.release.is_some() || self.description.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReleaseEditValues {
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub struct ReleaseFieldErrors {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl ReleaseFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.description.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DecisionEditValues {
    pub title: String,
    pub status: String,
    pub decision: String,
    pub context: String,
    pub consequences: String,
    pub superseded_by: String,
}

#[derive(Clone, Debug, Default)]
pub struct DecisionFieldErrors {
    pub title: Option<String>,
    pub status: Option<String>,
    pub decision: Option<String>,
    pub context: Option<String>,
    pub consequences: Option<String>,
    pub superseded_by: Option<String>,
}

impl DecisionFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some()
            || self.status.is_some()
            || self.decision.is_some()
            || self.context.is_some()
            || self.consequences.is_some()
            || self.superseded_by.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct GlossaryEditValues {
    pub title: String,
    pub definition: String,
}

#[derive(Clone, Debug, Default)]
pub struct GlossaryFieldErrors {
    pub title: Option<String>,
    pub definition: Option<String>,
}

impl GlossaryFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.definition.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiSpecEditValues {
    pub title: String,
    pub view: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub struct UiSpecFieldErrors {
    pub title: Option<String>,
    pub view: Option<String>,
    pub description: Option<String>,
}

impl UiSpecFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.view.is_some() || self.description.is_some()
    }
}

fn field_error(id: &str, message: Option<&str>) -> String {
    match message {
        Some(message) => format!(
            "<p class=\"error\" id=\"{}-error\">{}</p>",
            id,
            escape(message)
        ),
        None => String::new(),
    }
}

fn invalid_attrs(id: &str, message: Option<&str>) -> String {
    match message {
        Some(_) => format!(" aria-invalid=\"true\" aria-describedby=\"{}-error\"", id),
        None => String::new(),
    }
}

/// Render the edit form, including validation recovery.
pub fn edit_page(
    record: &Record,
    resolved_group: Option<&Record>,
    values: &EditValues,
    errors: &FieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard>\
<input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));

    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, message) in [
            ("title", errors.title.as_deref()),
            ("story", errors.story.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(message) = message {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(message)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }

    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label>\
<input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}>\
<p class=\"help\">Required \u{b7} 1 to 200 characters \u{b7} shown in lists and headings.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));

    if record.kind.supports_story() {
        form.push_str(&format!(
            "<div class=\"field\"><label for=\"story\">User story</label>\
<textarea class=\"textarea\" id=\"story\" name=\"story\" rows=\"4\"{}>{}</textarea>\
<p class=\"help\">Optional \u{b7} plain text \u{b7} line breaks are preserved.</p>{}</div>",
            invalid_attrs("story", errors.story.as_deref()),
            escape(&values.story),
            field_error("story", errors.story.as_deref())
        ));
    }

    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label>\
<textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea>\
<p class=\"help\">Optional \u{b7} a line starting with ## is not allowed.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));

    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button>\
<a class=\"link\" href=\"/doc/{}\">Cancel</a>\
<span class=\"form-note\" data-form-note>Returns to the record view without saving.</span></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div>\
<p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, resolved_group)),
            ("page_head", &page_head),
            ("meta", &metadata_block(record, resolved_group, true)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_task(
    record: &Record,
    values: &TaskEditValues,
    errors: &TaskFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("status", errors.status.as_deref()),
            ("milestone", errors.milestone.as_deref()),
            ("labels", errors.labels.as_deref()),
            ("dependencies", errors.dependencies.as_deref()),
            ("acceptance_criteria", errors.acceptance_criteria.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"status\">Status <span class=\"req\">(required)</span></label><input class=\"input\" id=\"status\" name=\"status\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Must be a configured status.</p>{}</div>",
        escape(&values.status),
        invalid_attrs("status", errors.status.as_deref()),
        field_error("status", errors.status.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"milestone\">Milestone</label><input class=\"input\" id=\"milestone\" name=\"milestone\" type=\"text\" value=\"{}\"{}><p class=\"help\">Optional milestone ID.</p>{}</div>",
        escape(&values.milestone),
        invalid_attrs("milestone", errors.milestone.as_deref()),
        field_error("milestone", errors.milestone.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"assignee\">Assignee</label><input class=\"input\" id=\"assignee\" name=\"assignee\" type=\"text\" value=\"{}\"></div>",
        escape(&values.assignee)
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"labels\">Labels</label><textarea class=\"textarea\" id=\"labels\" name=\"labels\" rows=\"4\"{}>{}</textarea><p class=\"help\">One value per line.</p>{}</div>",
        invalid_attrs("labels", errors.labels.as_deref()),
        escape(&values.labels),
        field_error("labels", errors.labels.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"dependencies\">Dependencies</label><textarea class=\"textarea\" id=\"dependencies\" name=\"dependencies\" rows=\"4\"{}>{}</textarea><p class=\"help\">One ID per line.</p>{}</div>",
        invalid_attrs("dependencies", errors.dependencies.as_deref()),
        escape(&values.dependencies),
        field_error("dependencies", errors.dependencies.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"acceptance_criteria\">Acceptance criteria</label><textarea class=\"textarea\" id=\"acceptance_criteria\" name=\"acceptance_criteria\" rows=\"4\"{}>{}</textarea><p class=\"help\">One value per line.</p>{}</div>",
        invalid_attrs("acceptance_criteria", errors.acceptance_criteria.as_deref()),
        escape(&values.acceptance_criteria),
        field_error("acceptance_criteria", errors.acceptance_criteria.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional \u{b7} a line starting with ## is not allowed.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_milestone(
    record: &Record,
    values: &MilestoneEditValues,
    errors: &MilestoneFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("release", errors.release.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"release\">Release</label><input class=\"input\" id=\"release\" name=\"release\" type=\"text\" value=\"{}\"{}><p class=\"help\">Optional release ID.</p>{}</div>",
        escape(&values.release),
        invalid_attrs("release", errors.release.as_deref()),
        field_error("release", errors.release.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional \u{b7} a line starting with ## is not allowed.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));
    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_release(
    record: &Record,
    values: &ReleaseEditValues,
    errors: &ReleaseFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional \u{b7} a line starting with ## is not allowed.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));
    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_decision(
    record: &Record,
    values: &DecisionEditValues,
    errors: &DecisionFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("status", errors.status.as_deref()),
            ("decision", errors.decision.as_deref()),
            ("context", errors.context.as_deref()),
            ("consequences", errors.consequences.as_deref()),
            ("superseded-by", errors.superseded_by.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"status\">Status</label><input class=\"input\" id=\"status\" name=\"status\" type=\"text\" value=\"{}\"{}><p class=\"help\">Optional status.</p>{}</div>",
        escape(&values.status),
        invalid_attrs("status", errors.status.as_deref()),
        field_error("status", errors.status.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"decision\">Decision</label><textarea class=\"textarea\" id=\"decision\" name=\"decision\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("decision", errors.decision.as_deref()),
        escape(&values.decision),
        field_error("decision", errors.decision.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"context\">Context</label><textarea class=\"textarea\" id=\"context\" name=\"context\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("context", errors.context.as_deref()),
        escape(&values.context),
        field_error("context", errors.context.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"consequences\">Consequences</label><textarea class=\"textarea\" id=\"consequences\" name=\"consequences\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("consequences", errors.consequences.as_deref()),
        escape(&values.consequences),
        field_error("consequences", errors.consequences.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"superseded-by\">Superseded by</label><input class=\"input\" id=\"superseded-by\" name=\"superseded-by\" type=\"text\" value=\"{}\"{}><p class=\"help\">Optional decision ID.</p>{}</div>",
        escape(&values.superseded_by),
        invalid_attrs("superseded-by", errors.superseded_by.as_deref()),
        field_error("superseded-by", errors.superseded_by.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

#[derive(Clone, Debug, Default)]
pub struct UiMockupEditValues {
    pub title: String,
    pub specs: Vec<String>,
    pub refs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct UiMockupFieldErrors {
    pub title: Option<String>,
    pub specs: Option<String>,
    pub refs: Option<String>,
}

impl UiMockupFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.specs.is_some() || self.refs.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BusinessRuleEditValues {
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub applicability: String,
    pub requirements: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct BusinessRuleFieldErrors {
    pub title: Option<String>,
    pub description: Option<String>,
    pub rationale: Option<String>,
    pub applicability: Option<String>,
    pub requirements: Option<String>,
}

impl BusinessRuleFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.rationale.is_some()
            || self.applicability.is_some()
            || self.requirements.is_some()
    }
}

pub fn edit_page_glossary(
    record: &Record,
    values: &GlossaryEditValues,
    errors: &GlossaryFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("definition", errors.definition.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Term <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"definition\">Definition</label><textarea class=\"textarea\" id=\"definition\" name=\"definition\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("definition", errors.definition.as_deref()),
        escape(&values.definition),
        field_error("definition", errors.definition.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_ui_spec(
    record: &Record,
    values: &UiSpecEditValues,
    errors: &UiSpecFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("view", errors.view.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"view\">View <span class=\"req\">(required)</span></label><input class=\"input\" id=\"view\" name=\"view\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.view),
        invalid_attrs("view", errors.view.as_deref()),
        field_error("view", errors.view.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
fn tag_field(
    field_id: &str,
    add_id: &str,
    label_html: &str,
    values: &[String],
    placeholder: &str,
    help: &str,
    error: Option<&str>,
    value_noun: &str,
) -> String {
    let mut tags = String::new();
    for (index, value) in values.iter().enumerate() {
        tags.push_str(&format!(
            "<span class=\"tag\"><input class=\"tag-input\" id=\"{}-tag-{}\" name=\"{}\" type=\"text\" value=\"{}\" aria-label=\"{} {}\"><button class=\"tag-remove\" type=\"button\" aria-label=\"Remove {}\" data-remove>&times;</button></span>",
            field_id,
            index,
            field_id,
            escape(value),
            value_noun,
            index + 1,
            escape(value)
        ));
    }
    let add_input = format!(
        "<input class=\"tag-input tag-add\" id=\"{}\" name=\"{}\" type=\"text\" placeholder=\"{}\"{}>",
        add_id,
        field_id,
        escape(placeholder),
        invalid_attrs(field_id, error)
    );
    format!(
        "<div class=\"field\" id=\"{}\"><label for=\"{}\">{}</label><div class=\"tagbox\" data-tagbox=\"{}\">{}{}</div><div class=\"add-line\"><button class=\"btn\" type=\"button\" data-add=\"{}\">Add</button></div><p class=\"help\">{}</p>{}</div>",
        field_id,
        add_id,
        label_html,
        field_id,
        tags,
        add_input,
        field_id,
        escape(help),
        field_error(field_id, error)
    )
}

pub fn edit_page_ui_mockup(
    record: &Record,
    values: &UiMockupEditValues,
    errors: &UiMockupFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("specs", errors.specs.as_deref()),
            ("refs", errors.refs.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required \u{b7} 1 to 200 characters \u{b7} shown in lists and headings.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&tag_field(
        "specs",
        "specs-add",
        "UI specifications <span class=\"req\">(required)</span>",
        &values.specs,
        "Add UI specification id (e.g. UI-003)",
        "Required \u{b7} one or more UI specification ids \u{b7} added one at a time.",
        errors.specs.as_deref(),
        "UI specification",
    ));
    form.push_str(&tag_field(
        "refs",
        "refs-add",
        "References",
        &values.refs,
        "Add a reference (e.g. docs/specifications/ui-mockups-management.md)",
        "Optional \u{b7} references to other documents.",
        errors.refs.as_deref(),
        "Reference",
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn edit_page_business_rule(
    record: &Record,
    values: &BusinessRuleEditValues,
    errors: &BusinessRuleFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("description", errors.description.as_deref()),
            ("rationale", errors.rationale.as_deref()),
            ("applicability", errors.applicability.as_deref()),
            ("requirements", errors.requirements.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"rationale\">Rationale</label><textarea class=\"textarea\" id=\"rationale\" name=\"rationale\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("rationale", errors.rationale.as_deref()),
        escape(&values.rationale),
        field_error("rationale", errors.rationale.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"applicability\">Applicability</label><textarea class=\"textarea\" id=\"applicability\" name=\"applicability\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("applicability", errors.applicability.as_deref()),
        escape(&values.applicability),
        field_error("applicability", errors.applicability.as_deref())
    ));
    form.push_str(&tag_field(
        "requirements",
        "requirements-add",
        "Requirements",
        &values.requirements,
        "Add a requirement id (e.g. REQ-001.001)",
        "Optional \\u{b7} business requirement ids.",
        errors.requirements.as_deref(),
        "Requirement",
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

#[derive(Clone, Debug, Default)]
pub struct AuditEditValues {
    pub title: String,
    pub audit_type: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct AuditFieldErrors {
    pub title: Option<String>,
    pub audit_type: Option<String>,
    pub content: Option<String>,
}

impl AuditFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.audit_type.is_some() || self.content.is_some()
    }
}

pub fn edit_page_audit_record(
    record: &Record,
    values: &AuditEditValues,
    errors: &AuditFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("audit_type", errors.audit_type.as_deref()),
            ("content", errors.content.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required \u{b7} 1 to 200 characters.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    // audit type select
    let mut select = String::new();
    select.push_str(&format!(
        "<div class=\"field\"><label for=\"audit_type\">Audit type <span class=\"req\">(required)</span></label><select class=\"select\" id=\"audit_type\" name=\"audit_type\"{}>",
        invalid_attrs("audit_type", errors.audit_type.as_deref())
    ));
    for (val, label) in [
        ("security", "security"),
        ("licence", "licence"),
        ("architecture", "architecture"),
    ] {
        let selected = if values.audit_type == val {
            " selected"
        } else {
            ""
        };
        select.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            val, selected, label
        ));
    }
    // allow custom stored value not in built-ins
    if !values.audit_type.is_empty()
        && !["security", "licence", "architecture"].contains(&values.audit_type.as_str())
    {
        select.push_str(&format!(
            "<option value=\"{}\" selected>{}</option>",
            escape(&values.audit_type),
            escape(&values.audit_type)
        ));
    }
    select.push_str("</select>");
    select.push_str(&field_error("audit_type", errors.audit_type.as_deref()));
    select.push_str("</div>");
    form.push_str(&select);
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"content\">Content</label><textarea class=\"textarea\" id=\"content\" name=\"content\" rows=\"10\"{}>{}</textarea><p class=\"help\">Optional \u{b7} up to 20000 characters.</p>{}</div>",
        invalid_attrs("content", errors.content.as_deref()),
        escape(&values.content),
        field_error("content", errors.content.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn conflict_page_audit_record(
    record: &Record,
    title: &str,
    audit_type: &str,
    content: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut body = String::new();
    body.push_str(
        "<h1 class=\"h1\">This record changed on disk</h1><p class=\"sub\">Your changes were not saved. The file was modified after this form was opened.</p><div class=\"conflict-box\"><p><strong>409 Conflict \u{2014} record changed on disk; reload the form.</strong></p></div>",
    );
    body.push_str(&format!(
        "<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Reload the form</a><a class=\"link\" href=\"/doc/{}\">View current record</a></div>",
        escape(&record.id),
        escape(&record.id)
    ));
    body.push_str("<div class=\"readonly-block\"><dl>");
    body.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(title)));
    body.push_str(&format!(
        "<dt>Audit type</dt><dd>{}</dd>",
        escape(audit_type)
    ));
    body.push_str(&format!(
        "<dt>Content</dt><dd class=\"text\">{}</dd>",
        escape(content)
    ));
    body.push_str("</dl></div>");

    let title_page = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title_page),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &body),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", ""),
        ],
    )
}

#[derive(Clone, Debug, Default)]
pub struct TechSpecEditValues {
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub struct TechSpecFieldErrors {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl TechSpecFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some() || self.description.is_some()
    }
}

pub fn edit_page_tech_spec(
    record: &Record,
    values: &TechSpecEditValues,
    errors: &TechSpecFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(version)
    ));
    if errors.any() {
        form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>");
        for (field, msg) in [
            ("title", errors.title.as_deref()),
            ("description", errors.description.as_deref()),
        ] {
            if let Some(m) = msg {
                form.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    field,
                    escape(m)
                ));
            }
        }
        form.push_str("</ul><p>Nothing was saved. Your typed values are kept below.</p></div>");
    }
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title <span class=\"req\">(required)</span></label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" required value=\"{}\"{}><p class=\"help\">Required \u{b7} whitespace is trimmed \u{b7} line breaks and control characters are not allowed.</p>{}</div>",
        escape(&values.title),
        invalid_attrs("title", errors.title.as_deref()),
        field_error("title", errors.title.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description <span class=\"req\">(required)</span></label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"12\" required{}>{}</textarea><p class=\"help\">Required \u{b7} multi-line \u{b7} line breaks are preserved when saved.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&read_only_timestamps(record));
    form.push_str(
        "<p class=\"hidden-field version-note\">Hidden field: version (optimistic-concurrency token)</p>",
    );
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let page_head = format!(
        "<div class=\"page-head\"><div><h1 class=\"h1\">Edit {}</h1><p class=\"sub\">{}</p></div><p class=\"status\" data-dirty-status hidden>Unsaved changes</p></div>",
        escape(&record.id),
        escape(&record.title)
    );
    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

pub fn conflict_page_tech_spec(
    record: &Record,
    title: &str,
    description: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut body = String::new();
    body.push_str(
        "<h1 class=\"h1\">This record changed on disk</h1><p class=\"sub\">Nothing was saved. The file was modified after this form was opened.</p><div class=\"conflict-box\"><p><strong>409 Conflict \u{2014} record changed on disk; reload the form.</strong></p></div>",
    );
    body.push_str(&format!(
        "<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Reload the form</a><a class=\"link\" href=\"/doc/{}\">View current record</a></div>",
        escape(&record.id),
        escape(&record.id)
    ));
    body.push_str("<div class=\"readonly-block\"><dl>");
    body.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(title)));
    body.push_str(&format!(
        "<dt>Description</dt><dd class=\"text\">{}</dd>",
        escape(description)
    ));
    body.push_str("</dl></div>");

    let title_page = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title_page),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &body),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", ""),
        ],
    )
}

/// Error-page copy for a status code.
pub fn error_copy(status: u16) -> (&'static str, &'static str, &'static str) {
    match status {
        400 => (
            "Bad request",
            "The request could not be understood.",
            "The address or submitted data is malformed. Return to the list and try again.",
        ),
        403 => (
            "Forbidden",
            "The request origin is not allowed.",
            "Return to the list and try again from this interface.",
        ),
        404 => (
            "Record not found",
            "That record does not exist or is not available.",
            "Check the address, or return to the requirements list.",
        ),
        405 => (
            "Method not allowed",
            "That action is not available on this address.",
            "Use a link on the page, or return to the requirements list.",
        ),
        408 => (
            "Request timeout",
            "The request took too long to arrive.",
            "Return to the list and try again.",
        ),
        409 => (
            "This record changed on disk",
            "Your changes were not saved. The file was modified after this form was opened.",
            "409 Conflict \u{2014} record changed on disk; reload the form.",
        ),
        411 => (
            "Length required",
            "The request body length is missing.",
            "Return to the list and try again.",
        ),
        413 => (
            "Content too large",
            "The request body is too large.",
            "Return to the list and try again.",
        ),
        414 => (
            "URI too long",
            "The request address is too long.",
            "Return to the list and try again.",
        ),
        415 => (
            "Unsupported media type",
            "That content type is not accepted.",
            "Return to the list and try again.",
        ),
        431 => (
            "Request header fields too large",
            "The request headers are too large.",
            "Return to the list and try again.",
        ),
        501 => (
            "Not implemented",
            "That transfer encoding is not supported.",
            "Return to the list and try again.",
        ),
        503 => (
            "Service unavailable",
            "The server is busy. Try again in a moment.",
            "Return to the list and try again.",
        ),
        505 => (
            "HTTP version not supported",
            "Only HTTP/1.0 and HTTP/1.1 are supported.",
            "Return to the list and try again.",
        ),
        _ => (
            "Something went wrong",
            "The request could not be completed.",
            "Return to the list and try again.",
        ),
    }
}

/// Render a generic error page with a single recovery link.
pub fn error_page(status: u16, cfg: &Config, chrome: Chrome<'_>, actions: Option<&str>) -> String {
    let (heading, message, hint) = error_copy(status);
    let default_actions =
        "<a class=\"link\" href=\"/\"><strong>Back to requirements</strong></a>".to_string();
    let actions = actions.unwrap_or(&default_actions);
    let title = format!("Papyrus \u{2014} {}", heading);
    let status_text = status.to_string();
    render_template(
        ERROR_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
            ("status_code", &status_text),
            ("heading", heading),
            ("message", message),
            ("hint", hint),
        ],
        &[("sidebar", &sidebar(None, cfg)), ("actions", actions)],
    )
}

/// Render the technical-specifications `404` page.
///
/// The same page is returned for a deprecated and an unknown technical
/// specification so the response never reveals whether a hidden record exists.
pub fn tech_spec_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let actions =
        "<a class=\"link\" href=\"/?type=tech-spec\">Back to technical specifications</a>";
    render_template(
        ERROR_TEMPLATE,
        &[
            ("title", "Papyrus \u{2014} Record not found"),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
            ("status_code", "404"),
            ("heading", "Record not found"),
            ("message", "That record does not exist or is not available."),
            (
                "hint",
                "Deprecated technical specifications are not listed and cannot be opened. \
The same page is used for an unknown address.",
            ),
        ],
        &[
            ("sidebar", &sidebar(Some(Category::TechSpec), cfg)),
            ("actions", actions),
        ],
    )
}

/// Render the 409 conflict page with the attempted values and recovery links.
///
/// The approved MOCK-0001 `edit-conflict` view uses the edit breadcrumb and a
/// bordered conflict notice, not the generic error layout with a large status
/// code block.
pub fn conflict_page(
    record: &Record,
    values: &EditValues,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut content = String::new();
    content.push_str(
        "<h1 class=\"h1\">This record changed on disk</h1>\
<p class=\"sub\">Your changes were not saved. The file was modified after this form was opened.</p>\
<div class=\"conflict-box\"><p><strong>409 Conflict \u{2014} record changed on disk; reload the form.</strong></p>\
<p>Reloading shows the current content and a new version token. Copy anything you need first.</p></div>",
    );
    content.push_str(&format!(
        "<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Reload the form</a>\
<a class=\"link\" href=\"/doc/{}\">View current record</a>\
<a class=\"link\" href=\"/?type={}\">Back to requirements</a></div>",
        escape(&record.id),
        escape(&record.id),
        record.kind.key()
    ));
    content.push_str(
        "<div class=\"readonly-block\"><h2>Your unsaved changes (read-only, not saved)</h2><dl>",
    );
    content.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(&values.title)));
    if record.kind.supports_story() {
        content.push_str(&format!(
            "<dt>User story</dt><dd class=\"text\">{}</dd>",
            escape(&values.story)
        ));
    }
    content.push_str(&format!(
        "<dt>Description</dt><dd class=\"text\">{}</dd>",
        escape(&values.description)
    ));
    content.push_str(
        "</dl><p class=\"help\">Copy these values into the form after reloading if you want to keep them.</p></div>",
    );

    let title = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", ""),
            ("form", ""),
        ],
    )
}

/// Render the 409 conflict page for a UI mockup, showing the attempted
/// title, UI specifications, and references read-only (MOCK-0006 conflict).
pub fn conflict_page_ui_mockup(
    record: &Record,
    title: &str,
    specs: &[String],
    refs: &[String],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut content = String::new();
    content.push_str(
        "<h1 class=\"h1\">This record changed on disk</h1><p class=\"sub\">Your changes were not saved. The file was modified after this form was opened.</p><div class=\"conflict-box\"><p><strong>409 Conflict \u{2014} record changed on disk; reload the form.</strong></p><p>Reloading shows the current content and a new version token. Copy anything you need first.</p></div>",
    );
    content.push_str(&format!(
        "<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Reload the form</a><a class=\"link\" href=\"/doc/{}\">View current record</a><a class=\"link\" href=\"/?type={}\">Back to requirements</a></div>",
        escape(&record.id),
        escape(&record.id),
        record.kind.key()
    ));
    content.push_str(
        "<div class=\"readonly-block\"><h2>Your unsaved changes (read-only, not saved)</h2><dl>",
    );
    content.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(title)));
    content.push_str(&format!(
        "<dt>UI specifications</dt><dd class=\"text\">{}</dd>",
        escape(&specs.join(" "))
    ));
    content.push_str(&format!(
        "<dt>References</dt><dd class=\"text\">{}</dd>",
        escape(&refs.join(" "))
    ));
    content.push_str(
        "</dl><p class=\"help\">Copy these values into the form after reloading if you want to keep them.</p></div>",
    );

    let title_text = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title_text),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", ""),
            ("form", ""),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn conflict_page_business_rule(
    record: &Record,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
    requirements: &[String],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut form = String::new();
    form.push_str(&format!(
        "<form class=\"form\" method=\"post\" action=\"/doc/{}\" data-dirty-guard><input type=\"hidden\" name=\"version\" value=\"{}\">",
        escape(&record.id),
        escape(&record.version)
    ));
    form.push_str("<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><p>record changed on disk; reload the form</p></div>");
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"title\">Title</label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" value=\"{}\"></div>",
        escape(title)
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"6\">{}</textarea></div>",
        escape(description)
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"rationale\">Rationale</label><textarea class=\"textarea\" id=\"rationale\" name=\"rationale\" rows=\"6\">{}</textarea></div>",
        escape(rationale)
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"applicability\">Applicability</label><textarea class=\"textarea\" id=\"applicability\" name=\"applicability\" rows=\"6\">{}</textarea></div>",
        escape(applicability)
    ));
    // requirements tags
    for (i, v) in requirements.iter().enumerate() {
        form.push_str(&format!(
            "<input type=\"hidden\" name=\"requirements\" value=\"{}\">",
            escape(v)
        ));
        let _ = i;
    }
    form.push_str(&format!(
        "<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{}\">Cancel</a></div></form>",
        escape(&record.id)
    ));

    let content = format!(
        "<div class=\"page-head\"><h1 class=\"h1\">Edit {}</h1></div>",
        escape(&record.id)
    );
    let title_text = format!("Papyrus \u{2014} Edit {}", record.id);
    render_template(
        EDIT_TEMPLATE,
        &[
            ("title", &title_text),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    )
}

/// Convert a validated change into a [`FieldChange`].
pub fn change(value: Option<String>) -> FieldChange {
    match value {
        Some(value) => FieldChange::Set(value),
        None => FieldChange::Clear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_all_five_characters() {
        assert_eq!(escape("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
        assert_eq!(escape("&amp;"), "&amp;amp;");
    }

    #[test]
    fn placeholder_values_are_literal() {
        let output = render_template("<p>{{name}}</p>", &[("name", "{{other}}")], &[]);
        assert_eq!(output, "<p>{{other}}</p>");
    }

    #[test]
    fn unknown_placeholders_are_left_untouched() {
        assert_eq!(render_template("a{{unknown}}b", &[], &[]), "a{{unknown}}b");
    }
}
