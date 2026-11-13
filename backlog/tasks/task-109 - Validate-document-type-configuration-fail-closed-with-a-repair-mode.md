---
id: TASK-109
title: Validate document-type configuration fail-closed with a repair mode
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
  - TASK-108
documentation:
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - docs/requirements/business/REQ-07.002.md
priority: high
ordinal: 41040
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Today `Config::validate` checks paths but not document slugs, entry prefixes, reserved names, status lists, or duplicate YAML keys; the defaults merge also makes removing a default entry impossible and re-adds `deploy`. REQ-07.002 requires a fail-closed, aggregated validator with a whole-map `document_types`, plus two repair surfaces so a broken file can still be fixed through Papyrus (`config get|set|unset|list` tolerant load with a validated write, and `init --force`). Without this, a typo locks every command out or silently produces a partial type set.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Config::validate is the only validator for document types and aggregates every violation into one AppError::Validation (exit 4) message in document_types.<slug>.<field> form, in deterministic order.
- [ ] #2 Slug validation rejects empty values, non-ASCII/unsupported characters, a leading -, whitespace, control characters, path separators, and .. without trimming or normalizing; reserved built-in names come from cli::BUILTIN_COMMANDS and are compared case-insensitively.
- [ ] #3 Prefix validation rejects empty, whitespace, control characters, path separators, and traversal, and rejects a prefix used by more than one configured entry including disabled ones; enabled entries reject duplicate directories; each directory stays within the project directory after symlink resolution.
- [ ] #4 Status lists are rejected when empty, duplicated, missing active or archived, or containing an invalid value; default_status must be declared and must not be archived; a statuses declaration on ui-mockup is rejected.
- [ ] #5 A repeated YAML mapping key is detected on the raw file before defaults are merged and rejected as invalid configuration; document_types is replaced wholesale when present, so removing an entry is expressible and no default is re-added.
- [ ] #6 config get|set|unset|list load tolerantly, apply the change, validate the resulting configuration with the same validator, and write only a valid result; an invalid result is refused and the file is byte-identical.
- [ ] #7 init --force replaces the configuration with the generated default set even when the existing file is invalid; repair mode never executes record operations or builds a registry from an invalid set.
- [ ] #8 ArtifactRegistry::build re-asserts the same invariants as defence in depth; tests cover each rejection, aggregated diagnostics, the tolerant-load/validated-write distinction, and a symlinked directory escape.
<!-- AC:END -->
