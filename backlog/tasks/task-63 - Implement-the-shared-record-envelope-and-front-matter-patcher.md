---
id: TASK-63
title: Implement the shared record envelope and front-matter patcher
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T16:48:00Z'
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
  - docs/requirements/non-functional/NFR-04.003.md
documentation:
  - docs/specifications/shared-record-envelope-and-front-matter-patcher.md
priority: high
type: feature
ordinal: 30010
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build ADR-0009 Stage 1: one shared record implementation inside the existing crate so every managed record type has one front-matter parser, one lossless document envelope, one patch API, and one atomic store. This is the foundation for front matter, CLI/browser parity, and the later lazy migration; it does not migrate the corpus and does not change command surfaces.

Scope: create src/record/ with frontmatter, document, patch, and store modules; define record descriptors for all managed Markdown record types except UI mockup HTML; make readers accept both legacy and front-matter shapes using front-matter-first fallback. Out of scope: identity translation (TASK-64), CLI option exposure (TASK-65), lazy migration (TASK-68/TASK-69), legacy removal (TASK-70), browser UI/projection (TASK-66/TASK-67/TASK-72), and any change to existing record files.

Assumptions: yaml_serde remains the only YAML dependency; Context::ensure_within is reused for path safety; atomic writes follow the ADR-0006 create_new + sync_all + rename sequence; no corpus migration occurs in this ticket. Dependencies: none.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/record/frontmatter detects a leading YAML block at byte 0, parses it with yaml_serde, preserves key order and unknown keys, and patches changed keys without re-serializing the block.
- [ ] #2 src/record/document splits a file into front matter plus ordered body units and preserves every unmodified line byte-for-byte.
- [ ] #3 src/record/patch is the only mutation API; an empty patch or a patch with unchanged values returns byte-identical text.
- [ ] #4 PatchOp includes SetFrontMatter, ClearFrontMatter, SetSection, ClearSection, and SetHeadingTitle.
- [ ] #5 Record descriptors exist for task, milestone, release, decision, business requirement, NFR, glossary term, UI specification, technical specification, business rule, and audit record; UI mockup HTML is excluded.
- [ ] #6 Readers accept both legacy files without front matter and files with front matter, using front matter first and section/heading fallback.
- [ ] #7 src/record/store provides safe path resolution, read, atomic write, and the ADR-0006 version token; no record module calls fs::write.
- [ ] #8 Atomic write failures leave the original file byte-identical and remove temporary files.
- [ ] #9 Malformed front matter produces a validation error rather than a panic.
- [x] #10 Tests cover no-op byte identity, unknown key preservation, unknown section preservation, and both read shapes; no-drift grep checks pass for this stage.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo test passes for the record module and affected integration tests.
- [ ] #2 No fs::write call on a record path exists outside src/record/store.
- [ ] #3 No format!("---") or equivalent front-matter delimiter formatting exists outside src/record/.
- [ ] #4 Feature specification is updated and linked from the ticket.
- [ ] #5 No existing corpus record file is modified by this stage.
<!-- DOD:END -->
