---
id: TASK-20
title: Package and verify the complete content-management milestone
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-4
dependencies:
  - TASK-19
references:
  - docs/requirements/business/REQ-01.md
  - docs/requirements/business/REQ-02.md
  - docs/requirements/business/REQ-03.md
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-05.md
  - docs/requirements/business/REQ-06.md
  - docs/requirements/non-functional/NFR-01.md
  - docs/requirements/non-functional/NFR-02.md
  - docs/requirements/non-functional/NFR-03.md
  - docs/requirements/non-functional/NFR-04.md
  - docs/requirements/non-functional/NFR-05.md
  - docs/requirements/non-functional/NFR-06.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfreq-command-behavior.md
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The final planned feature increment must be installable and expose all currently specified CLI, search, and browser content workflows from its Debian package.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An amd64 Debian package installs on Debian Trixie and Forky and provides all implemented Papyrus commands, search, and browser access.
- [ ] #2 Installation smoke tests verify every supported record family can be managed, searched, and viewed/edited in the offline browser.
- [ ] #3 The packaged application satisfies the recorded Rust, GPL-3.0-or-later, dependency compatibility, project-local text persistence, path-access, security, and browser requirements.
<!-- AC:END -->
