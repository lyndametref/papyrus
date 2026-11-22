---
id: TASK-293
title: "Record the accepted seed slot sets for built-in document types"
created_date: '2026-11-22T08:53:01Z'
updated_date: '2026-11-22T08:53:01Z'
labels: []
references:
  - ADR-039
  - ADR-036
  - ADR-0028
  - TECH-070
  - TECH-077
  - REQ-07.002
  - REQ-07.004
status: ToDo
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
