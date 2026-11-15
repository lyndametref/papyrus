//! The one shared list layout (ADR-0028 decision 2, REQ-06.016).
//!
//! A list page builds a [`ListPlan`] from declared data — the record index and
//! the presentation table's slot columns — and renders it through this module.
//! Family variation is limited to the declared family slot column(s) and the
//! empty-state copy; structural columns are fixed for every family.

use crate::browser::records::{Index, Record};
use crate::browser::render::escape;
use crate::browser::render::render_template;
use crate::browser::routes;
use crate::registry::TypeDescriptor;

/// The list section template: heading, count note, then the table or empty state.
const LIST_TEMPLATE: &str = "<section class=\"section\" aria-label=\"{{heading}}\">\
<div class=\"section-head\"><h2 class=\"h2\">{{heading}}</h2><p class=\"count\">{{count_note}}</p></div>\
{{body}}</section>";

/// Structural column keys. They may never be declared as family slots.
pub const RESERVED_COLUMN_KEYS: [&str; 6] =
    ["id", "title", "status", "created", "updated", "actions"];

/// One list page, fully assembled by the caller.
#[derive(Clone, Debug, Default)]
pub struct ListPlan {
    pub heading: String,
    pub count_note: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub empty: Option<EmptyState>,
}

/// One column of the shared table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// `id` | `title` | `status` | family slot key | `created` | `updated` | `actions`.
    pub key: &'static str,
    pub label: &'static str,
    /// Cell class; empty when none.
    pub class: &'static str,
}

/// One record row. Grouped families nest member rows in `children`.
#[derive(Clone, Debug, Default)]
pub struct Row {
    pub id: String,
    pub title: String,
    /// Route to the record view (`/doc/{id}`), never a stored value.
    pub href: String,
    /// Status, family slot(s), Created and Updated cells, in column order.
    pub cells: Vec<String>,
    /// Front-matter labels, rendered in the Title cell.
    pub labels: Vec<String>,
    /// Front-matter references, rendered in the Title cell.
    pub references: Vec<String>,
    /// `(label, href)` actions: View, Edit, then any family action.
    pub actions: Vec<(String, String)>,
    /// Member rows of a grouped family, rendered in the same table body.
    pub children: Vec<Row>,
}

/// The per-family empty state; copy is supplied by the caller, not by markup.
#[derive(Clone, Debug)]
pub struct EmptyState {
    pub message: String,
    pub help: Option<String>,
}

impl ListPlan {
    /// Build the reference-family plan from the index and declared slots.
    ///
    /// Rows keep `Index.records` order. Family variation comes only from
    /// `routes::list_slots(slug)` and the empty copy; there is no slug or
    /// handler match in this module.
    pub fn for_descriptor(
        descriptor: &TypeDescriptor,
        index: &Index,
        heading: &str,
        count_note: &str,
    ) -> ListPlan {
        let slug = descriptor.slug.as_str();
        let slots = routes::list_slots(slug);
        validate_slots(slots);
        let mut columns = vec![
            Column {
                key: "id",
                label: "ID",
                class: "id-cell",
            },
            Column {
                key: "title",
                label: "Title",
                class: "title-cell",
            },
            Column {
                key: "status",
                label: "Status",
                class: "status-cell",
            },
        ];
        for (key, label) in slots {
            columns.push(Column {
                key,
                label,
                class: "slot-cell",
            });
        }
        columns.push(Column {
            key: "created",
            label: "Created",
            class: "created-cell",
        });
        columns.push(Column {
            key: "updated",
            label: "Updated",
            class: "updated-cell",
        });
        columns.push(Column {
            key: "actions",
            label: "Actions",
            class: "actions-cell",
        });

        let rows = index
            .records
            .iter()
            .map(|record| row_for(record, slots))
            .collect();
        let empty = if index.records.is_empty() {
            Some(EmptyState {
                message: format!("No {} in this project.", routes::short(slug)),
                help: Some(
                    "Records are created from the command line; this browser does not create records."
                        .to_string(),
                ),
            })
        } else {
            None
        };
        ListPlan {
            heading: heading.to_string(),
            count_note: count_note.to_string(),
            columns,
            rows,
            empty,
        }
    }

