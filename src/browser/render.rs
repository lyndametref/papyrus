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
    }
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
        Form::Task | Form::Milestone | Form::Release | Form::Decision => {
            "<dd class=\"link-muted\">(none)</dd>".to_string()
        }
    }
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
            Form::Task | Form::Milestone | Form::Release | Form::Decision => {
                "<dd class=\"link-muted\">(none)</dd>".to_string()
            }
        }
    } else {
        owning_group_dd(record, resolved_group)
    };
    format!(
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
    )
}

fn requirement_link(record: &Record) -> String {
    format!(
        "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></li>",
        escape(&record.id),
        escape(&record.id),
        escape(&record.title)
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
    let count = if total == 0 {
        "No records".to_string()
    } else {
        let singular = match kind {
            Category::Task => "task",
            Category::Milestone => "milestone",
            Category::Release => "release",
            Category::Decision => "decision",
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
            _ => String::new(),
        };
        if meta.is_empty() {
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
    }
    body.push_str("</ul></section>");
    body
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
    };
    let subheading = match kind {
        Category::Req => "Business requirement records in this project.",
        Category::Nfr => "Non-functional requirement records in this project.",
        Category::Task => "Task records in this project.",
        Category::Milestone => "Milestone records in this project.",
        Category::Release => "Release records in this project.",
        Category::Decision => "Decision records in this project.",
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
    }

    let page_head = format!(
        "<div class=\"page-head\"><div><p class=\"id\">{}</p><h1 class=\"h1\">{}</h1></div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
        escape(&record.id),
        escape(&record.title),
        escape(&record.id)
    );
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
