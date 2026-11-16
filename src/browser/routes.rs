//! Startup route, navigation, and presentation tables (ADR-0020).
//!
//! [`BrowserRoutes`] is derived once from the validated [`ArtifactRegistry`]
//! before the browser binds its listener. Every membership decision comes from
//! the registry; the static [`PRESENTATION`] table only supplies labels,
//! headings, curated text, navigation order, and legacy URL keys.

use crate::registry::{
    ArtifactCategory as CategoryKind, ArtifactRegistry, IdShape, TypeDescriptor,
};

/// The three sidebar/lifecycle groups (REQ-06.003).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavGroup {
    Backlog,
    Decisions,
    Documents,
}

impl NavGroup {
    pub fn heading(self) -> &'static str {
        match self {
            NavGroup::Backlog => "Backlog",
            NavGroup::Decisions => "Decisions",
            NavGroup::Documents => "Documents",
        }
    }

    pub fn from_category(category: CategoryKind) -> NavGroup {
        match category {
            CategoryKind::Backlog => NavGroup::Backlog,
            CategoryKind::Decision => NavGroup::Decisions,
            CategoryKind::Document => NavGroup::Documents,
        }
    }
}

/// Which 404 page an ID shape selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotFoundPage {
    TechSpec,
    UseCase,
    TestCase,
    TestReport,
    GenericDocument,
}

/// Server-side content-width decision for a route (ADR-0028 decision 3).
///
/// `Capped` is the default for every route. `Full` is reserved for the backlog
/// kanban board and the UI-mockup preview route; assigning it to any other
/// route requires an ADR amendment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageWidth {
    #[default]
    Capped,
    Full,
}

/// Static presentation data for a built-in slug. It never decides existence.
#[derive(Clone, Copy, Debug)]
pub struct Presentation {
    pub slug: &'static str,
    /// Sidebar and inventory label.
    pub label: &'static str,
    /// List and lifecycle section heading.
    pub heading: &'static str,
    /// Lower-case noun used in empty-state copy.
    pub short: &'static str,
    /// Curated one-line lifecycle description.
    pub lifecycle: &'static str,
    /// Inventory note.
    pub note: &'static str,
    /// Legacy `?type=` value, when it differs from the slug.
    pub legacy_type: Option<&'static str>,
    /// Relative order inside the category.
    pub order: u32,
    /// Legacy list API paths kept as aliases of this type's list. Purely
    /// presentational: the table never decides whether a type exists.
    pub api_paths: &'static [&'static str],
    /// Not-found page rendered for this type's ID shape.
    pub not_found: NotFoundPage,
    /// Category whose aggregate navigation entry this slug represents.
    pub aggregate_group: Option<NavGroup>,
    /// Declared family slot columns `(key, label)` for the shared list layout,
    /// in order. Empty for families with no declared slot.
    pub list_slots: &'static [(&'static str, &'static str)],
}

#[allow(clippy::too_many_arguments)]
const fn builtin(
    slug: &'static str,
    label: &'static str,
    heading: &'static str,
    short: &'static str,
    lifecycle: &'static str,
    note: &'static str,
    legacy_type: Option<&'static str>,
    order: u32,
) -> Presentation {
    Presentation {
        slug,
        label,
        heading,
        short,
        lifecycle,
        note,
        legacy_type,
        order,
        api_paths: &[],
        not_found: NotFoundPage::GenericDocument,
        aggregate_group: None,
        list_slots: &[],
    }
}

const fn with_api(mut entry: Presentation, api_paths: &'static [&'static str]) -> Presentation {
    entry.api_paths = api_paths;
    entry
}

const fn with_not_found(mut entry: Presentation, not_found: NotFoundPage) -> Presentation {
    entry.not_found = not_found;
    entry
}

const fn as_aggregate(mut entry: Presentation, group: NavGroup) -> Presentation {
    entry.aggregate_group = Some(group);
    entry
}

const fn with_slots(
    mut entry: Presentation,
    slots: &'static [(&'static str, &'static str)],
) -> Presentation {
    entry.list_slots = slots;
    entry
}

