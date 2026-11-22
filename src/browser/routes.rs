//! Startup route, navigation, and presentation tables (ADR-0020).
//!
//! [`BrowserRoutes`] is derived once from the validated [`ArtifactRegistry`]
//! before the browser binds its listener. Every membership decision comes from
//! the registry. The static [`PRESENTATION`] table supplies only the Backlog
//! and Decision labels, headings, curated text, navigation order, and legacy
//! URL keys; every document presentation fact comes from its descriptor.

use crate::registry::{ArtifactCategory as CategoryKind, ArtifactRegistry, TypeDescriptor};

/// The three sidebar groups (REQ-06.003).
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

/// The route kind that owns a rendered page, for the width decision.
///
/// Width belongs to the route, not to a record type: a task detail or edit
/// page shares the `task` slug with the board but must stay `Capped`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageRoute {
    /// The backlog kanban board (`/?type=task` with its query variants).
    Board,
    /// Every other route: lists, detail, edit, inventory, errors.
    Other,
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
    /// List section heading.
    pub heading: &'static str,
    /// Lower-case noun used in empty-state copy.
    pub short: &'static str,
    /// Inventory note.
    pub note: &'static str,
    /// Legacy `?type=` value, when it differs from the slug.
    pub legacy_type: Option<&'static str>,
    /// Relative order inside the category.
    pub order: u32,
    /// Legacy list API paths kept as aliases of this type's list. Purely
    /// presentational: the table never decides whether a type exists.
    pub api_paths: &'static [&'static str],
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
    note: &'static str,
    legacy_type: Option<&'static str>,
    order: u32,
) -> Presentation {
    Presentation {
        slug,
        label,
        heading,
        short,
        note,
        legacy_type,
        order,
        api_paths: &[],
        aggregate_group: None,
        list_slots: &[],
    }
}

const fn with_api(mut entry: Presentation, api_paths: &'static [&'static str]) -> Presentation {
    entry.api_paths = api_paths;
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
                    "Decisions in this project.",
                    None,
                    200,
                ),
                &[("type", "Type"), ("superseded_by", "Superseded by")],
            ),
            &["/api/decisions"],
        ),
        NavGroup::Decisions,
    ),
];

/// Presentation lookup for a built-in slug.
pub fn presentation(slug: &str) -> Option<&'static Presentation> {
    PRESENTATION.iter().find(|entry| entry.slug == slug)
}

/// Static presentation entry for a backlog or decision descriptor.
fn presentation_for(descriptor: &TypeDescriptor) -> Option<&'static Presentation> {
    presentation(&descriptor.slug).or_else(|| {
        let group = match descriptor.category {
            CategoryKind::Decision => NavGroup::Decisions,
            CategoryKind::Backlog => NavGroup::Backlog,
            CategoryKind::Document => return None,
        };
        aggregate_slug(group).and_then(presentation)
    })
}

/// The static presentation slug that represents a non-document descriptor.
pub fn static_slug(descriptor: &TypeDescriptor) -> &str {
    presentation_for(descriptor)
        .map(|entry| entry.slug)
        .unwrap_or(&descriptor.slug)
}

/// Declared family slot columns `(key, label)` for a Backlog or Decision slug.
///
/// A document type's slots come from its descriptor, never this table.
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

/// Static heading for a backlog or decision surface.
pub fn static_heading(slug: &str) -> Option<&'static str> {
    presentation(slug).map(|entry| entry.heading)
}

/// Static lower-case noun for a backlog or decision surface.
pub fn static_short(slug: &str) -> Option<&'static str> {
    presentation(slug).map(|entry| entry.short)
}

/// Sidebar and inventory label derived from the descriptor.
pub fn label(descriptor: &TypeDescriptor) -> String {
    if descriptor.category == CategoryKind::Document {
        return capitalize_first(&descriptor.plural);
    }
    presentation_for(descriptor)
        .map(|entry| entry.label.to_string())
        .unwrap_or_else(|| pluralize(&title_case(&descriptor.slug)))
}

/// List heading and page title derived from the descriptor.
pub fn heading(descriptor: &TypeDescriptor) -> String {
    if descriptor.category == CategoryKind::Document {
        return capitalize_first(&descriptor.plural);
    }
    static_heading(&descriptor.slug)
        .map(str::to_string)
        .unwrap_or_else(|| label(descriptor))
}

