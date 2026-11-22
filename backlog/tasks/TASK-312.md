---
id: TASK-312
title: "Correct MS-035 audit findings from AUD-145"
created_date: '2026-11-22T19:08:46Z'
updated_date: '2026-11-22T19:37:27Z'
labels: []
references:
  - AUD-145
  - MS-035
  - TASK-269
  - TASK-309
  - TASK-270
  - TC-184
  - TC-183
  - ADR-039
  - TECH-040
  - REQ-06.014
  - REQ-06.019
status: Done
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-269
  - TASK-309
ordinal: 88
documentation:
  - tests/support/characterization.rs
  - tests/characterization_replay.rs
  - tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md
  - tests/document_type_no_drift.rs
  - tests/ui_spec_cli.rs
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/list.rs
  - src/browser/routes.rs
  - src/record/edit.rs
  - src/registry.rs
  - src/config.rs
  - scripts/build-package.sh
  - scripts/build-deb.sh
  - docs/specifications/technical/TECH-040.md
type: implementation
---

## Acceptance Criteria
- [ ] #1 H1: static per-case node-level allowlist; no form.form, section.section or main.main; allowance reasons validated against EXPECTED-DELTAS.md.
- [ ] #2 H1: negative tests mutate one label, one text, one field and one link inside each page kind and each mutation makes the comparison fail.
- [ ] #3 H1: a test asserts the TC-184 no-per-type-wrapper rule.
- [ ] #4 H2: build-package.sh cleans only a subdirectory the script owns, or requires an empty out directory, or requires an explicit force flag; refuses out paths inside the repository except the project-local .tmp directory, and refuses HOME and system directories; validates the path before creating directories; a guard test covers each refusal.
- [ ] #5 M1: field domains are declared on the descriptor (FieldSpec kind and values) and one validator serves both HTML and JSON document writes; option-string branches and field-map shape sniffing are removed; the HTML 422 tests for an invalid requirement reference and an invalid audit type are restored and pass.
- [ ] #6 M2: version, kind, form, path and metadata are reserved field keys with validation; a config regression test covers the reservation and the edit page asserts exactly one version control.
- [ ] #7 M3: the dead document None arms and the record.description assignment are deleted; D-15 matches a function name regardless of visibility; a structural D-18 variant covers description.
- [ ] #8 M4: reference visibility has one source through the descriptor helper; DocumentFieldMap::reference_visibility is consumed or deleted; a test covers an absent role visibility with a Hidden descriptor.
- [ ] #9 L1-L6: each item has a recorded result and test: the body-then-frontmatter fallback is removed or TECH-040 and ADR-039 are amended; dead code and duplicated helpers are removed; mockup references render once; packaging shares the out policy and tightens the version regex; role and visibility tests distinguish behaviour; the conflict sentence, 404 heading order and stale byte-for-byte naming are corrected.
- [ ] #10 The full suite, cargo clippy --all-targets -- -D warnings and cargo fmt --check pass; no production behaviour changes beyond these corrections; AUD-145 is addressed point by point with an evidence comment per item.
## Description
## Context
AUD-145 (code audit MS-035) returned "corrections required": 2 high, 4 medium and 6 low findings. This task applies the corrections in audit order and adds the regression tests the audit requires. It is a correction task inside MS-035, executed after TASK-269 and TASK-309 and before the milestone verification TASK-270.

## Scope
1. H1 - semantic equivalence gate. Replace the self-derived, pruned allowlist with a static per-case, node-level allowlist. Do not allow parent-container selectors (`form.form`, `section.section`, `main.main`). Validate each allowance reason against `EXPECTED-DELTAS.md`. Add negative tests that mutate one label, one text, one field and one link inside each page kind; each mutation must fail the comparison. Assert TC-184's no-per-type-wrapper rule.
2. H2 - packaging out-directory safety. Scope `--out` cleanup to a subdirectory the script owns, or require an empty out directory, or add an explicit force flag. Refuse out paths inside the repository except the project-local `.tmp` directory; refuse HOME and system directories. Validate the path before creating any directory. Add a guard test for every refusal.
3. M1 - one write validator. Declare field domains on the descriptor (`FieldSpec` kind and values) and use one validator for HTML and JSON document writes. Remove option-string branches and field-map shape sniffing. Restore the HTML 422 tests: invalid requirement reference and invalid audit type.
4. M2 - reserved field keys. Reserve `version`, `kind`, `form`, `path` and `metadata` as field keys with validation. Add a config regression test and an edit-page assertion of exactly one `version` control.
5. M3 - no-drift guards and dead code. Delete the dead document `None` arms and the `record.description` assignment. Tighten D-15 to match a function name regardless of visibility and add a structural D-18 variant covering `description`.
6. M4 - reference visibility. Use one reference-visibility source through the descriptor helper. Consume or delete `DocumentFieldMap::reference_visibility`. Test an absent role visibility with a `Hidden` descriptor.
7. L1-L6 - quality corrections: record the spec-vs-code fallback decision (remove the body-then-frontmatter fallback, or amend TECH-040 and ADR-039); remove dead code and duplicated helpers; de-duplicate the mockup references that render twice; share the packaging out policy and tighten the version regex; make the role and visibility tests distinguish behaviour; fix the conflict sentence, the 404 heading order and the stale byte-for-byte naming.

## Out of scope
- Implementing features not named by AUD-145, changing requirement meaning, or editing milestone text, TECH-068, the audit record, or the milestone verification logic.
- Producing or retaining a package; MS-035 stays an internal checkpoint.

## Definition of done
Each AUD-145 item has a recorded result and its regression test; the full suite, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass; no production behaviour changes beyond the corrections; an evidence comment maps every AUD-145 finding to its correction and test.
## Comments
- **rust-dev**: -
- **orchestrator**: Acceptance met: static node-level semantic allowlist with per-page mutation tests, scoped packaging out policy with guard tests, one field-domain validator for HTML and JSON, reserved envelope field keys, dead document reads removed with tighter guards, single visibility source, plus L1 to L6 cleanups; 1349 tests pass.
