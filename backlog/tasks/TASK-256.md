---
id: TASK-256
title: "Implement GroupedMarkdown create-group, group semantics, and HTML mockup supersede"
created_date: '2026-11-20T17:41:57Z'
updated_date: '2026-11-22T13:53:26Z'
labels: []
references:
  - ADR-036
  - REQ-005.001
  - REQ-005.002
  - REQ-005.008
  - BR-009
  - BR-010
  - TECH-066
  - REQ-007.004
  - TC-120
  - TC-121
  - ADR-042
status: Done
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 40
documentation:
  - src/document.rs
  - src/requirement.rs
  - src/ui_mockup.rs
  - src/record/identity.rs
  - tests/req_cli.rs
  - tests/ui_mockup_cli.rs
type: implementation
---

## Description
## Context
`req` and `nfr` are the only `GroupedMarkdown` seed types and `ui-mockup` is the only `SingleHtmlMockup` seed type. `src/requirement.rs` owns `create_group` (line ~144), member `create` (~179), `edit` (~581), group validation (~235-311), and merged listing (~360-430). `src/ui_mockup.rs` owns the HTML metadata surface: `handle` (~10), create, edit, archive, delete, and supersede. TASK-255 makes one engine for the baseline verbs; this task adds the two model-specific surfaces without reintroducing per-type dispatch.

## Scope
- GroupedMarkdown:
  - `create-group` allocates a plain group ID with `record::identity::next_number`/`format_id`, writes the title and the `group.fields` values, and writes a group record with no dotted ID.
  - A member `create` requires `--group`; it validates an existing active group of the same type (`record::identity::is_group_id`, cross-type check through the registry, archived-group check) and rejects an unknown, archived, or cross-type group with one validation outcome and no write.
  - A member `create` allocates a dotted ID with `record::identity::next_seq_number`/`format_seq_id` and writes the engine-owned `group` front-matter key; the key is reserved and never a field.
  - Membership is immutable after creation: `edit` exposes no group option and no field can set or change the group link.
  - Archive and delete act per record and never cascade; a list merges group records and their members in group order then member order, excluding archived records.
- SingleHtmlMockup:
  - `create` requires the title, `--task`, and at least one `--spec`; `--spec` and `--ref` are repeatable; the record starts `pending`.
  - The fixed metadata keys are `status`, `task`, `specs`, `refs`, `references`, and `superseded_by`, read from and written to the HTML metadata comment; the HTML file is Papyrus-owned and written only through `record::store`.
  - The CLI `edit` verb keeps `--status` and `--superseded-by` exactly as today (ADR-042 ruling 3): approval is recorded with `edit --status approved`; `archive` and `supersede` remain the only writers of `archived`. The browser edit form offers only the title, `specs`, and `refs` controls, and the `status` verb remains absent.
  - `archive` sets `archived` and moves the file to the `archived/` subfolder; `supersede <old> <new>` sets the superseded record to `archived` and records the successor in `superseded_by`, leaving the successor unchanged; a superseded mockup refuses further transitions; no `status` verb exists.
  - The HTML metadata title follows the uniform contract (TASK-257).
- Update `tests/req_cli.rs`, `tests/ui_mockup_cli.rs`, `tests/document_dispatch.rs`, and the characterization replay. Add coverage for the nfr `story` field, which the current module ignores although REQ-005.002 lists it.

## Out of scope
The title contract and `--term` removal (TASK-257), config selectors (TASK-258), deleting modules and transitional command tables (TASK-259), help-golden regeneration (TASK-260), and the browser.

## Grounding
ADR-036 decisions 7, 9, 10, 20, 23, 27; ADR-042 ruling 3; TECH-066 sections 6-7; REQ-005.001, REQ-005.002, REQ-005.008; TC-120, TC-121.

## Definition of done
Grouped create-group, member create, membership immutability, no-cascade archive/delete, and merged listing are implemented and tested; the mockup fixed metadata, archive, and supersede behavior are implemented and tested; the title contract is applied to the HTML metadata; the mockup approval path follows ADR-042 ruling 3: CLI edit keeps --status and --superseded-by, the browser edit form has no status control, and no status verb exists; the suites and quality gates are clean.

