# Feature Specification — Expose Front-Matter Metadata Options on All Record Commands

**Ticket:** TASK-65
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 3: every managed record command (create and edit) exposes an option for every front-matter field except `id`, `created_date`, and `updated_date`, and every write goes through the shared patch-preserving path delivered by TASK-63 and TASK-64. This ticket implements the option rule in REQ-05.011, the field map in REQ-05.010, the view parity contract in REQ-06.004, and the Backlog.md interoperability requirement NFR-04.003. It does not migrate the corpus (TASK-68, TASK-69), remove legacy serializers (TASK-70), or change browser UI (TASK-66, TASK-67, TASK-72).

## Scope

### In scope

- Create and update options for all front-matter keys except `id`, `created_date`, and `updated_date` on every managed record command.
- Managed record types: task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record.
- General front-matter fields on every type: `title`, `labels`, `references`.
- Task-specific fields: `status`, `assignee`, `milestone`, `dependencies`, `ordinal`, `documentation`, `type`.
- Milestone-specific field: `release`.
- Decision-specific field: `superseded_by`.
- Audit-specific field: `type` (exposed as `--audit-type`).
- Automatic `created_date` and `updated_date` handling and no-op detection.
- Defaulting of `ordinal` from the current maximum.
- Patch-preserving writes for CLI commands and the browser save contract.
- Unit and integration tests for option coverage, timestamps, ordinal defaults, and field preservation.

### Out of scope

- Lazy migration of document or task/milestone records — TASK-68, TASK-69.
- Browser UI changes and metadata projection — TASK-66, TASK-67, TASK-72 (this ticket only fixes the save contract they consume).
- Removal of legacy per-module serializers and no-drift enforcement beyond the checks inherited from TASK-63 — TASK-70.
- UI mockups and other HTML files (no front matter) — REQ-05.010.
- Backlog.md `docs` and `drafts` records — out of scope (REQ-05.010).
- New record types, new configuration keys, or changes to record identity rules (TASK-64).

## Behavior

### 1. General option rule

1. `id`, `created_date`, and `updated_date` are exempt from the option rule. They are never accepted as command options and are never edited by a command option.
2. Every other front-matter key of a managed record type is exposed as a create option and as an update option on that type's commands.
3. A command that already exposes an option for a field keeps that option; a command that lacks one gains it.
4. A create option initializes the field. An update option changes the field only when it is supplied; omitted options leave the existing value unchanged.
5. Options are type-specific where the field is type-specific and general where the field is general. No command exposes an option for a field its record type does not carry.

### 2. Front-matter field matrix

General fields carried by every managed record type:

| Field | Kind | Create/update option |
|---|---|---|
| `title` | scalar string | existing title/term positional flag or `--title` (no new flag) |
| `labels` | list of strings | `--label <label>` (repeatable) |
| `references` | list of strings | `--reference <value>` (repeatable) |

Type-specific fields:

| Record type | Field | Kind | Create/update option |
|---|---|---|---|
| task | `status` | scalar string | `--status <status>` |
| task | `assignee` | list of strings | `--assignee <name>` (repeatable; replaces the list) |
| task | `milestone` | optional scalar ID | `--milestone <id>` plus existing `--no-milestone` |
| task | `dependencies` | list of task IDs | `--depends-on <id>` (repeatable) |
| task | `ordinal` | non-negative integer | `--ordinal <n>` |
| task | `documentation` | list of paths | `--documentation <path>` (repeatable) |
| task | `type` | scalar string | `--type <type>` |
| milestone | `release` | optional scalar ID | `--release <id>` plus existing `--no-release` |
| decision | `superseded_by` | optional scalar ID | `--superseded-by <id>` |
| audit | `type` | scalar string | `--audit-type <type>` (existing) |

All other managed record types carry only the general fields plus their body fields, which are not front-matter keys and keep their existing options.

### 3. Command surface changes

#### 3.1 Task (`papyrus task`)

Create gains `--label`, `--reference`, `--documentation`, `--type`, and `--ordinal` in addition to the existing `--ac`, `--depends-on`, `--milestone`, `--assignee`, and `--status`-equivalent default behavior. `--status` is not a create option; a created task uses the configured `default_status`.

Edit gains `--label`, `--reference`, `--documentation`, `--type`, and `--ordinal` in addition to the existing edit options. `--status` already exists.

`--assignee` accepts one or more values and replaces the whole assignee list. When omitted, the assignee list is unchanged.

`--label`, `--depends-on`, `--documentation`, and `--reference` are repeatable and append unique values to the existing list. They do not remove existing values. Removal of individual values is outside this ticket and remains available only where an existing command already provides it (`--no-milestone`, `--no-release`, `--no-requirement`).

#### 3.2 Milestone (`papyrus milestone`)

Create gains `--label` and `--reference` in addition to `--release`.
Edit gains `--label` and `--reference` in addition to `--title`, `--description`, `--release`, and `--no-release`.

#### 3.3 Release (`papyrus release`)

