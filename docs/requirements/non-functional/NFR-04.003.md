# NFR-04.003 Keep Task and Milestone Files Interoperable with Backlog.md

## Description
Papyrus task and milestone files shall remain readable, listable, and updatable by Backlog.md without loss of the fields Backlog.md requires. Task files carry the Backlog.md front-matter key set. Milestone files may carry the general front-matter keys in addition to `id`, `title`, and `release`; Backlog.md must still read, list, and update these files, and the additive general keys are accepted and preserved. Release records have no Backlog.md counterpart and are outside this requirement's scope, as are Backlog.md `docs` and `drafts` records.

## Rationale
The project stores tasks and milestones using Backlog.md conventions, and the requested front matter must preserve that interoperability. The quality attributes are interoperability and maintainability: both tools operate on the same project files without a conversion step or a parallel store. Additive keys are tolerated rather than forbidden, because Papyrus needs metadata (general keys, milestone `release`) that Backlog.md does not write itself.

## Verification Method
Create and edit tasks and milestones with Papyrus, then run Backlog.md list, view, and update commands against the same project and confirm that every required field is read and preserved, including the additive milestone general keys and `release`. Repeat in the other direction: create and edit records with Backlog.md, then read, list, update, and view them with Papyrus. Run both directions on newly created records and on legacy records migrated lazily on first read and on first write, covering both migration input shapes: files already in Backlog.md shape keep a byte-identical body apart from the front-matter block, and legacy Papyrus-shape files are converted on first read or first write and then round-trip. Verify migration is idempotent across repeated reads and writes in both directions.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), clarification: "insure the compatibility with task and milestone from backlog md, only acceptable discrpancy is the addition of releases for milestone. check the files created with backlog md in @backlog/ and how the folder is structured and implement the same behavior in papyrus"
- User decision 2026-11-08 (NFR-04.003 resolution): milestones may carry the general front-matter keys in addition to `id`, `title`, and `release`; the round-trip verification runs in both directions; additive general keys are accepted and preserved; `docs`/`drafts` are out of scope.
- User decision 2026-11-08 (migration shapes, revised): the round-trip verification covers records migrated lazily on first read and first write from both input shapes, including legacy Papyrus-shape files converted at that time (REQ-05.011).
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source (field map, patch contract preserving foreign keys)
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (on-disk interoperability contract)
- backlog/tasks/*.md, backlog/milestones/*.md — observed Backlog.md front-matter conventions
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
