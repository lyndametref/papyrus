---
id: TASK-234
title: "Update TECH-040, TECH-042, TECH-043, TECH-066, and TECH-067 for front-matter keys, patch ops, and search"
created_date: '2026-11-20T17:40:19Z'
updated_date: '2026-11-21T07:45:39Z'
labels: []
references:
  - TECH-040
  - TECH-042
  - TECH-043
  - TECH-066
  - TECH-067
  - ADR-036
  - ADR-0023
  - ADR-0024
  - REQ-05.010
  - REQ-06.004
  - REQ-07.002
status: In-Progress
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-218
  - TASK-221
  - TASK-222
ordinal: 180
documentation:
  - docs/specifications/technical/TECH-040.md
  - docs/specifications/technical/TECH-042.md
  - docs/specifications/technical/TECH-043.md
  - docs/specifications/technical/TECH-066.md
  - docs/specifications/technical/TECH-067.md
type: docs
---

## Description
Context: the record envelope and front-matter patcher survive, but the field map, the option surface, and the migration path change.

Scope:
- TECH-040: the record envelope carries a model-neutral field map; state how required, list, and frontmatter_key fields map to PatchOps and that only PatchOps write records.
- TECH-042: front-matter options are generated from the field sequence; metadata options exist only for fields whose verbs include the command; title is not an option but a positional argument.
- TECH-043: lazy front-matter migration resolves descriptors from the snapshot instead of building a registry per record; state the one-scan guarantee.
- TECH-066: archive and status behavior is unchanged by the content model; group records archive without cascading; the HTML mockup has no status verb and keeps fixed metadata.
- TECH-067: search indexes config-declared fields through frontmatter_key and body headings; nested mapping sequences are not list-valued keys (ADR-0024).
Do not reference TASK, MS, or REL IDs in the body.
## Refinement inputs (TASK-290)

### TECH section map (real paths)
- TECH-040 — `docs/specifications/technical/TECH-040.md` — §2 record::frontmatter, §3 record::document, §4 record::patch, §5 record::store, §6 record descriptors, §7 error behavior.
- TECH-042 — `docs/specifications/technical/TECH-042.md` — §2 front-matter field matrix, §3 command surface changes, §7 list-field update semantics, §9 identity/reference handling, §10 validation.
- TECH-043 — `docs/specifications/technical/TECH-043.md` — §1 managed record set/descriptor claim, §2 first read/first write trigger, §3 migration patch contents, §4 created_date provenance, §6 idempotency.
- TECH-066 — `docs/specifications/technical/TECH-066.md` — `Directory resolution contract`, `Front-matter key contract`, `Public API consumed by downstream tasks`, `ADR-0022 decision mapping`, `ADR-0023 mapping`, `Validation and error behavior`.
- TECH-067 — `docs/specifications/technical/TECH-067.md` — §1 document set, §2 directory/files/identity, §3 matching/ordering/output, §4 visibility, plus `Acceptance criteria mapping`.
- Read-only decision inputs: ADR-0023 (`decisions/ADR-0023-consolidated-record-layout-and-field-map.md`), ADR-0024 (`decisions/ADR-0024-list-valued-configuration-keys.md`), ADR-0028, ADR-0036.

### Refinement assumptions / definition of done
All five TECH files are `verified` records; this task updates their bodies only and attaches links with `--reference`. No TASK, MS, or REL id appears in a body. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-7, AC-10).
## Acceptance Criteria
- [ ] TECH-040 states that the record envelope carries a model-neutral field map, how required, list, and frontmatter_key fields map to PatchOps, and that only PatchOps write records (MS-031 AC-7).
- [ ] TECH-042 states that front-matter options are generated from the field sequence, metadata options exist only for fields whose verbs include the command, and title is a positional argument rather than an option (MS-031 AC-7, RE-010 D13).
- [ ] TECH-043 resolves descriptors from the snapshot instead of building a registry per record and states the one-scan guarantee (MS-031 AC-7, RE-010 D11).
- [ ] TECH-066 keeps archive and status behavior unchanged by the content model, states that group records archive without cascading and that the HTML mockup has no status verb while preserving fixed metadata; TECH-067 indexes config-declared fields through frontmatter_key and body headings and treats nested mapping sequences as not list-valued keys (MS-031 AC-7, ADR-0024).
- [ ] No TASK, MS, or REL ids appear in any body, and the user has verified the five updates (MS-031 AC-7, AC-8).
