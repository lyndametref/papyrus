---
id: ADR-0024
title: "List-Valued Configuration Keys in `papyrus config set`"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-20T20:09:05Z'
labels: []
references: []
---
# List-Valued Configuration Keys in `papyrus config set`

## Status
accepted
## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-01.002 defines `papyrus config set <key> <value>` as persisting "the value as a YAML scalar". REQ-07.001/REQ-07.002 add list-valued keys — `document_types.<slug>.statuses` and the top-level `statuses` for tasks (REQ-01.002) — and REQ-01.002 states that `document_types.<slug>.statuses` and `document_types.<slug>.default_status` are ordinary configuration keys that `config get` reads and `config set` writes. Today the two statements cannot both hold: the current `config set` rejects a `yaml_serde::Value::Sequence` outright (`src/commands.rs`), so a status list can only be introduced by editing `papyrus.config.yml` by hand, which contradicts the configuration-edit surface the requirements promise.

ADR-0019 makes configuration fail-closed with two repair surfaces, one of which is `config get|set|unset|list`. A user who needs to fix a status list in a broken configuration must be able to do it through that surface, or the repair path is incomplete for exactly the keys that are easiest to get wrong.

Forces: keep `config set`'s single value argument and its scalar semantics for every non-list key; do not turn `config set` into a YAML parser; keep validation on the resulting configuration (ADR-0019); keep the file format stable and readable; introduce no new flag, key type, or dependency.

## Decision
1. **List-valued keys are declared in one constant.** `config::LIST_VALUED_KEYS` names the keys whose value is a YAML sequence: the top-level `statuses` (task statuses) and `document_types.<slug>.statuses` for every slug. Every other key — including `default_status`, `prefix`, `directory`, `enabled`, `id_padding`, and `browser_*` — keeps scalar semantics. The constant is the only place a key is classified; key parsing must not guess from the value's shape.

2. **`config set` with a list-valued key takes a comma-separated list.**
   - The value is split on `,`; each element has surrounding whitespace trimmed; empty elements are dropped only when the entire value is empty, which is rejected.
   - An empty value (`config set statuses ""`, or a value whose split yields no non-empty element) is a validation error (exit 4) naming the key, because "no statuses" is not a valid configuration for any of these keys (ADR-0019 decision 7).
   - The list is written to `papyrus.config.yml` as a YAML **flow sequence**, for example `statuses: [ToDo, Initiated, Done]`, and `config get`/`config list` print the same flow form.
   - Example: `papyrus config set document_types.audit.statuses 'active,archived,open'`.

3. **Commas are separators, never data, and there is no escaping.** The command does not interpret quotes, backslashes, or any escape sequence. A value that must itself contain a comma cannot be expressed through `config set`; the user edits `papyrus.config.yml` directly. For a document status list this is safe rather than lossy: a comma- or quote-containing element fails the status character rule (ADR-0019 decision 7), so a quoted attempt is rejected with a validation outcome instead of being silently split into wrong values. For the free-form task status list, a status containing a comma is simply not representable through `config set`, and that limitation is documented.

4. **Scalar keys are unchanged.** For a non-list key a comma stays an ordinary character of the scalar value; no existing `config set` invocation changes meaning.

5. **The resulting configuration is validated before it is persisted.** `config set` builds the effective configuration with the new list, runs the full ADR-0019 validator, and writes only if the result is valid; an invalid result is refused (exit 4) and nothing is written. When the existing configuration is already invalid, the repair-mode load of ADR-0019 decision 10 applies: the file is read tolerantly, only the result is validated, and the same sanitization and containment rules are enforced.

6. **`config unset` is unchanged.** Removing a list key reverts it to its default list; there is no separate "clear list" operation.

7. **The syntax is documented where the key is documented.** `papyrus-config(5)` and `config set`'s help state the comma-separated form, the flow-sequence storage, and the no-escape limitation. No new flag (`--append`, `--remove`, `--json`) is added.

### Amendment (2026-11-20) — descriptor model cross-check

