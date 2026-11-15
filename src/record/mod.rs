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
pub use descriptor::DESCRIPTORS as RECORD_DESCRIPTORS;
pub use descriptor::RecordDescriptor;
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
