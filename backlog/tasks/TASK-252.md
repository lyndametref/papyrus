---
id: TASK-252
title: "Verify MS-033 model foundation: identical old-config load and validated schema"
created_date: '2026-11-20T17:41:41Z'
updated_date: '2026-11-22T12:21:28Z'
labels: []
references:
  - REQ-07.002
  - REQ-07.004
  - TECH-077
  - ADR-036
  - TC-103
  - TC-104
  - TC-105
  - TC-106
  - TC-107
  - TC-108
  - TC-109
  - TC-128
  - TC-130
  - TC-151
  - TC-160
  - TC-171
  - TC-172
  - TC-183
  - TC-184
  - TC-185
  - TC-190
  - TC-191
  - TC-193
status: Done
assignee:
  - quality-gatekeeper
milestone: MS-033
dependencies:
  - TASK-251
  - TASK-306
ordinal: 90
documentation:
  - docs/audits
  - tests/descriptor_equivalence.rs
  - tests/fixtures/characterization/corpus
  - tests/golden/cli-help
type: milestone-verification
---

## Description
Context: this is the MS-033 milestone-boundary verification, the model and configuration layer only. MS-033 is an internal checkpoint of one deliverable: it replaces the `'static` descriptor assumption and the per-type hard-coded schemas with owned, seed-derived descriptors and a validated config-declared field schema, per ADR-036, ADR-037, and the amended ADR-0019 and ADR-0024. Old configuration files must load identically, and no CLI or browser behaviour may change (MS-033 AC-1 to AC-7). The verification mirrors the MS-031 and MS-032 boundary records (`docs/audits/AUD-134.md`, `docs/audits/AUD-138.md`).

Inputs: MS-033 AC-1 to AC-7; the implementation of TASK-246 to TASK-251; the pre-refactor oracles at the MS-033 start commit c3103fc (`tests/golden/cli-help/`, `tests/fixtures/cli-help/papyrus.config.yml`, `tests/fixtures/characterization/corpus/`, `tests/fixtures/characterization/lists/`, and the live `papyrus.config.yml`); ADR-036, ADR-037, ADR-039, ADR-0019 as amended, ADR-0024 as amended, TECH-070, TECH-071, TECH-072, TECH-077, TECH-043; NFR-007.001 and NFR-007.002; TC-103 to TC-109, TC-114 to TC-122, TC-128, TC-130, TC-152, TC-160, TC-171, TC-172, TC-183, TC-184, TC-185, TC-190, TC-191, TC-193.

Scope:
1. Check every MS-033 acceptance criterion and record pass or fail with an evidence pointer. Record the outcome in a quality audit record created with `papyrus audit create --audit-type quality --reference MS-033 --reference TASK-252`, stored under `docs/audits/`, and in a task comment, exactly as AUD-134 and AUD-138 do.
2. AC-1: for the live configuration and all five fixture configurations, load without editing and diff the effective descriptor projection (`tests/descriptor_equivalence.rs`) against the pre-refactor facts: prefix, directory, enabled, statuses, default status, model, ID shape, verb set, and per-role heading, frontmatter key, option, kind, required, and verbs. Confirm the only projected difference is the documented user-defined reference-visibility delta (REQ-06.019, TC-185), and confirm the configuration and corpus files are byte-identical after the load.
3. AC-2: run the fill and replace tests and record the seed-fill, collection-replace, `fields: []`, independent `group`/`member`, absent-slug, and never-filled `prefix`/`directory` results.
4. AC-3: run every rejected configuration case and confirm each diagnostic uses `document_types.<slug>.<path>`, names the field key when present, is collected in one pass in deterministic order, and exits 4; include the reserved-key, duplicate-key, unknown-key, unknown-enum, and `model`/`title`/`content`/`id_shape` cases.
5. AC-4: re-emit every fixture configuration and confirm the output reparses to an equal value, mapping sequences are block sequences, flat string lists stay inline, and `"## User Story"` is preserved.
6. AC-5: run the read-path source guard and the runtime proof; confirm no file under `src/record/` loads configuration or builds a registry, and that one configuration load and one registry build serve a whole pass.
7. AC-6: confirm the named unit tests exist and pass: reserved field keys, seed-only `GroupedMarkdown` and `SingleHtmlMockup`, user-defined defaults, and immutable group membership.
8. AC-7: confirm no CLI or browser behaviour changed. Run `tests/cli_help_golden.rs`, `tests/characterization_replay.rs`, `tests/list_characterization_replay.rs`, and the browser, CLI, JSON, and migration suites; confirm the CLI-help goldens are unchanged and no route, status code, JSON member, record byte, or page changed. The only configuration-surface differences are the D14 ones: minimal `init` entries and full `config list` with selector paths.
9. Run the full `cargo test` suite and `cargo clippy --all-targets -- -D warnings`; record pass and fail counts, the run date with timezone, the rustc and cargo versions, and the host description.
10. Scope of the change set: `git diff --name-only c3103fc..HEAD` must contain no file under `packaging/` or `scripts/`, no dependency or release-profile change in `Cargo.toml`, and no change to `docs/requirements`, `decisions`, or `docs/specifications`; a change there is a failure unless it is an accepted correction recorded in the audit.

