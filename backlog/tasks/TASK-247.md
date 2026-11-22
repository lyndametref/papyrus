---
id: TASK-247
title: "Add FieldSpec, DocumentModel, and the extended document-type entry"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-22T10:11:07Z'
labels: []
references:
  - ADR-036
  - ADR-0018
  - TECH-070
  - TECH-077
  - REQ-07.002
  - REQ-07.004
  - TC-102
  - TC-105
  - TC-106
  - TC-107
  - TC-108
  - TC-130
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-246
ordinal: 40
documentation:
  - src/config.rs
  - src/registry.rs
  - docs/specifications/technical/TECH-070.md
  - docs/specifications/technical/TECH-077.md
type: implementation
---

## Description
Context: MS-033 start commit c3103fc. The per-type content model and the minimal entry are still in place:
- `src/registry.rs::ContentModel` is `Generic | Typed(&'static TypedSchema) | HtmlMockup(&'static TypedSchema)`. Per-type fields live in `DOCUMENT_TEMPLATES` (`RecordFacts`, `frontmatter_extra`, `TemplateStatus`, `TemplateVerbs`) and in the nine typed modules. Consumers keyed on `ContentModel`: `src/browser/list.rs:153` (`Generic` hides list references), `src/browser/records.rs::load_record`, `src/document.rs`, and `src/cli/dispatch.rs`.
- `src/config.rs::DocTypeEntry` holds only `prefix`, `directory`, `enabled`, `statuses`, `default_status`; `DEFAULT_DOCUMENT_TYPES` (`src/config.rs:101`) is a four-field tuple table.
- ADR-036 decisions 5 to 27 replace this with one `DocumentModel`, one `FieldSpec` vocabulary, and one extended entry.

Scope:
1. Replace `ContentModel` with `DocumentModel { SingleMarkdown, GroupedMarkdown, SingleHtmlMockup }` and remove every `ContentModel` identifier from `src/`. Each consumer keeps today's behaviour: `SingleHtmlMockup` keeps the HTML-mockup path, `GroupedMarkdown` keeps the grouped path, and `SingleMarkdown` keeps both the typed and the generic path until TASK-249 derives the seed.
2. Add the field vocabulary: `FieldKind { Text, Multiline, List }`, a verb set restricted to `create` and `edit` with default `[create, edit]`, `FieldSpec { key, heading, frontmatter_key, option, label, kind, required, verbs, help }`, `Visibility { Visible, Hidden }`, `RoleFields { fields: Option<Vec<FieldSpec>>, reference_visibility: Option<Visibility> }`, and a `Slot` pair (`key`, `label`) matching `browser::routes::PRESENTATION` today.
3. Extend `DocTypeEntry` with `label`, `plural`, `order`, `summary`, `type_alias`, `reference_visibility`, `slots`, `api_aliases`, `fields`, `group`, and `member`. Every new member is optional: `None` means the key is absent and `Some(vec![])` means a present empty collection, so `fields: []` stays distinguishable from an absent `fields` (ADR-036 decision 32). `prefix`, `directory`, `enabled`, `statuses`, and `default_status` stay as they are.
4. Express the field rules in the type: a field has no `order` and no `enabled` member; `heading` and/or `frontmatter_key` is required; the reserved keys are `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`; field keys are unique within a role; `model`, `title`, `content`, and `id_shape` are never entry keys. Deny unknown keys at deserialization so TASK-248 can turn each into a `document_types.<slug>.<path>` diagnostic.
5. Keep the model seed-only: `req` and `nfr` resolve to `GroupedMarkdown`, `ui-mockup` to `SingleHtmlMockup`, every other seed slug to `SingleMarkdown`, and a slug with no seed entry resolves to `SingleMarkdown`. The seed table itself is TASK-249.
6. Keep `Config::default()` yielding the eleven seed slugs with today's prefix, directory, enabled flag, statuses, and default status, and keep the existing `DocTypeEntry { .. }` literals compiling; adding `..Default::default()` in tests is allowed.
7. Preserve behaviour at the former `Generic` call sites by using the existing `crate::registry::is_builtin_document_slug`: `src/browser/list.rs` keeps hiding list references for a non-seed slug, and `src/cli/dispatch.rs` keeps routing a non-seed slug to the generic document module. MS-034 and MS-035 replace these two checks with the seed-derived model and `reference_visibility`.

Invariants:
- Old configuration files must parse to the same effective document-type descriptor set: no entry's effective prefix, directory, enabled flag, statuses, default status, ID shape, verb set, section heading, or front-matter key changes in this task (MS-033 AC-1).
- No CLI or browser behaviour changes in this milestone (MS-033 AC-7). In particular the user-defined reference-visibility rule is still hidden until MS-035 applies the seed default, which is the documented expected delta in REQ-06.019 and TC-185.

Out of scope: parsing, validation messages, fill and merge, emission, selector paths, seed values, the document engine, and deleting the typed modules. Those belong to TASK-248, TASK-249, and MS-034. No dependency, ordinal, milestone, or assignee changes.

