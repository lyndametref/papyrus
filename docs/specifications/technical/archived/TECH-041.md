---
id: TECH-041
title: "Translate Papyrus and Backlog.md IDs at the Storage Boundary"
created_date: '2026-11-16T17:46:48Z'
updated_date: '2026-11-16T17:47:18Z'
labels: []
references:
  - ADR-0008
  - ADR-0009
  - ADR-0010
  - ADR-0013
  - NFR-004.004
  - REQ-001.005
  - TASK-64
status: archived
---
# TECH-041 Translate Papyrus and Backlog.md IDs at the Storage Boundary

## Description
## Purpose

This specification is **superseded in intent** and no longer states a live behaviour contract. Its subject — bidirectional translation between Papyrus display IDs and Backlog.md storage IDs, fixed per-type offsets (task `0`, milestone `1`), storage prefixes, and the `m-0` base — was renounced by NFR-004.004 and REQ-001.005. Papyrus now keeps a single ID space with no translation layer. The live contract is now defined by:

- `docs/specifications/papyrus-native-record-layout-and-single-id-space.md` (single ID space, allocation without offsets, milestones starting at 1)

Do not implement against this document.

## Scope

### In scope (historical)

None. This document is retained only as a traceability pointer to the superseded ADR-0008/ADR-0009 Stage 2 work.

### Out of scope

All former in-scope and out-of-scope items are withdrawn; see the superseding specification above.

## Superseding decisions

- ADR-0010 — Papyrus-Native On-Disk Record Layout and Path Convention (supersedes ADR-0008)
- ADR-0013 — Shared Record Envelope and Single Write Path Without a Storage Layer (supersedes ADR-0009)

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-64 (historical) |
| Requirements | NFR-004.004 (renunciation); REQ-001.005 (single ID space) |
| ADRs | ADR-0010, ADR-0013 (superseding); ADR-0008, ADR-0009 (superseded) |
| Superseding specification | `papyrus-native-record-layout-and-single-id-space.md` |
| UI mockups | none |
