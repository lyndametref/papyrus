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

/// The single navigation-toggle declaration (ADR-0028 decision 4).
///
/// The shell owns it, so every page carries the same control. `hidden` is in
/// the server output: without JavaScript the control cannot act, so it is not
/// offered. `/assets/nav-state.js` removes `hidden` for scripted clients.
pub const NAV_TOGGLE: &str = "<button class=\"nav-toggle\" type=\"button\" aria-expanded=\"true\" aria-controls=\"site-nav\" hidden>Navigation</button>";

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
    /// Raw internal head-asset fragment (math CSS and script), or `""`.
    pub assets: &'a str,
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
            ("nav_toggle", NAV_TOGGLE),
            ("asset_links", page.assets),
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
            assets: "",
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
    fn math_assets_default_to_nothing_and_are_raw_when_present() {
        let html = page(PageWidth::Capped);
        assert!(!html.contains("/assets/katex/"));
        let with_assets = render(Page {
            title: "Papyrus \u{2014} Math",
            project: "Demo",
            bind_note: "127.0.0.1:6420",
            sidebar: "<nav id=\"site-nav\">nav</nav>",
            main: "<main>math</main>",
            width: PageWidth::Capped,
            assets: "<link rel=\"stylesheet\" href=\"/assets/katex/katex.min.css\">",
        });
        assert!(with_assets.contains("/assets/katex/katex.min.css"));
        assert!(with_assets.find("app.css") < with_assets.find("katex.min.css"));
    }

    #[test]
    fn full_width_emits_the_reserved_class() {
        let html = page(PageWidth::Full);
        assert!(html.contains("<div class=\"app app--full\">"));
        assert!(include_str!("app.css").contains(".app--full"));
    }

    #[test]
    fn every_width_renders_one_hidden_nav_toggle() {
        for width in [PageWidth::Capped, PageWidth::Full] {
            let html = page(width);
            assert_eq!(html.matches("<button class=\"nav-toggle\"").count(), 1);
            assert_eq!(html.matches("</button>").count(), 1);
            assert!(html.contains("type=\"button\""));
            assert!(html.contains("aria-expanded=\"true\""));
            assert!(html.contains("aria-controls=\"site-nav\""));
            assert!(html.contains(">Navigation</button>"));
            assert!(html.contains("aria-controls=\"site-nav\" hidden>"));
            assert!(!html.contains("data-nav"));
        }
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
            assets: "",
        });
        assert!(html.contains("&lt;title&gt;"));
        assert!(html.contains("<span class=\"project\">&amp;</span>"));
        assert!(html.contains("offline &middot; &quot;"));
    }
}