/// Lower-case noun for count and empty-state copy.
pub fn short(descriptor: &TypeDescriptor) -> String {
    if descriptor.category == CategoryKind::Document {
        return descriptor.plural.clone();
    }
    static_short(&descriptor.slug)
        .map(str::to_string)
        .unwrap_or_else(|| title_case(&descriptor.slug).to_lowercase())
}

/// Singular or plural noun for a count and empty-state copy.
pub fn noun(descriptor: &TypeDescriptor, count: usize) -> String {
    noun_for_plural(&short(descriptor), count)
}

fn noun_for_plural(plural: &str, count: usize) -> String {
    if count == 1 {
        if let Some(singular) = plural.strip_suffix('s')
            && !plural.ends_with("ss")
        {
            return singular.to_string();
        }
        plural.to_string()
    } else if plural.ends_with('s') {
        plural.to_string()
    } else {
        format!("{plural}s")
    }
}

/// Inventory note derived from the descriptor.
pub fn inventory_note(descriptor: &TypeDescriptor) -> String {
    if descriptor.category == CategoryKind::Document {
        return descriptor
            .summary
            .clone()
            .unwrap_or_else(|| CONFIG_DEFINED_NOTE.to_string());
    }
    presentation_for(descriptor)
        .map(|entry| entry.note.to_string())
        .unwrap_or_else(|| CONFIG_DEFINED_NOTE.to_string())
}

/// Fixed generic inventory note for a type without a seed summary.
const CONFIG_DEFINED_NOTE: &str = "Config-defined document records in this project.";

/// Legacy `?type=` value for a descriptor (the slug itself when none).
pub fn legacy_url_key(descriptor: &TypeDescriptor) -> &str {
    if descriptor.category == CategoryKind::Document {
        return descriptor.type_alias.as_deref().unwrap_or(&descriptor.slug);
    }
    presentation_for(descriptor)
        .and_then(|entry| entry.legacy_type)
        .unwrap_or(&descriptor.slug)
}

/// Legacy list API aliases for a descriptor.
pub fn api_aliases(descriptor: &TypeDescriptor) -> Vec<&str> {
    if descriptor.category == CategoryKind::Document {
        return descriptor.api_aliases.iter().map(String::as_str).collect();
    }
    static_api_aliases(&descriptor.slug)
}

fn static_api_aliases(slug: &str) -> Vec<&str> {
    presentation(slug)
        .map(|entry| entry.api_paths.to_vec())
        .unwrap_or_default()
}

