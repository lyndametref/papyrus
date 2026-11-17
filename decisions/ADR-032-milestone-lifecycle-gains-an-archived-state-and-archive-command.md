---
id: ADR-032
title: "Milestone lifecycle gains an archived state and archive command"
created_date: '2026-11-17T18:51:43Z'
updated_date: '2026-11-17T18:54:26Z'
labels: []
references:
  - REQ-03.001
  - RE-002
  - BR-006
  - BR-001
  - GLO-006
  - TECH-009
  - ADR-0017
  - ADR-0022
  - ADR-0023
  - ADR-0010
  - ADR-0013
  - ADR-0006
superseded_by: null
---
# ADR-032 Milestone lifecycle gains an archived state and archive command

## Status
accepted
## Decision
1. Milestone lifecycle is fixed open to closed to archived. papyrus milestone close id remains the only open to closed transition and writes status and updated_date in place. 2. papyrus milestone archive id is the only transition to archived: it closes an open milestone if needed, then sets status to archived, refreshes updated_date, preserves title, description, labels, references and release assignment, and moves the file to the backlog archive milestones folder using the shared atomic record writer. An already archived milestone is a no-op success. Archive is refused for any status other than open or closed. 3. No restore command exists for milestones; only tasks keep archive and restore because REQ-01.005 requires task IDs to survive that cycle. 4. The milestone descriptor gains the archive verb and an archive location of the backlog archive milestones folder, mirroring the task backlog archive location, and its declared status vocabulary becomes open, closed, archived. ArchivePolicy for milestones is no longer None. 5. The backlog archive milestones folder is now both the lifecycle archive destination and the legacy read and import location: it remains readable by list, view, search and browser resolution and participates in ID allocation as before; the one-way Backlog.md import remains its only other writer. 6. Archived milestones are hidden from active listings because list reads only the active folder, but remain viewable by ID and editable in place as legacy data. 7. Configuration cannot change the milestone lifecycle, verbs, or archive location.

## Context
REQ-03.001 and RE-002 now require papyrus milestone archive id: close an open milestone if needed, then move its file to the backlog archive milestones folder and mark it archived, with no restore command (BR-006). This reverses the close-only milestone lifecycle in ADR-0022 decision 5 (milestones no longer keep archive or restore; fixed open to closed) and the milestone row of ADR-0023 decision 2 (archive location none; backlog archive milestones read and import only), and the milestone verb set in ADR-0017 (no archive verb). The milestone lifecycle becomes open to closed to archived. The backlog archive milestones folder, previously read and import only, becomes the milestone lifecycle archive location while retaining import readability and ID allocation participation. ADR-0022 and ADR-0023 remain Accepted for their other decisions; this ADR is the authoritative delta for the milestone archive clause.

## Consequences
Positive: milestones gain a cleanup and archive path consistent with task archive and backlog archive conventions; manual file moves are eliminated; BR-006 and REQ-03.001 are satisfied. Negative: reverses the prior close-only milestone decision, so code, descriptors and tests that assert milestones have no archive location or no archive verb must change, including the milestone has no lifecycle archive location assertion and the ADR-0017 revision notes. The archive folder now serves two roles, import destination and lifecycle archive, which must be documented to avoid confusion. Risks: collision between imported legacy archived milestones and newly archived ones is handled by max plus one allocation over active plus archive locations; no automatic migration of existing closed milestones; archived milestones remain readable legacy data.
