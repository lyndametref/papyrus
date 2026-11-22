---
id: TASK-308
title: "Seed the default status on document create for every model"
created_date: '2026-11-22T14:19:46Z'
updated_date: '2026-11-22T14:37:49Z'
labels: []
references:
  - REQ-05.011
  - TECH-042
  - TC-138
status: Done
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
## Comments
- **rust-dev**: -
- **rust-dev**: Implementation (rust-dev) - status written at create (REQ-05.011; TECH-042 7.4).
- **rust-dev**: SingleMarkdown (glossary, rule, ui-spec, audit, tech-spec, use-case, test-case, test-report, user-defined note): configured default_status, else active. Seed types write status active; note with default_status review writes review.
- **rust-dev**: GroupedMarkdown (req, nfr): member create and create-group write the configured default_status, else active. A created group gains a front-matter block holding only status.
- **rust-dev**: SingleHtmlMockup (ui-mockup): unchanged; keeps status pending in its HTML metadata. No create or create-group option sets status; existing records without status are not migrated.
- **rust-dev**: Code: src document module create_document seeds status for every StatusModel DocumentMain; create_group_document seeds it too. TC-138 updated in the glossary CLI tests. New tests cover seed create status, create-group status, and list, view and json visibility. The browser CLI t93 fixture is updated because created groups now carry status active.
- **rust-dev**: Characterization: regenerated through the documented capture command; second capture idempotent; replay green. Delta 8 in EXPECTED-DELTAS.md covers each create and create-json case plus req and nfr create-group; ui-mockup unchanged.
- **rust-dev**: Checks: env -u EDITOR -u VISUAL cargo test = 1258 passed, 0 failed, 26 ignored; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean; help goldens unchanged and no status option appears. Branch task-TASK-308; changes left uncommitted as instructed.
- **orchestrator**: Acceptance met: create and create-group seed status from default_status or active for every Markdown model, mockup keeps pending in HTML metadata, no status option, no migration; 22 characterization fixtures updated with EXPECTED-DELTAS item 8; TC-138 satisfied; 1258 tests pass.