/// The built-in presentation table, in navigation order.
pub const PRESENTATION: &[Presentation] = &[
    as_aggregate(
        with_api(
            with_slots(
                builtin(
                    "task",
                    "Backlog",
                    "Tasks",
                    "tasks",
                    "Tasks move through the configured task statuses and are archived under the backlog archive folder.",
                    "Backlog tasks in this project.",
                    None,
                    100,
                ),
                &[("milestone", "Milestone"), ("assignee", "Assignee")],
            ),
            &["/api/tasks"],
        ),
        NavGroup::Backlog,
    ),
    with_api(
        with_slots(
            builtin(
                "milestone",
                "Milestones",
                "Milestones",
                "milestones",
                "Milestones move from open to closed.",
                "Milestones in this project.",
                None,
                110,
            ),
            &[("release", "Release")],
        ),
        &["/api/milestones"],
    ),
    with_api(
        builtin(
            "release",
            "Releases",
            "Releases",
            "releases",
            "Releases move from planned to released.",
            "Releases in this project.",
            None,
            120,
        ),
        &["/api/releases"],
    ),
    as_aggregate(
        with_api(
            with_slots(
                builtin(
                    "decision",
                    "Decisions",
                    "Decisions",
                    "decisions",
                    "Decisions move from proposed to accepted or rejected; an accepted decision may be deprecated or superseded.",
                    "Decisions in this project.",
                    None,
                    200,
                ),
                &[("type", "Type")],
            ),
            &["/api/decisions"],
        ),
        NavGroup::Decisions,
    ),
    with_slots(
        builtin(
            "req",
            "Business requirements",
            "Business requirements",
            "business requirements",
            "Requirements are active until archived; archive moves the file to archived folder.",
            "Business requirements in this project.",
            None,
            10,
        ),
        &[("group", "Group")],
    ),
    with_api(
        with_slots(
            builtin(
                "rule",
                "Business Rules",
                "Business Rules",
                "business rules",
                "Business rules are active until archived; archive moves the file to archived folder.",
                "Business rules in this project.",
                Some("business-rule"),
                20,
            ),
            &[("applicability", "Applicability")],
        ),
        &["/api/business-rules", "/api/business-rule"],
    ),
    with_api(
        with_slots(
            builtin(
                "audit",
                "Audit Records",
                "Audit Records",
                "audit records",
                "Audit records are active until archived; archive moves the file to archived folder.",
                "Audit records in this project.",
                Some("audit-record"),
                30,
            ),
            &[("audit_type", "Audit type")],
        ),
        &["/api/audit-records", "/api/audit-record"],
    ),
    with_not_found(
        with_api(
            builtin(
                "tech-spec",
                "Technical Specifications",
                "Technical Specifications",
                "technical specifications",
                "Technical specifications are active until archived; archive moves the file to archived folder.",
                "Technical specifications in this project.",
                None,
                40,
            ),
            &["/api/tech-specs", "/api/tech-spec"],
        ),
        NotFoundPage::TechSpec,
    ),
    with_not_found(
        with_api(
            builtin(
                "use-case",
                "Use Cases",
                "Use Cases",
                "use cases",
                "Use cases are active until archived; archive moves the file to archived folder.",
                "Use cases in this project.",
                None,
                50,
            ),
            &["/api/use-cases", "/api/use-case"],
        ),
        NotFoundPage::UseCase,
    ),
    with_not_found(
        with_api(
            builtin(
                "test-case",
                "Test Cases",
                "Test Cases",
                "test cases",
                "Test cases are active until archived; archive moves the file to archived folder.",
                "Test cases in this project.",
                None,
                60,
            ),
            &["/api/test-cases", "/api/test-case"],
        ),
        NotFoundPage::TestCase,
    ),
    with_not_found(
        with_api(
            builtin(
                "test-report",
                "Test Reports",
                "Test Reports",
                "test reports",
                "Test reports are active until archived; archive moves the file to archived folder.",
                "Test reports in this project.",
                None,
                70,
            ),
            &["/api/test-reports", "/api/test-report"],
        ),
        NotFoundPage::TestReport,
    ),
    with_slots(
        builtin(
            "nfr",
            "Non-functional requirements",
            "Non-functional requirements",
            "non-functional requirements",
            "Requirements are active until archived; archive moves the file to archived folder.",
            "Non-functional requirements in this project.",
            None,
            80,
        ),
        &[("group", "Group")],
    ),
    with_api(
        builtin(
            "glossary",
            "Glossary",
            "Glossary",
            "glossary terms",
            "Glossary terms are active until archived; archive moves the file to archived folder.",
            "Glossary terms in this project.",
            None,
            90,
        ),
        &["/api/glossary"],
    ),
    with_api(
        builtin(
            "ui-spec",
            "UI Specifications",
            "UI Specifications",
            "UI specifications",
            "UI specifications are active until archived; archive moves the file to archived folder.",
            "UI specifications in this project.",
            None,
            100,
        ),
        &["/api/ui-specs", "/api/ui-specifications", "/api/ui-spec"],
    ),
    with_api(
        with_slots(
            builtin(
                "ui-mockup",
                "UI Mockups",
                "UI Mockups",
                "UI mockups",
                "UI mockups move from pending to approved, then archived.",
                "UI mockups in this project.",
                None,
                110,
            ),
            &[("task", "Task"), ("specs", "Specs")],
        ),
        &["/api/ui-mockups", "/api/ui-mockup", "/api/mockups"],
    ),
];

