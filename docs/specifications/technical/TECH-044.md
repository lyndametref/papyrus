---
id: TECH-044
title: "Migrate Tasks and Milestones to the Backlog.md Layout"
created_date: '2026-11-16T17:46:48Z'
updated_date: '2026-11-16T17:47:19Z'
labels: []
references:
  - ADR-0007
  - ADR-0008
  - ADR-0009
  - ADR-0010
  - ADR-0011
  - ADR-0012
  - ADR-0013
  - NFR-004.004
  - REQ-001.005
  - TASK-69
status: verified
---
# TECH-044 Migrate Tasks and Milestones to the Backlog.md Layout

## Description
## Purpose

This specification is **superseded in intent** and no longer states a live behaviour contract. Its subject — lazy migration of task and milestone files to the Backlog.md on-disk layout, Backlog.md storage IDs, the `m-0` offset, and the byte-identical Backlog.md body contract — was renounced by NFR-004.004 and replaced by the Papyrus-native layout of ADR-0010. The live contract is now defined by:

- `docs/specifications/papyrus-native-record-layout-and-single-id-space.md` (native layout and single ID space)
- `docs/specifications/import-backlog-md-records.md` (one-way Backlog.md import)

Do not implement against this document.

## Scope

### In scope (historical)

None. This document is retained only as a traceability pointer to the superseded ADR-0008/ADR-0009 Stage 4 work.

### Out of scope

All former in-scope and out-of-scope items are withdrawn; see the superseding specifications above.

## Superseding decisions

- ADR-0010 — Papyrus-Native On-Disk Record Layout and Path Convention (supersedes ADR-0008)
- ADR-0011 — One-Way Backlog.md Import as the Only Bridge (supersedes the round-trip gate)
- ADR-0012 — Papyrus-Owned Front Matter as the Single Record Metadata Source (supersedes ADR-0007)
- ADR-0013 — Shared Record Envelope and Single Write Path Without a Storage Layer (supersedes ADR-0009)

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-69 (historical) |
| Requirements | NFR-004.004 (renunciation); REQ-001.005 (single ID space) |
| ADRs | ADR-0010, ADR-0011, ADR-0012, ADR-0013 (superseding); ADR-0007, ADR-0008, ADR-0009 (superseded) |
| Superseding specifications | `papyrus-native-record-layout-and-single-id-space.md`, `import-backlog-md-records.md` |
| UI mockups | none |
