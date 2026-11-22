---
id: TASK-293
title: "Record the accepted seed slot sets for built-in document types"
created_date: '2026-11-22T08:53:01Z'
updated_date: '2026-11-22T09:43:32Z'
labels: []
references:
  - ADR-039
  - ADR-036
  - ADR-0028
  - TECH-070
  - TECH-077
  - REQ-07.002
  - REQ-07.004
status: Done
assignee:
  - architect
milestone: MS-033
dependencies:
  - TASK-245
ordinal: 25
documentation:
  - src/browser/routes.rs
  - tests/fixtures/characterization/corpus
  - decisions/ADR-039-one-descriptor-driven-document-view-and-edit-surface.md
type: architecture
---

## Description
Context: MS-033 refinement (TASK-245) is blocked by one ambiguity. ADR-039 decision 25 says a document descriptor's list slots come from its seed entry and are "currently none", and decision 6 forbids the reserved key `group` as a slot key. The implemented `src/browser/routes.rs::PRESENTATION` renders document slots today: `req` and `nfr` `[group]`, `rule` `[applicability]`, `audit` `[audit_type]`, `ui-mockup` `[task, specs]`; the characterization corpus (`tests/fixtures/characterization/corpus/expected/http/list-req/page.html`) expects the `Group` column. TASK-249 must seed those slot sets to keep MS-033 behaviour-neutral, so an accepted decision must settle them first.

Scope:
1. Amend ADR-039 decisions 6 and 25 to declare the exact seed slot sets for the eleven seed document types of REQ-07.002 (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`), including the engine-owned `group` link for the grouped types, or record a new accepted ADR that owns the seed slot sets. The decision must be internally consistent about the reserved `group` key in decision 6.
2. Keep the browser list columns unchanged so MS-033 stays behaviour-neutral: the declared sets reproduce today's `PRESENTATION` output (`req`/`nfr` `[group]`, `rule` `[applicability]`, `audit` `[audit_type]`, `ui-mockup` `[task, specs]`, every other seed type no slots).
3. Align the accepted decision text with the `PRESENTATION` slot data and the characterization corpus.
4. Update requirement and specification references only where needed; do not change their normative content.

Out of scope: no code change, no specification rewrite, no seed implementation (TASK-249), no browser list redesign.

Definition of done: one accepted decision names every seed type's slot set and removes the contradiction with `PRESENTATION`; ADR-039 (or the new ADR) is internally consistent about `group`; TASK-249 can add the seed `slots` values and its slot acceptance criterion without choosing behaviour.

## Acceptance Criteria
- [ ] The accepted decision names every one of the eleven seed document types' slot set exactly.
- [ ] ADR-039 or the new ADR is internally consistent about group: decision 6 no longer forbids a slot set the decision declares.
- [ ] No code or specification change is made; requirement and specification references are updated only where needed.
## Comments
- **architect**: Architecture evidence: ADR-041 (accepted) owns the seed slot sets; record file ADR-041-seed-list-slots-for-the-eleven-built-in-document-types.md. Per-type slots read from PRESENTATION and confirmed against the characterization corpus: req group=Group; nfr group=Group; glossary none; rule applicability=Applicability; ui-spec none; ui-mockup task=Task and specs=Specs; audit audit_type=Audit type; tech-spec none; use-case none; test-case none; test-report none. Browser list columns are unchanged; a slot key is not a field key; group is a permitted slot key only on GroupedMarkdown seed types and remains prohibited as a field key and frontmatter key. ADR-039 carries a dated clarification (2026-11-22) for decisions 5, 6 and 25 pointing to ADR-041; every other ADR-039 decision stands. papyrus check adds no finding. No code, TECH, requirement or test change.
- **orchestrator**: Acceptance met: ADR-041 accepted, naming the exact seed slot sets for all 11 seed types with the engine-owned group link for grouped types; ADR-039 decisions 6 and 25 clarified; browser list columns unchanged.