    /// Render the section, the table, or the empty state.
    pub fn render(&self) -> String {
        self.validate();
        let body = match &self.empty {
            Some(empty) => {
                let help = empty
                    .help
                    .as_deref()
                    .map(|help| format!("<p class=\"help\">{}</p>", escape(help)))
                    .unwrap_or_default();
                format!(
                    "<div class=\"empty\"><p>{}</p>{}</div>",
                    escape(&empty.message),
                    help
                )
            }
            None => self.render_table(),
        };
        render_template(
            LIST_TEMPLATE,
            &[("heading", &self.heading), ("count_note", &self.count_note)],
            &[("body", &body)],
        )
    }

    fn render_table(&self) -> String {
        let mut header = String::new();
        for column in &self.columns {
            header.push_str(&format!(
                "<th scope=\"col\" data-column=\"{}\">{}</th>",
                escape(column.key),
                escape(column.label)
            ));
        }
        let mut rows = String::new();
        for row in &self.rows {
            render_row(row, &self.columns, None, &mut rows);
            for child in &row.children {
                render_row(child, &self.columns, Some(&row.id), &mut rows);
            }
        }
        format!(
            "<div class=\"table-scroll\"><table class=\"table list list--shared\" data-list-layout=\"shared\" aria-label=\"{}\">\
<thead><tr>{}</tr></thead><tbody>{}</tbody></table></div>",
            escape(&self.heading),
            header,
            rows
        )
    }

    fn validate(&self) {
        assert!(
            self.columns.len() >= 6,
            "shared list needs the six structural columns"
        );
        assert_eq!(self.columns[0].key, "id", "first column is ID");
        assert_eq!(self.columns[1].key, "title", "second column is Title");
        assert_eq!(
            self.columns.last().map(|column| column.key),
            Some("actions"),
            "last column is Actions"
        );
        assert_eq!(self.columns[2].key, "status", "Status is the third column");
        assert_eq!(
            self.columns[self.columns.len() - 3].key,
            "created",
            "Created precedes Updated"
        );
        assert_eq!(
            self.columns[self.columns.len() - 2].key,
            "updated",
            "Updated precedes Actions"
        );
        let slots: Vec<&str> = self.columns[3..self.columns.len() - 3]
            .iter()
            .map(|column| column.key)
            .collect();
        for (index, key) in slots.iter().enumerate() {
            assert!(
                !RESERVED_COLUMN_KEYS.contains(key),
                "family slot key `{key}` is a reserved structural column"
            );
            assert!(
                !slots[..index].contains(key),
                "family slot key `{key}` is declared twice"
            );
        }
        let expected = self.columns.len() - 3;
        for row in &self.rows {
            assert_eq!(
                row.cells.len(),
                expected,
                "row {} must carry Status, slot(s), Created and Updated",
                row.id
            );
            if !row.actions.is_empty() {
                assert_eq!(row.actions[0].0, "View", "View is the first action");
                assert!(
                    row.actions.len() == 1 || row.actions[1].0 == "Edit",
                    "Edit is the second action"
                );
            }
            for child in &row.children {
                assert_eq!(child.cells.len(), expected, "child {} cell count", child.id);
            }
        }
    }
}

/// Reject reserved keys and duplicates in a declared slot list.
///
/// A violation is a programming error in the presentation table, not a page
/// error, so it panics in every build.
pub fn validate_slots(slots: &[(&'static str, &'static str)]) {
    for (index, (key, _)) in slots.iter().enumerate() {
        assert!(
            !RESERVED_COLUMN_KEYS.contains(key),
            "family slot key `{key}` is a reserved structural column"
        );
        assert!(
            !slots[..index].iter().any(|(earlier, _)| earlier == key),
            "family slot key `{key}` is declared twice"
        );
    }
}

