---
id: TASK-112
title: Update init defaults and seed document statuses
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
  - TASK-109
documentation:
  - docs/requirements/business/REQ-07.002.md
  - docs/requirements/business/REQ-01.001.md
priority: high
ordinal: 41060
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`Config::default()` still seeds the removed `deploy` entry, and the generated configuration declares no `statuses` for the default document types. REQ-07.002 fixes the default set at eleven entries without `deploy` and makes `papyrus init` the surface that writes it; REQ-01.006's walkthrough disables entries from the same list. This task aligns the generated configuration, the interactive walkthrough, and `init --force` so a new project starts from exactly the declared default set and a valid status model.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The generated default document_types map contains exactly req, nfr, glossary, rule, ui-spec, ui-mockup, audit, tech-spec, use-case, test-case, and test-report with their specified prefixes and directories; deploy is absent.
- [ ] #2 Every front-matter-backed default entry seeds statuses as active, archived with no custom status; ui-mockup declares no statuses; no entry seeds default_status.
- [ ] #3 papyrus init writes the same enabled set the interactive walkthrough presents, keeps the existing prefix/directory values, and disable choices still remove a type from management without touching files.
- [ ] #4 papyrus init --force replaces an existing configuration, including an invalid one, with this default set and the result passes the ADR-0019 validator.
- [ ] #5 A project may still add deploy as an ordinary user-defined document type after init.
- [ ] #6 Tests assert the exact default map, the seeded status lists, the absence of deploy, walkthrough disabling, and init --force against a broken configuration.
<!-- AC:END -->
