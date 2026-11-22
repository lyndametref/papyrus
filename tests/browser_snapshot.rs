//! Source audit for the ADR-037 serving snapshot (TASK-274).
//!
//! The browser must serve from one immutable `ServingSnapshot`. These checks
//! fail the default test run if a per-request configuration load, a second copy
//! of the snapshot parts on `ServerState`, or a handler that bypasses the
//! request handle returns. AUD-148 F9 replaced the brittle substring counts
//! with symbol and block assertions.

use std::fs;
use std::path::PathBuf;

fn browser_source() -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browser/mod.rs"))
        .expect("browser module source")
}

/// The source of one named item, from its signature to its closing brace.
///
/// A symbol assertion: the check is scoped to the named function or struct
/// instead of counting a substring across the whole module.
fn item_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing item: {signature}"));
    let mut depth = 0usize;
    let mut opened = false;
    for (offset, character) in source[start..].char_indices() {
        match character {
            '{' => {
                depth += 1;
                opened = true;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if opened && depth == 0 {
                    return &source[start..start + offset + 1];
                }
            }
            _ => {}
        }
    }
    &source[start..]
}

#[test]
fn configuration_is_loaded_only_by_the_snapshot_builder() {
    let source = browser_source();
    assert!(
        source.contains("use crate::commands::load_effective_config;"),
        "the browser imports the shared configuration loader"
    );
    assert!(
        !source.contains("fn load_effective_config("),
        "the browser must not define its own configuration loader"
    );
    assert!(
        item_source(&source, "fn load_serving_snapshot(").contains("load_effective_config("),
        "the serving-snapshot builder is the startup and reload construction path"
    );
    for accessor in [
        "fn request_state(",
        "fn serving_snapshot(",
        "fn reload_if_changed(",
    ] {
        assert!(
            !item_source(&source, accessor).contains("load_effective_config("),
            "{accessor} must serve or reload through the builder, never load configuration directly"
        );
    }
}

#[test]
fn server_state_holds_one_swap_point_and_no_snapshot_part() {
    let source = browser_source();
    let block = item_source(&source, "struct ServerState {");
    assert!(
        block.contains("current: RwLock<Arc<ServingSnapshot>>"),
        "{block}"
    );
    assert!(!block.contains("registry: ArtifactRegistry"), "{block}");
    assert!(!block.contains("routes: BrowserRoutes"), "{block}");
    assert!(!source.contains("fn config(&self)"));

    assert!(
        source.contains("fn request_state(&self) -> RequestState<'_>"),
        "one request accessor builds the request handle"
    );
    assert!(
        item_source(&source, "fn handle_connection(").contains("state: &ServerState"),
        "the connection entry keeps the server state"
    );
    assert!(
        item_source(&source, "fn serve(").contains("state: Arc<ServerState>"),
        "the accept loop keeps the server state"
    );
}

#[test]
fn request_path_takes_one_handle() {
    let source = browser_source();
    assert!(
        item_source(&source, "fn handle_connection(").contains("state.request_state()"),
        "the connection entry builds one request handle"
    );
    for handler in ["fn handle_request(", "fn route(", "fn validate_host("] {
        assert!(
            item_source(&source, handler).contains("state: &RequestState<'_>"),
            "{handler} takes the request handle, not the server state"
        );
    }
    assert!(
        !source.contains("state: &'a ServerState"),
        "no borrowed server state is stored alongside the request snapshot"
    );
}
