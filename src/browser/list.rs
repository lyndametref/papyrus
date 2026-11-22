//! The one shared list layout (ADR-0028 decision 2, REQ-06.016).
//!
//! A list page builds a [`ListPlan`] from declared data — the record index and
//! the descriptor (or static Backlog/Decision) slot columns — and renders it
//! through this module.
//! Family variation is limited to the declared family slot column(s) and the
//! empty-state copy; structural columns are fixed for every family.

use crate::browser::records::{Index, Record};
use crate::browser::render::escape;
use crate::browser::render::render_template;
use crate::browser::routes;
use crate::registry::{ArtifactCategory as CategoryKind, IdShape, TypeDescriptor};

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
    pub key: String,
    pub label: String,
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
    /// Front-matter references, rendered in the Title cell when visible.
    pub references: Vec<String>,
    /// Whether this row's reference list renders (ADR-039 decision 32).
    pub render_references: bool,
    /// `(label, href)` actions: View, Edit, then any family action.
    pub actions: Vec<(String, String)>,
    /// Member rows of a grouped family, rendered in the same table body.
    pub children: Vec<Row>,
}

/// The one shared empty state; copy is assembled from the presentation noun.
#[derive(Clone, Debug)]
pub struct EmptyState {
    pub message: String,
    pub help: Option<String>,
}

