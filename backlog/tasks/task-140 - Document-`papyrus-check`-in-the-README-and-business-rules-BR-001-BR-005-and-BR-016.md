---
id: TASK-140
title: >-
  Document `papyrus check` in the README and business rules BR-001, BR-005, and
  BR-016
status: To Do
assignee:
  - '@tech-writer'
created_date: '2026-11-15T16:12:00Z'
labels:
  - docs
  - cli
  - business-rules
milestone: m-21
dependencies:
  - TASK-139
references:
  - README.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
priority: high
ordinal: 41530
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** `papyrus check` is CLI-only and README is the behaviour source for the command (BR-001, BR-014), yet nothing documents it. The tolerant-surface change also alters the documented configuration and command contracts in BR-001, BR-005, and BR-016.

**What.** Document the command and re-align the business rules that quote the command surface.

**In scope.**
- README `COMMANDS` section gains `check`: synopsis, `--fix`, `--force`, output shape, finding kinds, exit statuses (0/4/1/2), the tolerant configuration behaviour, the read-only guarantee, and the intended `--fix` then clean-second-run loop.
- State that `--force` is meaningful only with `--fix` and that a non-TTY run without `--force` writes nothing and exits 2.
- BR-001: record `check` as a built-in command outside the document-family command set, add exit 4 as a findings outcome that is not an error, and note the read-only default.
- BR-005: re-sync the quoted README excerpt range if the new section shifts it; change no task behaviour text.
- BR-016: list `check` among the tolerant repair surfaces that run while the configuration is invalid, and add it to the reserved built-in names that a document-type slug may not take.
- Keep requirement files untouched; report any wording conflict with REQ-01.008 or REQ-07.002 instead of editing them.
- Regenerate the business-rules index if the change touches indexed headings.

**Out of scope.** Specifications (TASK-148), ADRs, requirement text, release notes, and man-page content.

**Assumptions.** ADR-0021 (man page) is still Proposed, so README remains the documented contract. BR-005 needs only range re-synchronisation, not new behaviour.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 README documents check with its synopsis, flags, finding kinds, exit statuses, tolerant configuration behaviour, and read-only guarantee.
- [ ] #2 README states the --fix then clean-second-run loop and the non-TTY and --force rules.
- [ ] #3 BR-001 records check as a built-in outside the document-family command set and adds exit 4 as a findings outcome that is not an error.
- [ ] #4 BR-005 quoted README excerpt ranges are re-synchronised if the new section shifts them, with no task behaviour text changed.
- [ ] #5 BR-016 lists check among the tolerant repair surfaces and among the reserved built-in names.
- [ ] #6 No requirement file, ADR, or specification is edited, and any wording conflict is reported instead.
<!-- AC:END -->
