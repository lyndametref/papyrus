---
id: TECH-010
title: "Built-in Record Attributes"
created_date: '2026-11-27T20:35:37Z'
updated_date: '2026-11-27T20:35:37Z'
labels: []
references:
  - REQ-002.001
  - REQ-002.005
  - REQ-002.006
  - REQ-004.001
status: verified
---
# TECH-010 Built-in Record Attributes

## Description
Covers the attributes of the record types that Papyrus defines rather than configures: task, milestone, release, and decision. Document-type attributes come from configuration. Identifiers, statuses, dates, create and edit behavior, archive and delete, command families, and views and lists are separate specifications.

## Task

| Attribute | Rule |
|---|---|
| Title | Required |
| Description | Optional; multiline |
| Acceptance criteria | Repeatable; one criterion each |
| Dependencies | Repeatable |
| Milestone | Optional; at most one |
| Labels | Repeatable; free-form classification |
| References | Repeatable; may name any managed record |
| Assignees | List |
| Ordinal | Optional non-negative whole number |

- The ordinal gives the task its execution order, is optional on create, and when omitted is derived from the current maximum.
- The ordinal is the value by which the Backlog board orders milestones and releases.
- Acceptance criteria are shown by the `--ac` view option and one criterion is stored per list item.

## Milestone

| Attribute | Rule |
|---|---|
| Title | Required |
| Description | Optional |
| Release | Optional; at most one |
| Acceptance criteria | Repeatable `--ac`; one bullet per criterion in the body |
| Labels | Repeatable |
| References | Repeatable; may name any managed record |

- A release may contain multiple milestones, and each milestone is assigned to at most one release.
- A milestone that names a release that no longer exists is a dangling reference reported by `papyrus check`.

## Release

| Attribute | Rule |
|---|---|
| Title | Required |
| Description | Optional |
| Labels | Repeatable |
| References | Repeatable; may name any managed record |

## Decision

| Attribute | Rule |
|---|---|
| Title | Required |
| Decision text | Multiline |
| Context | Multiline |
| Consequences | Multiline |
| Labels | Repeatable |
| References | Repeatable; may name any managed record |
| Superseded by | Optional successor identifier, written only by `supersede` |

- A decision follows the Nygard template: title, decision text, context, and consequences.
- A decision is superseded by at most one successor of the same decision type.

## Shared rules

- Every built-in record type has a required, single-line title.
- Labels are free-form classification and are not restricted by any list.
- A reference may name any managed record, and no record type is excluded.
- These attributes are fixed by Papyrus and are not declared in configuration.