impl ListPlan {
    /// Build one family's plan from the index and declared slots.
    ///
    /// Flat rows keep `Index.records` order; grouped families emit one group
    /// row, its members, then orphan members. Family variation comes only from
    /// the representative presentation slug and the empty copy; there is no
    /// slug or handler match in this module.
    pub fn for_descriptor(
        descriptor: &TypeDescriptor,
        index: &Index,
        heading: &str,
        count_note: &str,
    ) -> ListPlan {
        let slots = descriptor_slots(descriptor);
        let mut columns = vec![
            Column {
                key: "id".to_string(),
                label: "ID".to_string(),
                class: "id-cell",
            },
            Column {
                key: "title".to_string(),
                label: "Title".to_string(),
                class: "title-cell",
            },
            Column {
                key: "status".to_string(),
                label: "Status".to_string(),
                class: "status-cell",
            },
        ];
        for slot in &slots {
            columns.push(Column {
                key: slot.key.clone(),
                label: slot.label.clone(),
                class: "slot-cell",
            });
        }
        columns.push(Column {
            key: "created".to_string(),
            label: "Created".to_string(),
            class: "created-cell",
        });
        columns.push(Column {
            key: "updated".to_string(),
            label: "Updated".to_string(),
            class: "updated-cell",
        });
        columns.push(Column {
            key: "actions".to_string(),
            label: "Actions".to_string(),
            class: "actions-cell",
        });

        let single_references =
            descriptor.single_reference_visibility() == crate::config::Visibility::Visible;
        let group_references =
            descriptor.group_reference_visibility() == crate::config::Visibility::Visible;
        let member_references =
            descriptor.member_reference_visibility() == crate::config::Visibility::Visible;
        let rows = if descriptor.id_shape == IdShape::Grouped {
            grouped_rows(index, &slots, group_references, member_references)
        } else {
            index
                .records
                .iter()
                .map(|record| row_for(record, &slots, single_references))
                .collect()
        };
        let empty = if index.records.is_empty() {
            Some(EmptyState {
                message: format!("No {} in this project.", routes::noun(descriptor, 0)),
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
                escape(&column.key),
                escape(&column.label)
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
            "<div class=\"table-scroll\" role=\"region\" aria-label=\"{}\" tabindex=\"0\">\
<table class=\"table list list--shared\" data-list-layout=\"shared\" aria-label=\"{}\">\
<thead><tr>{}</tr></thead><tbody>{}</tbody></table></div>",
            escape(&self.heading),
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
            self.columns.last().map(|column| column.key.as_str()),
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
            .map(|column| column.key.as_str())
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

/// Resolve a descriptor's columns: document slots come from the seed; the
/// backlog and decision surfaces keep their static presentation sets.
fn descriptor_slots(descriptor: &TypeDescriptor) -> Vec<crate::config::Slot> {
    if descriptor.category == CategoryKind::Document {
        validate_descriptor_slots(&descriptor.slots);
        descriptor.slots.clone()
    } else {
        let slots = routes::list_slots(routes::static_slug(descriptor));
        validate_slots(slots);
        slots
            .iter()
            .map(|(key, label)| crate::config::Slot {
                key: (*key).to_string(),
                label: (*label).to_string(),
            })
            .collect()
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

fn validate_descriptor_slots(slots: &[crate::config::Slot]) {
    for (index, slot) in slots.iter().enumerate() {
        assert!(
            !RESERVED_COLUMN_KEYS.contains(&slot.key.as_str()),
            "family slot key `{}` is a reserved structural column",
            slot.key
        );
        assert!(
            !slots[..index].iter().any(|earlier| earlier.key == slot.key),
            "family slot key `{}` is declared twice",
            slot.key
        );
    }
}

/// Grouped rows: the group row first, its members as children, then every
/// member whose group row is absent from the filtered index as a top-level row.
fn grouped_rows(
    index: &Index,
    slots: &[crate::config::Slot],
    group_references: bool,
    member_references: bool,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for group in &index.groups {
        let mut row = row_for(group, slots, group_references);
        if let Some(positions) = index.members.get(&group.id) {
            for position in positions {
                if let Some(member) = index.requirements.get(*position) {
                    row.children.push(row_for(member, slots, member_references));
                }
            }
        }
        rows.push(row);
    }
    let mut other: Vec<&Record> = index
        .other
        .iter()
        .filter_map(|position| index.requirements.get(*position))
        .collect();
    other.sort_by(|left, right| left.id.cmp(&right.id));
    for record in other {
        rows.push(row_for(record, slots, member_references));
    }
    rows
}

fn row_for(record: &Record, slots: &[crate::config::Slot], render_references: bool) -> Row {
    let id = record.id.clone();
    let href = format!("/doc/{id}");
    let title = if record.title.trim().is_empty() {
        id.clone()
    } else {
        record.title.clone()
    };
    let mut cells = vec![status_cell(record.status.as_deref())];
    for slot in slots {
        cells.push(slot_cell(&slot.key, record));
    }
    cells.push(date_cell(record, "created_date"));
    cells.push(date_cell(record, "updated_date"));
    Row {
        id: id.clone(),
        title,
        href: href.clone(),
        cells,
        labels: record.labels.clone(),
        references: record.refs.clone(),
        render_references,
        actions: vec![
            ("View".to_string(), href),
            ("Edit".to_string(), format!("/doc/{id}/edit")),
        ],
        children: Vec::new(),
    }
}

/// Status cell: a status tag when the record has a status, muted text when it
/// is unset (UI description §4.2).
fn status_cell(status: Option<&str>) -> String {
    match status.filter(|value| !value.is_empty()) {
        Some(status) => format!("<span class=\"row-status tag\">{}</span>", escape(status)),
        None => "<span class=\"muted\">(none)</span>".to_string(),
    }
}

/// One family slot value. This is a match on the slot key only — never on a
/// slug, a handler, or a template.
///
/// A document row reads its slot values from the record's field map: a
/// document field is never a list column of its own, and no document content
/// is read from a typed `Record` field.
fn slot_cell(key: &str, record: &Record) -> String {
    let value = if record.descriptor.category == CategoryKind::Document {
        match key {
            "group" => record.group_id.clone(),
            other => record.field_map.display_value(other),
        }
    } else {
        match key {
            "milestone" => record.milestone_id.clone(),
            "assignee" => record.assignee.clone(),
            "release" => record.release_id.clone(),
            "type" => record.decision_type.clone(),
            "superseded_by" => record.superseded_by.clone(),
            "group" => record.group_id.clone(),
            _ => None,
        }
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

/// Count note for a flat shared list (`3 milestones`, `1 milestone`, `0 milestones`).
///
/// The noun is the descriptor's plural for a document type and the static
/// Backlog/Decision noun otherwise (`routes::noun`).
pub fn count_note(descriptor: &TypeDescriptor, count: usize) -> String {
    format!("{count} {}", routes::noun(descriptor, count))
}

/// Count note for a grouped family (`1 group · 3 requirements`).
pub fn grouped_count_note(index: &Index) -> String {
    format!(
        "{} \u{b7} {}",
        count_phrase(index.groups.len(), "group", "groups"),
        count_phrase(index.requirements.len(), "requirement", "requirements")
    )
}

fn count_phrase(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
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
    if row.render_references && !row.references.is_empty() {
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

    fn generic_descriptor(slug: &str) -> TypeDescriptor {
        let mut cfg = Config::default();
        cfg.document_types.insert(
            slug.to_string(),
            crate::config::DocTypeEntry {
                prefix: "NOTE".to_string(),
                directory: "notes/".to_string(),
                enabled: true,
                statuses: Some(vec!["active".to_string(), "archived".to_string()]),
                default_status: None,
                ..Default::default()
            },
        );
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
            render_references: true,
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
        let keys: Vec<&str> = plan
            .columns
            .iter()
            .map(|column| column.key.as_str())
            .collect();
        assert_eq!(
            keys,
            [
                "id", "title", "status", "release", "created", "updated", "actions"
            ]
        );
        let labels: Vec<&str> = plan
            .columns
            .iter()
            .map(|column| column.label.as_str())
            .collect();
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
        let keys: Vec<&str> = plan
            .columns
            .iter()
            .map(|column| column.key.as_str())
            .collect();
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
        for descriptor in crate::config::SEED_DOCUMENT_TYPES.iter() {
            let slots: Vec<crate::config::Slot> = descriptor
                .slots
                .iter()
                .map(|slot| crate::config::Slot {
                    key: slot.key.to_string(),
                    label: slot.label.to_string(),
                })
                .collect();
            validate_descriptor_slots(&slots);
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
        assert!(
            html.contains(
                "<div class=\"table-scroll\" role=\"region\" aria-label=\"Milestones\" tabindex=\"0\">"
            ),
            "the narrow-width scroll container is keyboard focusable and labelled"
        );
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
                    key: "id".to_string(),
                    label: "ID".to_string(),
                    class: "id-cell",
                },
                Column {
                    key: "title".to_string(),
                    label: "Title".to_string(),
                    class: "title-cell",
                },
                Column {
                    key: "status".to_string(),
                    label: "Status".to_string(),
                    class: "status-cell",
                },
                Column {
                    key: "group".to_string(),
                    label: "Group".to_string(),
                    class: "slot-cell",
                },
                Column {
                    key: "created".to_string(),
                    label: "Created".to_string(),
                    class: "created-cell",
                },
                Column {
                    key: "updated".to_string(),
                    label: "Updated".to_string(),
                    class: "updated-cell",
                },
                Column {
                    key: "actions".to_string(),
                    label: "Actions".to_string(),
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
    fn user_defined_family_defaults_to_visible_references() {
        let note_descriptor = generic_descriptor("note");
        let mut note = record("NOTE-001", false, None);
        note.descriptor = note_descriptor.clone();
        note.labels = vec!["note-label".to_string()];
        note.refs = vec!["NOTE-REF-1".to_string()];
        let index = Index {
            records: vec![note],
            ..Index::default()
        };
        let plan = ListPlan::for_descriptor(
            &note_descriptor,
            &index,
            "Notes",
            &count_note(&note_descriptor, 1),
        );
        assert!(plan.rows[0].render_references);
        assert_eq!(
            plan.rows[0].references,
            vec!["NOTE-REF-1".to_string()],
            "the Row keeps the RecordView reference projection"
        );
        let html = plan.render();
        let start = html.find("data-row-id=\"NOTE-001\"").unwrap();
        let end = html[start..].find("</tr>").unwrap() + start;
        let row = &html[start..end];
        assert!(row.contains("<ul class=\"row-tags\"><li class=\"tag\">note-label</li></ul>"));
        assert!(row.contains("<ul class=\"row-refs\"><li>NOTE-REF-1</li></ul>"));
    }

    #[test]
    fn req_and_nfr_hide_references_in_both_roles_but_keep_labels() {
        for slug in ["req", "nfr"] {
            let descriptor = descriptor(slug);
            let mut group = record("REQ-001", true, None);
            group.descriptor = descriptor.clone();
            group.labels = vec!["group-label".to_string()];
            group.refs = vec!["REF-GROUP".to_string()];
            let mut member = record("REQ-001.001", false, Some("REQ-001"));
            member.descriptor = descriptor.clone();
            member.labels = vec!["member-label".to_string()];
            member.refs = vec!["REF-MEMBER".to_string()];
            let mut index = Index {
                groups: vec![group.clone()],
                requirements: vec![member.clone()],
                records: vec![group, member],
                ..Index::default()
            };
            index.members.insert("REQ-001".to_string(), vec![0]);
            let plan = ListPlan::for_descriptor(
                &descriptor,
                &index,
                "Requirements",
                &grouped_count_note(&index),
            );
            assert!(!plan.rows[0].render_references);
            assert!(!plan.rows[0].children[0].render_references);
            let html = plan.render();
            assert!(
                !html.contains("row-refs"),
                "{slug} hides every reference list"
            );
            assert!(!html.contains("REF-GROUP"));
            assert!(!html.contains("REF-MEMBER"));
            assert!(html.contains("<li class=\"tag\">group-label</li>"));
            assert!(html.contains("<li class=\"tag\">member-label</li>"));
        }
    }

    #[test]
    fn other_seed_families_keep_visible_references() {
        for (slug, presentation) in [("milestone", "milestone"), ("ui-mockup", "ui-mockup")] {
            let mut doc = record("X-001", false, None);
            doc.descriptor = descriptor(slug);
            doc.refs = vec!["REF-001".to_string()];
            let index = Index {
                records: vec![doc],
                ..Index::default()
            };
            let plan = ListPlan::for_descriptor(&descriptor(slug), &index, "List", "1 item");
            assert!(plan.rows[0].render_references, "{slug} keeps references");
            let html = plan.render();
            assert!(
                html.contains("<ul class=\"row-refs\"><li>REF-001</li></ul>"),
                "{slug} renders the reference block"
            );
            let _ = presentation;
        }
    }

    #[test]
    fn unset_status_is_muted_text_not_a_tag() {
        assert_eq!(
            status_cell(Some("open")),
            "<span class=\"row-status tag\">open</span>"
        );
        assert_eq!(status_cell(None), "<span class=\"muted\">(none)</span>");
        assert_eq!(status_cell(Some("")), "<span class=\"muted\">(none)</span>");
    }

    fn record(id: &str, is_group: bool, group_id: Option<&str>) -> Record {
        Record {
            id: id.to_string(),
            descriptor: descriptor("req"),
            field_map: crate::record::field_map::DocumentFieldMap::default(),
            is_group,
            title: format!("{id} title"),
            group_id: group_id.map(str::to_string),
            group_title: None,
            description: None,
            status: None,
            ordinal: None,
            milestone_id: None,
            milestone_title: None,
            assignee: None,
            labels: Vec::new(),
            dependencies: Vec::new(),
            acceptance_criteria: Vec::new(),
            comments: Vec::new(),
            release_id: None,
            release_title: None,
            decision: None,
            context: None,
            consequences: None,
            decision_type: None,
            superseded_by: None,
            superseded_title: None,
            refs: Vec::new(),
            doc: crate::record::document::Document {
                source: String::new(),
                front_matter: None,
                frontmatter: None,
                heading: None,
                preamble: Vec::new(),
                sections: Vec::new(),
            },
            stored_source: String::new(),
            path: std::path::PathBuf::new(),
            rel_path: String::new(),
            version: String::new(),
        }
    }

    fn grouped_index() -> Index {
        let first = record("REQ-001", true, None);
        let empty = record("REQ-002", true, None);
        let child_one = record("REQ-001.001", false, Some("REQ-001"));
        let child_two = record("REQ-001.002", false, Some("REQ-001"));
        let orphan = record("REQ-001.009", false, Some("REQ-999"));
        let mut index = Index {
            records: vec![
                first.clone(),
                empty.clone(),
                child_one.clone(),
                child_two.clone(),
                orphan.clone(),
            ],
            groups: vec![first, empty],
            requirements: vec![child_one, child_two, orphan],
            ..Index::default()
        };
        index.members.insert("REQ-001".to_string(), vec![0, 1]);
        index.other = vec![2];
        index
    }

    #[test]
    fn grouped_rows_are_one_body_with_children_and_a_top_level_orphan() {
        let index = grouped_index();
        let plan = ListPlan::for_descriptor(
            &descriptor("req"),
            &index,
            "Business requirements",
            &grouped_count_note(&index),
        );
        let html = plan.render();
        assert_eq!(html.matches("<tbody>").count(), 1);
        let order = [
            "REQ-001",
            "REQ-001.001",
            "REQ-001.002",
            "REQ-002",
            "REQ-001.009",
        ];
        let mut cursor = 0;
        for id in order {
            let marker = format!("data-row-id=\"{id}\"");
            let position = html[cursor..]
                .find(&marker)
                .unwrap_or_else(|| panic!("row {id} missing"));
            cursor += position + marker.len();
        }
        assert!(html.contains("<tr data-row-id=\"REQ-001.001\" data-child-of=\"REQ-001\">"));
        assert!(html.contains("<tr data-row-id=\"REQ-001.002\" data-child-of=\"REQ-001\">"));
        assert!(html.contains("<tr data-row-id=\"REQ-002\">"));
        assert!(
            !html.contains("data-row-id=\"REQ-001.009\" data-child-of"),
            "an unresolved member is a top-level row"
        );
        assert!(html.contains("<p class=\"count\">2 groups · 3 requirements</p>"));
    }

    #[test]
    fn grouped_count_note_handles_empty_and_singular_counts() {
        assert_eq!(
            grouped_count_note(&Index::default()),
            "0 groups · 0 requirements"
        );
        let index = Index {
            groups: vec![record("REQ-001", true, None)],
            requirements: vec![record("REQ-001.001", false, Some("REQ-001"))],
            ..Index::default()
        };
        assert_eq!(grouped_count_note(&index), "1 group · 1 requirement");
    }

    #[test]
    fn empty_message_uses_the_count_note_noun() {
        let note_descriptor = generic_descriptor("note");
        let plan = ListPlan::for_descriptor(
            &note_descriptor,
            &Index::default(),
            "Notes",
            &count_note(&note_descriptor, 0),
        );
        let html = plan.render();
        assert!(html.contains("<p class=\"count\">0 notes</p>"));
        assert!(html.contains("No notes in this project."));
    }

    #[test]
    fn decision_superseded_by_slot_renders_the_value_or_muted_text() {
        let mut superseded = record("ADR-001", false, None);
        superseded.descriptor = descriptor("adr");
        superseded.decision_type = Some("adr".to_string());
        superseded.superseded_by = Some("ADR-002".to_string());
        let mut open = record("BDR-001", false, None);
        open.descriptor = descriptor("bdr");
        open.decision_type = Some("bdr".to_string());
        let index = Index {
            records: vec![superseded, open],
            ..Index::default()
        };
        let plan = ListPlan::for_descriptor(&descriptor("adr"), &index, "Decisions", "2 decisions");
        let keys: Vec<&str> = plan
            .columns
            .iter()
            .map(|column| column.key.as_str())
            .collect();
        assert_eq!(
            keys,
            [
                "id",
                "title",
                "status",
                "type",
                "superseded_by",
                "created",
                "updated",
                "actions"
            ]
        );
        let html = plan.render();
        let start = html.find("data-row-id=\"ADR-001\"").unwrap();
        let end = html[start..].find("</tr>").unwrap() + start;
        let row = &html[start..end];
        assert!(row.contains("<td class=\"slot-cell\">adr</td>"));
        assert!(row.contains("<td class=\"slot-cell\">ADR-002</td>"));
        let start = html.find("data-row-id=\"BDR-001\"").unwrap();
        let end = html[start..].find("</tr>").unwrap() + start;
        let row = &html[start..end];
        assert!(row.contains("<td class=\"slot-cell\">bdr</td>"));
        assert!(row.contains("<td class=\"slot-cell\"><span class=\"muted\">(none)</span></td>"));
    }

    #[test]
    fn count_note_handles_singular_and_plural() {
        assert_eq!(count_note(&descriptor("milestone"), 0), "0 milestones");
        assert_eq!(count_note(&descriptor("milestone"), 1), "1 milestone");
        assert_eq!(count_note(&descriptor("milestone"), 3), "3 milestones");
        assert_eq!(count_note(&descriptor("rule"), 2), "2 business rules");
        let note = generic_descriptor("note");
        assert_eq!(count_note(&note, 2), "2 notes");
    }
}
