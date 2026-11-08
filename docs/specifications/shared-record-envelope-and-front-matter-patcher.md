# Feature Specification — Shared Record Envelope and Front-Matter Patcher

**Ticket:** TASK-63
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 1: a single shared record implementation inside the existing Rust crate so every managed record type has one front-matter parser, one lossless document envelope, one patch API, and one atomic store. This ticket implements the infrastructure required by REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, and NFR-04.003. It does not migrate the corpus (TASK-68, TASK-69), does not implement identity translation (TASK-64), and does not change any record command surface (TASK-65).

## Scope

### In scope

- A new `src/record/` module with four submodules:
  - `record::frontmatter` — detect, parse, patch, and serialize the leading YAML block; own the key registry (general keys plus per-type keys) from ADR-0007.
  - `record::document` — the lossless envelope: front matter plus ordered body units (optional heading line, preamble lines, `## ` sections with raw bodies).
  - `record::patch` — the only mutation API: a list of `PatchOp` values.
  - `record::store` — record location, safe path resolution, read, atomic write, and the ADR-0006 version token.
- Record descriptors for every managed record type: task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record. UI mockup HTML is excluded.
- Readers that accept both legacy files without front matter and files with front matter, using front matter first and section/heading fallback.
- Unit and integration tests for no-op byte identity, unknown-key preservation, unknown-section preservation, and both read shapes.
- No-drift grep checks for this stage (no `fs::write` on record paths, no `format!("---` outside the shared module, no ID formatting outside `record::identity` where applicable).

### Out of scope

- Identity translation between Papyrus display IDs and Backlog.md storage IDs — TASK-64.
- CLI option exposure for front-matter fields — TASK-65.
- Lazy migration of document record types — TASK-68.
- Lazy migration of tasks and milestones to the Backlog.md layout — TASK-69.
- Removal of legacy per-module parsers/writers and no-drift enforcement beyond this stage — TASK-70.
- Backlog.md round-trip verification — TASK-71.
- Browser UI changes and metadata projection — TASK-66, TASK-67, TASK-72.
- UI mockups and other HTML files — excluded by REQ-05.010 and ADR-0007 decision 11.
- Backlog.md `docs` and `drafts` records — out of scope (REQ-05.010).
- Corpus migration or any write that changes an existing record body.

## Behavior

### 1. Module layout and ownership

1. All record parsing, metadata formatting, and record writing live under `src/record/`. No other module may hand-format a `---` block, re-serialize an untouched block, or parse front matter with ad-hoc string splitting.
2. `record::frontmatter`, `record::document`, `record::patch`, and `record::store` are the only entry points used by later tasks.
3. `record::identity` is not part of this ticket; it is delivered by TASK-64. Descriptors may reserve storage-ID fields but must not implement translation here.
4. Every record module becomes a descriptor consumer in later tasks; this ticket only defines the descriptor shape and the shared implementation.

### 2. `record::frontmatter`

1. Detection: a leading YAML block is recognized only when the file's first byte is the start of a line containing exactly `---`, followed by a closing line containing exactly `---`. Nothing may precede the opening delimiter.
2. Parsing uses `yaml_serde`. The parse result preserves key order, raw scalar text, and unknown keys.
3. Typed accessors cover the general keys (`id`, `title`, `created_date`, `updated_date`, `labels`, `references`) and the per-type extras declared in the key registry from ADR-0007.
4. Serialization emits the canonical key order and Backlog-compatible quoting. General keys precede type-specific keys.
5. Patching rewrites only changed keys and preserves unknown keys, key order, and comments. A full document re-serialization that reorders or requotes foreign keys is prohibited.
6. Timestamps are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.
7. For tasks and milestones, YAML key order and quoting follow the Backlog.md-compatible serialization. `assignee` is a YAML list.

### 3. `record::document`

1. The envelope splits a file into:
   - an optional front-matter block;
   - an optional heading line (`# <ID> <title>`) where the record type keeps one;
   - preamble lines before the first `## ` section;
   - ordered `## ` sections, each retaining its heading line and raw body lines.
2. Section text and unknown content are kept as raw lines and are never re-rendered.
3. Every unmodified line is preserved byte-for-byte, including line endings as stored.
4. Tasks and milestones have no `# <ID> <title>` heading. All other Markdown record types keep a body title line.
5. Frozen ADR bodies keep their existing heading and sections; the envelope does not alter them.

