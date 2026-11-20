---
id: REQ-05.008
title: "Manage UI Mockups"
created_date: '2026-11-13T20:21:14Z'
updated_date: '2026-11-20T20:45:01Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - ADR-033
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.007
  - REQ-05.010
---
# REQ-05.008 Manage UI Mockups

## User Story
As a project contributor, I want to maintain UI mockups associated with UI specifications and tickets so that UI design artifacts can be found, reviewed, approved, updated, and retired.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`ui-mockup` is a document type of the `SingleHtmlMockup` model and the only seed type of that model (RE-010 D6, REQ-07.004). It has one record per plain ID `MOCK-<n>`, stored by default under `docs/specifications/ui/mockups/`, and its record body is HTML. The type declares no field sequence (`fields: []`): its content is presented through a fixed metadata block, not through fields (ADR-036 decisions 7 and 29).

**Verbs.** The command family is `papyrus ui-mockup create|list|view|edit|archive|delete|supersede`. The verb set is derived from the model: the baseline verbs plus `supersede`; there is no `status` verb (ADR-036 decisions 8 and 10, REQ-07.003).

**Fixed metadata.** A mockup's metadata lives in its HTML metadata comment, and the HTML file carries no YAML front matter (REQ-05.010). The metadata keys are fixed:

- `status` — one of `pending`, `approved`, or `archived`. The states are fixed: configuration cannot add, remove, or rename them, and a `statuses` or `default_status` declaration on this type is rejected (REQ-07.002).
- `task` — the associated task, recorded as its Papyrus ID.
- `specs` — the list of UI specification spec IDs the mockup realizes.
- `references` — the list of references to other documents.
- `superseded_by` — the back-reference to the mockup that superseded this one.

Because the model has no field sequence, no key of this metadata block is a field, a CLI field option, or a configuration key: the metadata is engine-owned record data read from and written to the HTML metadata comment. The HTML file remains Papyrus-owned: Papyrus creates it and maintains the metadata comment on create, edit, archive, and supersede, and Papyrus does not version a mockup — version control is provided by git.

**Title.** The uniform title contract applies to the title in the HTML metadata (RE-010 D13, ADR-036 decision 10): the title is always present, single-line and non-empty, is required on create, and can never be cleared by an edit. `papyrus ui-mockup create <TITLE> --task <TASK> --spec <SPEC-ID>` creates a mockup; `--title` is accepted on create and edit, and supplying both the positional `TITLE` and `--title` is rejected and writes nothing.

**Create, view, and edit.** Creating a mockup requires the title, the associated task, and at least one UI specification reference; `--spec` and `--ref` are repeatable. A mockup can be listed, viewed, and edited. The view uses the uniform document head and chrome with the fixed metadata block in place of field sections, and the edit form offers the title and the two list-valued metadata controls `specs` and `references` only: `status`, `task`, and `superseded_by` are read-only there, and there is no field sequence and no `status` control (ADR-039 decision 21 and its clarification).

**State transitions.** The states follow the linear order `pending` -> `approved` -> `archived`. A new mockup starts `pending`; `archive` and `supersede` are the only commands that set `archived`, and `archive` is the only path to `archived` (REQ-07.001). A transition that goes backwards or skips a state is rejected, and status transitions are blocked for a mockup that has been superseded. An archived mockup has no restore command (REQ-05.007).

**Supersession.** A mockup may supersede an older mockup. Supersession is performed by command only — `papyrus ui-mockup supersede <old> <new>` — and never through a form control or a field. The superseded mockup becomes `archived` and records its successor in `superseded_by`; the successor is unchanged.

**Archive and delete.** Archiving sets the HTML metadata state to `archived`, moves the file into the `archived/` subfolder of the mockup directory, hides it from lists, search, and the web UI, and preserves its ID (REQ-05.007). `papyrus ui-mockup delete <id>` permanently removes a mockup, archiving an active mockup first, with confirmation and `--force` as for other document types (REQ-05.007). Reference visibility for this type is `visible`, so a list row renders its reference list (ADR-036 decision 29).

