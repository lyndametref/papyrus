---
id: TECH-066
title: "Refactor the Record Layer for Document Archive and Status"
created_date: '2026-11-16T17:46:53Z'
updated_date: '2026-11-21T15:36:11Z'
labels: []
references:
  - ADR-0006
  - ADR-0013
  - ADR-0022
  - ADR-0023
  - ADR-0036
  - ADR-0039
  - REQ-05.007
  - REQ-05.008
  - REQ-05.010
  - REQ-05.011
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - ADR-032
  - REQ-03.001
status: verified
---
# TECH-066 Refactor the Record Layer for Document Archive and Status

## Description
Archive, delete, status storage, and visibility are defined once in the record
layer and are the same for every document model. A type's field sequence adds no
state, no lifecycle, and no storage location.
## Behavior

### 1. Status is independent of the content model

1. A `SingleMarkdown` or `GroupedMarkdown` record stores its main status in the
   front-matter `status` key. The value is `active`, `archived`, or a custom
   status declared for the type.
2. A `SingleHtmlMockup` record stores its state in the HTML metadata comment.
   The states are fixed to `pending`, `approved`, and `archived`; no custom or
   default status is declared for that model.
3. A declared field, a role field set, and a field sequence add no state. They
   change no status location, introduce no per-type lifecycle, and define no
   folder per status.
4. Only `active` and `archived` affect storage location. A custom status is a
   front-matter value only: it causes no move and has no folder.
5. The only writers of a document status are the `status` verb and the `archive`
   verb, plus `supersede` for the HTML model. No create, edit, or field option
   writes it.
6. A non-conforming status value already on disk is preserved verbatim, reported
   in diagnostics, and blocks status writes until the value or the configuration
   is corrected. Archive and delete still work.

### 2. Directory resolution

1. Every document descriptor resolves its active directory, its `archived/`
   subdirectory, and its read directories from configuration, contained by the
   project-boundary check.
2. Read directories are the active directory plus `archived/`. A legacy inactive
   folder is not a read directory.
3. No production path resolves, reads, scans, lists, migrates, or inventories a
   legacy inactive folder.
4. A task descriptor keeps its own archive location, and its legacy read-only
   alias remains readable where it already exists.
5. A milestone lifecycle is fixed `open -> closed -> archived` and is not
   configurable. `close` is the only `open -> closed` transition: it writes
   status and `updated_date` in place and moves no file.
6. `milestone archive` is the only transition to `archived`. It closes an open
   milestone first, then patches `status` to `archived`, refreshes
   `updated_date`, preserves the title, description, labels, references, and
   release assignment, and writes atomically into the backlog archive milestones
   folder. An already archived milestone is a no-op success. Any other status,
   including a non-conforming value, is refused with the file left untouched.
7. The milestone descriptor gains the archive verb and the backlog archive
   milestones folder as its archive location, and declares `open`, `closed`, and
   `archived` as its status vocabulary; its archive policy is no longer `none`.
   Configuration cannot change the milestone lifecycle, its verbs, or its
   archive location.
8. The backlog archive milestones folder serves two roles: it is the lifecycle
   archive destination and the legacy read and import location. It stays
   readable by list, view, search, and browser resolution, and it participates in
   allocation. The one-way import remains its only other writer.
9. Archived milestones are hidden from active listings because a listing reads
   only the active directory. They remain viewable by ID and editable in place as
   legacy data.
10. No restore command exists for milestones: nothing returns an archived
    milestone to `open` or `closed`. Only tasks keep archive and restore.
11. A decision descriptor has no archive directory: a decision is never moved and
    its state stays in its status section.

### 3. Archive contract

`store::archive(ctx, descriptor, id)` behaves as follows:

1. It resolves the record from the active and archived locations.
2. If the record is already archived, it returns the existing path with no write
   and no `updated_date` change. Archive is therefore idempotent.
3. If the record is active with status `archived`, it relocates the file to the
   archived directory using the destination-write-then-unlink sequence, leaves
   the status value as it is, and returns the archived path. This is the store
   counterpart of the read-path relocation.
4. Otherwise it patches `status` to `archived`, writes the destination
   atomically, and removes the source only after that write succeeds.
5. On a destination-write failure the source is untouched and the error is
   returned. On a source-unlink failure after a successful destination write the
   source is untouched and the destination exists; the runtime error names both
   paths, and duplicate detection then makes the record unreadable for writes
   until one copy is removed.
6. It creates the destination directory when absent, contained in the project.
7. It preserves the record ID and refreshes `updated_date` only because the
   content changed.
8. A custom status is replaced by `archived` and the previous value is not
   retained anywhere.

### 4. Read-path relocation

1. When a read, list, view, search, or browser lookup finds a record in the
   active directory whose stored status is `archived`, it relocates the file
   with the sequence of section 3 before returning the record.
2. A successful relocation is reported in diagnostics.
3. If the relocation fails, the source stays intact, no partial state is left,
   the record is returned read-only with a validation outcome naming both paths,
   and writes to it are refused until the conflict is resolved.
