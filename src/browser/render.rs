//! Server-rendered HTML for the local browser increment.
//!
//! Templates are embedded text files using a fixed `{{name}}` placeholder
//! syntax. Every user-controlled value is escaped at render time; the only raw
//! placeholders are internal fragments whose values are escaped as they are
//! built.

use crate::browser::records::{Index, Record, filter_statuses};
use crate::browser::routes::{self, BrowserRoutes, NavGroup};
use crate::config::Config;
use crate::record::edit::FieldChange;
use crate::registry::{
    ArtifactCategory as CategoryKind, StatusModel, TypeDescriptor, TypedHandler,
};

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

pub(crate) fn render_template(
    template: &str,
    escaped: &[(&str, &str)],
    raw: &[(&str, &str)],
) -> String {
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
    pub routes: &'a BrowserRoutes,
}

/// Wrap a rendered page body in the page shell (ADR-0028 decision 1).
fn page(
    chrome: Chrome<'_>,
    slug: Option<&str>,
    title: &str,
    sidebar: &str,
    body: String,
) -> String {
    crate::browser::shell::render(crate::browser::shell::Page {
        title,
        project: chrome.project,
        bind_note: chrome.bind_note,
        sidebar,
        main: &body,
        width: chrome.routes.page_width(slug.unwrap_or_default()),
        nav_toggle: "",
    })
}

fn category_name(slug: &str) -> String {
    routes::heading(slug)
}

fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {}", singular)
    } else {
        format!("{} {}", count, plural)
    }
}

/// Sidebar fragment generated from the startup route table.
pub fn sidebar(selected: Option<&str>, _cfg: &Config, routes: &BrowserRoutes) -> String {
    build_sidebar(selected, routes, false, false, false)
}

/// Sidebar fragment with optional lifecycle-page active state.
pub fn sidebar_with_lifecycle(
    selected: Option<&str>,
    _cfg: &Config,
    lifecycle_active: bool,
    routes: &BrowserRoutes,
) -> String {
    build_sidebar(selected, routes, false, lifecycle_active, false)
}

/// Sidebar fragment with optional skills-page active state.
pub fn sidebar_with_skills(
    selected: Option<&str>,
    _cfg: &Config,
    skills_active: bool,
    routes: &BrowserRoutes,
) -> String {
    build_sidebar(selected, routes, false, false, skills_active)
}

/// Sidebar fragment with optional configuration-page active state.
pub fn sidebar_with_config(
    selected: Option<&str>,
    _cfg: &Config,
    config_active: bool,
    routes: &BrowserRoutes,
) -> String {
    build_sidebar(selected, routes, config_active, false, false)
}

