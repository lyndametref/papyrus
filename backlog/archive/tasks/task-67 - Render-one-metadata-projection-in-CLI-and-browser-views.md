---
id: TASK-67
title: Render one metadata projection in CLI and browser views
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-09T19:25:00Z'
labels:
  - front-matter
  - cli
  - browser
  - metadata
milestone: m-12
dependencies:
  - TASK-63
  - TASK-64
  - TASK-65
  - TASK-66
references:
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-01.005.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - >-
    docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md
  - docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md
  - docs/requirements/business/REQ-06.003.md
priority: high
type: feature
ordinal: 30050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver ADR-0009 decision 5: one RecordView projection is the only source of record metadata in CLI list/view (plain text and --json), browser list/detail JSON, and browser HTML. Record identity is the Papyrus display ID everywhere; the Backlog.md storage ID appears only inside the metadata object (metadata.id) and in labelled diagnostics. --json gains an ordered metadata object (canonical front-matter key order) without changing existing keys. Browser editable field set is unchanged; created_date and updated_date are read-only. Browser saves patch front matter, preserve other keys and the rest of the file, refresh updated_date, and never copy metadata into the body. Tests cover CLI/browser parity, front-matter round trip on save, read-only timestamps, and absence of storage IDs in identity positions. Prerequisite: correct the three browser spec pages that currently claim front-matter preservation (browser-audit-records, browser-business-rules, browser-technical-specifications). See docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 One RecordView projection in src/record/ feeds CLI list/view plain text, CLI --json, browser list/detail JSON, and browser HTML; no surface re-parses front matter.
- [ ] #2 CLI and browser show identical metadata values for the same record: display id, title, created_date, updated_date, labels, references, and the type-specific keys.
- [ ] #3 --json adds an ordered metadata object in canonical front-matter key order; the top-level identity remains the display ID and any storage ID appears only inside metadata or a labelled diagnostic.
- [ ] #4 Browser editable fields are unchanged and created_date and updated_date are read-only (disabled inputs, never accepted from the request).
- [ ] #5 A browser save patches front matter, preserves other front-matter keys and the rest of the file, refreshes updated_date, and never copies metadata into the body.
- [ ] #6 Tests assert CLI/browser parity, front-matter round trip on save, read-only timestamps, and absence of storage IDs in identity positions.
- [ ] #7 The three browser specification pages that currently claim front-matter preservation (browser-audit-records, browser-business-rules, browser-technical-specifications) are corrected to state that front matter is patched, not preserved as a blob.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 RecordView type defined in src/record/ and used by every CLI and browser read surface.
- [ ] #2 CLI --json and browser list/detail JSON include the metadata object; existing keys unchanged.
- [ ] #3 Browser save uses record::patch + record::store::write_atomic; no fs::write on record paths.
- [ ] #4 Read-only timestamp and identity-position tests pass.
- [ ] #5 Three browser spec pages corrected.
- [ ] #6 cargo test (or project test suite) passes.
<!-- DOD:END -->
