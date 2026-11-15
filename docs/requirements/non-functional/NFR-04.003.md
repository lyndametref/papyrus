---
id: NFR-04.003
title: "Keep Task and Milestone Files Interoperable with Backlog.md"
created_date: '2026-11-11T19:56:15Z'
updated_date: '2026-11-15T15:47:30Z'
labels: []
references: []
---
# NFR-04.003 Keep Task and Milestone Files Interoperable with Backlog.md

## Description
Status: **Superseded by NFR-04.004.** Retained for traceability; no longer part of the governing persistence contract and no longer imposing behavior.

Papyrus task and milestone files were required to remain readable, listable, and updatable by Backlog.md without loss of the fields Backlog.md requires, including the additive milestone general keys and `release`, and legacy Papyrus-shape task and milestone files were converted to the Backlog.md on-disk layout on first read and first write. NFR-04.004 renounces file-level interoperability in favor of a one-way import script, so this requirement is deprecated in full.

## Rationale
The project originally stored tasks and milestones using Backlog.md conventions, and the requested front matter had to preserve that interoperability so both tools could operate on the same project files without a conversion step or a parallel store. Maintainer decision 2026-11-11 reversed that direction: ongoing compatibility and the associated lazy Backlog-layout conversion are removed (NFR-04.004), and an explicit one-way import replaces them.

## Verification Method
Not verified; superseded by NFR-04.004. The historical verification (two-direction round trip with Backlog.md on newly created and lazily migrated records) is retained only as a record of what the requirement asked.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), clarification: "insure the compatibility with task and milestone from backlog md, only acceptable discrpancy is the addition of releases for milestone. check the files created with backlog md in @backlog/ and how the folder is structured and implement the same behavior in papyrus"
- Superseded by NFR-04.004 (maintainer decision 2026-11-11).
- Historical user decisions 2026-11-08 (milestone additive keys, round-trip verification in both directions, migration shapes) are recorded here only as superseded history.
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (superseded design)
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