Out of scope: fixing findings, changing goldens, fixtures, audits, or source, redefining the acceptance criteria, and verifying MS-034 or MS-035 behaviour. A failed criterion is recorded with a finding and left open.

Definition of done: the audit record covers AC-1 to AC-7 with evidence pointers, every projection and diagnostic result is recorded, the emitter round-trip is proven byte-stable, the read-path guard is proven, the four named unit-test groups are present and passing, the no-behaviour-change evidence is recorded, the suite and clippy results are recorded with date, toolchain, and host, the change-set scope check is recorded, and the task comment summarises the outcome.
## Acceptance Criteria
- [ ] Each MS-033 acceptance criterion AC-1 to AC-7 is checked and recorded as pass or fail with an evidence pointer in a quality audit record that references MS-033 and TASK-252, created with `papyrus audit create --audit-type quality`, and the task carries a summary comment (mirrors AUD-134 and AUD-138).
- [ ] AC-1: the live configuration and the five fixture configurations load unedited, and the effective descriptor projection matches the pre-refactor facts from the cli-help goldens, the corpus, and ADR-036 decision 29, with the user-defined reference-visibility delta as the only named difference; the configuration and corpus files are byte-identical after the load.
- [ ] AC-2: the fill and replace results are recorded for seed fill, collection replace including `fields: []`, independent `group` and `member` blocks, an absent slug staying absent, and `prefix` and `directory` never filled.
- [ ] AC-3: every rejected case produces a `document_types.<slug>.<path>` diagnostic naming the field key when present, aggregated in one pass in deterministic order, with exit code 4, including reserved keys, duplicate keys, unknown keys, unknown enums, and `model`, `title`, `content`, and `id_shape`.
- [ ] AC-4: every fixture configuration re-emits to output that reparses equal, with mapping sequences block-style, flat string lists inline, and `"## User Story"` preserved.
- [ ] AC-5: the read-path source guard passes and the runtime proof records that a configuration-defined record still resolves after the configuration file is removed between the registry build and the read, so no read path builds a configuration or registry per record.
- [ ] AC-6: the named unit tests are present and passing for reserved field keys, seed-only `GroupedMarkdown` and `SingleHtmlMockup`, user-defined defaults, and immutable group membership.
- [ ] AC-7: the CLI-help goldens are unchanged and green, the characterization and list replays pass, the browser, CLI, JSON, and migration suites pass, and no route, status code, JSON member, record byte, or page changed; the only configuration-surface differences are minimal `init` entries and full `config list` with selector paths (MS-033 AC-7).
- [ ] The full `cargo test` suite passes with zero failures and `cargo clippy --all-targets -- -D warnings` is clean, and the run date with timezone, rustc and cargo versions, and host are recorded.
- [ ] The change set from MS-033 start commit c3103fc touches no file under packaging or scripts, changes no dependency or release-profile entry in `Cargo.toml`, and changes no requirement, decision, or specification file; any such change is recorded as a failure unless it is an accepted correction.
- [ ] Verification closes only when AC-1 to AC-7 all have a pass; any fail is recorded with a finding and the affected criterion is left open.
## Comments
- **task-refiner**: Refined for MS-033: description follows the AUD-134 and AUD-138 boundary pattern; 11 testable acceptance criteria added covering AC-1 to AC-7, the descriptor projection diff, diagnostics, emitter round-trip, read-path guard, no-behaviour-change evidence, suite and clippy record, and change-set scope; docs audits and oracle paths documented. No dependency, ordinal, milestone, or assignee change.
- **orchestrator**: Verification PASS (AUD-140): AC-1 to AC-7 all pass, 0 blocker, 0 high, 1 medium, 4 low; suite 1224 passed; packaging waived; M-1 and L-1 to L-4 routed as follow-ups.