fn row_for(record: &Record, slots: &[(&'static str, &'static str)]) -> Row {
    let id = record.id.clone();
    let href = format!("/doc/{id}");
    let mut cells = vec![status_cell(record)];
    for (key, _) in slots {
        cells.push(slot_cell(key, record));
    }
    cells.push(date_cell(record, "created_date"));
    cells.push(date_cell(record, "updated_date"));
    Row {
        id: id.clone(),
        title: record.title.clone(),
        href: href.clone(),
        cells,
        labels: record.labels.clone(),
        references: record.refs.clone(),
        actions: vec![
            ("View".to_string(), href),
            ("Edit".to_string(), format!("/doc/{id}/edit")),
        ],
        children: Vec::new(),
    }
}

fn status_cell(record: &Record) -> String {
    format!(
        "<span class=\"row-status tag\">{}</span>",
        escape(&record.status_text())
    )
}

/// One family slot value. This is a match on the slot key only — never on a
/// slug, a handler, or a template.
fn slot_cell(key: &str, record: &Record) -> String {
    let value = match key {
        "milestone" => record.milestone_id.clone(),
        "assignee" => record.assignee.clone(),
        "release" => record.release_id.clone(),
        "type" => record.decision_type.clone(),
        "group" => record.group_id.clone(),
        "audit_type" => record.audit_type.clone(),
        "applicability" => record.applicability.clone(),
        "task" => Some(record.mockup_task.clone()),
        "specs" => Some(record.specs.join(", ")),
        _ => None,
    };
    match value.as_deref().filter(|value| !value.is_empty()) {
        Some(value) => escape(value),
        None => "<span class=\"muted\">(none)</span>".to_string(),
    }
}

fn date_cell(record: &Record, field: &str) -> String {
    let value = crate::record::view::RecordView::from_content(
        &record.doc.source,
        &record.id,
        &record.version,
    )
    .ok()
    .map(|view| view.value_text(field))
    .unwrap_or_default();
    if value.is_empty() {
        "<span class=\"muted\">(none)</span>".to_string()
    } else {
        escape(&value)
    }
}

/// Count note for a shared list (`3 milestones`, `1 milestone`, `0 milestones`).
///
/// The plural noun is declared in the presentation table (`routes::short`);
/// config-defined types fall back to their lower-case slug.
pub fn count_note(slug: &str, count: usize) -> String {
    let plural = routes::short(slug);
    let noun = if count == 1 {
        singular(&plural)
    } else if plural.ends_with('s') {
        plural
    } else {
        format!("{plural}s")
    };
    format!("{count} {noun}")
}

fn singular(plural: &str) -> String {
    if plural.ends_with('s') && !plural.ends_with("ss") {
        plural[..plural.len() - 1].to_string()
    } else {
        plural.to_string()
    }
}

