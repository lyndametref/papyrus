//! Shared record envelope: front matter, document, patch, store.

pub mod descriptor;
pub mod document;
pub mod edit;
pub mod frontmatter;
pub mod identity;
pub mod line_endings;
pub mod migrate;
pub mod patch;
pub mod relocate;
pub mod resolve;
pub mod scan;
pub mod store;
pub mod view;

// Convenience re-exports (non-ambiguous).
pub use descriptor::RecordDescriptor;
pub use descriptor::descriptors as record_descriptors;
pub use document::{Document, Section, parse as parse_document, render as render_document};
pub use frontmatter::FrontMatter;
pub use identity::*;
pub use patch::{Patch, PatchOp, apply_patch};
pub use store::{
    NormalizedRead, Store, archive, archive_content, delete, read as read_record, read_document,
    read_document_normalized, resolve_path, set_status, write_atomic,
};
pub use view::RecordView;

pub use relocate::{RelocationReport, move_folder, relocate, relocate_folder};

pub use store::RelocateError;

// Read-only diagnostic access to the retired-section table and reader rules.
pub(crate) use migrate::{
    SectionList, SectionValue, retired_sections, section_list, section_scalar,
};

#[cfg(test)]
mod read_path_guard_tests {
    use std::fs;
    use std::path::Path;

    const FORBIDDEN: &[&str] = &[
        "load_effective_config",
        "load_effective_value",
        "ArtifactRegistry::build",
        "load_cfg_for",
        "fn load_cfg(",
    ];

    fn production_source(source: &str) -> &str {
        // Only production code counts: helpers in the test module may build a
        // configuration and registry for fixtures.
        source.split("#[cfg(test)]").next().unwrap_or(source)
    }

    fn assert_no_reconstruction(path: &Path) {
        let source =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        let production = production_source(&source);
        for forbidden in FORBIDDEN {
            assert!(
                !production.contains(forbidden),
                "{} constructs configuration or a registry in the record read path: {}",
                path.display(),
                forbidden
            );
        }
    }

    #[test]
    fn record_module_builds_no_per_record_configuration_or_registry() {
        let record_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/record");
        let mut checked = 0usize;
        for entry in fs::read_dir(&record_dir).expect("record module directory") {
            let path = entry.expect("record module entry").path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                assert_no_reconstruction(&path);
                checked += 1;
            }
        }
        assert!(
            checked >= 10,
            "expected the record module sources, found {}",
            checked
        );
    }

    #[test]
    fn browser_record_read_helpers_build_no_configuration_or_registry() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/browser/records.rs");
        assert_no_reconstruction(&path);
    }
}