fn build_sidebar(
    selected: Option<&str>,
    routes: &BrowserRoutes,
    config_active: bool,
    lifecycle_active: bool,
    skills_active: bool,
) -> String {
    let effective_selected = selected.and_then(|value| routes.nav_slug_for(value));
    let mut items = String::new();
    for group in [NavGroup::Backlog, NavGroup::Decisions, NavGroup::Documents] {
        let entries: Vec<&crate::browser::routes::NavItem> = routes
            .nav
            .iter()
            .filter(|item| item.group == group)
            .collect();
        if entries.is_empty() {
            continue;
        }
        items.push_str(&format!(
            "<li class=\"sidebar-group\"><p class=\"sidebar-label\">{}</p><ul class=\"sidebar-group-list\">",
            escape(group.heading())
        ));
        for item in entries {
            let current = if effective_selected == Some(item.slug.as_str()) {
                " aria-current=\"page\""
            } else {
                ""
            };
            items.push_str(&format!(
                "<li><a href=\"/?type={}\"{}>{}</a></li>",
                escape(&item.url_key),
                current,
                escape(&item.label)
            ));
        }
        items.push_str("</ul></li>");
    }
    let utility = [
        ("/config", "Configuration", config_active),
        (
            "/lifecycle",
            "Managed artifacts &amp; lifecycles",
            lifecycle_active,
        ),
        ("/skills", "Agent skills &amp; workflows", skills_active),
    ];
    for (href, label, active) in utility {
        let current = if active { " aria-current=\"page\"" } else { "" };
        items.push_str(&format!(
            "<li><a href=\"{}\"{}>{}</a></li>",
            href, current, label
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

fn breadcrumb_label(record: &Record) -> String {
    match record.handler() {
        Some(TypedHandler::Requirement(_)) => "Requirements".to_string(),
        Some(TypedHandler::Task) => "Tasks".to_string(),
        Some(TypedHandler::Milestone) => "Milestones".to_string(),
        Some(TypedHandler::Release) => "Releases".to_string(),
        Some(TypedHandler::Decision) => "Decisions".to_string(),
        Some(TypedHandler::Glossary) => "Glossary".to_string(),
        Some(TypedHandler::UiSpec) => "UI Specifications".to_string(),
        Some(TypedHandler::UiMockup) => "UI Mockups".to_string(),
        Some(TypedHandler::Rule) => "Business Rules".to_string(),
        Some(TypedHandler::Audit) => "Audit Records".to_string(),
        Some(TypedHandler::TechSpec) => "Technical Specifications".to_string(),
        Some(TypedHandler::UseCase) => "Use Cases".to_string(),
        Some(TypedHandler::TestCase) => "Test Cases".to_string(),
        Some(TypedHandler::TestReport) => "Test Reports".to_string(),
        None => record.type_label(),
    }
}

fn breadcrumb(
    record: &Record,
    _resolved_group: Option<&Record>,
    _related_milestone: Option<&Record>,
    _related_release: Option<&Record>,
) -> String {
    let label = breadcrumb_label(record);
    let type_key = routes::legacy_url_key(record.slug());
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">{}</a></li>",
        escape(type_key),
        escape(&label)
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
    let label = breadcrumb_label(record);
    let type_key = routes::legacy_url_key(record.slug());
    let mut items = String::new();
    items.push_str(&format!(
        "<li><a href=\"/?type={}\">{}</a></li>",
        escape(type_key),
        escape(&label)
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
    if record.is_group {
        return "<dd class=\"link-muted\">None (group record)</dd>".to_string();
    }
    if !record.is_requirement() {
        return "<dd class=\"link-muted\">(none)</dd>".to_string();
    }
    match (record.group_id.as_deref(), resolved_group) {
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
    if rows.is_empty() {
        String::new()
    } else {
        format!("<div class=\"meta\"><dl class=\"dl\">{}</dl></div>", rows)
    }
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
        if record.is_group {
            "<dd class=\"link-muted\">None (group record)</dd>".to_string()
        } else if record.is_requirement() {
            match (record.group_id.as_deref(), resolved_group) {
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
            }
        } else {
            "<dd class=\"link-muted\">(none)</dd>".to_string()
        }
    } else {
        owning_group_dd(record, resolved_group)
    };
    let view = record_view(record);
    let created = view
        .as_ref()
        .map(|v| v.value_text("created_date"))
        .unwrap_or_default();
    let created = if created.is_empty() {
        "(none)".to_string()
    } else {
        created
    };
    let updated = view
        .as_ref()
        .map(|v| v.value_text("updated_date"))
        .unwrap_or_default();
    let updated = if updated.is_empty() {
        "(none)".to_string()
    } else {
        updated
    };
    let include_owning = record.is_requirement() && !record.is_group;
    let owning_row = if include_owning {
        format!("<div><dt>Owning group</dt>{}</div>", owning)
    } else {
        String::new()
    };
    format!(
        "<div class=\"meta summary-card\"><dl class=\"dl\">\
<div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div>\
<div><dt>Created date</dt><dd>{}</dd></div>\
<div><dt>Updated date</dt><dd>{}</dd></div>\
{}\
</dl></div>",
        escape(&record.id),
        escape(&created),
        escape(&updated),
        owning_row
    )
}

fn requirement_link(record: &Record) -> String {
    format!(
        "<li>{}<a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a>{} {}</li>",
        row_status(record),
        escape(&record.id),
        escape(&record.id),
        escape(&record.title),
        row_dates(record),
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

fn form_label(record: &Record) -> String {
    if record.is_group {
        return "Group".to_string();
    }
    if record.is_requirement() {
        return "Requirement".to_string();
    }
    match record.handler() {
        Some(TypedHandler::Requirement(_)) => "Requirement".to_string(),
        Some(TypedHandler::Task) => "Task".to_string(),
        Some(TypedHandler::Milestone) => "Milestone".to_string(),
        Some(TypedHandler::Release) => "Release".to_string(),
        Some(TypedHandler::Decision) => "Decision".to_string(),
        Some(TypedHandler::Glossary) => "Glossary term".to_string(),
        Some(TypedHandler::UiSpec) => "UI specification".to_string(),
        Some(TypedHandler::UiMockup) => "UI mockup".to_string(),
        Some(TypedHandler::Rule) => "Business rule".to_string(),
        Some(TypedHandler::Audit) => "Audit record".to_string(),
        Some(TypedHandler::TechSpec) => "Technical specification".to_string(),
        Some(TypedHandler::UseCase) => "Use case".to_string(),
        Some(TypedHandler::TestCase) => "Test case".to_string(),
        Some(TypedHandler::TestReport) => "Test report".to_string(),
        None => "Document".to_string(),
    }
}

/// Read-only status cell, immediately before the ID.
fn row_status(record: &Record) -> String {
    format!(
        "<span class=\"row-status\">{}</span> ",
        escape(&record.status_text())
    )
}

/// Read-only Created and Updated cells, in that order.
fn row_dates(record: &Record) -> String {
    let view = record_view(record);
    let created = view
        .as_ref()
        .map(|v| v.value_text("created_date"))
        .unwrap_or_default();
    let created = if created.is_empty() {
        "(none)"
    } else {
        created.as_str()
    };
    let updated = view
        .as_ref()
        .map(|v| v.value_text("updated_date"))
        .unwrap_or_default();
    let updated = if updated.is_empty() {
        "(none)"
    } else {
        updated.as_str()
    };
    format!(
        " <span class=\"row-created\">{}</span> <span class=\"row-updated\">{}</span>",
        escape(created),
        escape(updated)
    )
}

/// Read-only status filter form for a document list.
fn status_filter_html(descriptor: &TypeDescriptor, selected: Option<&str>) -> String {
    let mut options = String::new();
    // A disabled, default-selected placeholder keeps an untouched form from
    // submitting `status` while still showing a prompt (REQ-07.005, AC6/AC7).
    // `Show all` is the explicit reset; there is no empty selectable option.
    if selected.is_none() {
        options.push_str("<option value=\"\" disabled selected>Status</option>");
    } else {
        options.push_str("<option value=\"\" disabled>Status</option>");
    }
    for value in filter_statuses(descriptor) {
        let current = if selected == Some(value.as_str()) {
            " selected"
        } else {
            ""
        };
        options.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            escape(&value),
            current,
            escape(&value)
        ));
    }
    format!(
        "<form class=\"filter\" method=\"get\" action=\"/\">\
<input type=\"hidden\" name=\"type\" value=\"{}\">\
<div class=\"field\"><label for=\"status\">Status</label>\
<select class=\"input\" id=\"status\" name=\"status\">{}</select></div>\
<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Apply filter</button>\
<a class=\"link\" href=\"/?type={}\">Show all</a></div></form>",
        escape(routes::legacy_url_key(&descriptor.slug)),
        options,
        escape(routes::legacy_url_key(&descriptor.slug))
    )
}

/// One failure message shown above the backlog board groups.
#[derive(Clone, Copy)]
pub struct BoardAlert<'a> {
    pub heading: &'a str,
    pub message: &'a str,
    /// Per-item failure lines rendered as a list below the message.
    pub details: &'a [String],
    /// Page subtitle shown while the alert is on screen.
    pub subheading: &'a str,
}

/// The open `Clean up all` confirmation region of one milestone board.
#[derive(Clone, Copy)]
pub struct BoardCleanup<'a> {
    pub milestone: &'a Record,
    pub status: &'a str,
    pub tasks: &'a [&'a Record],
}

fn board_milestone_for<'a>(task: &Record, milestones: &'a Index) -> Option<&'a Record> {
    task.milestone_id.as_deref().and_then(|id| {
        milestones
            .records
            .iter()
            .find(|milestone| milestone.id == id)
    })
}

fn board_card(task: &Record, cfg: &Config, expand: Option<&str>) -> String {
    let mut card = format!(
        "<article class=\"board-card\" draggable=\"true\" data-task=\"{}\">",
        escape(&task.id)
    );
    card.push_str(&format!(
        "<p class=\"board-card-title\"><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></p>",
        escape(&task.id),
        escape(&task.id),
        escape(&task.title)
    ));
    let assignee = task
        .assignee
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("(none)");
    card.push_str(&format!(
        "<p class=\"board-assignee\">Assignee: {}</p>",
        escape(assignee)
    ));
    if !task.labels.is_empty() {
        let tags = task
            .labels
            .iter()
            .map(|label| format!("<span class=\"tag\">{}</span>", escape(label)))
            .collect::<Vec<_>>()
            .join("");
        card.push_str(&format!(
            "<p class=\"board-labels\">Labels: <span class=\"taglist\">{}</span></p>",
            tags
        ));
    }
    let mut options = String::new();
    for status in &cfg.statuses {
        let selected = if task.status.as_deref() == Some(status.as_str()) {
            " selected"
        } else {
            ""
        };
        options.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            escape(status),
            selected,
            escape(status)
        ));
    }
    let expand_value = expand.unwrap_or("");
    card.push_str(&format!(
        "<form method=\"post\" action=\"/doc/{}\" class=\"card-status\">\
<input type=\"hidden\" name=\"version\" value=\"{}\">\
<input type=\"hidden\" name=\"status_change\" value=\"1\">\
<input type=\"hidden\" name=\"expand\" value=\"{}\">\
<label for=\"status-{}\">Status</label>\
<select id=\"status-{}\" name=\"status\" aria-label=\"Status for {}\">{}</select>\
<button type=\"submit\">Change status</button>\
</form>",
        escape(&task.id),
        escape(&task.version),
        escape(expand_value),
        escape(&task.id),
        escape(&task.id),
        escape(&task.id),
        options
    ));
    card.push_str("</article>");
    card
}

/// The `Clean up all` link shown in the final configured status column header
/// of a milestone board.
fn board_cleanup_action(milestone: &Record, status: &str) -> String {
    format!(
        "<p class=\"board-column-action\"><a class=\"link board-cleanup\" href=\"/?type=task&expand={id}&cleanup={id}\" aria-label=\"Clean up all {status} tasks in {id}\">Clean up all</a></p>",
        id = escape(&milestone.id),
        status = escape(status)
    )
}

/// The server-rendered confirmation region of the `Clean up all` action.
fn board_cleanup_confirm(cleanup: &BoardCleanup<'_>) -> String {
    let id = escape(&cleanup.milestone.id);
    let status = escape(cleanup.status);
    let title = escape(&cleanup.milestone.title);
    let cancel = format!(
        "<a class=\"link\" href=\"/?type=task&expand={}\">Cancel</a>",
        id
    );
    let mut html = format!(
        "<section class=\"board-confirm\" role=\"region\" aria-label=\"Clean up all {} tasks in {}\">\
<h3 class=\"h3\">Warning: archive all {} tasks in {}</h3>",
        status, id, status, id
    );
    if cleanup.tasks.is_empty() {
        html.push_str(&format!(
            "<p>Milestone <strong>{} \u{2014} {}</strong>. The <strong>{}</strong> column is empty for this milestone, so <strong>no task will be archived</strong>; confirming is not offered.</p>",
            id, title, status
        ));
        html.push_str(&format!(
            "<p class=\"board-confirm-actions\">{}</p>",
            cancel
        ));
    } else {
        let count = plural(cleanup.tasks.len(), "task", "tasks");
        html.push_str(&format!(
            "<p>Milestone <strong>{} \u{2014} {}</strong>. The <strong>{}</strong> column holds <strong>{}</strong> for this milestone; archiving removes them from this board. No other milestone and no task without a milestone is affected.</p>",
            id, title, status, count
        ));
        html.push_str("<p>Task IDs to be archived, in ascending order:</p><ul>");
        for task in cleanup.tasks {
            html.push_str(&format!(
                "<li><strong>{}</strong> \u{2014} {}</li>",
                escape(&task.id),
                escape(&task.title)
            ));
        }
        html.push_str("</ul>");
        html.push_str(&format!(
            "<form class=\"board-confirm-actions\" method=\"post\" action=\"/?type=task\">\
<input type=\"hidden\" name=\"cleanup\" value=\"1\">\
<input type=\"hidden\" name=\"milestone\" value=\"{id}\">\
<input type=\"hidden\" name=\"expand\" value=\"{id}\">\
<button type=\"submit\">Archive {count}</button>{cancel}</form>",
            id = id,
            count = count,
            cancel = cancel
        ));
    }
    html.push_str("</section>");
    html
}

