---
id: TECH-012
title: "Record Archive, Restore, Delete, and Lifecycle"
created_date: '2026-11-27T20:35:38Z'
updated_date: '2026-11-28T07:27:17Z'
labels: []
references:
  - REQ-002.001
  - REQ-002.005
  - REQ-002.006
  - REQ-004.002
  - REQ-005.007
  - REQ-005.008
status: verified
---
# TECH-012 Record Archive, Restore, Delete, and Lifecycle

## Description
Covers the archive, restore, and delete commands and the lifecycle effects they produce for every record category. Storage layout is a separate specification; status vocabularies and transitions are separate specifications.

## Archive command

- `papyrus <command> archive <ID>` moves the record's file into the `archive/` subfolder of the folder that holds it and preserves the record identifier (for milestones `<backlog_dir>/<milestone_dir>/archive/`; for releases `<backlog_dir>/<release_dir>/archive/`).
- The file move is the state change; archiving changes no status for any category.
- Archiving a task preserves its dependencies and references.
- An archived record is hidden from lists, search, and the browser, except that a search may explicitly include archived records; the browser shows the shared not-found page.
- Archiving is allowed for any task, any milestone or release, any `rejected`, `superseded`, or `deprecated` decision, any non-archived Markdown document, and any `draft` or `approved` formatted-record.

- Archiving a decision that is `proposed` or `accepted` is rejected with a validation outcome and writes nothing.
- No command returns an archived milestone, release, decision, or document to a non-archived state; there is no restore command for them.
- A document has no supersede verb; decision supersession is a status transition, not an archive operation.

## Restore command

- `papyrus task restore <ID>` returns an archived task to the working set with the status it had before archiving and the same identifier.
- `restore` exists for tasks only.

## Delete command

- `papyrus <command> delete <ID>` permanently removes a record; a record that is not archived is archived first.
- Deletion asks for confirmation, and `--force` skips the prompt.
- A deleted record leaves no file, and the deleted identifier remains reusable only under the identifier allocation rule.
- A decision is deletable only while its status is `proposed`; a decision that has left `proposed` cannot be deleted and is archived instead.


## grouped-record records

- Archiving or deleting a group record changes no member's file, status, or identifier.
- Each member is archived or deleted only by a command that names that member.

## Model independence

- The archive and delete rules apply to every document type and do not depend on a type's document model, field sequence, or role field sets.
