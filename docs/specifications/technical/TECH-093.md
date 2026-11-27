---
id: TECH-093
title: "Unified Search"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T17:03:33Z'
labels: []
references:
  - REQ-009
  - REQ-009.001
status: verified
---
# TECH-093 Unified Search

## Description
Covers the single free-text search command across the backlog, decision, and document categories: its query, record type filters, result limit, archive handling, and output modes. Per-type list and view commands are outside this specification.

## Command

- `papyrus search <QUERY>` runs one free-text search across every managed record: tasks, milestones, releases, decisions, and every document type declared in the configuration.
- Search runs locally and requires no network access.
- The output is plain text by default and honours the global `--json` option.

## Matching

- A query matches record values, including record metadata, titles, and body text.
- A quoted phrase containing spaces is treated as one query.

## Type filters

- `--type` is repeatable and restricts the search to the selected record types.
- The accepted values are `task`, `milestone`, `release`, and the slug of any configured decision type or document type.
- A value that is neither a built-in type nor a configured type slug is rejected.
- With no `--type`, all record types are searched.

## Limit

- `--limit <n>` caps the number of returned results.

## Archive handling

- Archived records are excluded by default. A document, milestone, release, or decision moved into an archive folder is not returned, and neither is an archived task.
- `--include-archive` adds archived records of every category to the results.
- A returned archived record shows its archived state in the plain-text and JSON output.
- `--include-archive` combines with `--type` and `--limit`.