fn board_columns(
    cfg: &Config,
    tasks: &[&Record],
    milestone: Option<&Record>,
    cleanup: Option<&BoardCleanup<'_>>,
) -> String {
    let final_status = cfg.statuses.last().map(String::as_str);
    let mut html = String::from("<div class=\"board-columns\">");
    for status in &cfg.statuses {
        let matching: Vec<&Record> = tasks
            .iter()
            .copied()
            .filter(|task| task.status.as_deref() == Some(status.as_str()))
            .collect();
        html.push_str(&format!(
            "<section class=\"board-column\" data-status=\"{}\"><h3 class=\"h3\">{}</h3>",
            escape(status),
            escape(status)
        ));
        // The action belongs to the final configured status column of a
        // milestone board only, never to the unconfigured column or to the
        // without-milestone section, and never when the status is `active`.
        if let Some(milestone) = milestone
            && final_status == Some(status.as_str())
            && final_status != Some("active")
            && cleanup.is_none()
        {
            html.push_str(&board_cleanup_action(milestone, status));
        }
        if matching.is_empty() {
            html.push_str("<p class=\"board-empty\">No tasks in this status.</p>");
        } else {
            for task in matching {
                html.push_str(&board_card(task, cfg, milestone.map(|m| m.id.as_str())));
            }
        }
        html.push_str("</section>");
    }
    let unconfigured: Vec<&Record> = tasks
        .iter()
        .copied()
        .filter(|task| match task.status.as_deref() {
            Some(status) => !cfg.statuses.iter().any(|configured| configured == status),
            None => true,
        })
        .collect();
    if !unconfigured.is_empty() {
        html.push_str(
            "<section class=\"board-column\" data-status=\"\"><h3 class=\"h3\">Unconfigured status</h3>",
        );
        for task in unconfigured {
            html.push_str(&board_card(task, cfg, None));
        }
        html.push_str("</section>");
    }
    html.push_str("</div>");
    html
}

fn board_milestone_section(
    milestone: &Record,
    tasks: &[&Record],
    cfg: &Config,
    open: bool,
    cleanup: Option<&BoardCleanup<'_>>,
) -> String {
    let cleanup = cleanup.filter(|cleanup| cleanup.milestone.id == milestone.id);
    let confirm = cleanup.map(board_cleanup_confirm).unwrap_or_default();
    format!(
        "<details class=\"board-milestone\"{}><summary>{} \u{2014} {}</summary>{}{}</details>",
        if open { " open" } else { "" },
        escape(&milestone.id),
        escape(&milestone.title),
        confirm,
        board_columns(cfg, tasks, Some(milestone), cleanup)
    )
}

/// The board's empty-project fallback. It is board copy, not a list: no table,
/// no Actions cell, and no status filter.
fn board_empty_section(slug: &str) -> String {
    format!(
        "<section class=\"section\" aria-label=\"{0}\">\
<div class=\"section-head\"><h2 class=\"h2\">{0}</h2><p class=\"count\">No records</p></div>\
<div class=\"empty\"><p>No {1} in this project.</p><p class=\"help\">\
Records are created from the command line; this browser does not create records.</p></div></section>",
        escape(&category_name(slug)),
        escape(&routes::short(slug)),
    )
}

/// Render the combined Backlog board at `GET /?type=task`.
#[allow(clippy::too_many_arguments)]
pub fn board_page(
    cfg: &Config,
    tasks: &Index,
    milestones: &Index,
    releases: &Index,
    expand: Option<&str>,
    alert: Option<BoardAlert<'_>>,
    cleanup: Option<&BoardCleanup<'_>>,
    chrome: Chrome<'_>,
) -> String {
    let mut section = String::new();
    if let Some(alert) = alert {
        section.push_str(&format!(
            "<div class=\"alert\" role=\"alert\"><h2>{}</h2><p>{}</p>",
            escape(alert.heading),
            escape(alert.message)
        ));
        if !alert.details.is_empty() {
            section.push_str("<ul>");
            for detail in alert.details {
                section.push_str(&format!("<li>{}</li>", escape(detail)));
            }
            section.push_str("</ul>");
        }
        section.push_str("</div>");
    }

    if tasks.is_empty() && milestones.is_empty() {
        let task_slug = chrome
            .routes
            .nav
            .iter()
            .find(|item| item.group == NavGroup::Backlog)
            .map(|item| item.slug.as_str())
            .unwrap_or_default();
        section.push_str(&board_empty_section(task_slug));
    } else {
        // The no-release group comes first, then every active release in
        // ascending ID order. A dangling release reference belongs to the
        // no-release group because it names no active release.
        let mut groups: Vec<(Option<&Record>, Vec<&Record>)> = Vec::new();
        let no_release: Vec<&Record> = milestones
            .records
            .iter()
            .filter(|milestone| match milestone.release_id.as_deref() {
                Some(release_id) => !releases
                    .records
                    .iter()
                    .any(|release| release.id == release_id),
                None => true,
            })
            .collect();
        if !no_release.is_empty() {
            groups.push((None, no_release));
        }
        for release in &releases.records {
            let group: Vec<&Record> = milestones
                .records
                .iter()
                .filter(|milestone| milestone.release_id.as_deref() == Some(release.id.as_str()))
                .collect();
            groups.push((Some(release), group));
        }

        // Exactly one milestone is open: the valid `expand` target, otherwise
        // the first milestone in render order that has at least one active task.
        let first_open: Option<String> = groups
            .iter()
            .flat_map(|(_, group)| group.iter())
            .find(|milestone| {
                tasks
                    .records
                    .iter()
                    .any(|task| task.milestone_id.as_deref() == Some(milestone.id.as_str()))
            })
            .map(|milestone| milestone.id.clone());
        let open_id: Option<&str> = expand
            .filter(|value| {
                milestones
                    .records
                    .iter()
                    .any(|milestone| milestone.id == *value)
            })
            .or(first_open.as_deref());

        for (release, group) in &groups {
            match release {
                None => section.push_str(
                    "<h2 class=\"h2 board-group-head\">Milestones without a release</h2>",
                ),
                Some(release) => section.push_str(&format!(
                    "<h2 class=\"h2 release-sep\">{} \u{2014} {}</h2>",
                    escape(&release.id),
                    escape(&release.title)
                )),
            }
            if release.is_some() && group.is_empty() {
                section.push_str("<p class=\"board-empty\">No milestones in this release.</p>");
            }
            for milestone in group {
                let milestone_tasks: Vec<&Record> = tasks
                    .records
                    .iter()
                    .filter(|task| task.milestone_id.as_deref() == Some(milestone.id.as_str()))
                    .collect();
                section.push_str(&board_milestone_section(
                    milestone,
                    &milestone_tasks,
                    cfg,
                    open_id == Some(milestone.id.as_str()),
                    cleanup,
                ));
            }
        }

        let unassigned: Vec<&Record> = tasks
            .records
            .iter()
            .filter(|task| board_milestone_for(task, milestones).is_none())
            .collect();
        if !unassigned.is_empty() {
            section.push_str(&format!(
                "<details class=\"board-milestone\" open><summary>Tasks without a milestone</summary>{}</details>",
                board_columns(cfg, &unassigned, None, None)
            ));
        }
    }

    let subheading = alert.map_or(
        "Tasks grouped by milestone and status; releases separate milestones.",
        |alert| alert.subheading,
    );
    let body = render_template(
        LIST_TEMPLATE,
        &[("heading", "Backlog"), ("subheading", subheading)],
        &[("section", &section)],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Backlog",
        &sidebar(
            chrome
                .routes
                .nav
                .iter()
                .find(|item| item.group == NavGroup::Backlog)
                .map(|item| item.slug.as_str()),
            cfg,
            chrome.routes,
        ),
        body,
    )
}