fn capitalize_first(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn pluralize(label: &str) -> String {
    match label.chars().last() {
        Some(character) if character.is_ascii_alphabetic() && character != 's' => {
            format!("{}s", label)
        }
        _ => label.to_string(),
    }
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

/// Number of leading built-in document cards rendered before the Backlog and
/// Decisions aggregate cards (presentation only).
pub const INVENTORY_AGGREGATE_AFTER: usize = 5;

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
        }
        if let Some(aggregate) = backlog_aggregate
            && backlog_items
                .iter()
                .any(|descriptor| descriptor.slug == aggregate)
        {
            nav.push(NavItem {
                slug: aggregate.clone(),
                label: NavGroup::Backlog.heading().to_string(),
                url_key: registry
                    .descriptor(&aggregate)
                    .map(|descriptor| legacy_url_key(descriptor).to_string())
                    .unwrap_or_else(|| aggregate.clone()),
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
        }
        if let Some(aggregate) = &decision_aggregate {
            type_values.insert(aggregate.clone(), aggregate.clone());
            nav_selection.insert(aggregate.clone(), aggregate.clone());
            nav.push(NavItem {
                slug: aggregate.clone(),
                label: NavGroup::Decisions.heading().to_string(),
                url_key: registry
                    .descriptor(aggregate)
                    .map(|descriptor| legacy_url_key(descriptor).to_string())
                    .unwrap_or_else(|| aggregate.clone()),
                group: NavGroup::Decisions,
            });
        }

        let mut documents: Vec<&TypeDescriptor> =
            registry.enabled(CategoryKind::Document).collect();
        documents.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.slug.cmp(&right.slug))
        });
        for descriptor in &documents {
            type_values.insert(descriptor.slug.clone(), descriptor.slug.clone());
            let legacy = legacy_url_key(descriptor);
            if legacy != descriptor.slug {
                type_values.insert(legacy.to_string(), descriptor.slug.clone());
            }
            nav_selection.insert(descriptor.slug.clone(), descriptor.slug.clone());
            nav.push(nav_item(descriptor));
            inventory.push(InventoryEntry {
                slug: descriptor.slug.clone(),
                title: label(descriptor),
                note: inventory_note(descriptor),
                url_key: legacy_url_key(descriptor).to_string(),
            });
        }

        for descriptor in registry.descriptors() {
            if !descriptor.enabled {
                continue;
            }
            let slug = descriptor.slug.clone();
            if descriptor.category == CategoryKind::Document {
                api_lists.insert(format!("/api/docs/{}", slug), slug.clone());
                for path in api_aliases(descriptor) {
                    api_lists.insert(path.to_string(), slug.clone());
                }
            } else if let Some(target) = match descriptor.category {
                CategoryKind::Decision => decision_aggregate.as_deref(),
                CategoryKind::Backlog | CategoryKind::Document => Some(slug.as_str()),
            } {
                for path in static_api_aliases(target) {
                    api_lists.insert(path.to_string(), target.to_string());
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

    /// The backlog board's slug, resolved from the Backlog nav group.
    ///
    /// The board is the only route whose descriptor handler is `Task`, so the
    /// aggregate Backlog group names the only uncapped surface.
    pub fn board_slug(&self) -> Option<&str> {
        self.nav
            .iter()
            .find(|item| item.group == NavGroup::Backlog)
            .map(|item| item.slug.as_str())
    }

    /// Server-side content-width decision for a route (ADR-0028 decision 3).
    ///
    /// `Full` is reserved for the backlog board route. Every other route,
    /// including task detail and edit pages that share the `task` slug, is
    /// `Capped`.
    pub fn page_width(&self, route: PageRoute) -> PageWidth {
        match route {
            PageRoute::Board => PageWidth::Full,
            PageRoute::Other => PageWidth::Capped,
        }
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
        label: label(descriptor),
        url_key: legacy_url_key(descriptor).to_string(),
        group: NavGroup::from_category(descriptor.category),
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
    fn page_width_is_full_only_for_the_board_route() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let routes = BrowserRoutes::build(&registry);
        assert_eq!(routes.board_slug(), Some("task"));
        assert_eq!(routes.page_width(PageRoute::Board), PageWidth::Full);
        assert_eq!(routes.page_width(PageRoute::Other), PageWidth::Capped);

        // The route identity cannot collide with a record type: a task detail
        // or edit page shares the `task` slug with the board and stays capped.
        assert_eq!(routes.board_slug(), Some("task"));
        assert_eq!(routes.page_width(PageRoute::Other), PageWidth::Capped);
    }

    #[test]
    fn reference_family_declares_one_slot() {
        assert_eq!(list_slots("milestone"), &[("release", "Release")]);
    }

    #[test]
    fn declared_slots_match_the_amended_adr_table_for_static_surfaces() {
        assert_eq!(
            list_slots("task"),
            &[("milestone", "Milestone"), ("assignee", "Assignee")]
        );
        assert_eq!(list_slots("release"), &[]);
        assert_eq!(
            list_slots("decision"),
            &[("type", "Type"), ("superseded_by", "Superseded by")]
        );
        assert_eq!(list_slots("note"), &[]);
    }

    #[test]
    fn presentation_holds_no_document_entry() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        for descriptor in registry.enabled(CategoryKind::Document) {
            assert!(
                presentation(&descriptor.slug).is_none(),
                "{} must not have a static presentation entry",
                descriptor.slug
            );
            assert_eq!(list_slots(&descriptor.slug), &[]);
        }
        for slug in [
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
        ] {
            assert!(presentation(slug).is_none(), "{slug} is a document type");
        }
    }

    #[test]
    fn document_presentation_facts_match_the_seed() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        for seed in crate::config::SEED_DOCUMENT_TYPES.iter() {
            let descriptor = registry.descriptor(seed.slug).expect(seed.slug);
            assert_eq!(descriptor.plural, seed.plural, "{} plural", seed.slug);
            assert_eq!(
                descriptor.summary.as_deref(),
                Some(seed.summary),
                "{} summary",
                seed.slug
            );
            assert_eq!(descriptor.order, seed.order, "{} order", seed.slug);
            assert_eq!(
                descriptor.type_alias.as_deref(),
                seed.type_alias,
                "{} type alias",
                seed.slug
            );
            assert_eq!(
                descriptor.api_aliases,
                seed.api_aliases
                    .iter()
                    .map(|alias| (*alias).to_string())
                    .collect::<Vec<_>>(),
                "{} api aliases",
                seed.slug
            );
            assert_eq!(
                label(descriptor),
                {
                    let mut chars = seed.plural.chars();
                    let first = chars.next().unwrap().to_uppercase().collect::<String>();
                    first + chars.as_str()
                },
                "{} label",
                seed.slug
            );
            assert_eq!(heading(descriptor), label(descriptor));
            assert_eq!(short(descriptor), seed.plural);
            assert_eq!(inventory_note(descriptor), seed.summary);
            assert_eq!(
                legacy_url_key(descriptor),
                seed.type_alias.unwrap_or(seed.slug)
            );
        }
    }

    #[test]
    fn seed_reference_visibility_matrix_is_resolved_per_role() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        for seed in crate::config::SEED_DOCUMENT_TYPES.iter() {
            let descriptor = registry.descriptor(seed.slug).expect(seed.slug);
            let expected = if seed.slug == "req" || seed.slug == "nfr" {
                crate::config::Visibility::Hidden
            } else {
                crate::config::Visibility::Visible
            };
            if let Some(group) = seed.group {
                assert_eq!(
                    descriptor.group_reference_visibility(),
                    group.reference_visibility,
                    "{} group",
                    seed.slug
                );
            }
            if let Some(member) = seed.member {
                assert_eq!(
                    descriptor.member_reference_visibility(),
                    member.reference_visibility,
                    "{} member",
                    seed.slug
                );
            }
            if seed.group.is_none() {
                assert_eq!(
                    descriptor.single_reference_visibility(),
                    expected,
                    "{} single",
                    seed.slug
                );
            }
        }
    }

    #[test]
    fn user_defined_type_gets_slug_derived_presentation_defaults() {
        let mut cfg = Config::default();
        cfg.document_types.insert(
            "note".to_string(),
            crate::config::DocTypeEntry {
                prefix: "NOTE".to_string(),
                directory: "notes/".to_string(),
                ..Default::default()
            },
        );
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let descriptor = registry.descriptor("note").expect("note");
        assert_eq!(descriptor.plural, "notes");
        assert_eq!(descriptor.summary, None);
        assert!(descriptor.order > 110);
        assert_eq!(descriptor.type_alias, None);
        assert!(descriptor.api_aliases.is_empty());
        assert!(descriptor.slots.is_empty());
        assert_eq!(
            descriptor.single_reference_visibility(),
            crate::config::Visibility::Visible
        );
        assert_eq!(label(descriptor), "Notes");
        assert_eq!(legacy_url_key(descriptor), "note");
    }

    #[test]
    fn inventory_and_navigation_follow_descriptor_order_with_slug_tie_break() {
        let cfg = Config::default();
        let registry = ArtifactRegistry::build(&cfg).expect("registry");
        let routes = BrowserRoutes::build(&registry);
        let inventory: Vec<&str> = routes
            .inventory
            .iter()
            .map(|entry| entry.slug.as_str())
            .collect();
        assert_eq!(
            inventory,
            [
                "req",
                "rule",
                "audit",
                "tech-spec",
                "use-case",
                "test-case",
                "test-report",
                "nfr",
                "glossary",
                "ui-spec",
                "ui-mockup",
            ]
        );
        let nav_documents: Vec<&str> = routes
            .nav
            .iter()
            .filter(|item| item.group == NavGroup::Documents)
            .map(|item| item.slug.as_str())
            .collect();
        assert_eq!(nav_documents, inventory);
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