## Acceptance Criteria
- `ui-mockup` is a `SingleHtmlMockup` document type and the only seed type of that model; its model comes from the seed table and is never declared in configuration (REQ-07.004).
- A mockup has one record per plain ID `MOCK-<n>`, and the record body is HTML.
- The type declares no field sequence (`fields: []`), and its view and edit surface uses the fixed metadata block instead of field sections.
- The verb family is `create`, `list`, `view`, `edit`, `archive`, `delete`, and `supersede`; no `status` verb exists for this type.
- A `statuses` or `default_status` declaration on the `ui-mockup` entry is rejected (REQ-07.002).
- The metadata keys are exactly `status`, `task`, `specs`, `references`, and `superseded_by`, and they are read from and written to the HTML metadata comment.
- The metadata states are exactly `pending`, `approved`, and `archived`; configuration cannot add, remove, or rename a state.
- A mockup HTML file carries no YAML front matter, and the metadata comment is the single source of its metadata (REQ-05.010).
- The title is the title in the HTML metadata: present, single-line, non-empty, required on create, and never cleared by an edit.
- `papyrus ui-mockup create <TITLE> --task <TASK> --spec <SPEC-ID>` creates a mockup, `--spec` and `--ref` are repeatable, and a create that omits the title, the task, or every specification reference is rejected with one validation outcome and writes no file.
- Supplying both the positional `TITLE` and `--title` on one invocation is rejected and writes nothing.
- A new mockup starts `pending`; a transition to `approved` and then to `archived` is accepted, while a transition that goes backwards or skips a state is rejected with no write.
- `archive` and `supersede` are the only commands that set `archived`, and `archive` is the only path to `archived`.
- Status transitions are blocked for a mockup that has been superseded.
- `papyrus ui-mockup supersede <old> <new>` sets the superseded mockup to `archived` and records the successor in `superseded_by`; supersession has no form control and no field.
- The edit form offers the title and the `specs` and `references` list controls only; `status`, `task`, and `superseded_by` are read-only there, and no field sequence or `status` control exists.
- A mockup can be listed, viewed, and edited, and its metadata records the associated task and its UI specification references in the HTML metadata comment.
- A list row for a `ui-mockup` record renders its reference list because the seed reference visibility of the type is `visible`.
- Archiving a mockup sets its HTML metadata state to `archived`, moves it to the `archived/` subfolder, hides it from lists, search, and the web UI, and preserves its ID.
- No restore command exists for mockups (REQ-05.007).
- `papyrus ui-mockup delete <id>` permanently removes a mockup, archiving an active mockup first, with confirmation and `--force` as for other document types.
- Mockups are not versioned by Papyrus; version control is provided by git.

## Open Questions
- Whether the CLI `edit` verb keeps a status option for recording `approved`. This requirement keeps the linear `pending` -> `approved` -> `archived` order and fixes `archive` as the only path to `archived`, while ADR-039 decision 21 removes the status control from the browser form; no accepted source states whether the approval transition is offered on the CLI edit verb or only by editing the HTML metadata.

## Traceability
- `docs/specifications/ui/mockups/README.md` — mockup registry; status values pending/approved
- `docs/specifications/ui/browser-requirements-ui.md` — MOCK-0001 package
- User decision 2026-11-08: UI mockups are excluded from YAML front matter; the HTML-embedded metadata remains their single source.
- User decision 2026-11-08: supersession is performed by command only (`papyrus mockup supersede <old> <new>`); status transitions follow the linear table pending -> approved -> deprecated (state name superseded by the 2026-11-13 decision below).
- Project-owner decision 2026-11-13: mockups keep HTML metadata but adopt the `archived` state in place of `deprecated`; archiving moves the file to `archived/`; no restore command exists.
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D6 (only `ui-mockup` is `SingleHtmlMockup`, and the HTML model is seed-only), D8 (reference visibility is seed data), D10 (presentation facts in the seed), D13 (the title contract applies to the HTML metadata).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (`SingleHtmlMockup` row: one record per plain ID, baseline verbs plus `supersede`, no field sequence, fixed metadata states, HTML body, title contract in the HTML metadata), decision 8 (model-derived verbs, no `status` verb), decision 10 (fixed states, `archive` the only path to `archived`, `supersede` records the superseding ID), decision 29 (seed table: `ui-mockup` has no fields and fixed states).
- ADR-039 — One descriptor-driven document view and edit surface: decision 7 (the mockup view uses the same head and chrome with the fixed metadata block), decision 15 (no field sequence, fixed metadata), decision 21 and its clarification (fixed metadata block; the form offers the title and the `specs` and `references` controls; `status`, `task`, `references`, and `superseded_by` read-only; supersede is not a form field).
- ADR-033 — Uniform single-document browser view layout for managed Markdown records, as amended by ADR-039 decision 7 (the mockup view keeps the shared head and chrome).
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter (front-matter scope excludes mockups)
- REQ-07.001 — Organize Artifacts into Three Categories (fixed mockup states, `archive` as the only path to `archived`)
- REQ-07.002 — Configure Document Types and Slugs (rejection of `statuses` and `default_status` on this model)
- REQ-07.003 — Dispatch Config-Defined Document Commands (model-derived `supersede`, title contract)
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (the `SingleHtmlMockup` model)
- README.md, `papyrus(1)` → COMMANDS → ui-spec and Documents lifecycle (lines 538–583, 750–757) — pattern reference