pub fn list_page(
    descriptor: &TypeDescriptor,
    index: &Index,
    status: Option<&str>,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let slug = descriptor.slug.as_str();
    let presentation_slug = chrome.routes.list_slug_for(slug);
    let heading = routes::heading(&presentation_slug);
    let subheading = routes::inventory_note(&presentation_slug);
    let count_note = if descriptor.id_shape == crate::registry::IdShape::Grouped {
        crate::browser::list::grouped_count_note(index)
    } else {
        crate::browser::list::count_note(&presentation_slug, index.records.len())
    };
    let section_body = crate::browser::list::ListPlan::for_descriptor(
        descriptor,
        &presentation_slug,
        index,
        &heading,
        &count_note,
    )
    .render();
    let section = if descriptor.category == CategoryKind::Document {
        format!(
            "<div class=\"filter-wrap\">{}</div>{}",
            status_filter_html(descriptor, status),
            section_body
        )
    } else {
        section_body
    };
    let title = format!("Papyrus \u{2014} {heading}");
    let body = render_template(
        LIST_TEMPLATE,
        &[("heading", &heading), ("subheading", &subheading)],
        &[("section", &section)],
    );
    page(
        chrome,
        Some(slug),
        &title,
        &sidebar(Some(slug), cfg, chrome.routes),
        body,
    )
}

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
    let mut details = String::new();
    match record.handler() {
        Some(TypedHandler::Requirement(_)) => {
            if record.supports_story()
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
            if record.is_group {
                content.push_str(&format!(
                    "<section class=\"record-section\"><h2>Requirements in this group</h2>{}</section>",
                    member_list(members)
                ));
            }
        }
        Some(TypedHandler::Task) => {
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
            // Dependencies (rendered at the end as a disclosure section)
            if !record.dependencies.is_empty() {
                details.push_str(&format!(
                    "<details><summary>Dependencies ({})</summary><ul class=\"req-list\">",
                    record.dependencies.len()
                ));
                for (i, dep) in record.dependencies.iter().enumerate() {
                    let resolved = resolved_dependencies.get(i).and_then(|r| r.as_ref());
                    match resolved {
                        Some(rel) => details.push_str(&format!(
                            "<li><a href=\"/doc/{}\"><span class=\"req-id\">{}</span> {}</a></li>",
                            escape(&rel.id),
                            escape(dep),
                            escape(&rel.title)
                        )),
                        None => details.push_str(&format!("<li>{}</li>", escape(dep))),
                    }
                }
                details.push_str("</ul></details>");
            }
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
        Some(TypedHandler::Milestone) => {
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
        Some(TypedHandler::Release) => {
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">(none)</p>"),
            }
            content.push_str("</section>");
        }
        Some(TypedHandler::Decision) => {
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
        Some(TypedHandler::Glossary) => {
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
        Some(TypedHandler::UiSpec) => {
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
        Some(TypedHandler::UiMockup) => {
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
            if !refs.is_empty() {
                content.push_str(&format!(
                    "<section class=\"record-section\"><h2>References</h2><p class=\"text\" style=\"white-space: pre-wrap\">{}</p></section>",
                    escape(&refs)
                ));
                details.push_str(&format!(
                    "<details><summary>References (1)</summary><p class=\"text\" style=\"white-space: pre-wrap\">{}</p></details>",
                    escape(&refs)
                ));
            }
        }
        Some(TypedHandler::Rule) => {
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
            // Requirements (rendered at the end as a disclosure section)
            if !record.requirements.is_empty() {
                details.push_str(&format!(
                    "<details><summary>Requirements ({})</summary><ul class=\"req-list\">",
                    record.requirements.len()
                ));
                for (i, req) in record.requirements.iter().enumerate() {
                    let resolved = resolved_refs.get(i).and_then(|r| r.as_ref());
                    match resolved {
                        Some(rel) => details.push_str(&format!(
                            "<li><a href=\"/doc/{}\">{}</a></li>",
                            escape(&rel.id),
                            escape(req)
                        )),
                        None => details.push_str(&format!("<li>{}</li>", escape(req))),
                    }
                }
                details.push_str("</ul></details>");
            }
        }
        Some(TypedHandler::Audit) => {
            content.push_str(&format!(
                "<p class=\"text\">Audit type: {}</p>",
                escape(record.audit_type.as_deref().unwrap_or("security"))
            ));
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
        Some(TypedHandler::TechSpec) => {
            // The description is body text: no "Description" heading. The
            // shared `.text` class already applies `white-space: pre-wrap`.
            let content_text = record.description.as_deref().unwrap_or("");
            if content_text.trim().is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!("<p class=\"text\">{}</p>", escape(content_text)));
            }
        }
        Some(TypedHandler::UseCase)
        | Some(TypedHandler::TestCase)
        | Some(TypedHandler::TestReport) => {
            let content_text = record.description.as_deref().unwrap_or("");
            if content_text.trim().is_empty() {
                content.push_str("<p class=\"text empty-text\">(none)</p>");
            } else {
                content.push_str(&format!(
                    "<p class=\"text\" style=\"white-space: pre-wrap\">{}</p>",
                    escape(content_text)
                ));
            }
            if let Some(app) = &record.applicability
                && !app.trim().is_empty()
            {
                content.push_str(&format!(
                    "<section class=\"record-section\"><h2>Applicability</h2><p class=\"text\" style=\"white-space: pre-wrap\">{}</p></section>",
                    escape(app)
                ));
            }
        }
        None => {
            content.push_str("<section class=\"record-section\"><h2>Description</h2>");
            match &record.description {
                Some(description) => {
                    content.push_str(&format!("<p class=\"text\">{}</p>", escape(description)))
                }
                None => content.push_str("<p class=\"text empty-text\">No description.</p>"),
            }
            content.push_str("</section>");
        }
    }

    if !resolved_refs.is_empty() && record.handler() != Some(TypedHandler::Rule) {
        details.push_str(&format!(
            "<details><summary>References ({})</summary><ul class=\"req-list\">",
            resolved_refs.len()
        ));
        for (i, rref) in record.refs.iter().enumerate() {
            let resolved = resolved_refs.get(i).and_then(|r| r.as_ref());
            match resolved {
                Some(rel) => details.push_str(&format!(
                    "<li><a href=\"/doc/{}\">{}</a></li>",
                    escape(&rel.id),
                    escape(rref)
                )),
                None => details.push_str(&format!("<li>{}</li>", escape(rref))),
            }
        }
        details.push_str("</ul></details>");
    }
    let form_line = format!(
        "<p class=\"record-meta\">{}</p>",
        escape(&form_label(record))
    );
    content = format!("{}{}", form_line, content);
    if record.handler() == Some(TypedHandler::Decision) {
        content = format!(
            "<p class=\"record-meta\">Decision type: {}</p><p class=\"record-meta\">Status: {}</p>{}",
            escape(record.decision_type.as_deref().unwrap_or("(none)")),
            escape(record.status.as_deref().unwrap_or("(none)")),
            content
        );
    }
    let extra = record_view_extra_rows(record, &["created_date", "updated_date", "labels"]);
    if !extra.is_empty() {
        content.push_str(&extra);
    }
    if !details.is_empty() {
        content.push_str(&details);
    }
    let taglist = if record.labels.is_empty() {
        String::new()
    } else {
        let mut out = String::from("<div class=\"taglist\">Labels: ");
        for l in &record.labels {
            out.push_str(&format!("<span class=\"tag\">{}</span>", escape(l)));
        }
        out.push_str("</div>");
        out
    };
    // UI mockups store their status in HTML metadata rather than in a
    // rendered section, so surface it with the same read-only status line as
    // the generic document detail page (specification 4.1.3).
    let status_line = if record.handler() == Some(TypedHandler::UiMockup) {
        format!(
            "<p class=\"status-line\"><span class=\"badge\">Status</span> {}</p>",
            escape(&record.status_text())
        )
    } else {
        String::new()
    };
    let page_head = if record.handler() == Some(TypedHandler::Audit) {
        let audit_type = record.audit_type.as_deref().unwrap_or("security");
        format!(
            "<div class=\"page-head\"><div><p class=\"id\">{} <span class=\"badge\">{}</span></p><h1 class=\"h1\">{}</h1>{}</div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
            escape(&record.id),
            escape(audit_type),
            escape(&record.title),
            taglist,
            escape(&record.id)
        )
    } else {
        format!(
            "<div class=\"page-head\"><div><p class=\"id\">{}</p><h1 class=\"h1\">{}</h1>{}{}</div>\
<div class=\"actions\"><a class=\"btn btn-primary\" href=\"/doc/{}/edit\">Edit</a></div></div>",
            escape(&record.id),
            escape(&record.title),
            taglist,
            status_line,
            escape(&record.id)
        )
    };
    let title = format!("Papyrus \u{2014} {} {}", record.id, record.title);
    let breadcrumb_html = breadcrumb(record, resolved_group, related_milestone, related_release);
    let meta_html = if record.is_requirement() {
        metadata_block(record, resolved_group, false)
    } else {
        metadata_block_backlog(record, related_milestone, related_release)
    };
    let body = render_template(
        VIEW_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &breadcrumb_html),
            ("page_head", &page_head),
            ("meta", &meta_html),
            ("content", &content),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

fn metadata_block_backlog(
    record: &Record,
    _related_milestone: Option<&Record>,
    _related_release: Option<&Record>,
) -> String {
    metadata_block(record, None, false)
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

    if record.supports_story() {
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, resolved_group)),
            ("page_head", &page_head),
            ("meta", &metadata_block(record, resolved_group, true)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &body),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", ""),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title_page,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

#[derive(Clone, Debug, Default)]
pub struct TechSpecEditValues {
    pub title: String,
    pub description: String,
}

pub struct UseCaseEditValues {
    pub title: String,
    pub description: String,
    pub applicability: String,
    pub labels: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct UseCaseFieldErrors {
    pub title: Option<String>,
    pub description: Option<String>,
    pub applicability: Option<String>,
    pub labels: Option<String>,
    pub references: Option<String>,
}

impl UseCaseFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.applicability.is_some()
            || self.labels.is_some()
            || self.references.is_some()
    }
}

pub type TestCaseEditValues = UseCaseEditValues;
pub type TestReportEditValues = UseCaseEditValues;
pub type TestCaseFieldErrors = UseCaseFieldErrors;
pub type TestReportFieldErrors = UseCaseFieldErrors;

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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

pub fn edit_page_use_case(
    record: &Record,
    values: &UseCaseEditValues,
    errors: &UseCaseFieldErrors,
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
            ("applicability", errors.applicability.as_deref()),
            ("labels", errors.labels.as_deref()),
            ("references", errors.references.as_deref()),
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
        "<div class=\"field\"><label for=\"description\">Description <span class=\"req\">(required)</span></label><textarea class=\"textarea\" id=\"description\" name=\"description\" rows=\"12\" required{}>{}</textarea><p class=\"help\">Required \u{b7} multi-line.</p>{}</div>",
        invalid_attrs("description", errors.description.as_deref()),
        escape(&values.description),
        field_error("description", errors.description.as_deref())
    ));
    form.push_str(&format!(
        "<div class=\"field\"><label for=\"applicability\">Applicability</label><textarea class=\"textarea\" id=\"applicability\" name=\"applicability\" rows=\"6\"{}>{}</textarea><p class=\"help\">Optional.</p>{}</div>",
        invalid_attrs("applicability", errors.applicability.as_deref()),
        escape(&values.applicability),
        field_error("applicability", errors.applicability.as_deref())
    ));
    form.push_str(&tag_field(
        "labels",
        "labels-add",
        "Labels",
        &values.labels,
        "Add a label",
        "Optional.",
        errors.labels.as_deref(),
        "Label",
    ));
    form.push_str(&tag_field(
        "references",
        "references-add",
        "References",
        &values.references,
        "Add a reference",
        "Optional.",
        errors.references.as_deref(),
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

pub fn edit_page_test_case(
    record: &Record,
    values: &TestCaseEditValues,
    errors: &TestCaseFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    edit_page_use_case(record, values, errors, version, cfg, chrome)
}

pub fn edit_page_test_report(
    record: &Record,
    values: &TestReportEditValues,
    errors: &TestReportFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    edit_page_use_case(record, values, errors, version, cfg, chrome)
}

#[allow(clippy::too_many_arguments)]
pub fn conflict_page_use_case(
    record: &Record,
    title: &str,
    description: &str,
    applicability: &str,
    labels: &[String],
    references: &[String],
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
    body.push_str(&format!(
        "<dt>Applicability</dt><dd class=\"text\">{}</dd>",
        escape(applicability)
    ));
    body.push_str(&format!(
        "<dt>Labels</dt><dd class=\"text\">{}</dd>",
        escape(&labels.join(" "))
    ));
    body.push_str(&format!(
        "<dt>References</dt><dd class=\"text\">{}</dd>",
        escape(&references.join(" "))
    ));
    body.push_str("</dl></div>");

    let title_page = format!("Papyrus \u{2014} Edit {}", record.id);
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &body),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", ""),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title_page,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn conflict_page_test_case(
    record: &Record,
    title: &str,
    description: &str,
    applicability: &str,
    labels: &[String],
    references: &[String],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    conflict_page_use_case(
        record,
        title,
        description,
        applicability,
        labels,
        references,
        cfg,
        chrome,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn conflict_page_test_report(
    record: &Record,
    title: &str,
    description: &str,
    applicability: &str,
    labels: &[String],
    references: &[String],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    conflict_page_use_case(
        record,
        title,
        description,
        applicability,
        labels,
        references,
        cfg,
        chrome,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &body),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", ""),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title_page,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        ERROR_TEMPLATE,
        &[
            ("status_code", &status_text),
            ("heading", heading),
            ("message", message),
            ("hint", hint),
        ],
        &[("actions", actions)],
    );
    page(
        chrome,
        None,
        &title,
        &sidebar(None, cfg, chrome.routes),
        body,
    )
}

/// Render the technical-specifications `404` page.
///
/// The same page is returned for an archived and an unknown technical
/// specification so the response never reveals whether a hidden record exists.
pub fn tech_spec_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let actions =
        "<a class=\"link\" href=\"/?type=tech-spec\">Back to technical specifications</a>";
    let body = render_template(
        ERROR_TEMPLATE,
        &[
            ("status_code", "404"),
            ("heading", "Record not found"),
            ("message", "That record does not exist or is not available."),
            (
                "hint",
                "Archived technical specifications are not listed and cannot be opened. \
The same page is used for an unknown address.",
            ),
        ],
        &[("actions", actions)],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Record not found",
        &sidebar(
            routes::not_found_slug(routes::NotFoundPage::TechSpec),
            cfg,
            chrome.routes,
        ),
        body,
    )
}

/// Render the use-cases `404` page.
pub fn use_case_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let actions = "<a class=\"link\" href=\"/?type=use-case\">Back to Use Cases</a>";
    let body = render_template(
        ERROR_TEMPLATE,
        &[
            ("status_code", "404"),
            ("heading", "Record not found"),
            ("message", "That record does not exist or is not available."),
            (
                "hint",
                "Archived use cases are not listed and cannot be opened. The same page is used for an unknown address.",
            ),
        ],
        &[("actions", actions)],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Record not found",
        &sidebar(
            routes::not_found_slug(routes::NotFoundPage::UseCase),
            cfg,
            chrome.routes,
        ),
        body,
    )
}