Assumptions:
- The field verb set can be an owned two-element set; only `create` and `edit` exist (ADR-036 decision 19).
- `order` is an integer and keeps the existing `PRESENTATION` values (10, 20, 30, ... 110) so a later milestone can adopt them without reordering anything.
- `api_aliases` and `slots` are ordinary declarative sequences in the file; ADR-0024 keeps them out of the comma-separated surface.
- `FieldSpec` carries no short option: the current `-d` shorts on `description` are retained by the per-type argument metadata until MS-034, and no short option is added or removed here.

Definition of done: `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean; `rg ContentModel src/` is empty; the new entry deserializes every documented key and rejects every unknown one; the eleven default slugs, prefixes, directories, and statuses are unchanged.
## Acceptance Criteria
- [ ] `DocumentModel` has exactly `SingleMarkdown`, `GroupedMarkdown`, and `SingleHtmlMockup`, and no `ContentModel` identifier remains under the source tree (ADR-036 decision 5, REQ-07.004).
- [ ] `FieldSpec` has exactly `key`, `heading`, `frontmatter_key`, `option`, `label`, `kind`, `required`, `verbs`, and `help`, and no `order`, `enabled`, or short-option member; a field item declaring `order` or `enabled` fails deserialization (ADR-036 decision 11, REQ-07.002).
- [ ] `kind` accepts exactly `text`, `multiline`, and `list`; `verbs` defaults to `[create, edit]` and rejects any other verb; `reference_visibility` accepts exactly `visible` and `hidden` (ADR-036 decisions 17 and 19, REQ-07.002).
- [ ] `DocTypeEntry` deserializes `fields`, `group.fields`, `member.fields`, `group.reference_visibility`, `member.reference_visibility`, `statuses`, `default_status`, `slots`, `api_aliases`, `label`, `plural`, `order`, `summary`, `type_alias`, and `reference_visibility`; an absent key is `None` and `fields: []` is `Some([])`, so the two are distinguishable (ADR-036 decision 32, TC-107).
- [ ] `DocTypeEntry` and `FieldSpec` reject unknown keys at deserialization, including entry-level `model`, `title`, `content`, and `id_shape` and field-level `order` and `enabled` (ADR-036 decision 34, TC-130).
- [ ] The model is seed-only: `req` and `nfr` resolve to `GroupedMarkdown`, `ui-mockup` to `SingleHtmlMockup`, the other eight seed slugs to `SingleMarkdown`, and a slug with no seed entry to `SingleMarkdown` (ADR-036 decisions 6 and 31, REQ-07.004).
- [ ] `Config::default()` still yields the eleven seed slugs with the prefix, directory, enabled flag, statuses, and default status of the start commit, and every existing `DocTypeEntry { .. }` construction site compiles (REQ-07.002).
- [ ] Behaviour is unchanged for the former `Generic` call sites: the browser list hides references for a non-seed slug exactly as before and CLI dispatch routes a non-seed slug to the generic document module, both via `registry::is_builtin_document_slug` (MS-033 AC-7).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounded in ContentModel, DOCUMENT_TEMPLATES, DocTypeEntry and the four ContentModel call sites; 9 testable acceptance criteria added; TECH-070 and TECH-077 documented. A transitional is_builtin_document_slug bridge keeps browser list and CLI dispatch behaviour identical. No dependency, ordinal, milestone, or assignee change.
- **rust-dev**: Changed: registry DocumentModel enum with SingleMarkdown, GroupedMarkdown, SingleHtmlMockup replaces the former content model; TypeDescriptor now carries document_model plus a transitional schema field so typed and generic paths keep today's behaviour; DocumentModel::for_slug is seed-only (req and nfr grouped, ui-mockup HTML, other seed and user-defined slugs single markdown); Config FieldSpec, FieldKind, FieldVerb, Visibility, RoleFields, Slot and extended DocTypeEntry with optional fields, group, member, slots, api_aliases, reference_visibility, label, plural, order, summary, type_alias; deny_unknown_fields rejects entry model, title, content, id_shape and field order, enabled; reserved field key constants and helpers; is_config_defined_document bridge uses is_builtin_document_slug for browser list and CLI dispatch.
- **rust-dev**: Tests: new config unit tests cover the exact FieldSpec key set and defaults, field order and enabled rejection, kind and verb value sets, visibility values, all eight reserved keys as key and frontmatter_key, nested group and member blocks with absent versus empty collections, full presentation and slot deserialization, and entry behaviour-key rejection; new registry test covers seed-only model resolution, user-defined SingleMarkdown default, and the seed bridge. Verification: cargo test 1155 passed, 0 failed, 26 ignored; characterization replay 3 of 3, list characterization replay 3 of 3, cli help golden 2 of 2; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. No commit made per instruction; status left In-Progress.
- **orchestrator**: Acceptance met: DocumentModel replaces ContentModel, FieldSpec vocabulary with reserved keys and verb defaults, nested group and member role blocks with reference visibility, extended entry with deny_unknown_fields; 1155 tests pass, characterization and help goldens unchanged, clippy clean.
