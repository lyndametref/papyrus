---
id: TECH-097
title: "Browser Backlog Board"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T18:10:35Z'
labels: []
references:
  - REQ-006.013
status: verified
---
# TECH-097 Browser Backlog Board

## Description
Covers the implementation of the browser Backlog board: ordering computation, status-change submission, progressive drag enhancement, cleanup execution, and failure rendering. Routes and server-side rendering are a separate specification. The board's content, ordering rules, markers, and cleanup rules are specified by REQ-006.013; visual layout is specified by the UI specification.


## Board assembly

- Status columns are the configured task statuses plus a trailing `Unconfigured status` column for tasks whose stored status is not configured.
- A task whose milestone is absent or unresolved renders in the expandable `Tasks without a milestone` section after the milestone and release groups.
- The first milestone that has a non-archived task starts expanded; every other milestone section starts collapsed.
- A card carries the `POST /records/{id}` form for its status control, and the drag script reuses that form; no drag-specific endpoint or payload exists.

## Ordering computation

- Each render derives a sort key per milestone and per release from the non-archived tasks: the lowest task ordinal for a milestone, and the lowest ordinal across a release's milestones for a release.
- A milestone or release with no qualifying task sorts after every one that has a key, by identifier.
- Equal keys are ordered by identifier.
- The computation uses the record set of one request; changing an ordinal changes the order on the next render.

## Status change

- Without JavaScript, a card offers a `POST /records/{id}` form containing a select over the configured task statuses and a `Change status` submit control.
- Dragging a card is a progressive enhancement that submits the same change to the same target; there is no separate drag endpoint.
- The change uses the shared single-record write path and its version token. A conflict or validation failure writes nothing and re-renders the board with an alert carrying `role="alert"` above the groups; the card stays in its stored column.
- The board holds no staging state; a successful change is read back from the record on the next render.

## Cleanup execution

- `Clean up all` opens a server-rendered confirmation region below the milestone heading; the region is not a dialog, overlay, or script-driven prompt.
- The confirmation submits, and the cancel control is a plain link that performs no write.
- On confirmation each affected task is archived individually; the operation is not a transaction, a failure is reported for that task, the remaining tasks are still archived, and failed tasks stay on the board.
- A column with no affected task still opens the confirmation region but renders no confirm form, only the cancel link.
- A failed cleanup re-renders with an alert above the release groups that states the archived count out of the total, names the milestone, and lists each failed identifier with its message; successful tasks are absent from the re-read board.
- The cancel link returns to the board with the milestone expanded and performs no write.
- A closed milestone's cleanup archives that milestone; a released release's cleanup archives each closed milestone one at a time and never the release itself.
- Cleanup introduces no new write path, no batch transaction, and no command-line equivalent.

## Markers

- Lifecycle markers are rendered server-side from the stored status; no script derives or changes them.

## Constraints

- Every board function works without JavaScript; drag is the only enhancement.
- The board issues no outbound network request.
