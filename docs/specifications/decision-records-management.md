# Feature Specification — Decision Records Management from the CLI

**Ticket:** TASK-13
**Milestone:** m-3 (Decision Records)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `decision` commands so a project contributor can create, list, view, edit, supersede, and archive decision records, with stable IDs, documented Nygard fields, fixed status vocabulary and transitions, and text-file persistence inside the project directory. This ticket implements REQ-04, REQ-04.001, and REQ-04.002, together with the documented command contract in BR-008. It provides the decision records that later tickets search (TASK-14) and expose in the browser (TASK-15).

## Scope

### In scope

- `papyrus decision create <title> [-t, --decision-type <type>] [-d <decision>] [--context <text>] [--consequences <text>]` (BR-008, REQ-04.001).
- `papyrus decision list [-t, --decision-type <type>]` (BR-008, REQ-04.001).
- `papyrus decision view <id>` (BR-008, REQ-04.001).
- `papyrus decision edit <id> [--title <title>] [-d <decision>] [--status <status>] [--context <text>] [--consequences <text>] [--superseded-by <id>]` (BR-008, REQ-04.002).
- `papyrus decision archive <id>` (BR-008, REQ-04.002).
- Decision identity: stable IDs of the form `<prefix>-<number>` (for example `ADR-001`, `BDR-001`), using the configured `decision_types` prefix for the chosen type and `id_padding` (REQ-04.001, BR-008, BR-016).
- Built-in types `adr` (default) and `bdr`; additional types may be configured in `decision_types` (REQ-04.001, BR-008, BR-016).
- Decision fields: title, status, decision, context, consequences (Nygard template) (REQ-04.001, BR-008).
- Status vocabulary: proposed, accepted, rejected, deprecated, superseded, with documented transitions (REQ-04.001, BR-008, README STATUSES).
- Type filtering on list (BR-008).
- Supersession by at most one decision of the same type; `--superseded-by` cannot be combined with `--status` (REQ-04.002, BR-008).
- Archival of decisions (REQ-04.002, BR-008).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search and browser access for decisions — TASK-14 and TASK-15.
- Packaging and verification of the decision-record milestone — TASK-16.
- Managing references from a decision to business or non-functional requirements via CLI flags — BR-008 defines no such flags; requirement references, when present, appear as literal text in the decision fields (consistent with TASK-3 and TASK-9).
- Restoring an archived decision — BR-008 defines no `restore` command.
- Deleting a decision — BR-008 defines no `delete` command.
- Editing or removing supersession once recorded beyond overwriting with a new `--superseded-by` value — BR-008 documents only the set operation.
- UI mockups or web UI — none for this ticket; CLI output only.
- New configuration keys beyond those already delivered by TASK-2 (`decision_directory`, `decision_types`).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `decision_directory` (default `decisions/`) and `decision_types` (default `adr` → `ADR`, `bdr` → `BDR`, both enabled), plus `id_padding`.
4. Disabled decision types are rejected by `create` and hidden from `list` and search (BR-016).

### 2. Decision type validation

1. `-t, --decision-type <type>` accepts a configured decision type key (for example `adr`, `bdr`).
2. An unknown or disabled type fails with exit code 4 (validation).
3. When `-t` is omitted, the default type `adr` is used. If `adr` is disabled or absent from configuration, creation fails with exit code 4.

### 3. Create

1. `papyrus decision create <title> [-t <type>] [-d <decision>] [--context <text>] [--consequences <text>]` creates a decision.
2. The positional `<title>` is required and is sanitized before use.
3. The decision type defaults to `adr` when `-t` is omitted.
4. The initial status is `proposed`.
5. The decision ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `ADR-001`), where `<number>` is one greater than the highest existing number found in both the active and `archived/` folders for that type's prefix. IDs are never reused.
6. The `-d/--decision`, `--context`, and `--consequences` options are optional; omitted fields are not written.
7. `create` prints the generated decision ID (and, in `--json` mode, returns it as JSON).
8. If any user-controlled input cannot be sanitized, creation fails with exit code 4.

### 4. List

1. `papyrus decision list` lists active decisions (not archived) of all enabled types.
2. `papyrus decision list -t <type>` filters the listing to that type.
3. List output includes at least the ID and title of each decision in a stable order (by ID ascending).
4. Archived decisions and decisions of disabled types are hidden from `list`.
5. `--json` wraps the output as a JSON array.

### 5. View

1. `papyrus decision view <id>` prints the full persisted text of the decision.
2. Viewing an archived decision by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` wraps the output as a JSON object containing at least ID, path, and content.

### 6. Edit

1. `papyrus decision edit <id> [--title <title>] [-d <decision>] [--status <status>] [--context <text>] [--consequences <text>] [--superseded-by <id>]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values.
3. `--title` updates the title (and the top-level heading).
4. `-d/--decision` updates the decision text.
5. `--context` updates the context text.
6. `--consequences` updates the consequences text.
7. `--status <status>` sets the status. The value must be one of `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`. The transition must be valid per the documented graph:
   - `proposed` → `accepted` or `rejected`;
   - `accepted` → `deprecated` or `superseded`;
   - setting the same status as the current one is allowed (no-op success);
   - any other transition is a validation error (exit code 4).
