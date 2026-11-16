---
id: NFR-04.004
title: "Renounce Backlog.md Compatibility and Provide Import"
created_date: '2026-11-11T20:17:13Z'
updated_date: '2026-11-16T20:31:52Z'
labels: []
references: []
status: verified
---
# NFR-04.004 Renounce Backlog.md Compatibility and Provide Import

## Group
NFR-04 Persistence


## Description
Papyrus shall no longer maintain file-level interoperability with Backlog.md. Instead, Papyrus shall provide a one-way script that imports Backlog.md backlog records into Papyrus records. The one-way import covers Backlog.md task and milestone records; Backlog.md `docs`, `drafts`, and `decisions` are out of scope. Imported records receive Papyrus IDs (REQ-01.005) and are written in the Papyrus record format with the front matter of REQ-05.010.

The following Backlog.md compatibility behavior shall be removed: the Backlog.md key-set and body-layout contract for task and milestone files, the byte-identical Backlog.md body contract, the translation between Papyrus display IDs and Backlog.md storage IDs, and the lazy conversion of legacy task and milestone files to the Backlog.md on-disk layout. No read or write path shall rewrite a record into a Backlog.md shape or assign a Backlog.md storage ID. The import is explicit and one-way: after import Papyrus owns the records, Backlog.md compatibility is not guaranteed, and an existing Backlog.md project is brought over by running the import script once. Lazy adoption of YAML front matter by managed Markdown records (REQ-05.011) is a Papyrus-native behavior and is unaffected by this renunciation.

## Rationale
This records the strategic decision to renounce Backlog.md compatibility in favor of a one-way import script, removing the ongoing compatibility and lazy-migration burden carried by NFR-04.003, REQ-05.010, and REQ-05.011. A single Papyrus ID space removes the dual-identity mapping and its failure modes, and the import keeps a supported path for projects that already hold Backlog.md backlog files. Maintainer decision 2026-11-11 authorized this direction.

## Verification Method
Review the storage, identity, and write paths to confirm that no Backlog.md compatibility layer remains: no Backlog.md key-set or body contract, no display-to-storage ID translation, no lazy conversion to the Backlog.md layout, and no read or write path that rewrites a record into a Backlog.md shape. Run the import script against a sample Backlog.md backlog and confirm that each task and milestone becomes a Papyrus record with a Papyrus ID, its metadata, and its body content preserved. Then run Papyrus commands against the imported project and confirm that Backlog.md `docs`, `drafts`, and `decisions` are neither read nor rewritten.

## Open Questions
- Native on-disk file layout and path convention for task and milestone records (Papyrus-owned; an architecture decision, not fixed by this requirement). **Resolved 2026-11-11 by ADR-0010** — Papyrus-native `<ID>.md` records under configuration-driven roots, one record per file, one Papyrus ID, no storage ID.
- Collision handling when the import meets an existing Papyrus record with the same number or title. **Resolved 2026-11-11 by ADR-0011** — allocation makes number collisions impossible, a same-title existing record is imported as a new record with a "possible duplicate" warning, no merge or overwrite.

## Traceability
- Source: customer-feedback handoff — maintainer, Item 12.
- Maintainer decision 2026-11-11: adopt this requirement and supersede NFR-04.003; import covers tasks and milestones only; single Papyrus ID scheme; existing Backlog.md projects are imported once; lazy front-matter adoption (REQ-05.011) is unaffected.
- NFR-04.003 — Keep Task and Milestone Files Interoperable with Backlog.md (superseded)
- REQ-01.005 — Assign Stable Record IDs (single ID space)
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source (superseded by ADR-0012)
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (superseded by ADR-0010)
- ADR-0010 — Papyrus-Native On-Disk Record Layout and Path Convention (native layout, single ID space)
- ADR-0011 — One-Way Backlog.md Import as the Only Bridge (import scope, ID assignment, collisions)
- ADR-0012 — Papyrus-Owned Front Matter as the Single Record Metadata Source
- ADR-0013 — Shared Record Envelope and Single Write Path Without a Storage Layer
