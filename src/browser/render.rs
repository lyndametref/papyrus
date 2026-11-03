//! Server-rendered HTML for the local browser increment.
//!
//! Templates are embedded text files using a fixed `{{name}}` placeholder
//! syntax. Every user-controlled value is escaped at render time; the only raw
//! placeholders are internal fragments whose values are escaped as they are
//! built.

use crate::browser::records::{FieldChange, Form, Index, Record};
use crate::config::Config;
use crate::requirement::Kind;

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

fn category_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Req => "Business requirements",
        Kind::Nfr => "Non-functional requirements",
    }
}

fn category_short(kind: Kind) -> &'static str {
    match kind {
        Kind::Req => "business requirements",
        Kind::Nfr => "non-functional requirements",
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
pub fn sidebar(selected: Option<Kind>, cfg: &Config) -> String {
    let mut items = String::new();
    for kind in [Kind::Req, Kind::Nfr] {
        let enabled = cfg
            .document_types
            .get(kind.key())
            .map(|entry| entry.enabled)
            .unwrap_or(false);
        if !enabled {
            continue;
        }
        let current = if selected == Some(kind) {
            " aria-current=\"page\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"/?type={}\"{}>{}</a></li>",
            kind.key(),
            current,
            escape(category_name(kind))
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

fn breadcrumb(record: &Record, resolved_group: Option<&Record>) -> String {
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">Requirements</a></li>",
        record.kind.key()
    ));
    if let Some(group) = resolved_group {
        items.push_str(&format!(
            "<li><a href=\"/doc/{}\">{}</a></li>",
            escape(&group.id),
            escape(&group.id)
        ));
    }
    items.push_str(&format!(
        "<li><span aria-current=\"page\">{}</span></li>",
        escape(&record.id)
    ));
    format!(
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol>{}</ol></nav>",
        items
    )
}

fn edit_breadcrumb(record: &Record, resolved_group: Option<&Record>) -> String {
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">Requirements</a></li>",
        record.kind.key()
    ));
    items.push_str(&format!(
        "<li><a href=\"/doc/{}\">{}</a></li>",
        escape(&record.id),
        escape(&record.id)
    ));
    if resolved_group.is_some() {
        // The owning group is shown in the metadata block; the breadcrumb stays
        // short so the current Edit crumb is always visible.
    }
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

fn list_section(kind: Kind, index: &Index, other_has_records: bool) -> String {
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
            Kind::Req => "Non-functional requirements",
            Kind::Nfr => "Business requirements",
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
pub fn list_page(
    kind: Kind,
    index: &Index,
    other_has_records: bool,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let section = list_section(kind, index, other_has_records);
    let heading = "Requirements";
    let subheading = match kind {
        Kind::Req => "Business requirement records in this project.",
        Kind::Nfr => "Non-functional requirement records in this project.",
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
pub fn view_page(
    record: &Record,
    resolved_group: Option<&Record>,
    members: &[Record],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut content = String::new();
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

    let page_head = format!(
        "<div class=\"page-head\"><div><p class=\"id\">{}</p><h1 class=\"h1\">{}</h1></div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
        escape(&record.id),
        escape(&record.title),
        escape(&record.id)
    );
    let title = format!("Papyrus \u{2014} {} {}", record.id, record.title);
    render_template(
        VIEW_TEMPLATE,
        &[
            ("title", &title),
            ("project", chrome.project),
            ("bind_note", chrome.bind_note),
        ],
        &[
            ("sidebar", &sidebar(Some(record.kind), cfg)),
            ("breadcrumb", &breadcrumb(record, resolved_group)),
            ("page_head", &page_head),
            ("meta", &metadata_block(record, resolved_group, false)),
            ("content", &content),
        ],
    )
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
        if errors.title.is_some() {
            form.push_str("<li><a href=\"#title\">Title is required.</a></li>");
        }
        if errors.story.is_some() {
            form.push_str(
                "<li><a href=\"#story\">User story contains a line starting with ## .</a></li>",
            );
        }
        if errors.description.is_some() {
            form.push_str("<li><a href=\"#description\">Description contains a line starting with ## .</a></li>");
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
pub fn conflict_page(
    record: &Record,
    values: &EditValues,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let mut actions = format!(
        "<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Reload the form</a>\
<a class=\"link\" href=\"/doc/{}\">View current record</a>\
<a class=\"link\" href=\"/?type={}\">Back to requirements</a></div>",
        escape(&record.id),
        escape(&record.id),
        record.kind.key()
    );
    actions.push_str(
        "<div class=\"readonly-block\"><h2>Your unsaved changes (read-only, not saved)</h2><dl>",
    );
    actions.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(&values.title)));
    if record.kind.supports_story() {
        actions.push_str(&format!(
            "<dt>User story</dt><dd class=\"text\">{}</dd>",
            escape(&values.story)
        ));
    }
    actions.push_str(&format!(
        "<dt>Description</dt><dd class=\"text\">{}</dd>",
        escape(&values.description)
    ));
    actions.push_str(
        "</dl><p class=\"help\">Copy these values into the form after reloading if you want to keep them.</p></div>",
    );
    error_page(409, cfg, chrome, Some(&actions))
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