/// Presentation lookup for a built-in slug.
pub fn presentation(slug: &str) -> Option<&'static Presentation> {
    PRESENTATION.iter().find(|entry| entry.slug == slug)
}

/// Declared family slot columns `(key, label)` for a slug.
///
/// A config-defined type has no presentation entry, therefore no slots.
pub fn list_slots(slug: &str) -> &'static [(&'static str, &'static str)] {
    presentation(slug)
        .map(|entry| entry.list_slots)
        .unwrap_or(&[])
}

/// The presentation slug that represents a category's aggregate list.
fn aggregate_slug(group: NavGroup) -> Option<&'static str> {
    PRESENTATION
        .iter()
        .find(|entry| entry.aggregate_group == Some(group))
        .map(|entry| entry.slug)
}

/// The presentation slug whose not-found template a page selector names.
pub fn not_found_slug(page: NotFoundPage) -> Option<&'static str> {
    PRESENTATION
        .iter()
        .find(|entry| entry.not_found == page)
        .map(|entry| entry.slug)
}

/// Legacy list API aliases for a presentation slug.
fn api_aliases(slug: &str) -> &'static [&'static str] {
    presentation(slug)
        .map(|entry| entry.api_paths)
        .unwrap_or(&[])
}

/// Legacy `?type=` value for a slug (the slug itself when none).
pub fn legacy_url_key(slug: &str) -> &str {
    presentation(slug)
        .and_then(|entry| entry.legacy_type)
        .unwrap_or(slug)
}

/// Number of leading built-in document cards rendered before the Backlog and
/// Decisions aggregate cards (presentation only).
pub const INVENTORY_AGGREGATE_AFTER: usize = 5;

/// Inventory card order for built-in document cards. It is presentation only;
/// membership still comes from the route table.
pub fn inventory_order() -> &'static [&'static str] {
    &[
        "req",
        "rule",
        "audit",
        "tech-spec",
        "nfr",
        "glossary",
        "ui-spec",
        "ui-mockup",
    ]
}

/// Sidebar label for any document slug.
pub fn label(slug: &str) -> String {
    presentation(slug)
        .map(|entry| entry.label.to_string())
        .unwrap_or_else(|| pluralize(&title_case(slug)))
}

fn pluralize(label: &str) -> String {
    let last = label.chars().last();
    match last {
        Some(character) if character.is_ascii_alphabetic() && character != 's' => {
            format!("{}s", label)
        }
        _ => label.to_string(),
    }
}

/// List heading for any document slug.
pub fn heading(slug: &str) -> String {
    presentation(slug)
        .map(|entry| entry.heading.to_string())
        .unwrap_or_else(|| label(slug))
}

/// Lower-case noun for empty-state copy.
pub fn short(slug: &str) -> String {
    presentation(slug)
        .map(|entry| entry.short.to_string())
        .unwrap_or_else(|| title_case(slug).to_lowercase())
}

/// Singular or plural noun for a count and empty-state copy.
///
/// The presentation table already declares the plural noun; a config-defined
/// type falls back to its lower-case slug (`notes` for `note`).
pub fn noun(slug: &str, count: usize) -> String {
    let plural = short(slug);
    if count == 1 {
        if plural.ends_with('s') && !plural.ends_with("ss") {
            plural[..plural.len() - 1].to_string()
        } else {
            plural
        }
    } else if plural.ends_with('s') {
        plural
    } else {
        format!("{plural}s")
    }
}

/// Curated or shared lifecycle description.
pub fn lifecycle_description(slug: &str) -> String {
    presentation(slug)
        .map(|entry| entry.lifecycle.to_string())
        .unwrap_or_else(|| {
            "Config-defined document records are active until archived; archive moves the file to archived folder."
                .to_string()
        })
}

/// Inventory note.
pub fn inventory_note(slug: &str) -> String {
    presentation(slug)
        .map(|entry| entry.note.to_string())
        .unwrap_or_else(|| "Config-defined document records in this project.".to_string())
}