/// Render the test-cases `404` page.
pub fn test_case_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let actions = "<a class=\"link\" href=\"/?type=test-case\">Back to Test Cases</a>";
    let body = render_template(
        ERROR_TEMPLATE,
        &[
            ("status_code", "404"),
            ("heading", "Record not found"),
            ("message", "That record does not exist or is not available."),
            (
                "hint",
                "Archived test cases are not listed and cannot be opened. The same page is used for an unknown address.",
            ),
        ],
        &[("actions", actions)],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Record not found",
        &sidebar(
            routes::not_found_slug(routes::NotFoundPage::TestCase),
            cfg,
            chrome.routes,
        ),
        body,
    )
}

/// Render the test-reports `404` page.
pub fn test_report_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let actions = "<a class=\"link\" href=\"/?type=test-report\">Back to Test Reports</a>";
    let body = render_template(
        ERROR_TEMPLATE,
        &[
            ("status_code", "404"),
            ("heading", "Record not found"),
            ("message", "That record does not exist or is not available."),
            (
                "hint",
                "Archived test reports are not listed and cannot be opened. The same page is used for an unknown address.",
            ),
        ],
        &[("actions", actions)],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Record not found",
        &sidebar(
            routes::not_found_slug(routes::NotFoundPage::TestReport),
            cfg,
            chrome.routes,
        ),
        body,
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
        record.slug()
    ));
    content.push_str(
        "<div class=\"readonly-block\"><h2>Your unsaved changes (read-only, not saved)</h2><dl>",
    );
    content.push_str(&format!("<dt>Title</dt><dd>{}</dd>", escape(&values.title)));
    if record.supports_story() {
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", ""),
            ("form", ""),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
        record.slug()
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", ""),
            ("form", ""),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title_text,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
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
    let body = render_template(
        EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &content),
            ("meta", &metadata_block_backlog(record, None, None)),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &title_text,
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

/// Convert a validated change into a [`FieldChange`].
pub fn change(value: Option<String>) -> FieldChange {
    match value {
        Some(value) => FieldChange::Set(value),
        None => FieldChange::Clear,
    }
}

#[allow(clippy::items_after_test_module)]
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

/// Render the registry-derived managed artifacts and lifecycles page.
pub fn lifecycle_page(
    cfg: &Config,
    registry: &crate::registry::ArtifactRegistry,
    chrome: Chrome<'_>,
) -> String {
    fn storage_folder(base: &str, sub: &str) -> String {
        let base = base.trim_end_matches('/');
        let sub = sub.trim_matches('/');
        if sub.is_empty() {
            base.to_string()
        } else if base.is_empty() {
            sub.to_string()
        } else {
            format!("{}/{}", base, sub)
        }
    }

    fn section(descriptor: &TypeDescriptor, cfg: &Config) -> String {
        let heading = routes::heading(&descriptor.slug);
        let description = routes::lifecycle_description(&descriptor.slug);
        let lifecycle = match &descriptor.status_model {
            StatusModel::TaskConfigurable { statuses, .. } => statuses.join(" \u{2192} "),
            StatusModel::Fixed { states } => states.join(" \u{2192} "),
            StatusModel::DocumentMain { custom, .. } => {
                let mut values = vec!["active \u{2192} archived".to_string()];
                if !custom.is_empty() {
                    values.push(format!(
                        "custom front-matter statuses: {} (do not move files)",
                        custom.join(", ")
                    ));
                } else {
                    values.push("custom front-matter statuses: (none)".to_string());
                }
                values.join("; ")
            }
            StatusModel::HtmlMetadata { states } => states.join(" \u{2192} "),
        };
        let folder = match descriptor.directory.config_key {
            "backlog_directory" => {
                storage_folder(&cfg.backlog_directory, &descriptor.directory.relative)
            }
            "decision_directory" => {
                storage_folder(&cfg.decision_directory, &descriptor.directory.relative)
            }
            _ => storage_folder(&cfg.doc_directory, &descriptor.directory.relative),
        };
        format!(
            "<section><h2>{}</h2><p>Curated description: {}</p><p>Lifecycle: {}.</p><p>Read-only config \u{2014} enabled: {}; ID prefix: {}; storage folder: {}. Changes via <a class=\"link\" href=\"/config\">Configuration</a>.</p></section>",
            escape(&heading),
            escape(&description),
            escape(&lifecycle),
            if descriptor.enabled { "true" } else { "false" },
            escape(&descriptor.prefix),
            escape(&folder)
        )
    }

    let mut sections = String::new();
    for (group, category) in [
        (NavGroup::Backlog, CategoryKind::Backlog),
        (NavGroup::Decisions, CategoryKind::Decision),
        (NavGroup::Documents, CategoryKind::Document),
    ] {
        sections.push_str(&format!("<h2 class=\"h2\">{}</h2>", group.heading()));
        for descriptor in registry.in_category(category) {
            sections.push_str(&section(descriptor, cfg));
        }
    }
    sections.push_str(
        "<p>The three categories are Backlog, Decisions, and Documents. Backlog.md <code>docs</code>, <code>drafts</code>, and project configuration files are excluded from every category. Configuration is a startup snapshot: document-type changes take effect on the next browser start.</p>",
    );

    let body = render_template(
        VIEW_TEMPLATE,
        &[],
        &[
            ("breadcrumb", ""),
            (
                "page_head",
                "<h1 class=\"h1\">Managed artifacts &amp; lifecycles</h1>",
            ),
            ("meta", ""),
            ("content", &sections),
        ],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Managed artifacts & lifecycles",
        &sidebar_with_lifecycle(None, cfg, true, chrome.routes),
        body,
    )
}

pub fn skills_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    // Status fallback mirrors the lifecycle page.
    let statuses = if cfg.statuses.is_empty() || !cfg.statuses.contains(&cfg.default_status) {
        Config::default().statuses
    } else {
        cfg.statuses.clone()
    };
    let status_chain_raw = statuses.join(" \u{2192} ");

    // id_padding fallback.
    let padding = if (1..=6).contains(&cfg.id_padding) {
        cfg.id_padding
    } else {
        3
    };
    let task_example = crate::record::identity::format_id(&cfg.task_prefix, 1, padding as usize);
    let milestone_example =
        crate::record::identity::format_id(&cfg.milestone_prefix, 1, padding as usize);
    let release_example =
        crate::record::identity::format_id(&cfg.release_prefix, 1, padding as usize);
    let id_padding_str = padding.to_string();

    let task_prefix = cfg.task_prefix.clone();
    let milestone_prefix = cfg.milestone_prefix.clone();
    let release_prefix = cfg.release_prefix.clone();
    let default_status = cfg.default_status.clone();
    let date_format = cfg.date_format.clone();
    let backlog_directory = cfg.backlog_directory.clone();
    let decision_directory = cfg.decision_directory.clone();
    let doc_directory = cfg.doc_directory.clone();

    // Display order for the skills summary; membership still comes from the
    // configured document types.
    let known_doc_order = [
        "req",
        "nfr",
        "glossary",
        "rule",
        "ui-spec",
        "ui-mockup",
        "audit",
        "tech-spec",
        "use-case",
        "test-case",
        "test-report",
    ];
    let mut enabled_doc = Vec::new();
    for key in known_doc_order {
        if let Some(entry) = cfg.document_types.get(key)
            && entry.enabled
        {
            enabled_doc.push(key.to_string());
        }
    }
    let mut unknown: Vec<String> = cfg
        .document_types
        .iter()
        .filter(|(key, entry)| entry.enabled && !known_doc_order.contains(&key.as_str()))
        .map(|(key, _)| key.clone())
        .collect();
    unknown.sort();
    enabled_doc.extend(unknown);
    let enabled_doc_types = enabled_doc.join(", ");
    let enabled_doc_types_display = if enabled_doc_types.is_empty() {
        "(none)".to_string()
    } else {
        enabled_doc_types.clone()
    };

    // Enabled decision types (BTreeMap order = sorted).
    let enabled_decision: Vec<String> = cfg
        .decision_types
        .iter()
        .filter(|(_, e)| e.enabled)
        .map(|(k, _)| k.clone())
        .collect();
    let enabled_decision_types = enabled_decision.join(", ");
    let enabled_decision_types_display = if enabled_decision_types.is_empty() {
        "(none)".to_string()
    } else {
        enabled_decision_types.clone()
    };

    // Variables for substitution.
    let vars: Vec<(&str, String)> = vec![
        ("task_prefix", task_prefix.clone()),
        ("milestone_prefix", milestone_prefix.clone()),
        ("release_prefix", release_prefix.clone()),
        ("id_padding", id_padding_str.clone()),
        ("task_example", task_example.clone()),
        ("milestone_example", milestone_example.clone()),
        ("release_example", release_example.clone()),
        ("status_chain", status_chain_raw.clone()),
        ("default_status", default_status.clone()),
        ("enabled_doc_types", enabled_doc_types_display.clone()),
        (
            "enabled_decision_types",
            enabled_decision_types_display.clone(),
        ),
        ("doc_directory", doc_directory.clone()),
        ("decision_directory", decision_directory.clone()),
        ("backlog_directory", backlog_directory.clone()),
        ("date_format", date_format.clone()),
    ];

    fn substitute(text: &str, vars: &[(&str, String)]) -> String {
        let mut out = text.to_string();
        for (key, val) in vars {
            out = out.replace(&format!("{{{}}}", key), val);
        }
        out
    }

    // Config block (dl).
    let mut config_block = String::new();
    config_block.push_str("<section><h2>Configuration in effect</h2><dl>");
    let pairs: Vec<(&str, String)> = vec![
        ("Task prefix", task_prefix.clone()),
        ("Milestone prefix", milestone_prefix.clone()),
        ("Release prefix", release_prefix.clone()),
        ("ID padding", format!("{} (e.g. {})", padding, task_example)),
        ("Date format", date_format.clone()),
        ("Statuses", status_chain_raw.clone()),
        ("Enabled document types", enabled_doc_types_display.clone()),
        (
            "Enabled decision types",
            enabled_decision_types_display.clone(),
        ),
        ("Backlog directory", backlog_directory.clone()),
        ("Decision directory", decision_directory.clone()),
        ("Doc directory", doc_directory.clone()),
    ];
    for (label, value) in pairs {
        config_block.push_str(&format!(
            "<div><dt>{}</dt><dd>{}</dd></div>",
            escape(label),
            escape(&value)
        ));
    }
    config_block.push_str("</dl></section>");

    // Skill groups.
    // (group title, Vec of (title, desc_template, snippet_template))
    #[allow(clippy::type_complexity)]
    let groups: Vec<(&str, Vec<(&str, &str, &str)>)> = vec![
        (
            "Project setup and init",
            vec![
                (
                    "Initialize a new project",
                    "Create the project skeleton and write the initial configuration.",
                    "papyrus init --dir ./my-project",
                ),
                (
                    "Inspect effective configuration",
                    "Prefixes, padding, statuses, enabled types, and directories are read from papyrus.config.yml with built-in defaults.",
                    "papyrus config list",
                ),
            ],
        ),
        (
            "Backlog, tasks, milestones, releases",
            vec![
                (
                    "Task IDs and statuses",
                    "New tasks use prefix {task_prefix} with padding {id_padding}: {task_example}. Statuses flow {status_chain}.",
                    "papyrus task create \"Title\" --status {default_status}",
                ),
                (
                    "Milestones and releases",
                    "Milestone IDs use prefix {milestone_prefix} ({milestone_example}); release IDs use prefix {release_prefix} ({release_example}). Milestones move open \u{2192} closed; releases move planned \u{2192} released. Release lifecycle is separate from task workflow status.",
                    "papyrus milestone create \"Title\"\npapyrus milestone close {milestone_example}\npapyrus release create \"Title\"",
                ),
            ],
        ),
        (
            "Requirements and documentation management",
            vec![
                (
                    "Document types and directories",
                    "Enabled types ({enabled_doc_types}) are stored under {doc_directory} in their configured directories. Disabled types are hidden.",
                    "papyrus req create \"Title\"\npapyrus nfr create \"Title\"",
                ),
                (
                    "Business rules, glossary, specifications",
                    "Rules, glossary terms, UI specs, technical specs, and mockups follow the same record lifecycle: active \u{2192} archived.",
                    "papyrus rule create \"Title\"\npapyrus glossary create \"Term\"",
                ),
            ],
        ),
        (
            "Search",
            vec![(
                "Search across records",
                "Search spans enabled document and decision types; disabled types are excluded.",
                "papyrus search \"query\"",
            )],
        ),
        (
            "Browser usage",
            vec![(
                "Serve the local browser",
                "Offline, loopback by default. No internet connection required.",
                "papyrus browser --port 6420",
            )],
        ),
        (
            "Release and audit workflows",
            vec![(
                "Releases and audits",
                "Releases move planned \u{2192} released; audit records are created for release audits. Decision types enabled: {enabled_decision_types}.",
                "papyrus release close {release_example}",
            )],
        ),
        (
            "Configuration management",
            vec![(
                "Set and unset keys",
                "Prefixes, padding, statuses, enabled types, and directories are configuration keys (BR-016).",
                "papyrus config set task_prefix {task_prefix}\npapyrus config unset statuses",
            )],
        ),
        (
            "Decisions",
            vec![(
                "Decision records",
                "Enabled decision types: {enabled_decision_types}. Lifecycle: proposed \u{2192} accepted | rejected; accepted \u{2192} deprecated | superseded.",
                "papyrus decision create --type adr \"Title\"",
            )],
        ),
    ];

    let mut sections = String::new();
    sections.push_str(&config_block);
    for (group, articles) in &groups {
        sections.push_str(&format!("<section><h2>{}</h2>", escape(group)));
        for (title, desc_tpl, snippet_tpl) in articles {
            let desc = substitute(desc_tpl, &vars);
            let snippet = substitute(snippet_tpl, &vars);
            sections.push_str(&format!(
                "<article><h3>{}</h3><p>{}</p><pre>{}</pre><button type=\"button\">Copy</button></article>",
                escape(title),
                escape(&desc),
                escape(&snippet)
            ));
        }
        sections.push_str("</section>");
    }

    let title = "Papyrus \u{2014} Agent skills and workflows".to_string();
    let subheading = "Agent-facing instructions for configuring and operating Papyrus. Values reflect the effective project configuration. Read-only reference.";
    let body = render_template(
        LIST_TEMPLATE,
        &[
            ("heading", "Agent skills and workflows"),
            ("subheading", subheading),
        ],
        &[("section", &sections)],
    );
    page(
        chrome,
        None,
        &title,
        &sidebar_with_skills(None, cfg, true, chrome.routes),
        body,
    )
}