- **Decision 1 — list-valued key set.** `LIST_VALUED_KEYS` keeps exactly the scalar status lists it names: the top-level task `statuses` and `document_types.<slug>.statuses`. The descriptor's `fields`, `group.fields`, and `member.fields` collections are sequences of mappings, not list-valued keys: a comma-separated scalar list cannot express a mapping, so they are never classified as list-valued and `config set` never writes them. Presentation collections (`slots`, `api_aliases`) are not classified here and remain declarative file data; this note does not add them to the comma-separated surface.
- **Consequence for fields.** Adding, removing, or reordering a field, and every per-field property edit, is a configuration file edit (RE-010 D14; ADR-0019 decision 10 as amended). `config set` keeps its scalar semantics (decision 4) and its validation-before-write rule (decision 5).
- **Not changed:** decisions 2-7, the comma syntax, the flow-sequence storage for the status lists, the no-escape rule, and this record's accepted status.
## Consequences

**Positive:**

- Status lists become editable through the documented configuration surface, which completes the ADR-0019 repair path for the keys most likely to be invalid.
- The file stays a plain YAML mapping with readable flow sequences, and `config get` round-trips what `config set` wrote.
- One key-classification constant prevents a value-shape guess from turning a scalar containing commas into a list.

**Negative:**

- A comma can never be part of a configured status through `config set`; a project that needs one must edit the file, and the limitation must be documented rather than discovered.
- An empty list cannot be expressed through `config set`; only `config unset` (revert to defaults) or a file edit expresses it.
- Trimming means a status with significant leading or trailing whitespace is unrepresentable through this surface.
- REQ-01.002 still says "persists the value as a YAML scalar"; the requirement must be updated by `requirements-engineer` to record the owner's list exception (traceability follow-up, not a code question).

**Risks and mitigations:**

- *Silent mis-split*: for document statuses the charset validation rejects quote- or comma-bearing elements; for task statuses the split is the documented behaviour, and the command prints the resulting list under `config get`.
- *List/scalar classification drift*: `LIST_VALUED_KEYS` is the only classifier and a test asserts that every key with a sequence type in `Config` appears in it.
- *Repair-mode bypass*: the list value goes through the same validator as a file-sourced list; a test asserts an invalid list is refused and the file is byte-identical.
- *Documentation drift*: the man page and help text are updated with the syntax, and a test asserts the documented example parses.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Comma-separated value written as a YAML flow sequence (this ADR) | Selected | Owner's decision; keeps one value argument, one key classifier, and the existing validator. |
| Accept arbitrary YAML in the value and store it as parsed | Rejected | Turns `config set` into a second YAML entry point, lets a user write mappings where a scalar is expected, and changes error behaviour for every key. |
| Add repeated `--append`/`--remove` options | Rejected | New surface and new semantics (ordering, duplicates) not requested; the owner chose the comma form. |
| Require file editing for every list key | Rejected by the owner | Leaves the repair path and the documented `config set` promise incomplete. |
| Keep `config set` scalar-only and exempt list keys | Rejected | Contradicts REQ-01.002's statement that these keys are written by `config set`. |

## Open Questions

- Should a status value containing a comma be supported later through a documented escape, or through `--append`? Current decision: neither; file edit only.
- Should `config get` gain a `--json` rendering for list values? Current decision: no; the flow-sequence text is the documented form.

## Downstream constraints

1. `LIST_VALUED_KEYS` is the only list-key classification; `config set` must not infer list-ness from the value.
2. List values are stored as YAML flow sequences and validated by the ADR-0019 validator before any write.
3. An empty list value is rejected; `config unset` is the only way to return to defaults.
4. No escaping and no quote interpretation may be added without a new ADR.
5. `papyrus-config(5)` and `config set --help` must state the comma syntax and the limitation.

## References

- `docs/requirements/business/REQ-01.002.md` — `config set`, list-valued keys, repair surfaces.
- `docs/requirements/business/REQ-07.002.md` — `statuses`/`default_status` validation.
- `docs/requirements/business/REQ-07.001.md` — custom document statuses.
- `docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md` — validator, repair-mode load, status-list rules.
- `docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md` — field map that consumes the configured status lists.
- Implementation: `src/commands.rs` (`config_set`, `config_get`, `config_list`), `src/config.rs` (`LIST_VALUED_KEYS`, `validate`).
