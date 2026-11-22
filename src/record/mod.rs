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
    use std::path::{Path, PathBuf};

    /// Modules that own the configuration and registry lifecycle. Every other
    /// top-level `src/*.rs` module is a read/command module and must receive a
    /// ready `&Config` and `&ArtifactRegistry` instead of building them.
    const LIFECYCLE_MODULES: &[&str] = &[
        "commands.rs",
        "config.rs",
        "context.rs",
        "error.rs",
        "init.rs",
        "lib.rs",
        "main.rs",
        "registry.rs",
        "sanitize.rs",
    ];

    const FORBIDDEN: &[&str] = &[
        "load_effective_config",
        "load_effective_value",
        "ArtifactRegistry::build",
        "load_cfg_for",
        "fn load_cfg(",
    ];

    /// Only production code counts: a test module may build a configuration
    /// and registry for fixtures.
    fn production_source(source: &str) -> &str {
        source.split("#[cfg(test)]").next().unwrap_or(source)
    }

    /// Match a forbidden construction after removing all whitespace, so
    /// `ArtifactRegistry :: build` and `fn  load_cfg (` cannot slip through.
    fn violation(source: &str) -> Option<&'static str> {
        let production = production_source(source);
        let normalized: String = production.chars().filter(|c| !c.is_whitespace()).collect();
        FORBIDDEN.iter().copied().find(|forbidden| {
            let needle: String = forbidden.chars().filter(|c| !c.is_whitespace()).collect();
            normalized.contains(&needle)
        })
    }

    fn assert_no_reconstruction(path: &Path) {
        let source =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        if let Some(forbidden) = violation(&source) {
            panic!(
                "{} constructs configuration or a registry in a read path: {}",
                path.display(),
                forbidden
            );
        }
    }

    fn read_module_paths() -> Vec<PathBuf> {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut paths: Vec<PathBuf> = fs::read_dir(&src)
            .expect("src directory")
            .map(|entry| entry.expect("src entry").path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| !LIFECYCLE_MODULES.contains(&name))
            })
            .collect();
        paths.sort();
        paths
    }

    #[test]
    fn every_read_module_builds_no_per_record_configuration_or_registry() {
        let paths = read_module_paths();
        assert!(
            paths.len() >= 15,
            "expected the top-level read modules, found {}",
            paths.len()
        );
        for path in &paths {
            assert_no_reconstruction(path);
        }
        // The record module tree and the browser record reader stay covered.
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
        assert_no_reconstruction(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/browser/records.rs"),
        );
    }

    #[test]
    fn planted_violation_in_a_previously_unscanned_read_module_is_detected() {
        // `src/document.rs` is one of the read modules the former guard did not
        // scan. Plant the violation in a temporary copy of that module path.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join("read-path-guard");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("guard scratch directory");
        let path = root.join("document.rs");
        fs::write(
            &path,
            "pub fn read() {\n    let _ = crate::commands::load_effective_config;\n}\n",
        )
        .expect("planted module");
        let result = std::panic::catch_unwind(|| assert_no_reconstruction(&path));
        let _ = fs::remove_dir_all(&root);
        assert!(
            result.is_err(),
            "the guard must detect a violation in document.rs"
        );
    }

    #[test]
    fn guard_normalizes_whitespace_before_matching() {
        let planted = "pub fn read() {\n    let _ = ArtifactRegistry :: build;\n}\n";
        assert!(
            violation(planted).is_some(),
            "spaced construction must still be detected"
        );
        assert!(violation("pub fn read() {}\n").is_none());
    }
}