fn title_case(slug: &str) -> String {
    slug.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One generated sidebar entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub slug: String,
    pub label: String,
    pub url_key: String,
    pub group: NavGroup,
}

/// One generated inventory card for a document type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryEntry {
    pub slug: String,
    pub title: String,
    pub note: String,
    pub url_key: String,
}

/// One enabled descriptor's prefix and ID shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdRule {
    pub prefix: String,
    pub id_shape: IdShape,
    pub not_found: NotFoundPage,
}

/// The immutable startup route table.
#[derive(Clone, Debug)]
pub struct BrowserRoutes {
    /// Canonical and legacy list API path -> slug.
    pub api_lists: std::collections::BTreeMap<String, String>,
    /// Canonical and legacy `?type=` value -> slug.
    pub type_values: std::collections::BTreeMap<String, String>,
    /// Sidebar entries for all three groups.
    pub nav: Vec<NavItem>,
    /// Document inventory cards.
    pub inventory: Vec<InventoryEntry>,
    /// Enabled descriptor ID rules for not-found selection.
    pub id_rules: Vec<IdRule>,
    /// Descriptor slug -> representative sidebar entry slug.
    pub nav_selection: std::collections::BTreeMap<String, String>,
    /// The aggregate slug that serves every enabled decision descriptor.
    pub decision_view: Option<String>,
    /// The list shown when no `?type=` value is given.
    pub default_type: String,
}

impl BrowserRoutes {
    /// Derive the route table from the validated startup registry.
    pub fn build(registry: &ArtifactRegistry) -> BrowserRoutes {
        let mut api_lists = std::collections::BTreeMap::new();
        let mut type_values = std::collections::BTreeMap::new();
        let mut nav = Vec::new();
        let mut inventory = Vec::new();
        let mut id_rules = Vec::new();
        let mut nav_selection = std::collections::BTreeMap::new();

        let mut backlog_items: Vec<&TypeDescriptor> =
            registry.enabled(CategoryKind::Backlog).collect();
        backlog_items.sort_by_key(|descriptor| presentation_order(descriptor));
        let mut decision_items: Vec<&TypeDescriptor> =
            registry.enabled(CategoryKind::Decision).collect();
        decision_items.sort_by_key(|descriptor| presentation_order(descriptor));

        // The Backlog and Decisions groups render one aggregate list (the task
        // board and the combined decision list). The presentation table names
        // the aggregate slug; the descriptor set still decides existence.
        let backlog_aggregate = aggregate_slug(NavGroup::Backlog)
            .map(str::to_string)
            .or_else(|| {
                backlog_items
                    .first()
                    .map(|descriptor| descriptor.slug.clone())
            });
        for descriptor in &backlog_items {
            type_values.insert(descriptor.slug.clone(), descriptor.slug.clone());
            if let Some(aggregate) = &backlog_aggregate {
                nav_selection.insert(descriptor.slug.clone(), aggregate.clone());
            }
            push_id_rule(descriptor, &mut id_rules);
        }
        if let Some(aggregate) = backlog_aggregate
            && backlog_items
                .iter()
                .any(|descriptor| descriptor.slug == aggregate)
        {
            nav.push(NavItem {
                slug: aggregate.clone(),
                label: NavGroup::Backlog.heading().to_string(),
                url_key: legacy_url_key(&aggregate).to_string(),
                group: NavGroup::Backlog,
            });
        }

        let decision_aggregate = if decision_items.is_empty() {
            None
        } else {
            aggregate_slug(NavGroup::Decisions)
                .map(str::to_string)
                .or_else(|| {
                    decision_items
                        .first()
                        .map(|descriptor| descriptor.slug.clone())
                })
        };
        for descriptor in &decision_items {
            type_values.insert(descriptor.slug.clone(), descriptor.slug.clone());
            if let Some(aggregate) = &decision_aggregate {
                nav_selection.insert(descriptor.slug.clone(), aggregate.clone());
            }
            push_id_rule(descriptor, &mut id_rules);
        }
        if let Some(aggregate) = &decision_aggregate {
            type_values.insert(aggregate.clone(), aggregate.clone());
            nav_selection.insert(aggregate.clone(), aggregate.clone());
            nav.push(NavItem {
                slug: aggregate.clone(),
                label: NavGroup::Decisions.heading().to_string(),
                url_key: legacy_url_key(aggregate).to_string(),
                group: NavGroup::Decisions,
            });
        }

        let mut documents: Vec<&TypeDescriptor> =
            registry.enabled(CategoryKind::Document).collect();
        documents.sort_by(|left, right| {
            presentation_order(left)
                .cmp(&presentation_order(right))
                .then_with(|| left.slug.cmp(&right.slug))
        });
        for descriptor in &documents {
            type_values.insert(descriptor.slug.clone(), descriptor.slug.clone());
            if let Some(entry) = presentation(&descriptor.slug)
                && let Some(legacy) = entry.legacy_type
            {
                type_values.insert(legacy.to_string(), descriptor.slug.clone());
            }
            nav_selection.insert(descriptor.slug.clone(), descriptor.slug.clone());
            nav.push(nav_item(descriptor));
            inventory.push(InventoryEntry {
                slug: descriptor.slug.clone(),
                title: label(&descriptor.slug),
                note: inventory_note(&descriptor.slug),
                url_key: legacy_url_key(&descriptor.slug).to_string(),
            });
            push_id_rule(descriptor, &mut id_rules);
        }

        for descriptor in registry.descriptors() {
            if !descriptor.enabled {
                continue;
            }
            let slug = descriptor.slug.clone();
            if descriptor.category == CategoryKind::Document {
                api_lists.insert(format!("/api/docs/{}", slug), slug.clone());
                for path in api_aliases(&slug) {
                    api_lists.insert((*path).to_string(), slug.clone());
                }
            } else if let Some(target) = match descriptor.category {
                CategoryKind::Decision => decision_aggregate.as_deref(),
                CategoryKind::Backlog | CategoryKind::Document => Some(slug.as_str()),
            } {
                for path in api_aliases(target) {
                    api_lists.insert((*path).to_string(), target.to_string());
                }
            }
        }

        let default_type = nav
            .iter()
            .find(|item| item.group == NavGroup::Documents)
            .map(|item| item.slug.clone())
            .unwrap_or_default();

        BrowserRoutes {
            api_lists,
            type_values,
            nav,
            inventory,
            id_rules,
            nav_selection,
            decision_view: decision_aggregate,
            default_type,
        }
    }