### 4. `record::patch`

1. `PatchOp` supports at least:
   - `SetFrontMatter(key, value)`
   - `ClearFrontMatter(key)`
   - `SetSection(heading, value)`
   - `ClearSection(heading)`
   - `SetHeadingTitle(title)`
2. Applying a patch returns the new file text.
3. Applying an empty patch returns byte-identical text.
4. Applying a patch whose values equal the current values returns byte-identical text.
5. A patch that changes no byte does not result in a write; `updated_date` is therefore not refreshed by a no-op.
6. `record::patch` is the only mutation API. Lazy migration in later tasks must be expressed as patches through this API.

### 5. `record::store`

1. Provides safe path resolution using `Context::ensure_within` before any read or write.
2. Provides read of a record file, returning the document envelope and the ADR-0006 version token.
3. Provides atomic write using the ADR-0006 sequence: create a new temporary file in the same directory, write, `sync_all`, set permissions, rename over the target, and clean up on failure.
4. Provides the version token derived from file metadata as `seconds.nanos.length`.
5. No record module calls `fs::write` on a record path; all writes go through `record::store::write_atomic` or an equivalent store method.
6. Read accepts both legacy files without front matter and files with front matter. When front matter is present it is used first; otherwise the store falls back to section and heading parsing.
7. Read does not migrate or modify the file in this ticket. Migration is added by TASK-68 and TASK-69.

### 6. Record descriptors

1. A descriptor exists for each managed record type listed in scope.
2. Each descriptor declares:
   - record type name;
   - display ID pattern and directory;
   - type-specific front-matter keys and their order after the general keys;
   - known body sections and their order;
   - field validation rules.
3. UI mockup HTML is excluded; no descriptor claims `.html` mockup files.
4. Backlog.md `docs` and `drafts` records are not claimed by any descriptor.
5. Prose Markdown that no descriptor claims is never migrated or modified.

### 7. Error behavior

1. A file that cannot be read or whose path escapes the project directory produces a runtime error (exit code 1) consistent with existing commands.
2. A malformed front-matter block that cannot be parsed produces a validation error (exit code 4) rather than a panic.
3. A missing record produces a not-found error (exit code 3) consistent with existing commands.
4. Atomic write failures leave the original file byte-identical and remove any temporary file.
5. No partial write is reported as success.

## Inputs and outputs

- Input: a project `Context` and a record path or descriptor-claimed location.
- Output: a `Document` envelope (front matter plus body units) and a version token on read; new file text on patch; success or error on write.
- No user-facing command output is added by this ticket.

## Validation and error behavior

- Path escape or symlink traversal → exit code 1.
- Missing file → exit code 3.
- Malformed front matter or invalid YAML → exit code 4.
- IO failure during atomic write → exit code 1; original file unchanged.

## Assumptions

- Assumption: `yaml_serde` 0.10 remains the only YAML dependency; no new crate is added (ADR-0005).
- Assumption: the existing `Context::ensure_within` implementation is reused for path safety (NFR-04.002).
- Assumption: the existing `browser::records::write_atomic` sequence is the reference for atomic writes; `record::store` may wrap or reimplement it without changing the contract (ADR-0006).
- Assumption: no corpus migration occurs in this ticket; readers must tolerate both shapes, and migration is delivered later.
- Assumption: UI mockups remain HTML-only and are not represented by a record descriptor.
- Assumption: the single-crate layout is retained; `src/record/` is a module inside the existing crate (ADR-0004, ADR-0005).

## Open questions

- OQ-1: Whether `record::store` should expose a separate `read` function returning a borrowed document or an owned envelope; the implementation may choose, provided the byte-identity and fallback contracts hold.
- OQ-2: Whether descriptors are static constants or constructed values; either is acceptable as long as the no-drift checks and later tasks can consume them.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-63 |
| Requirements | REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, NFR-04.003 |
| ADRs | ADR-0006, ADR-0007, ADR-0009 |
| Related tickets | TASK-64, TASK-65, TASK-68, TASK-69, TASK-70 |
| Source | src/browser/records.rs (`parse_doc`, `apply_*_edit`, `write_atomic`, `version_token`), src/context.rs (`ensure_within`) |
| Tests | tests/browser_cli.rs, src/browser/records.rs unit tests |