fn render_row(row: &Row, columns: &[Column], parent: Option<&str>, out: &mut String) {
    let child_of = parent
        .map(|parent| format!(" data-child-of=\"{}\"", escape(parent)))
        .unwrap_or_default();
    out.push_str(&format!(
        "<tr data-row-id=\"{}\"{}>",
        escape(&row.id),
        child_of
    ));
    out.push_str(&format!("<td class=\"id-cell\">{}</td>", escape(&row.id)));
    let mut title_cell = format!(
        "<td class=\"title-cell\"><a class=\"link\" href=\"{}\">{}</a>",
        escape(&row.href),
        escape(&row.title)
    );
    if !row.labels.is_empty() {
        let tags: String = row
            .labels
            .iter()
            .map(|label| format!("<li class=\"tag\">{}</li>", escape(label)))
            .collect();
        title_cell.push_str(&format!("<ul class=\"row-tags\">{tags}</ul>"));
    }
    if !row.references.is_empty() {
        let references: String = row
            .references
            .iter()
            .map(|reference| format!("<li>{}</li>", escape(reference)))
            .collect();
        title_cell.push_str(&format!("<ul class=\"row-refs\">{references}</ul>"));
    }
    title_cell.push_str("</td>");
    out.push_str(&title_cell);
    for (cell, column) in row.cells.iter().zip(&columns[2..]) {
        out.push_str(&format!(
            "<td class=\"{}\">{}</td>",
            escape(column.class),
            cell
        ));
    }
    out.push_str("<td class=\"actions-cell\">");
    for (index, (label, href)) in row.actions.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(&format!(
            "<a class=\"link\" href=\"{}\" aria-label=\"{} {}\">{}</a>",
            escape(href),
            escape(label),
            escape(&row.id),
            escape(label)
        ));
    }
    out.push_str("</td></tr>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::registry::ArtifactRegistry;

    fn descriptor(slug: &str) -> TypeDescriptor {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        registry
            .descriptors()
            .find(|descriptor| descriptor.slug == slug)
            .cloned()
            .unwrap_or_else(|| panic!("descriptor {slug}"))
    }

    fn plan(columns: Vec<Column>, rows: Vec<Row>) -> ListPlan {
        ListPlan {
            heading: "Milestones".to_string(),
            count_note: "2 milestones".to_string(),
            columns,
            rows,
            empty: None,
        }
    }

    fn row(id: &str, cells: Vec<&str>) -> Row {
        Row {
            id: id.to_string(),
            title: format!("{id} title"),
            href: format!("/doc/{id}"),
            cells: cells.into_iter().map(str::to_string).collect(),
            labels: Vec::new(),
            references: Vec::new(),
            actions: vec![
                ("View".to_string(), format!("/doc/{id}")),
                ("Edit".to_string(), format!("/doc/{id}/edit")),
            ],
            children: Vec::new(),
        }
    }

    fn milestone_plan() -> ListPlan {
        ListPlan::for_descriptor(
            &descriptor("milestone"),
            &Index::default(),
            "Milestones",
            "0 milestones",
        )
    }

    #[test]
    fn reference_family_columns_are_the_amended_fixed_order() {
        let plan = milestone_plan();
        let keys: Vec<&str> = plan.columns.iter().map(|column| column.key).collect();
        assert_eq!(
            keys,
            [
                "id", "title", "status", "release", "created", "updated", "actions"
            ]
        );
        let labels: Vec<&str> = plan.columns.iter().map(|column| column.label).collect();
        assert_eq!(
            labels,
            [
                "ID", "Title", "Status", "Release", "Created", "Updated", "Actions"
            ]
        );
    }

    #[test]
    fn families_without_a_presentation_entry_have_no_slots() {
        assert_eq!(routes::list_slots("note"), &[]);
        let plan = ListPlan::for_descriptor(
            &descriptor("release"),
            &Index::default(),
            "Releases",
            "0 releases",
        );
        let keys: Vec<&str> = plan.columns.iter().map(|column| column.key).collect();
        assert_eq!(
            keys,
            ["id", "title", "status", "created", "updated", "actions"]
        );
    }

    #[test]
    fn every_declared_slot_set_is_valid() {
        for entry in routes::PRESENTATION {
            validate_slots(entry.list_slots);
        }
    }

    #[test]
    #[should_panic(expected = "reserved structural column")]
    fn reserved_slot_keys_are_rejected() {
        validate_slots(&[("status", "Status")]);
    }

    #[test]
    #[should_panic(expected = "declared twice")]
    fn duplicate_slot_keys_are_rejected() {
        validate_slots(&[("release", "Release"), ("release", "Release")]);
    }

    #[test]
    fn render_emits_the_table_hooks_and_header_sequence() {
        let html = plan(
            milestone_plan().columns,
            vec![row("MS-001", vec!["open", "REL-001", "c", "u"])],
        )
        .render();
        assert!(
            html.contains("<table class=\"table list list--shared\" data-list-layout=\"shared\"")
        );
        for key in [
            "id", "title", "status", "release", "created", "updated", "actions",
        ] {
            assert!(
                html.contains(&format!("<th scope=\"col\" data-column=\"{key}\">")),
                "missing header hook {key}"
            );
        }
        assert!(html.contains("<tbody><tr data-row-id=\"MS-001\">"));
    }

    #[test]
    fn rows_render_cells_actions_and_record_specific_names() {
        let plan = plan(
            milestone_plan().columns,
            vec![row("MS-001", vec!["open", "REL-001", "c", "u"])],
        );
        let html = plan.render();
        assert!(html.contains("<tr data-row-id=\"MS-001\">"));
        assert_eq!(html.matches("<td class=").count(), 7);
        assert!(html.contains("<td class=\"slot-cell\">REL-001</td>"));
        assert!(html.contains("<td class=\"created-cell\">c</td>"));
        assert!(html.contains("<td class=\"updated-cell\">u</td>"));
        let actions = &html[html.find("actions-cell").unwrap()..];
        let view = actions.find("aria-label=\"View MS-001\"").unwrap();
        let edit = actions.find("aria-label=\"Edit MS-001\"").unwrap();
        assert!(view < edit, "View precedes Edit");
    }

    #[test]
    fn family_action_follows_view_and_edit() {
        let mut first = row("MS-001", vec!["open", "REL-001", "c", "u"]);
        first
            .actions
            .push(("Close".to_string(), "/doc/MS-001/close".to_string()));
        let html = plan(milestone_plan().columns, vec![first]).render();
        let actions = &html[html.find("actions-cell").unwrap()..];
        let view = actions.find(">View</a>").unwrap();
        let edit = actions.find(">Edit</a>").unwrap();
        let family = actions.find(">Close</a>").unwrap();
        assert!(view < edit && edit < family, "View, Edit, family action");
    }

    #[test]
    fn grouped_children_are_rows_in_the_same_body() {
        let mut group = row("REQ-001", vec!["", "REQ-001", "c", "u"]);
        group.children = vec![row("REQ-001.001", vec!["active", "REQ-001", "c", "u"])];
        let columns = ListPlan {
            heading: "Requirements".to_string(),
            count_note: "1 group".to_string(),
            columns: vec![
                Column {
                    key: "id",
                    label: "ID",
                    class: "id-cell",
                },
                Column {
                    key: "title",
                    label: "Title",
                    class: "title-cell",
                },
                Column {
                    key: "status",
                    label: "Status",
                    class: "status-cell",
                },
                Column {
                    key: "group",
                    label: "Group",
                    class: "slot-cell",
                },
                Column {
                    key: "created",
                    label: "Created",
                    class: "created-cell",
                },
                Column {
                    key: "updated",
                    label: "Updated",
                    class: "updated-cell",
                },
                Column {
                    key: "actions",
                    label: "Actions",
                    class: "actions-cell",
                },
            ],
            rows: vec![group],
            empty: None,
        };
        let html = columns.render();
        assert_eq!(html.matches("<tbody>").count(), 1);
        assert!(html.contains("<tr data-row-id=\"REQ-001.001\" data-child-of=\"REQ-001\">"));
    }

    #[test]
    fn empty_state_replaces_the_table() {
        let html = milestone_plan().render();
        assert!(html.contains("<p class=\"count\">0 milestones</p>"));
        assert!(html.contains("No milestones in this project."));
        assert!(html.contains("Records are created from the command line"));
        assert!(!html.contains("<table"));
    }

    #[test]
    fn title_cell_escapes_labels_and_references() {
        let mut first = row("MS-001", vec!["open", "REL-001", "c", "u"]);
        first.labels = vec!["<b>".to_string()];
        first.references = vec!["a&b".to_string()];
        let html = plan(milestone_plan().columns, vec![first]).render();
        assert!(html.contains("<li class=\"tag\">&lt;b&gt;</li>"));
        assert!(html.contains("<li>a&amp;b</li>"));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn count_note_handles_singular_and_plural() {
        assert_eq!(count_note("milestone", 0), "0 milestones");
        assert_eq!(count_note("milestone", 1), "1 milestone");
        assert_eq!(count_note("milestone", 3), "3 milestones");
        assert_eq!(count_note("business-rule", 2), "2 business rules");
        assert_eq!(count_note("note", 2), "2 notes");
    }
}