    /// Resolve a `?type=` value (alias or canonical) to an enabled slug.
    pub fn slug_for_type(&self, value: &str) -> Option<&str> {
        self.type_values.get(value).map(String::as_str)
    }

    /// Resolve an API path to its enabled slug.
    pub fn slug_for_api(&self, path: &str) -> Option<&str> {
        self.api_lists.get(path).map(String::as_str)
    }

    /// The sidebar entry selected for a record type or list value.
    pub fn nav_slug_for(&self, slug: &str) -> Option<&str> {
        self.nav_selection.get(slug).map(String::as_str)
    }

    /// The presentation slug that supplies a list's heading, count noun, and
    /// declared slots. Decision type aliases share the aggregate decisions
    /// presentation; every other family uses its own slug.
    pub fn list_slug_for(&self, slug: &str) -> String {
        if self.is_decision_view(slug) {
            self.nav_selection
                .get(slug)
                .cloned()
                .unwrap_or_else(|| slug.to_string())
        } else {
            slug.to_string()
        }
    }

    /// The aggregate decision view (every enabled decision descriptor).
    pub fn is_decision_view(&self, slug: &str) -> bool {
        let selected = self.nav_selection.get(slug).map(String::as_str);
        selected.is_some() && selected == self.decision_view.as_deref()
    }

    /// Server-side content-width decision for a route (ADR-0028 decision 3).
    ///
    /// Every route is `Capped` in this ticket. The backlog board is the only
    /// future `Full` surface (TASK-144); no other route may return `Full`
    /// without an ADR amendment.
    pub fn page_width(&self, _slug: &str) -> PageWidth {
        // TASK-144 owns the board's `Full` classification. No route is
        // uncapped in this ticket, and a new `Full` route needs an ADR
        // amendment.
        PageWidth::Capped
    }

    /// Choose the not-found page for an ID, if an enabled type claims it.
    pub fn not_found_page(&self, id: &str) -> Option<NotFoundPage> {
        self.id_rules
            .iter()
            .find(|rule| id_matches_rule(rule, id))
            .map(|rule| rule.not_found)
    }
}

fn presentation_order(descriptor: &TypeDescriptor) -> u32 {
    presentation(&descriptor.slug)
        .map(|entry| entry.order)
        .unwrap_or(10_000)
}

fn nav_item(descriptor: &TypeDescriptor) -> NavItem {
    NavItem {
        slug: descriptor.slug.clone(),
        label: label(&descriptor.slug),
        url_key: legacy_url_key(&descriptor.slug).to_string(),
        group: NavGroup::from_category(descriptor.category),
    }
}

