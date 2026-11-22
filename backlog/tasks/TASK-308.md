---
id: TASK-308
title: "Seed the default status on document create for every model"
created_date: '2026-11-22T14:19:46Z'
updated_date: '2026-11-22T14:20:16Z'
labels: []
references:
  - REQ-05.011
  - TECH-042
  - TC-138
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-257
ordinal: 65
documentation:
  - src/document.rs
  - src/record/document.rs
  - src/ui_mockup.rs
  - tests/glossary_cli.rs
  - tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md
  - tests/fixtures/characterization/corpus/cases.json
  - tests/characterization_replay.rs
  - tests/characterization_capture.rs
  - tests/fixtures/characterization/corpus
type: implementation
---

## Description
## Context
MS-034 execution found a requirement violation in the field-driven create path: a user-defined type writes a `status` front-matter key (the corpus `note` record has `status: active`), but seed types do not. The captured baseline has no `status` key in `expected/req/create/record.md`, `expected/rule/create/record.md`, or `expected/glossary/create/record.md`. REQ-05.011 and TECH-042 section 7.4 require a new document record to take the type's configured `default_status`, or `active` when none is declared; `SingleHtmlMockup` declares none and starts `pending`. TC-138 expects `status: active` on a created glossary record, and MS-034 AC-1 requires baseline verb behaviour, so the current baseline itself violates the requirement.

## Scope
- In the field-driven create and create-group path (`src/document.rs`, `src/record/document.rs`), write `status` into front matter for every Markdown model using the type's configured `default_status`, or `active` when the type declares none.
- `SingleHtmlMockup` keeps `pending` in its HTML metadata; no other model writes `pending`.
- No create or create-group option supplies status.
- Existing records without a `status` key are not migrated and keep their current behaviour.
- Update the characterization expectations for every create, create-group, and create-json case through the documented capture path (`PAPYRUS_CAPTURE_COMMIT=... cargo test --test characterization_capture -- --ignored`), and record the delta in `tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md`.
- Align the TC-138 expectation: the approved test case already expects `status: active`; implement the path so TC-138 passes and leave the approved case unchanged unless a wording mismatch remains.

## Out of scope
Status verb behaviour, archive and delete behaviour, config selectors (TASK-258), typed module deletion (TASK-259), help-golden regeneration (TASK-260), and migration of existing records.

## Grounding
REQ-05.011; TECH-042 section 7.4; MS-034 AC-1; TC-138; corpus `expected/req|rule|glossary/create/record.md`.

## Definition of done
Every create, create-group, and create-json fixture records the expected status; `list`, `view`, and `json` show the status written at create; the status verb still changes a document's status; the characterization replay passes with the updated expectations; `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; the delta is documented.
## Acceptance Criteria
- [ ] Every create, create-group, and create-json case writes the expected status: the type's default_status, or active when none is declared, and pending for SingleHtmlMockup. (REQ-05.011; TECH-042 7.4)
- [ ] list, view, and json show the status written at create. (REQ-05.011)
- [ ] The status verb still changes a document's status. (MS-034 AC-1)
- [ ] The characterization suite replays with the updated expectations, and the delta is recorded in EXPECTED-DELTAS.md. (TECH-042 7.4)
- [ ] cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