/// An inventory hub card for one record type or virtual page.
#[derive(Clone)]
pub struct InventoryCard {
    pub title: String,
    pub count: String,
    pub href: Option<String>,
    pub note: String,
}

/// Render the inventory hub at `GET /`.
/// Render the configuration view page.
pub fn config_page(chrome: Chrome<'_>, section_html: &str, sidebar_html: &str) -> String {
    let title = "Papyrus \u{2014} Configuration".to_string();
    let subheading = "Effective configuration for the current project. Values come from built-in defaults and the config file.";
    let body = render_template(
        LIST_TEMPLATE,
        &[("heading", "Configuration"), ("subheading", subheading)],
        &[("section", section_html)],
    );
    page(chrome, None, &title, sidebar_html, body)
}

/// Render the single-key configuration edit page.
pub fn config_edit_page(chrome: Chrome<'_>, sidebar_html: &str, section_html: &str) -> String {
    let title = "Papyrus \u{2014} Edit configuration".to_string();
    let subheading = "Edit one configuration key.";
    let body = render_template(
        LIST_TEMPLATE,
        &[
            ("heading", "Edit configuration"),
            ("subheading", subheading),
        ],
        &[("section", section_html)],
    );
    page(chrome, None, &title, sidebar_html, body)
}

pub fn inventory_page(cfg: &Config, chrome: Chrome<'_>, cards: &[InventoryCard]) -> String {
    let mut section = String::from("<section class=\"inventory\">");
    for card in cards {
        section.push_str("<article class=\"card\">");
        section.push_str(&format!(
            "<span class=\"badge\">{} &middot; {}</span>",
            escape(&card.title),
            escape(&card.count)
        ));
        section.push_str(&format!("<h3>{}</h3>", escape(&card.title)));
        if let Some(href) = &card.href {
            section.push_str(&format!(
                "<p><a class=\"btn\" href=\"{}\">Open</a></p>",
                escape(href)
            ));
        }
        if !card.note.is_empty() {
            section.push_str(&format!("<p class=\"note\">{}</p>", escape(&card.note)));
        }
        section.push_str("</article>");
    }
    section.push_str("</section>");
    let title = "Papyrus \u{2014} Browser inventory".to_string();
    let subheading = "All Papyrus record types in one navigation (REQ-06.002, NFR-01.002).";
    let body = render_template(
        LIST_TEMPLATE,
        &[("heading", "Browser inventory"), ("subheading", subheading)],
        &[("section", &section)],
    );
    page(
        chrome,
        None,
        &title,
        &sidebar(None, cfg, chrome.routes),
        body,
    )
}