fn push_id_rule(descriptor: &TypeDescriptor, rules: &mut Vec<IdRule>) {
    let not_found = presentation(&descriptor.slug)
        .map(|entry| entry.not_found)
        .unwrap_or(NotFoundPage::GenericDocument);
    if descriptor.prefix.is_empty() {
        return;
    }
    rules.push(IdRule {
        prefix: descriptor.prefix.clone(),
        id_shape: descriptor.id_shape,
        not_found,
    });
}

fn id_matches_rule(rule: &IdRule, id: &str) -> bool {
    match rule.id_shape {
        IdShape::Grouped => {
            crate::record::identity::parse_id_parts(id, &rule.prefix, true).is_some()
                || crate::record::identity::parse_id_parts(id, &rule.prefix, false).is_some()
        }
        IdShape::Dotted => {
            crate::record::identity::parse_id_parts(id, &rule.prefix, true).is_some()
        }
        IdShape::Plain => {
            crate::record::identity::parse_id_parts(id, &rule.prefix, false).is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn default_routes_are_registry_derived() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let routes = BrowserRoutes::build(&registry);

        let mut document_slugs: Vec<String> = registry
            .enabled(CategoryKind::Document)
            .map(|descriptor| descriptor.slug.clone())
            .collect();
        document_slugs.sort();
        let mut nav_documents: Vec<String> = routes
            .nav
            .iter()
            .filter(|item| item.group == NavGroup::Documents)
            .map(|item| item.slug.clone())
            .collect();
        nav_documents.sort();
        let mut inventory: Vec<String> = routes
            .inventory
            .iter()
            .map(|item| item.slug.clone())
            .collect();
        inventory.sort();
        let mut canonical_api: Vec<String> = routes
            .api_lists
            .iter()
            .filter(|(path, _)| path.starts_with("/api/docs/"))
            .map(|(_, slug)| slug.clone())
            .collect();
        canonical_api.sort();
        canonical_api.dedup();
        let canonical_types: Vec<String> = document_slugs
            .iter()
            .filter(|slug| routes.type_values.get(*slug) == Some(*slug))
            .cloned()
            .collect();

        assert_eq!(nav_documents, document_slugs);
        assert_eq!(inventory, document_slugs);
        assert_eq!(canonical_api, document_slugs);
        assert_eq!(canonical_types, document_slugs);
    }

    #[test]
    fn page_width_is_capped_for_every_enabled_slug() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let routes = BrowserRoutes::build(&registry);
        for descriptor in registry.descriptors() {
            if !descriptor.enabled {
                continue;
            }
            assert_eq!(
                routes.page_width(&descriptor.slug),
                PageWidth::Capped,
                "slug {}",
                descriptor.slug
            );
        }
        assert_eq!(routes.page_width(&routes.default_type), PageWidth::Capped);
        assert_eq!(routes.page_width(""), PageWidth::Capped);
    }

    #[test]
    fn reference_family_declares_one_slot() {
        assert_eq!(list_slots("milestone"), &[("release", "Release")]);
    }

    #[test]
    fn declared_slots_match_the_amended_adr_table() {
        assert_eq!(
            list_slots("task"),
            &[("milestone", "Milestone"), ("assignee", "Assignee")]
        );
        assert_eq!(list_slots("release"), &[]);
        assert_eq!(list_slots("decision"), &[("type", "Type")]);
        assert_eq!(list_slots("req"), &[("group", "Group")]);
        assert_eq!(list_slots("nfr"), &[("group", "Group")]);
        assert_eq!(list_slots("audit"), &[("audit_type", "Audit type")]);
        assert_eq!(list_slots("rule"), &[("applicability", "Applicability")]);
        assert_eq!(
            list_slots("ui-mockup"),
            &[("task", "Task"), ("specs", "Specs")]
        );
        assert_eq!(list_slots("note"), &[]);
    }

    #[test]
    fn sidebar_groups_are_ordered() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let routes = BrowserRoutes::build(&registry);
        let groups: Vec<NavGroup> = routes.nav.iter().map(|item| item.group).collect();
        let mut expected = Vec::new();
        for group in [NavGroup::Backlog, NavGroup::Decisions, NavGroup::Documents] {
            let count = groups.iter().filter(|value| **value == group).count();
            expected.extend(std::iter::repeat_n(group, count));
        }
        assert_eq!(groups, expected);
    }
}
