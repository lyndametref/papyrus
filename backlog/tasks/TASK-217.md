---
id: TASK-217
title: "Record request RE-010 for the descriptor-driven document engine refactor"
created_date: '2026-11-20T17:35:52Z'
updated_date: '2026-11-20T17:35:52Z'
labels: []
references:
  - REQ-07.004
  - ADR-0017
  - ADR-0018
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies: []
ordinal: 10
documentation:
  - docs/requests/RE-010.md
type: requirements
---

## Description
Context: the refactor of the document subsystem currently has no request artifact, so its owner decisions are only in conversation. Governance needs a durable, citable record before any ADR or requirement is rewritten.

Scope: create request RE-010 through the Papyrus request command and record the twelve fixed owner decisions verbatim as accepted and not reopenable:
1. Supersede ADR-0018 in full, not only decisions 9-10, and replace ADR-0017's descriptor decisions (1-4, 7, 9, 10) with a new model ADR.
2. One deliverable only; no intermediate release; internally staged into milestones.
3. No new Rust crates; keep clap, serde, serde_json, yaml_serde.
4. Backlog (task/milestone/release) and decision records stay bespoke; only documents are unified.
5. DocumentModel enum: SingleMarkdown, GroupedMarkdown, SingleHtmlMockup; DocumentModel supersedes ContentModel.
6. GroupedMarkdown and SingleHtmlMockup are seed-only; only req/nfr are grouped; only ui-mockup is HTML; user-defined types default to SingleMarkdown.
7. Group membership is immutable after creation; preserve required existing active group of the same type, cross-type rejection, archived-group rejection, no cascade on archive/delete, merged list ordering.
8. Reference visibility is seed data per type/role; ADR-0035 is amended.
9. Equivalence gate: record files, JSON, and status codes strict; rendered HTML semantic.
10. Presentation facts live in the config seed.
11. Keep the mtime config/registry cache.
12. Asset caching: versioned immutable vendor files; app.css/app.js no-cache with ETag/304; assets only; HTML/JSON stay no-store.
13. Title is fixed and non-configurable; single-line, non-empty, always present, required on create, never clearable on edit; not a field; reserved field key; uniform CLI positional title plus --title on create/create-group and --title on edit; --term removed; audit and create-group gain a positional title additively; HTML mockup metadata uses the same contract.
14. Config format: nested group/member blocks; fields is a YAML sequence; no per-field order or enabled; scalars fill from seed; collections replace; model/title/content/id_shape are not config keys; init writes a minimal file; config list is full and uses selector paths; adding a field is file-edit-only.

Also record the target architecture: three category handlers (Backlog, Decision, Document) and one schema-driven document engine with three models.

Out of scope: any requirement, ADR, TECH, or code change; those are separate tasks in this milestone.