Create gains `--label` and `--reference`.
Edit gains `--label` and `--reference` in addition to `--title` and `--description`.

#### 3.4 Decision (`papyrus decision`)

Create gains `--label` and `--reference` in addition to the existing decision fields.
Edit gains `--label` and `--reference` in addition to `--title`, `-d`, `--status`, `--context`, `--consequences`, and `--superseded-by`.

`superseded_by` validation is unchanged: the superseding decision must exist and share the same decision type.

#### 3.5 Business and non-functional requirements (`papyrus req`, `papyrus nfreq`)

Create and edit gain `--label` and `--reference` in addition to the existing title, story, description, and group options. Requirement groups are not front-matter keys and keep their existing handling.

#### 3.6 Glossary (`papyrus glossary`)

Create and edit gain `--label` and `--reference` in addition to `--term` and `--definition`.

#### 3.7 UI specification (`papyrus ui-spec`)

Create and edit gain `--label` and `--reference` in addition to `--view` and `--description`.

#### 3.8 Technical specification (`papyrus tech-spec`)

Create and edit gain `--label` and `--reference` in addition to `--title` and `--description`.

#### 3.9 Business rule (`papyrus rule`)

Create and edit gain `--label` and `--reference` in addition to `--title`, `--description`, `--rationale`, `--applicability`, and `--requirement`/`--no-requirement`.

The existing `--requirement` option continues to manage requirement references stored in the rule body. The new `--reference` option manages the general front-matter `references` list. The two lists are independent.

#### 3.10 Audit record (`papyrus audit`)

Create and edit gain `--label` and `--reference` in addition to `--title`, `--content`, and `--audit-type`.

### 4. Timestamps

1. `created_date` is set once, at creation, to the current UTC time in quoted ISO-8601 form `'YYYY-MM-DDTHH:MM:SSZ'`. It is never changed by any later command, migration, or browser save.
2. `updated_date` is set at creation to the same value as `created_date`.
3. `updated_date` is refreshed to the current UTC time on every write that changes at least one byte of the record file.
4. A write that changes no byte does not refresh `updated_date`. This includes an edit where every supplied value equals the current value, an empty patch, and a no-op browser save.
5. Commands that only move or remove a file (archive, restore, deprecate, restore, delete) do not rewrite its content and therefore do not refresh `updated_date`. Superseding a decision rewrites the decision file and refreshes `updated_date`.
6. Browser saves follow the same rule: a save that changes a metadata-backed or body field refreshes `updated_date`; a save with no effective change does not.

### 5. Ordinal

1. `ordinal` is an opaque non-negative integer. Negative values and non-integer values are validation errors (exit code 4).
2. On create, when `--ordinal` is omitted, the default is `max(observed ordinals for that record type across active and archived or deprecated locations) + 1`. If no record of that type exists, the default is `0`.
3. The allocated ordinal is written once at creation and is never recomputed by a later read or write.
4. On edit, `--ordinal` replaces the stored value. Omitting it leaves the stored value unchanged.
5. Ordinal is only a front-matter field for tasks. Other record types do not gain an ordinal option.

### 6. Patch-preserving writes

1. Every managed record write performed by this ticket uses `record::patch` to build the change and `record::store::write_atomic` to persist it. No command calls `fs::write` on a record path.
2. A patch contains only the keys the operation changes. Keys not mentioned by the patch are left byte-for-byte intact, including unknown keys, their order, and their quoting.
3. Body lines, section order, marker comments, and line endings are preserved byte-for-byte unless a patch operation explicitly changes them.
4. An empty patch produces byte-identical text and results in no write and no `updated_date` refresh.
5. A patch whose values equal the current values produces byte-identical text and results in no write and no `updated_date` refresh.
6. Create renders the initial document through the shared envelope and store rather than through ad-hoc string formatting or `fs::write`.
7. Browser saves use the same patch and store path and therefore inherit the same preservation guarantees.

### 7. List-field update semantics

1. Scalar fields (`title`, `status`, `assignee` when supplied, `milestone`, `release`, `superseded_by`, `type`, `ordinal`) are replaced by the supplied value.
2. Repeatable list fields (`labels`, `dependencies`, `documentation`, `references`) append unique sanitized values when the option is supplied. Omitting the option leaves the list unchanged.
3. `--milestone` and `--no-milestone`, and `--release` and `--no-release`, remain mutually exclusive as documented by the existing command specifications.
4. Reference existence is not validated for the general `references` list or for `documentation` paths, consistent with the existing business-rule reference handling. Existence continues to be validated for `milestone`, `dependencies`, `release`, and `superseded_by`.

### 8. Browser save contract

1. The browser keeps its current editable field set. `created_date` and `updated_date` remain read-only.
2. A browser save of a metadata-backed field writes the new value into the front matter.
3. A browser save preserves the other front-matter fields, key order, unknown keys, and the rest of the file.
4. A browser save does not copy metadata back into the document core.
5. A browser save that changes no effective value does not refresh `updated_date`.

### 9. Identity and reference handling

