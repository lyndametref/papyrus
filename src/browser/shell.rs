//! Page shell: the document skeleton every page renders inside (ADR-0028
//! decision 1).
//!
//! This module owns the head, header, navigation slot, `<main>`, and footer,
//! plus the server-side width class. It knows nothing about records,
//! descriptors, or lists; pages render a body fragment and pass it here.

use crate::browser::render::render_template;
use crate::browser::routes::PageWidth;

/// The shell template, embedded at compile time.
pub const SHELL_TEMPLATE: &str = include_str!("templates/shell.html");

/// One page wrapped in the shell.
#[derive(Clone, Copy)]
pub struct Page<'a> {
    /// Document title (`Papyrus — ...`).
    pub title: &'a str,
    /// Project name shown in the header.
    pub project: &'a str,
    /// Bind note shown in the header.
    pub bind_note: &'a str,
    /// Rendered navigation fragment.
    pub sidebar: &'a str,
    /// Rendered page body fragment.
    pub main: &'a str,
    /// Server-side content-width decision for the route.
    pub width: PageWidth,
    /// Navigation-toggle slot; empty until TASK-144 fills it.
    pub nav_toggle: &'a str,
}

/// The `class` attribute value for a page width.
pub fn width_class(width: PageWidth) -> &'static str {
    match width {
        PageWidth::Capped => "app",
        PageWidth::Full => "app app--full",
    }
}

/// Render a page inside the shell, exactly once.
pub fn render(page: Page<'_>) -> String {
    let app_class = width_class(page.width);
    render_template(
        SHELL_TEMPLATE,
        &[
            ("title", page.title),
            ("project", page.project),
            ("bind_note", page.bind_note),
            ("app_class", app_class),
        ],
        &[
            ("nav_toggle", page.nav_toggle),
            ("sidebar", page.sidebar),
            ("main", page.main),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(width: PageWidth) -> String {
        render(Page {
            title: "Papyrus \u{2014} Milestones",
            project: "Demo",
            bind_note: "127.0.0.1:6420",
            sidebar: "<nav id=\"site-nav\">nav</nav>",
            main: "<h1 class=\"h1\">Milestones</h1>",
            width,
            nav_toggle: "",
        })
    }

    #[test]
    fn width_classes_are_server_decided() {
        assert_eq!(width_class(PageWidth::Capped), "app");
        assert_eq!(width_class(PageWidth::Full), "app app--full");
    }

    #[test]
    fn shell_regions_and_values_render_once() {
        let html = page(PageWidth::Capped);
        assert_eq!(html.matches("<div class=\"app").count(), 1);
        assert_eq!(html.matches("<header").count(), 1);
        assert_eq!(html.matches("<nav").count(), 1);
        assert_eq!(html.matches("<main").count(), 1);
        assert_eq!(html.matches("<footer").count(), 1);
        assert_eq!(html.matches("Demo").count(), 1);
        assert_eq!(html.matches("127.0.0.1:6420").count(), 1);
        assert_eq!(html.matches("Milestones</h1>").count(), 1);
        assert_eq!(html.matches("nav</nav>").count(), 1);
        assert!(html.contains("class=\"app\""));
    }

    #[test]
    fn full_width_emits_the_reserved_class_without_css() {
        let html = page(PageWidth::Full);
        assert!(html.contains("<div class=\"app app--full\">"));
        assert!(!include_str!("app.css").contains(".app--full"));
    }

    #[test]
    fn escaped_values_are_escaped_once() {
        let html = render(Page {
            title: "<title>",
            project: "&",
            bind_note: "\"",
            sidebar: "",
            main: "",
            width: PageWidth::Capped,
            nav_toggle: "",
        });
        assert!(html.contains("&lt;title&gt;"));
        assert!(html.contains("<span class=\"project\">&amp;</span>"));
        assert!(html.contains("offline &middot; &quot;"));
    }
}
