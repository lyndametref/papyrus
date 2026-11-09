//! Shared record envelope: front matter, document, patch, store.

pub mod descriptor;
pub mod descriptors;
pub mod document;
pub mod frontmatter;
pub mod identity;
pub mod migrate;
pub mod patch;
pub mod store;
pub mod view;

// Convenience re-exports (non-ambiguous).
pub use descriptor::DESCRIPTORS as RECORD_DESCRIPTORS;
pub use descriptor::RecordDescriptor;
pub use document::{Document, Section, parse as parse_document, render as render_document};
pub use frontmatter::FrontMatter;
pub use identity::*;
pub use patch::{Patch, PatchOp, apply_patch};
pub use store::{Store, read as read_record, read_document, resolve_path, write_atomic};
pub use view::RecordView;