pub const DOCUMENT_DETAIL_TEMPLATE: &str = include_str!("templates/document_detail.html");
pub const DOCUMENT_EDIT_TEMPLATE: &str = include_str!("templates/document_edit.html");

/// Generic document detail page (section 4.1).
pub fn document_detail_page(record: &Record, cfg: &Config, chrome: Chrome<'_>) -> String {
    let view = record_view(record);
    let created = view
        .as_ref()
        .map(|v| v.value_text("created_date"))
        .unwrap_or_default();
    let created = if created.is_empty() {
        "(none)".to_string()
    } else {
        created
    };
    let updated = view
        .as_ref()
        .map(|v| v.value_text("updated_date"))
        .unwrap_or_default();
    let updated = if updated.is_empty() {
        "(none)".to_string()
    } else {
        updated
    };

    let mut tags = String::new();
    for label in &record.labels {
        tags.push_str(&format!("<span class=\"tag\">{}</span>", escape(label)));
    }
    let label_tags = if tags.is_empty() {
        String::new()
    } else {
        format!("<p class=\"taglist\">{}</p>", tags)
    };
    let page_head = format!(
        "<h1 class=\"h1\">{} {}</h1>{}<p class=\"status-line\"><span class=\"badge\">Status</span> {}</p>",
        escape(&record.id),
        escape(&record.title),
        label_tags,
        escape(&record.status_text())
    );
    let meta = format!(
        "<div class=\"meta summary-card\"><dl class=\"dl\">\
<div><dt>Record ID</dt><dd class=\"mono\">{}</dd></div>\
<div><dt>Created date</dt><dd>{}</dd></div>\
<div><dt>Updated date</dt><dd>{}</dd></div>\
</dl></div>",
        escape(&record.id),
        escape(&created),
        escape(&updated)
    );
    let description = record
        .description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<p>{}</p>", escape(value)))
        .unwrap_or_else(|| "<p>No description.</p>".to_string());
    let skip = [
        "id",
        "title",
        "created_date",
        "updated_date",
        "labels",
        "references",
    ];
    let metadata = record_view_extra_rows(record, &skip);
    let mut references = String::new();
    if !record.refs.is_empty() {
        let items: String = record
            .refs
            .iter()
            .map(|value| format!("<li>{}</li>", escape(value)))
            .collect();
        references = format!(
            "<details class=\"references\"><summary>References</summary><ul>{}</ul></details>",
            items
        );
    }
    let content = format!(
        "<section class=\"section\"><h2 class=\"h2\">Description</h2>{}</section>{}{}<p><a class=\"btn\" href=\"/doc/{}/edit\">Edit</a></p>",
        description,
        metadata,
        references,
        escape(&record.id)
    );
    let body = render_template(
        DOCUMENT_DETAIL_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &breadcrumb(record, None, None, None)),
            ("page_head", &page_head),
            ("meta", &meta),
            ("content", &content),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &format!("Papyrus \u{2014} {} {}", record.id, record.title),
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

/// Submitted values for the generic document edit form.
#[derive(Clone, Debug, Default)]
pub struct DocumentEditValues {
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub references: Vec<String>,
}

/// Field errors for the generic document edit form.
#[derive(Clone, Debug, Default)]
pub struct DocumentFieldErrors {
    pub title: Option<String>,
    pub description: Option<String>,
    pub labels: Option<String>,
    pub references: Option<String>,
}

impl DocumentFieldErrors {
    pub fn any(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.labels.is_some()
            || self.references.is_some()
    }
}

fn document_edit_form(
    record: &Record,
    values: &DocumentEditValues,
    errors: &DocumentFieldErrors,
    version: &str,
) -> String {
    let mut alert = String::new();
    let mut messages = Vec::new();
    for message in [
        errors.title.as_deref(),
        errors.description.as_deref(),
        errors.labels.as_deref(),
        errors.references.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        messages.push(format!("<li>{}</li>", escape(message)));
    }
    if !messages.is_empty() {
        alert = format!(
            "<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><ul>{}</ul><p>Nothing was saved. Your typed values are kept below.</p></div>",
            messages.join("")
        );
    }
    let labels: String = values
        .labels
        .iter()
        .map(|value| {
            format!(
                "<input class=\"input\" name=\"labels\" type=\"text\" value=\"{}\">",
                escape(value)
            )
        })
        .collect();
    let references: String = values
        .references
        .iter()
        .map(|value| {
            format!(
                "<input class=\"input\" name=\"references\" type=\"text\" value=\"{}\">",
                escape(value)
            )
        })
        .collect();
    format!(
        "{alert}<form class=\"form\" method=\"post\" action=\"/doc/{id}\">\
<input type=\"hidden\" name=\"version\" value=\"{version}\">\
<div class=\"field\"><label for=\"title\">Title</label><input class=\"input\" id=\"title\" name=\"title\" type=\"text\" value=\"{title}\" required maxlength=\"200\"></div>\
<div class=\"field\"><label for=\"description\">Description</label><textarea class=\"input\" id=\"description\" name=\"description\" rows=\"8\">{description}</textarea></div>\
<div class=\"field\"><label>Labels</label>{labels}<input class=\"input\" name=\"labels\" type=\"text\" value=\"\"></div>\
<div class=\"field\"><label>References</label>{references}<input class=\"input\" name=\"references\" type=\"text\" value=\"\"></div>\
{timestamps}\
<div class=\"form-actions\"><button class=\"btn btn-primary\" type=\"submit\">Save changes</button><a class=\"link\" href=\"/doc/{id}\">Cancel</a></div></form>",
        alert = alert,
        id = escape(&record.id),
        version = escape(version),
        title = escape(&values.title),
        description = escape(&values.description),
        labels = labels,
        references = references,
        timestamps = read_only_timestamps(record),
    )
}

/// Generic document edit form (section 4.2); never renders status.
pub fn document_edit_page(
    record: &Record,
    values: &DocumentEditValues,
    errors: &DocumentFieldErrors,
    version: &str,
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let page_head = format!(
        "<h1 class=\"h1\">Edit {} {}</h1>",
        escape(&record.id),
        escape(&record.title)
    );
    let body = render_template(
        DOCUMENT_EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", ""),
            ("form", &document_edit_form(record, values, errors, version)),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &format!("Papyrus \u{2014} Edit {}", record.id),
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

/// Stale-version conflict page that preserves the submitted values.
#[allow(clippy::too_many_arguments)]
pub fn document_conflict_page(
    record: &Record,
    title: &str,
    description: &str,
    labels: &[String],
    references: &[String],
    cfg: &Config,
    chrome: Chrome<'_>,
) -> String {
    let values = DocumentEditValues {
        title: title.to_string(),
        description: description.to_string(),
        labels: labels.to_vec(),
        references: references.to_vec(),
    };
    let errors = DocumentFieldErrors::default();
    let mut form = document_edit_form(record, &values, &errors, &record.version);
    form = format!(
        "<div class=\"alert\" role=\"alert\"><h2>There is a problem</h2><p>record changed on disk; reload the form</p></div>{}",
        form
    );
    let page_head = format!(
        "<h1 class=\"h1\">Edit {} {}</h1>",
        escape(&record.id),
        escape(&record.title)
    );
    let body = render_template(
        DOCUMENT_EDIT_TEMPLATE,
        &[],
        &[
            ("breadcrumb", &edit_breadcrumb(record, None)),
            ("page_head", &page_head),
            ("meta", ""),
            ("form", &form),
        ],
    );
    page(
        chrome,
        Some(record.slug()),
        &format!("Papyrus \u{2014} Edit {}", record.id),
        &sidebar(Some(record.slug()), cfg, chrome.routes),
        body,
    )
}

/// Generic document not-found page. It never echoes the requested ID.
pub fn document_not_found_page(cfg: &Config, chrome: Chrome<'_>) -> String {
    let content = "<p>The requested record was not found.</p><p><a class=\"link\" href=\"/\">Back to requirements</a></p>";
    let body = render_template(
        VIEW_TEMPLATE,
        &[],
        &[
            ("breadcrumb", ""),
            ("page_head", "<h1 class=\"h1\">Not found</h1>"),
            ("meta", ""),
            ("content", content),
        ],
    );
    page(
        chrome,
        None,
        "Papyrus \u{2014} Not found",
        &sidebar(None, cfg, chrome.routes),
        body,
    )
}
