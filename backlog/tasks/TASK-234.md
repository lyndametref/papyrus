---
id: TASK-234
title: "Update TECH-040, TECH-042, TECH-043, TECH-066, and TECH-067 for front-matter keys, patch ops, and search"
created_date: '2026-11-20T17:40:19Z'
updated_date: '2026-11-20T17:40:19Z'
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
status: ToDo
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
