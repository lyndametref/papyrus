---
id: TASK-232
title: "Design the equivalence gate, fixture corpus, and benchmark protocol test cases"
created_date: '2026-11-20T17:39:46Z'
updated_date: '2026-11-20T17:39:46Z'
labels: []
references:
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
status: ToDo
assignee:
  - test-manager
milestone: MS-031
dependencies:
  - TASK-223
  - TASK-224
  - TASK-225
  - TASK-226
  - TASK-227
ordinal: 160
documentation:
  - docs/test-cases
type: test-design
---

## Description
Context: the refactor is only safe if behavior is proven equivalent. Record files, JSON payloads, and status codes are strict; rendered HTML is semantic. This gate must be designed before implementation so the fixtures exist from MS-032.

Scope: author Gherkin test cases for every changed requirement and business rule in this milestone, plus the gate and measurement protocol.
- Equivalence gate: record file bytes strict; JSON strict including key order where it is documented; HTTP status codes strict; rendered HTML compared by selectors, text, navigation, form fields, and absence of per-type wrappers, not by bytes.
- Fixture corpus: at least one type per model, a grouped type with multiple members, an HTML mockup in each metadata state, a user-defined SingleMarkdown type, a record with unresolved references, and a record at the archived boundary.
- Benchmark protocol: 100 and 1,000 records, warm-up and measured runs, recorded hardware, p50 and p95, and the exact commands.
- Deliberate behavior changes are listed as expected deltas, never as equivalence failures: uniform title CLI, --term removal, help prose, asset headers, latency.
- Cover config diagnostics: unknown key, unknown enum, reserved field key, duplicate field key, bad selector path, collection unset, and fields: [].
