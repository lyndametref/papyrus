---
id: TASK-232
title: "Design the equivalence gate, fixture corpus, and benchmark protocol test cases"
created_date: '2026-11-20T17:39:46Z'
updated_date: '2026-11-21T00:05:05Z'
labels: []
references:
  - REQ-007.001
  - REQ-007.002
  - REQ-007.003
  - REQ-007.004
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - BR-001
  - BR-009
  - BR-010
  - BR-011
  - BR-012
  - BR-013
  - BR-014
  - BR-016
  - BR-017
  - BR-018
  - BR-019
  - BR-020
status: Done
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
## Refinement inputs (TASK-290)

### Coverage map — changed requirements and rules (one TC each)
Changed requirements:
- REQ-007.001 (§7.1) — docs/requirements/business/REQ-007.001.md — Organize Artifacts into Three Categories
- REQ-007.002 (§7.2) — docs/requirements/business/REQ-007.002.md — Configure Document Types and Slugs
- REQ-007.003 (§7.3) — docs/requirements/business/REQ-007.003.md — Dispatch Config-Defined Document Commands
- REQ-007.004 (§7.4) — docs/requirements/business/REQ-007.004.md — Provide a Generic Content Model for Config-Defined Document Types
- REQ-005.001 (§5.1) — docs/requirements/business/REQ-005.001.md — Manage Business Requirements
- REQ-005.002 (§5.2) — docs/requirements/business/REQ-005.002.md — Manage Non-Functional Requirements
- REQ-005.003 (§5.3) — docs/requirements/business/REQ-005.003.md — Manage Glossary Terms
- REQ-005.004 (§5.4) — docs/requirements/business/REQ-005.004.md — Manage UI Specifications
- REQ-005.005 (§5.5) — docs/requirements/business/REQ-005.005.md — Manage Business Rules
- REQ-005.006 (§5.6) — docs/requirements/business/REQ-005.006.md — Manage Audit Records
- REQ-005.007 (§5.7) — docs/requirements/business/REQ-005.007.md — Manage Document Archive State and Deletion
- REQ-005.008 (§5.8) — docs/requirements/business/REQ-005.008.md — Manage UI Mockups
- REQ-005.009 (§5.9) — docs/requirements/business/REQ-005.009.md — Manage Technical Specifications
- REQ-005.010 (§5.10) — docs/requirements/business/REQ-005.010.md — Record Metadata in YAML Front Matter
- REQ-005.011 (§5.11) — docs/requirements/business/REQ-005.011.md — Maintain Record Metadata Through All Commands
- REQ-005.012 (§5.12) — docs/requirements/business/REQ-005.012.md — Manage Use Cases
- REQ-005.013 (§5.13) — docs/requirements/business/REQ-005.013.md — Manage Test Cases
- REQ-005.014 (§5.14) — docs/requirements/business/REQ-005.014.md — Manage Test Reports
- REQ-006.004 (§6.4) — docs/requirements/business/REQ-006.004.md — Use Front-Matter Metadata in CLI and Browser Views
- REQ-006.014 (§6.14) — docs/requirements/business/REQ-006.014.md — Display Config-Defined Document Types in the Browser
- REQ-006.019 (§6.19) — docs/requirements/business/REQ-006.019.md — Do not Show the Reference List in Document-Type Browser List Views
- REQ-001.002 (§1.2) — docs/requirements/business/REQ-001.002.md — Read and Edit Project Configuration
- REQ-001.005 (§1.5) — docs/requirements/business/REQ-001.005.md — Assign Stable Record IDs
- REQ-001.008 (§1.8) — docs/requirements/business/REQ-001.008.md — Run Project Diagnostics (`papyrus check`)
Changed NFRs:
- NFR-007.001 (NFR §7.1) — docs/requirements/non-functional/NFR-007.001.md — Inventory latency at 1,000 records (p95 under 250 ms)
- NFR-007.002 (NFR §7.2) — docs/requirements/non-functional/NFR-007.002.md — View and edit latency at 1,000 records (p95 under 100 ms)
- NFR-007.003 (NFR §7.3) — docs/requirements/non-functional/NFR-007.003.md — Browser asset caching contract (versioned immutable vendor assets and validated app assets)
Changed business rules:
- BR-001 — docs/business-rules/BR-001-papyrus-command-behavior.md
- BR-009 — docs/business-rules/BR-009-req-command-behavior.md
- BR-010 — docs/business-rules/BR-010-nfr-command-behavior.md
- BR-011 — docs/business-rules/BR-011-glossary-command-behavior.md
- BR-012 — docs/business-rules/BR-012-ui-spec-command-behavior.md
- BR-013 — docs/business-rules/BR-013-rule-command-behavior.md
- BR-014 — docs/business-rules/BR-014-audit-command-behavior.md
- BR-016 — docs/business-rules/BR-016-configuration-keys-and-precedence.md
- BR-017 — docs/business-rules/BR-017-tech-spec-command-behavior.md
- BR-018 — docs/business-rules/BR-018-use-case-document-type-behavior.md
- BR-019 — docs/business-rules/BR-019-test-case-document-type-behavior.md
- BR-020 — docs/business-rules/BR-020-test-report-document-type-behavior.md

