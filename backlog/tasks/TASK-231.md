---
id: TASK-231
title: "Finalize NFR-007.001 to NFR-007.003 performance and asset-caching requirements"
created_date: '2026-11-20T17:39:46Z'
updated_date: '2026-11-20T21:26:52Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
  - NFR-007
  - TECH-008
  - TECH-069
status: Done
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-219
  - TASK-220
ordinal: 150
documentation:
  - docs/requirements/non-functional/NFR-007.001.md
  - docs/requirements/non-functional/NFR-007.002.md
  - docs/requirements/non-functional/NFR-007.003.md
  - docs/requirements/non-functional/NFR-007.md
type: requirements
---

## Description
Context: a draft NFR-007 group and its three NFRs were captured while planning: NFR-007.001 inventory p95 under 250 ms at 1,000 records, NFR-007.002 view and edit p95 under 100 ms at 1,000 records, NFR-007.003 asset caching contract. They are drafts and have not been user-verified.

Scope: review, correct, and take the three NFRs through user verification.
- Confirm the reference hardware statement: 4 vCPU, 8 GiB RAM, SSD, Linux, release build with LTO and strip, loopback HTTP, single server process.
- Confirm the measurement method: generated 1,000-record corpus including one GroupedMarkdown type with 200 members and one SingleHtmlMockup type with 50 records; 10 warm-up requests then 100 measured requests; p50 and p95 reported; cold start and config-reload requests excluded.
- Confirm that NFR-007.003 owns header-level assertions for immutable vendor assets, no-cache plus ETag and 304 for app.css and app.js, no-store for HTML and JSON, assets only, and no compression.
- If the project declines asset caching, replace NFR-007.003 with an explicit exclusion recorded in ADR-038 rather than leaving it unmet.
- Record the baseline figures that motivate the targets: inventory 14.2 s and a document view 1.8 s at 382 active records.
## Refinement inputs (TASK-290)

### Requirement file map (NFR group and stubs)
- NFR-007 group — `docs/requirements/non-functional/NFR-007.md` — group label `NFR-007 07 Document Engine Performance`; fix only if the label is inconsistent, never renumber.
- NFR-007.001 (NFR §7.1) — docs/requirements/non-functional/NFR-007.001.md — Inventory latency at 1,000 records (p95 under 250 ms) — draft, not user-verified
- NFR-007.002 (NFR §7.2) — docs/requirements/non-functional/NFR-007.002.md — View and edit latency at 1,000 records (p95 under 100 ms) — draft, not user-verified
- NFR-007.003 (NFR §7.3) — docs/requirements/non-functional/NFR-007.003.md — Browser asset caching contract (versioned immutable vendor assets and validated app assets) — draft, not user-verified
- Baseline figures named by the task: inventory 14.2 s and document view 1.8 s at 382 active records.
- Read-only decision inputs: ADR-037 (`decisions/ADR-037-in-server-configuration-and-registry-snapshot.md`) and ADR-038 (`decisions/ADR-038-cacheable-immutable-browser-assets.md`).

### Refinement assumptions / definition of done
The three NFR records already exist as drafts; the task reviews and verifies them, it does not create a new group. User verification is explicit; record the approval as task evidence because NFR records have no approval status. Done means all acceptance criteria pass and no source, test, or benchmark harness is written in MS-031 (MS-031 AC-5, AC-10).
## Acceptance Criteria
- [ ] NFR-007.001, NFR-007.002, and NFR-007.003 exist in the NFR-007 group with the targets, reference hardware, and measurement method stated in the task (MS-031 AC-5).
- [ ] All three records state the same reference hardware, the 1,000-record generated corpus with a 200-member GroupedMarkdown type and a 50-record SingleHtmlMockup type, 10 warm-up plus 100 measured requests, p50 and p95 reporting, and cold-start and config-reload exclusions (MS-031 AC-5, AC-9).
- [ ] NFR-007.003 owns the header-level caching assertions (immutable vendor assets; no-cache plus ETag and 304 for app.css and app.js; no-store for HTML and JSON; assets only; no compression) and states the explicit-exclusion path in ADR-038 if caching is declined (MS-031 AC-5).
- [ ] The motivating baseline figures (inventory 14.2 s, document view 1.8 s at 382 active records) are recorded in the requirements or traceability (MS-031 AC-5).
- [ ] Each NFR's references point to ADR-037 and ADR-038, and the user has explicitly approved all three records; the approval is recorded as task evidence (MS-031 AC-5, AC-8).
## Comments
- **requirements-engineer**: Finalized NFR-007 group title to Document Engine Performance, removing the duplicate 07. NFR-007.001, NFR-007.002 and NFR-007.003 now state the reference hardware, the generated 1,000-record corpus with a 200-member GroupedMarkdown group and 50 SingleHtmlMockup records, 10 warm-up plus 100 measured requests, p50 and p95 reporting, and the cold-start and config-reload exclusions, and reference RE-010, ADR-037, ADR-038, REQ-006.014 and MS-032. Baseline 14.2 s inventory and 1.8 s view at 382 active records recorded. NFR-007.003 owns the header-level assertions and notes the ADR-038 exclusion path. All three statuses set to active. papyrus check shows the same three pre-existing frontmatter-conflict findings and no new findings.
- **orchestrator**: Acceptance met: NFR-007 group title fixed and NFR-007.001..003 finalized with targets, reference hardware, measurement method, cache header contract and baseline figures; no new check findings.
