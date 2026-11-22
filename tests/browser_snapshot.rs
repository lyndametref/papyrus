//! Source audit for the ADR-037 serving snapshot (TASK-274).
//!
//! The browser must serve from one immutable `ServingSnapshot`. These checks
//! fail the default test run if a per-request configuration load, a second copy
//! of the snapshot parts on `ServerState`, or a handler that bypasses the
//! request handle returns.

use std::fs;
use std::path::PathBuf;

fn browser_source() -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browser/mod.rs"))
        .expect("browser module source")
}

#[test]
fn configuration_is_loaded_only_by_the_snapshot_builder() {
    let source = browser_source();
    let calls = source.matches("load_effective_config(").count();
    assert_eq!(
        calls, 1,
        "only the serving-snapshot builder may load configuration"
    );
    let builder = source
        .find("fn load_serving_snapshot(")
        .expect("snapshot builder exists");
    let next_item = source[builder..]
        .find("\n/// The fingerprint of the last failed reload")
        .expect("builder ends before the next item")
        + builder;
    assert!(
        source[builder..next_item].contains("load_effective_config("),
        "the reload is the startup construction path"
    );
}

#[test]
fn server_state_holds_one_swap_point_and_no_snapshot_part() {
    let source = browser_source();
    let start = source.find("struct ServerState {").expect("ServerState");
    let end = source[start..].find("\n}").expect("ServerState end") + start;
    let block = &source[start..end];
    assert!(
        block.contains("current: RwLock<Arc<ServingSnapshot>>"),
        "{block}"
    );
    assert!(!block.contains("registry: ArtifactRegistry"), "{block}");
    assert!(!block.contains("routes: BrowserRoutes"), "{block}");
    assert!(!source.contains("fn config(&self)"));
}

#[test]
fn request_path_takes_one_handle() {
    let source = browser_source();
    assert!(
        source.contains("fn request_state(&self) -> RequestState<'_>"),
        "one request accessor builds the request handle"
    );
    assert_eq!(
        source.matches("state: &ServerState").count(),
        1,
        "only the connection entry keeps the server state"
    );
    assert!(
        source.matches("state: &RequestState<'_>").count() > 20,
        "handlers, routing, validation, and rendering take the request handle"
    );
}