## Acceptance Criteria
- [ ] #1 create-group and --group are exposed only for GroupedMarkdown (req and nfr); supersede is exposed only for SingleHtmlMockup (ui-mockup); SingleHtmlMockup exposes no status verb. (MS-034 AC-1, AC-6)
- [ ] #2 create-group allocates a plain group ID (prefix plus zero-padded sequence) and writes the title plus the group.fields values; it does not allocate a dotted member ID. (MS-034 AC-1; ADR-036 d9)
- [ ] #3 A member create requires --group and accepts only an existing active group of the same type; an unknown, archived, or cross-type group fails with one validation outcome and writes nothing. (MS-034 AC-1; ADR-036 d9)
- [ ] #4 A member create allocates a dotted ID under the group prefix and writes the engine-owned group front-matter key; membership is immutable after creation and no field or edit option changes it. (MS-034 AC-1; ADR-036 d9, d23)
- [ ] #5 Archiving or deleting a group changes no member file, status, or ID, and archiving or deleting a member changes no other record; a list merges group records then their members in group order and member order, and archived records are excluded. (MS-034 AC-1; TECH-066 section 6)
- [ ] #6 A ui-mockup create requires the title, --task, and at least one --spec; it starts at pending and writes the fixed metadata keys status, task, specs, refs, references, and superseded_by in the HTML metadata comment, with the HTML file written only by the single writer. (MS-034 AC-1; REQ-005.008)
- [ ] #7 ui-mockup archive sets archived and moves the file to the archived subfolder; ui-mockup supersede sets the superseded mockup to archived and records the successor in superseded_by while leaving the successor unchanged; a superseded mockup refuses further transitions; no status verb exists. (MS-034 AC-1, AC-6; ADR-036 d10)
- [ ] #8 The CLI ui-mockup edit verb keeps --status and --superseded-by unchanged: edit --status approved records approval; archive and supersede remain the only writers of archived; the browser edit form offers only title, specs, and refs, and no status verb exists. (ADR-042 ruling 3; REQ-005.008; MS-034 AC-1)
- [ ] #9 The ui-mockup HTML metadata title follows the uniform contract: present, single-line, non-empty, required on create, never clearable on edit. (MS-034 AC-2; REQ-005.008)
- [ ] #10 create-group derives its options from group.fields and a member create derives its options from member.fields; options follow the role sequence and each field verbs. (MS-034 AC-6; ADR-036 d20, d27)
- [ ] #11 Grouped and mockup suites pass: req and nfr CLI tests, ui-mockup CLI tests, document dispatch, and the characterization replay; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean. (DoD)
## Comments
- **task-refiner**: Grounding (grouped): src requirement.rs create_group (line 144), create (179), edit (581), group validation (235-311), and list (360-430); ID helpers in src record identity.rs (is_group_id, next_number, format_id, next_seq_number, format_seq_id). The seed roles are REQ_FIELDS for both req and nfr (src config.rs 339-362). nfr currently ignores --story because Kind::supports_story is false, while REQ-005.002 lists story for both roles; the field-driven engine will start writing it, so cover it with a test and record it as a corrected behavior.
- **task-refiner**: Grounding (mockup): src ui_mockup.rs handle (line 10), create, edit, archive, delete, and supersede; fixed metadata parsing and rendering; HTML file is Papyrus-owned. REQ-005.008 fixes create <TITLE> --task <TASK> --spec <SPEC-ID>, repeatable --spec and --ref, and metadata keys status, task, specs, refs, references, superseded_by.
- **planner**: ADR-042 ruling 3 clears the mockup approval blocker: the CLI edit verb keeps --status and --superseded-by exactly as today; the browser edit form continues to offer only title, specs, and refs; the status verb remains absent. TASK-256 stays one task and is not split: a split would add a dependency edge to TASK-259 and change ordinals, which the split condition forbids.
- **rust-dev**: TASK-256 evidence: branch task-TASK-256 (working tree, not committed per orchestrator instruction). GroupedMarkdown: create-group keeps a plain-ID group record with title plus group.fields sections; member create requires --group, resolves only an existing active same-type group, rejects missing, unknown, archived, and cross-type with one exit-4 validation and no write; dotted ID via next_seq_number and format_seq_id; the reserved front-matter group key is written by the engine (no legacy ## Group section, no field or edit option, immutable); archive and delete resolve plain group IDs via a store fallback and never cascade; list merges groups then members by ID and excludes archived. nfr now writes --story for both roles. SingleHtmlMockup: kept ui_mockup handle because the engine patch path cannot reproduce the HTML metadata block byte-for-byte; create requires title, --task, and at least one --spec and starts pending; the fixed metadata block carries status, task, specs, and refs comments plus references and superseded_by visible lines; archive and supersede remain the only writers of archived and edit --status archived is rejected; supersede leaves the successor unchanged; no status verb; fields: empty. Browser records now read the front-matter group key with a legacy ## Group fallback. Deltas: characterization req create, req create-json, nfr create, nfr create-json store the group as front matter (documented in EXPECTED-DELTAS item 7); help goldens verb-req-create, verb-nfr-create, err-missing-required change because --group is engine-validated (exit 4) instead of clap-required; search tests updated because a member no longer embeds the group title. Checks: env -u EDITOR -u VISUAL cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check all clean. TCs: TC-120 and TC-121 covered in tests req_cli.rs and tests ui_mockup_cli.rs; engine integration coverage added in tests document_dispatch.rs.
- **orchestrator**: Acceptance met: create-group plain IDs with engine-validated active same-type group, dotted members, engine-owned group link, no cascade, merged order; mockup fixed metadata with archive and supersede, CLI status path preserved, no status verb; group-link storage delta documented as EXPECTED-DELTAS item 7; suite and characterization green.
