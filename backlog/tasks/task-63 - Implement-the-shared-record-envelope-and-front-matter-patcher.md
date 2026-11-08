---
id: TASK-63
title: Implement the shared record envelope and front-matter patcher
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - front-matter
  - record-envelope
  - architecture
milestone: m-12
dependencies: []
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: feature
ordinal: 30010
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build the single shared record implementation required by ADR-0009 Stage 1 so that every record type has one front-matter parser, one lossless document envelope, one patch API, and one atomic store. This is the foundation for front matter, CLI/browser parity, and the bulk migration; it does not migrate the corpus.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/record/frontmatter detects a leading YAML block at byte 0, parses it with yaml_serde, keeps key order and unknown keys, and patches changed keys without re-serializing the block.
- [ ] #2 src/record/document splits a file into front matter plus ordered body units and preserves every unmodified line byte-for-byte.
- [ ] #3 src/record/patch is the only mutation API; an empty patch or a patch with unchanged values returns byte-identical text.
- [ ] #4 Record descriptors exist for task, milestone, release, decision, business requirement, NFR, glossary term, UI specification, technical specification, business rule, and audit record; UI mockup HTML is excluded.
- [ ] #5 Readers accept both legacy files without front matter and files with front matter, using front matter first and section/heading fallback.
- [ ] #6 src/record/store provides safe path resolution, read, atomic write, and the ADR-0006 version token; no record module calls fs::write.
- [ ] #7 Tests cover no-op byte identity, unknown key preservation, unknown section preservation, and both read shapes; the no-drift grep checks pass for this stage.
<!-- AC:END -->
