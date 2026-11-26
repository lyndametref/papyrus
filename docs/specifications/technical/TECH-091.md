---
id: TECH-091
title: "Backlog: Milestones and Releases"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-002.002
  - REQ-002.005
  - REQ-002.006
status: draft
---
# TECH-091 Backlog: Milestones and Releases

## Description
Covers the milestone and release records, their attributes, their commands,
their fixed lifecycles, the assignment between them, and their archive behavior.
Tasks, the shared view and list contract, and the Backlog board are outside this
specification.

## Milestone record

| Attribute | Rule |
|---|---|
| Identifier | Configured milestone prefix plus the shared `id_padding`; three digits with default configuration |
| Title | Required |
| Description | Optional |
| Release | Optional; at most one release |
| Acceptance criteria | Repeatable `--ac`; stored as one bullet per criterion in the body |
| Labels | Repeatable |
| References | Repeatable |
| Status | Stored in the front-matter `status` field |

## Milestone lifecycle

The lifecycle is fixed, not configurable: `open`, then `closed`, then `archived`.
A new milestone starts `open`. There is no restore command.

- `papyrus milestone create <TITLE>` creates a milestone and accepts an optional
  description, an optional release assignment, repeatable acceptance criteria,
  labels, and references.
- `papyrus milestone close <ID>` changes `open` to `closed`. Closing a milestone
  that is not open is rejected with a validation outcome and writes nothing.
- `papyrus milestone archive <ID>` closes an open milestone and then writes
  `archived`, moving the file to `backlog/archive/milestones/`.
- An archived milestone is hidden from active lists and from the Backlog board,
  and keeps its identifier.
- `papyrus milestone edit <ID>` changes the title, the description, the release
  assignment, the acceptance criteria, the labels, or the references. No option
  sets the lifecycle state apart from `close` and `archive`.
- `papyrus milestone delete <ID>` permanently removes the milestone, asks for
  confirmation, and `--force` skips the prompt.
- `papyrus milestone view <ID> --ac` prints only the acceptance criteria: one
  criterion per line in plain text, and an array under `--json`.

## Release record

| Attribute | Rule |
|---|---|
| Identifier | Configured release prefix plus the shared `id_padding`; three digits with default configuration |
| Title | Required |
| Description | Optional |
| Labels | Repeatable |
| References | Repeatable |
| Status | Stored in the front-matter `status` field |

## Release lifecycle

The lifecycle is fixed, not configurable: `planned`, then `released`, then
`archived`. A new release starts `planned`. There is no restore command.

- `papyrus release create <TITLE>` creates a release and accepts an optional
  description, labels, and references.
- `papyrus release close <ID>` changes `planned` to `released`; `released` means
  the delivery happened. Closing a release that is not planned is rejected with
  a validation outcome and writes nothing.
- `papyrus release archive <ID>` closes a release that is still planned and then
  writes `archived`, moving the file to `backlog/archive/releases/`.
- An archived release is hidden from active lists and from the Backlog board,
  and keeps its identifier.
- `papyrus release edit <ID>` changes the title, the description, the labels, or
  the references. No option sets the lifecycle state apart from `close` and
  `archive`.
- `papyrus release delete <ID>` permanently removes the release, asks for
  confirmation, and `--force` skips the prompt.

## View and list

- `papyrus milestone view <ID>` shows the identifier, title, status, description,
  assigned release, acceptance criteria, labels, and references (REQ-002.002).
- `papyrus release view <ID>` shows the identifier, title, status, description,
  labels, and references (REQ-002.002).
- `papyrus milestone view <ID> --ac` prints only the acceptance criteria: one
  criterion per line in plain text, and an array under `--json` (REQ-002.002).
- `papyrus milestone list` and `papyrus release list` print their records as plain
  text or JSON and accept no filter beyond the global options (REQ-002.002).

## Milestone-to-release assignment

- A release may contain multiple milestones; each milestone is assigned to at
  most one release.
- Archiving or deleting a release changes no milestone: each milestone keeps its
  file, its state, and its release assignment.
- A milestone that still names a release that no longer exists is reported as a
  dangling reference by `papyrus check`.