1. Commands continue to accept and display Papyrus display IDs only. Storage IDs are translated at the boundary by `record::identity` (TASK-64).
2. `milestone`, `dependencies`, `release`, and `superseded_by` values are translated to storage form on write and to display form on read, as defined by TASK-64.
3. The general `references` list is stored verbatim after sanitization and is not translated by this ticket.

### 10. Validation and error behavior

1. Missing or malformed ID input → exit code 3 or 4 as defined by TASK-64 and the existing command specifications.
2. Unknown record for `milestone`, `dependencies`, `release`, or `superseded_by` → exit code 3.
3. Invalid `ordinal` (negative or non-integer) → exit code 4.
4. Invalid or empty sanitized field value → exit code 4.
5. Mutually exclusive flags supplied together → exit code 4.
6. Storage ID supplied as input → exit code 4 with a hint naming the equivalent display ID (TASK-64).
7. A no-op write leaves the original file byte-identical, removes no temporary file beyond the store's own cleanup, and exits successfully.
8. Atomic write failure leaves the original file byte-identical and removes temporary files (inherited from TASK-63).

### 11. Tests

1. Option coverage: for each managed record type, tests verify that create and edit expose an option for every front-matter key except `id`, `created_date`, and `updated_date`. Tests may assert through CLI help output and through create/edit invocations followed by front-matter inspection.
2. Timestamps: tests verify `created_date` is written at creation, never changes on edit, and `updated_date` changes on a content-changing write but not on a no-op write.
3. Ordinal: tests verify a supplied ordinal is stored, and an omitted ordinal on create is derived from the current maximum plus one (or `0` when no records exist).
4. Field preservation: tests verify that writing with a subset of options preserves all other front-matter keys (including unknown keys) and body content byte-for-byte, and that a no-op write is byte-identical.
5. Browser: tests verify a metadata-backed save writes the new value, preserves the remaining fields and body, and does not refresh `updated_date` on a no-op save.
6. Interoperability: task and milestone files created or edited with the new options remain readable, listable, and updatable by Backlog.md, including additive keys (NFR-04.003).

## Inputs and outputs

- Input: a project `Context`, a record command, and the supplied option values.
- Output: a created or updated managed record file whose front matter contains every supplied field, with `created_date` and `updated_date` maintained automatically and all unmodified content preserved byte-for-byte.
- No new user-facing command output is required beyond the existing success output; option values are reflected in the persisted front matter and in CLI/browser views.

## Validation and error behavior

- Unknown or invalid record IDs → exit code 3 or 4 per TASK-64 and existing command specifications.
- Invalid `ordinal`, empty sanitized values, or invalid status/type values → exit code 4.
- Mutually exclusive flags → exit code 4.
- No-op write → exit code 0, original file byte-identical.
- Atomic write failure → exit code 1, original file unchanged.

## Assumptions

- Assumption: option flag names follow the existing Papyrus conventions (`--label`, `--status`, `--milestone`, `--depends-on`, `--assignee`, `--release`, `--audit-type`, `--superseded-by`) and add `--reference`, `--documentation`, `--ordinal`, and `--type` only where missing.
- Assumption: repeatable list fields append unique values on edit, matching the existing task behavior for labels and dependencies. Removal of individual list values is not added by this ticket except where an existing command already provides it.
- Assumption: `--assignee` accepts one or more values and replaces the entire assignee list; omitting it leaves the list unchanged.
- Assumption: ordinal defaults use `max(observed) + 1` with `0` when no records exist, scanning active and archived or deprecated locations for the same record type.
- Assumption: the general `references` list is stored verbatim after sanitization and is not translated by this ticket.
- Assumption: this ticket consumes the read fallback and patch API from TASK-63 and the identity translation from TASK-64 but does not implement lazy migration.
- Assumption: no UI design or mockup is required because the browser keeps its current editable field set and only the save contract changes.

## Open questions

- OQ-1: Whether list-field options should gain replace/remove variants (as Backlog.md supports) in addition to append. This ticket specifies append-only for newly exposed list fields; if stakeholders want full CRUD parity, a follow-up ticket is required.
- OQ-2: Whether `documentation` paths and general `references` values should be validated for existence. This ticket specifies no existence validation, consistent with the existing business-rule reference handling.
- OQ-3: Whether archive, restore, deprecate, and restore operations should rewrite front matter to refresh `updated_date`. This ticket specifies that they do not rewrite content; if a refresh is required, the scope of "write path" in REQ-05.011 must be clarified.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-65 |
| Requirements | REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.003 |
| ADRs | ADR-0007, ADR-0008, ADR-0009 |
| Related tickets | TASK-63 (Done), TASK-64 (Done), TASK-66, TASK-67, TASK-68, TASK-69, TASK-70, TASK-72 |
| Source | src/task.rs, src/milestone.rs, src/release.rs, src/decision.rs, src/requirement.rs, src/glossary.rs, src/ui_spec.rs, src/tech_spec.rs, src/rule.rs, src/audit.rs, src/record/* |
| Tests | tests/browser_cli.rs, tests/*_cli.rs |