4. Relocation on read is the only write permitted on a read path.
5. A record carrying a custom status is not relocated.

### 5. Delete contract

`store::delete(ctx, descriptor, id)` behaves as follows:

1. An active record is archived first, then the archived file is removed.
2. An archived record is removed directly.
3. No confirmation and no force flag belong to the store; they are a command
   concern.
4. A deleted record leaves no file. Allocation is `max(observed) + 1` over active
   and archived locations with no high-water mark, so deleted numbers generally
   remain gaps while the highest number can be reused.
5. No document restore operation exists: no command returns an archived record to
   `active`.

### 6. Group records

1. Archive and delete apply per record. Archiving or deleting a group record
   changes no member's file, status, or ID, and archiving or deleting a member
   changes no other member and no group record.
2. Each record is archived or deleted only by a command that names it.
3. Archiving a member preserves its ID, and the engine-owned group link is
   unchanged by an archive or a delete of either role.
4. An archived group cannot accept a new member; a member create requires an
   existing active group of the same type.
5. A list shows the surviving group records and their members in group order,
   then member order; archived records are not listed.

### 7. HTML mockup

1. The type has no `status` verb and no front-matter block. Its metadata lives in
   the HTML metadata comment.
2. The fixed metadata keys are `status`, `task`, `specs`, `refs`, `references`,
   and `superseded_by`, and all of them are read from the HTML metadata comment.
   `specs` and `refs` are the two list-valued keys the edit surface writes;
   `status` and `superseded_by` are engine-written, and `task` and `references`
   are read-only.
3. The states are fixed to `pending`, `approved`, and `archived`; configuration
   cannot add, remove, or rename one, and a status list or default status on this
   model is rejected.
4. `archive` is the only path to `archived`; `supersede` also sets `archived` and
   records the successor in `superseded_by`. Supersession has no form control
   and no field.
5. `archive` moves the file to the `archived/` subdirectory, keeps the ID, and
   hides the record from lists, search, and the browser.
6. A superseded mockup refuses status transitions.
7. The edit surface offers the title and the `specs` and `refs` controls only;
   `status`, `task`, `references`, and `superseded_by` are read-only there.
8. No restore operation exists for a mockup.

### 8. Front-matter key contract

1. The general keys keep their canonical order.
2. For a record whose model stores status in front matter, the key order is: the
   general keys, then `status`, then the declared-field keys in field sequence
   order.
3. For a task record, the existing key set is unchanged and `assignee` remains a
   YAML list.
4. For a milestone, the declared key `release` follows `status`. For a release,
   the declared keys follow `status`.
5. For a decision, the only type-specific key is `superseded_by`; status and type
   stay in the document core.
6. For an HTML mockup, there are no front-matter keys.
7. A key that no field declares and that already exists on disk is preserved in
   its stored position and surfaced read-only.

### 9. Surface consumed downstream

1. Descriptor resolution supplies the archived directory, the read directories,
   the front-matter key order, and the declared status vocabulary.
2. Record-store operations supply archive, delete, and content-level archive for
   a caller that patches its own content before moving it, plus atomic write and
   the version token.
3. Duplicate detection spans every passed directory; callers pass the active and
   archived locations only.
4. Allocation helpers scan active and archived locations only.
5. Adoption seeds `status` for a model that stores it, and never normalizes a
   non-conforming value.

### 10. Validation and error behavior

1. A record in the archived directory whose status is `active` or a custom value
   is a validation outcome (exit code 4) naming the value and the location, with
   no write and no relocation.
2. A record in the active directory whose status is `archived` is relocated on
   read; a relocation failure yields the read-only plus validation outcome of
   section 4.
3. A non-conforming status is preserved and flagged; the status verb refuses it.
4. The `status` verb rejects `archived` and names `archive` as the only archiving
   path, and it rejects an archived record.
5. The `status` verb performs no move and creates no directory.
6. Duplicate copies of one ID across active and archived locations are a
   validation outcome naming both paths.
7. Archive is idempotent and reports success without a write for an already
   archived record.
8. A milestone archive of an already archived milestone reports success without
   a write; any other status value is refused with no write and no relocation.

## Inputs and outputs

- Input: a project context, a record descriptor, and a record ID or path.
- Output: the resolved or written path on archive; success on delete; the record
  envelope and version token on read.

## Assumptions

- Assumption: `store::archive` creates the destination directory when absent.
- Assumption: archived records other than milestones are not resolvable for
  view, list, or search; archived directories are read only for adoption,
  duplicate detection, conflict resolution, and milestone view by ID.
- Assumption: allocation remains `max(observed) + 1` with no high-water mark.
- Assumption: a legacy inactive folder stays on disk untouched and inert.

## Open questions

- Whether an inventory of records left unreadable by a removed custom status
  should be provided.
- Whether the archive directory is version-controlled or ignored by the version
  control system.
