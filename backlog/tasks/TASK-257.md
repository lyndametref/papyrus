---
id: TASK-257
title: "Implement the uniform title contract and remove --term"
created_date: '2026-11-20T17:42:06Z'
updated_date: '2026-11-22T12:33:50Z'
labels: []
references:
  - ADR-036
  - REQ-07.003
  - REQ-05.003
  - REQ-05.006
  - BR-011
  - BR-014
  - TECH-070
  - REQ-05.008
  - TC-125
  - TC-126
  - TC-138
status: In-Progress
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 50
documentation:
  - src/document.rs
  - src/registry.rs
  - tests/glossary_cli.rs
  - tests/req_cli.rs
  - tests/audit_records_cli.rs
  - tests/ui_mockup_cli.rs
type: implementation
---

## Description
## Context
Title handling is still per-type. `src/registry.rs` gives `req create` an optional positional `[title]` plus `--title` (`requirement_verbs`, ~771-836), `audit create|edit` only `--title` (`audit_verbs`, ~1265-1368), `ui-mockup create` a positional `TITLE` (`ui_mockup_verbs`, ~1800-1840), `glossary create|edit` `--term` (`glossary_verbs`, ~941-1010), and the remaining types a positional `TITLE`. ADR-036 decision 24, REQ-07.003, and REQ-05.001/05.003/05.006/05.008 fix one contract: the title is not a field; it is the first positional `TITLE` on `create` and `create-group` plus `--title` on `create`, `create-group`, and `edit`; required on create, single-line, non-empty, never clearable on edit; `--term` is removed; `audit` and `create-group` gain the positional additively; the HTML mockup metadata uses the same contract.

## Scope
- In the TASK-255 engine (`src/document.rs`) and the descriptor-derived command metadata (`src/registry.rs`):
  - `create` and `create-group` emit the positional `TITLE` first, then `--title`; `edit` emits `ID` then `--title`; both title sources on one invocation are rejected.
  - Apply the title through `PatchOp::SetHeadingTitle` and the front-matter `title` key only through the single writer path; the title is never a field.
  - Validate single-line, non-empty, no control characters. Empty, whitespace-only, or multi-line titles on create or edit fail with one validation outcome and write nothing.
  - Edit never clears the title: an empty or whitespace-only `--title` is rejected; omitting it leaves the title unchanged.
  - Remove `--term` from glossary create/edit command metadata. Glossary is `glossary create <TERM> --definition <TEXT>` and `glossary edit <ID> --title <TERM> --definition ...`; existing glossary records keep their term as title and body heading.
  - `audit create` gains the positional `TITLE`; `audit create|edit` keep `--title` with its current name and meaning; no existing positional is renamed, removed, or reordered.
  - `create-group` gains the positional `TITLE` additively; `--title` keeps its current name and meaning.
  - The `ui-mockup` title is the HTML metadata title and follows the same validation and never-clear rule.
- Update the affected per-type CLI tests: `tests/glossary_cli.rs`, `tests/req_cli.rs`, `tests/audit_records_cli.rs`, `tests/ui_mockup_cli.rs`, and the goldens in `tests/golden/cli-help/` through TASK-260. Cover TC-125, TC-126, and TC-138.

## Out of scope
Field-option generation (TASK-255), create-group/group semantics and mockup supersede (TASK-256), config selectors (TASK-258), module deletion (TASK-259), help-golden regeneration (TASK-260), and the browser.

## Grounding
ADR-036 decisions 23-24; REQ-07.003; REQ-05.001, REQ-05.003, REQ-05.006, REQ-05.008; BR-011, BR-014; TECH-070 section 1; TC-125, TC-126, TC-138.

## Definition of done
The engine and command metadata implement the contract for every model; the listed tests and the help goldens (regenerated in TASK-260) are consistent; `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; no `--term` remains in production command metadata.
## Acceptance Criteria
- [ ] Every document type create and create-group has the first positional TITLE and also accepts --title; edit accepts --title only. (MS-034 AC-2; ADR-036 d24)
- [ ] A create with no title, an empty or whitespace-only title, or a multi-line title fails with one validation outcome (exit 4) and writes no file. (MS-034 AC-2; TC-126)
- [ ] Supplying both the positional TITLE and --title on one invocation fails with one validation outcome and writes nothing. (MS-034 AC-2; TC-125)
- [ ] An edit cannot clear the title: an empty or whitespace-only --title is rejected and writes nothing; omitting --title leaves the title unchanged. (MS-034 AC-2)
- [ ] No document command accepts --term; a glossary create or edit that supplies --term is a usage error (exit 2). (MS-034 AC-2; REQ-05.003)
- [ ] A glossary term is the record title: glossary create with a positional TERM and --definition writes the term as the title and heading; existing term records keep their title and body. (MS-034 AC-2; TC-138)
- [ ] audit create accepts the positional TITLE additively while --title keeps its current name and meaning on create and edit; no existing positional argument is renamed, removed, or reordered. (MS-034 AC-2; REQ-05.006)
- [ ] create-group accepts the positional TITLE additively while --title keeps its current name and meaning. (MS-034 AC-2; REQ-05.001 and REQ-05.002)
- [ ] A field whose key or frontmatter_key is title is rejected with a document_types diagnostic naming the field path. (MS-034 AC-2; ADR-036 d23)
- [ ] The ui-mockup HTML metadata title follows the same contract: present, single-line, non-empty, required on create, never clearable on edit. (MS-034 AC-2; REQ-05.008)
## Comments
- **task-refiner**: Grounding: current title surfaces are requirement_verbs (registry lines 771), glossary_verbs (941), audit_verbs (1265), ui_mockup_verbs (1800), and the remaining verb builders; the engine and descriptor command builder are TASK-255. AUD-140 L-3 (TECH-077 section 6 step 4 still says merge_yaml wholesale, contradicting corrected section 5) is recorded here per the AUD-140 routing; it is a TECH-077 wording fix owned by architect with the MS-034 spec pass, not implementation scope for this title task.
