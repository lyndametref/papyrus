//! TASK-240 / MS-032 AC-2: capture the pre-refactor behaviour corpus.
//!
//! This test is **not** part of the normal suite. It replays the committed
//! `project/` seed through every case in `cases.json` and writes the expected
//! tree under `tests/fixtures/characterization/corpus/expected/`. It was run
//! against the pre-refactor binary and is regenerated only from an unchanged
//! behaviour baseline.
//!
//! Reproduction:
//!
//! ```text
//! PAPYRUS_CAPTURE_COMMIT=$(git rev-parse HEAD) \
//!   cargo test --test characterization_capture -- --ignored --nocapture
//! ```

#[path = "support/characterization.rs"]
mod characterization;

use characterization::{Ctx, Runner, corpus_root};

#[test]
#[ignore = "characterization capture: writes the expected tree from the seed project"]
fn capture_characterization_corpus() {
    let commit = std::env::var("PAPYRUS_CAPTURE_COMMIT")
        .expect("PAPYRUS_CAPTURE_COMMIT must name the commit the snapshot is captured from");
    assert!(
        !commit.trim().is_empty(),
        "PAPYRUS_CAPTURE_COMMIT must not be empty"
    );
    let root = corpus_root();
    let ctx = Ctx::new("capture");
    Runner {
        root,
        ctx: &ctx,
        write: true,
        only: None,
    }
    .run();
    println!("captured characterization corpus at commit {commit}");
}