8. `--superseded-by <id>` marks this decision as superseded by the specified decision. It cannot be combined with `--status` (exit code 2 when both are supplied). It sets the status to `Superseded by <id>` and records the superseding decision ID in the status line (Nygard convention). Validation:
   - the superseding decision must exist (active or archived); unknown ID exits 3;
   - it must be of the same decision type as this decision; type mismatch exits 4;
   - it must not be the same decision as this one; self-reference exits 4.
9. Editing an archived decision is allowed and keeps it archived.
10. Editing an unknown ID fails with exit code 3.
11. If any supplied value cannot be sanitized, edit fails with exit code 4.

### 7. Archive

1. `papyrus decision archive <id>` moves the decision file from the active folder to the `archived/` folder.
2. Archiving an already-archived decision fails with exit code 4.
3. Archiving an unknown ID fails with exit code 3.
4. Archived decisions are hidden from `list` but remain viewable and editable by ID.

### 8. Persistence and sanitization

1. Decisions are stored as UTF-8 Markdown text files named `<ID>.md` in the configured `decision_directory` (active) or its `archived/` subfolder (archived). The `archived/` subfolder is created on first archive if absent.
2. File content follows the Nygard template: a top-level heading `# <ID> <title>`, then `## Status`, `## Decision`, `## Context`, and `## Consequences` sections. Empty optional fields are omitted. When a decision has been superseded, the `## Status` section reads `Superseded by <id>`.
3. All user-controlled input (title, decision, context, consequences, type, ID) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 9. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing or invalid usage (including `--superseded-by` combined with `--status`) exits 2 when it is a usage error, or 4 when it is a validation error, following clap's classification for mutually exclusive flags.
3. Unknown decision ID exits 3.
4. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `decision create`: inputs title (positional), optional type, optional decision/context/consequences; output is the generated decision ID (text) or JSON.
- `decision list [-t <type>]`: output is IDs and titles of active decisions (text table or JSON array).
- `decision view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `decision edit <id>`: inputs optional fields and flags; output minimal success text or JSON.
- `decision archive <id>`: moves file; minimal success output.

## Validation and error behavior

- Missing required arguments (for example the positional title) → exit code 2 (usage).
- Unknown decision ID or superseding decision ID → exit code 3 (not found).
- Validation/conflict (unknown or disabled type, invalid status value or transition, type mismatch on supersession, self-reference, already-archived decision, unnormalizable input, mutually exclusive flags handled as usage) → exit code 4, except mutually exclusive flags which are exit code 2 per clap.
- IO failure or boundary violation → exit code 1.
- `--json` error output includes the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: single-crate layout is retained; a `decision` module may be added within the crate without a new crate or ADR.
- Assumption: IDs are generated by scanning active and `archived/` folders for the highest existing number for that type's prefix; `id_padding` (default 3) applies.
- Assumption: `init` already creates the decision directory; this ticket creates the `archived/` subfolder on first archive if absent.
- Assumption: archive moves the decision file into `<decision_directory>/archived/` rather than recording status in place, consistent with the milestone pattern (TASK-8) and keeping archived decisions out of `list`.
- Assumption: status transition enforcement follows the documented graph; setting the current status again is a no-op success.
- Assumption: `--superseded-by` sets the status to `Superseded by <id>` (Nygard convention) and does not require a prior `accepted` status; the superseding decision must exist and share the same type.
- Assumption: requirement references are not CLI-managed for decisions because BR-008 defines no flags; any requirement IDs appearing in the fields are literal text.
- Assumption: list output order is by ID ascending.

## Open questions

- OQ-1: Whether `--status superseded` (the bare word) should be permitted in addition to `--superseded-by` is unspecified; this specification permits it as a valid status value without recording a superseding ID.
- OQ-2: Whether an archived decision's status can be changed via `--status`/`--superseded-by` is unspecified; this specification allows edits to archived decisions, including status changes, consistent with the milestone pattern.
- OQ-3: The exact Markdown section order and omission of empty fields follow the Nygard template as listed in BR-008; tests verify field presence rather than byte-for-byte formatting.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-04, REQ-04.001, REQ-04.002 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-008 |
| ADR | ADR-0001 (Rust), ADR-0002 (amd64 .deb) |
| Ticket | TASK-13 |
| UI mockups | none |
| Related specification | docs/specifications/project-initialization-and-configuration.md (TASK-2) |
path: docs/specifications/decision-records-management.md