### User-verification path (test cases)
MS-031 produces no UI specification and no UI mockup, so the `manage-ui-specs` verification step does not apply; the test-case verification flow below is the gate.
1. Create each test case with `papyrus test-case create "<title>" --description - --reference <REQ|NFR> [--reference <BR>]`. Papyrus assigns the TC id and file under `docs/test-cases/`; the initial status is `draft`.
2. Present every draft to the user: TC id, title, the single GIVEN/WHEN/THEN scenario, and traceability.
3. Only explicit user approval changes status: `papyrus test-case status <TC-ID> approved`. Silence or another reviewer's approval is not approval.
4. On change requests: `papyrus test-case edit <TC-ID> --description -`, keep the status `draft`, and present again.
5. Coverage gate: every changed REQ/NFR and BR in the map above has at least one `approved` TC; a missing TC is a coverage gap and blocks TASK-235 and TASK-237.
6. TASK-232 is Done only when the coverage matrix is complete and every one of its test cases is `approved`.

### Refinement assumptions / definition of done
The equivalence gate and benchmark protocol are designs, not code; no test implementation is written in MS-031. Done means all acceptance criteria pass and no source, test, or packaging file changes (MS-031 AC-8, AC-9, AC-10).
## Acceptance Criteria
- [ ] Every changed REQ, NFR, and BR in the coverage map has at least one Gherkin test case with one scenario, one WHEN block, and traceability (MS-031 AC-8).
- [ ] The equivalence gate is defined: record-file bytes strict, JSON strict including documented key order, HTTP status codes strict, and rendered HTML semantic by selectors, text, navigation, form fields, and absence of per-type wrappers (MS-031 AC-9).
- [ ] The fixture corpus includes at least one type per model, a grouped type with multiple members, an HTML mockup in each metadata state, a user-defined SingleMarkdown type, a record with unresolved references, and a record at the archived boundary (MS-031 AC-9).
- [ ] The benchmark protocol covers 100 and 1,000 records, warm-up and measured runs, recorded hardware, p50 and p95, and the exact commands (MS-031 AC-9).
- [ ] Deliberate behavior changes are listed as expected deltas, never as equivalence failures: uniform title CLI, --term removal, help prose, asset headers, and latency (MS-031 AC-9).
- [ ] Config-diagnostic scenarios cover unknown key, unknown enum, reserved field key, duplicate field key, bad selector path, collection unset, and fields: [] (MS-031 AC-9).
- [ ] All test cases are `approved` only after explicit user verification, and the coverage matrix has zero gaps; dependent tasks TASK-235 and TASK-237 remain blocked until then (MS-031 AC-8).
## Comments
- **test-manager**: Designed 94 Gherkin test cases TC-098 through TC-193 for milestone MS-031. Coverage is complete: 27 changed requirements and NFRs plus 12 business rules, zero gaps. The set includes the seven config diagnostics, the strict record JSON and status-code gate, the semantic rendered-HTML gate, the fixture corpus, the expected deltas, and the 100 and 1000-record benchmark protocol referencing the MS-032 harness. All test cases remain draft, awaiting user verification; none was approved. papyrus check result is unchanged: 3 pre-existing frontmatter conflicts and no new findings.
- **orchestrator**: Acceptance met: 94 Gherkin test cases TC-098..TC-193 cover every changed REQ, BR and NFR plus the strict equivalence gate and the 100 and 1000-record benchmark protocol; user approved all test cases on 2026-11-21.
