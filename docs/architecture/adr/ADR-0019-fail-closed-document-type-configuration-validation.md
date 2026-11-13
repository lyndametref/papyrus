# Fail-Closed Document-Type Configuration Validation

## Status

Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13.

## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-07.002 requires every `document_types` entry to be validated at configuration load: the slug must be usable as a command name (non-empty, ASCII letters, digits, `-`, `_`, not starting with `-`, no whitespace, control characters, or path separators); the slug must not equal a built-in command name (`init`, `config`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, `version`); the entry must have a non-empty `prefix` and a `directory` that is relative and inside the project directory. A violating configuration is rejected with a validation outcome naming the offending slug or key, and **Papyrus does not start with a partially valid document-type set**.

Current code shape:

- `Config::validate` (`src/config.rs`) checks `id_padding`, task/milestone/release prefixes, folder values, and each `document_types.<key>.directory` through `sanitize::sanitize_path_string`. It does **not** check the slug, the entry prefix, or collisions with built-in command names, and it is not the only entry point: `load_effective_config` (`src/commands.rs`) calls it, but `config get/set/unset` each merge and validate separately.
- `load_effective_value` deep-merges `Config::default()` with the file (`merge_yaml`). Because the default `document_types` map supplies all eleven default entries, **removing a default entry from the file is impossible**: the merge re-adds it. `enabled: false` is the only expressible form of "not this type", which contradicts REQ-07.002's "removing an entry stops managing those records". `deploy` is still seeded by `Config::default()` although REQ-07.002 removed it from the default set.
- Status vocabulary is not validated at all: `DocTypeEntry` has no `statuses`/`default_status`, so there is no check that a document type's list is non-empty, unique, or contains the fixed `active` and `archived` values, no check that `default_status` is a declared value and is not `archived`, and no check that `ui-mockup` declares no statuses (REQ-07.001, REQ-07.002).
- YAML duplicate keys are not detected: `document_types` with the same slug twice is currently resolved by the parser's last-wins behaviour, which REQ-07.002 forbids.
- Validation failures surface as `AppError::Validation`, exit code 4 (REQ-01.004), with a message that names the offending key for some keys and not for others.
- Sanitizers exist in `src/sanitize.rs` (`sanitize_path_string`, `sanitize_path_component`) and are applied in some paths, which risks silently normalizing a value instead of rejecting it.

Forces: NFR-02.002 requires every user-supplied value to be sanitized **and** rejected when it fails the criteria — no silent cleaning; NFR-04.002 requires every path to stay inside the canonical project directory with symlinks resolved, and rejects escapes; REQ-01.003 requires a missing or explicitly named-but-absent configuration to fail every command except `init`; REQ-01.002 requires `config set` to be the supported way to change configuration; and the command tree, the browser views, and the record layer must all consume the same validated set (ADR-0016, ADR-0017, ADR-0020).

A typo in a slug therefore has two conflicting demands: the tool must refuse to run (REQ-07.002), and the user must be able to repair it with `papyrus config set` (REQ-01.002). The owner decided on 2026-11-13 (D4) that both hold through two repair surfaces, and decision 10 records the resulting constraint.

## Decision

1. **One validation function, one failure outcome.** `Config::validate` remains the single validator; it is extended with a document-type section and reused by every loader. Failure is always `AppError::Validation` (exit 4), and the message names every offending key in the form `document_types.<slug>.<field>` (or `document_types.<slug>` for a slug-level violation), in `BTreeMap` order, so the outcome is deterministic and one startup attempt reports all violations rather than only the first.

2. **Slug grammar is reject-only, exactly as specified.**
   - non-empty;
   - every character is ASCII `A-Z`, `a-z`, `0-9`, `-`, or `_`;
   - the first character is not `-`;
   - no whitespace, control character, path separator (including `\`), or `..` segment.

   No trimming, lowercasing, or normalization is performed: `" Req"`, `"req "`, and `"REQ"` are rejected or accepted on their own merits, never repaired. This is NFR-02.002's "rejected rather than silently cleaned" applied to configuration.

3. **Reserved names come from one constant.** `cli::BUILTIN_COMMANDS` (ADR-0016) is the only list of built-in command names; validation compares each slug against it and rejects a match. The comparison is **case-insensitive** (`Req` is rejected as well as `req`), because a case-variant slug would make help text, documentation, and diagnostics ambiguous even though clap's matching is case-sensitive. This is deliberately stricter than REQ-07.002's literal wording and is flagged in the intake report for requirement confirmation.

4. **Duplicate keys are a parse error, not a merge.** The configuration file is read with a loader that detects a repeated mapping key, and a repeated `document_types.<slug>` (or any repeated key) is rejected with a validation outcome naming the key. Neither the first nor the last occurrence is silently kept, because either choice makes the effective type set depend on file order. The check happens on the raw file before defaults are merged, so a duplicate cannot be hidden by the merge.

5. **Prefix constraints.** `prefix` must be non-empty after no normalization at all; it must contain no whitespace, control character, path separator, or `..`; and it must be unique among all configured document types, **including disabled ones**. Prefix uniqueness is what makes the record claim rule (`<PREFIX>-<digits>`) and ID allocation unambiguous (ADR-0018 decision 7); uniqueness across disabled entries prevents a later re-enable from creating a duplicate. This uniqueness rule is also stricter than the requirement text and is listed for confirmation.

6. **Directory constraints.** `directory` must be non-empty, relative, free of traversal segments and backslashes, and its resolved path must pass `Context::ensure_within` against the canonical project directory (NFR-04.002) — including after symlink resolution. Duplicate directories among **enabled** entries are rejected, because two enabled types sharing a folder make listing, archive location, and ID resolution ambiguous. The check is performed at load and again on every resolution through `record::store`; load-time validation is not treated as a substitute for the runtime boundary check.

7. **Status lists are validated as a closed vocabulary.** For each entry:
   - `statuses`, when present, must be a non-empty list with no duplicates and must contain `active` and `archived` exactly once. Those two values are fixed: configuration cannot remove, rename, or replace them, and a custom status may not reuse either value.
   - Every declared status, including the two fixed ones, follows the slug character rule of decision 2 (non-empty, ASCII letters/digits/`-`/`_`, not starting with `-`, no whitespace, control character, or path separator). The same reject-only discipline applies: no trimming or case folding.
   - `default_status`, when present, must be one of the declared statuses and must not be `archived`, so a new record never starts archived.
   - A `statuses` declaration on `ui-mockup` is rejected, because that type keeps its state in HTML metadata and declares no custom statuses (REQ-05.008).
   - The main statuses are not configurable: `active` and `archived` are accepted only as the fixed members of the list, and there is no key that renames or redefines them.

8. **The default set is a file-level default, not a per-entry merge.** `document_types` is replaced wholesale when present in `papyrus.config.yml`:
   - key absent → the built-in default set applies (REQ-01.003: defaults apply to a key absent from a file that was found);
   - key present → exactly the listed entries exist, with no default entry re-added.

   This is a change to `load_effective_value`'s merge behaviour for this one key. It is what makes REQ-07.002's "removing an entry" expressible at all, and it keeps `disabled` as the non-destructive option.

9. **`config set` materializes before it edits.** Because of decision 8, a `config set document_types.<slug>.<field>` against a file with no `document_types` key would otherwise drop the default set. The write path therefore builds the full effective `document_types` map first, applies the single change, validates the **result**, and writes the whole map. The write is refused if the result is invalid, so the configuration file never contains a state that the loader rejects.

10. **Fail-closed startup, with two repair surfaces (owner decision of 2026-11-13).** Every command except `init` aborts before dispatch when validation fails (ADR-0016 phase 2). The repair surfaces are:
    - `papyrus config get|set|unset|list`, which load the configuration **tolerantly** — `document_types` is not validated on entry, so a broken slug or status list cannot lock the user out of the command that fixes it — apply the requested change, validate the **resulting** configuration with the full validator (decisions 1–7), and only then write. An invalid result is refused (exit 4) and nothing is written.
    - `papyrus init --force`, which replaces the file with the generated default set.

    Repair mode is not a bypass: it never executes record operations, never builds a command tree or registry from an invalid set, and never relaxes NFR-02.002 or NFR-04.002 for the key it edits. The changed key goes through the same slug/prefix/directory/status sanitization and rejection rules, the resulting map still passes `Context::ensure_within`, and the tolerant load is limited to reading the pre-existing invalid entries — never to writing one. The browser configuration view is not a repair surface: it cannot start when the configuration is invalid, so repair is CLI-only.

11. **Validation precedes registry construction and is repeated there.** `ArtifactRegistry::build` re-asserts the invariants it depends on (unique slug, unique prefix, non-empty resolved directory, no built-in collision, a document status list that contains `active` and `archived` and a `default` that is a declared, non-`archived` value) and returns an error rather than assuming the validator ran. This is defence in depth for tests, for the tolerant repair load, and for any future loader.

12. **No partially valid set is ever observable.** There is no "best effort" mode, no skipping of invalid entries, no warning-then-continue, and no environment flag to bypass validation. A type either exists and is valid or the process does not start.

13. **Diagnostics are actionable.** A rejected entry reports the slug, the field, the offending value, and the rule it violated (for example `document_types.req.directory: value must be relative and stay inside the project directory: ../shared`). The same message is used by the CLI and by `config list`'s validation report so a fix can be made from either surface.

## Consequences

**Positive:**

- A malicious or accidental slug cannot shadow a built-in command, escape the project directory, or inject terminal control characters; the guarantee is enforced at load and independent of dispatch order.
- Whole-map semantics make "remove a type" and "disable a type" distinct and both expressible, which is what REQ-07.002 promises.
- One validator with one outcome and one aggregated message shortens the repair loop: one attempt reports every invalid entry.
- Prefix and directory uniqueness eliminate two classes of ambiguous record resolution before they can occur.
- The status vocabulary is validated once, so a `status` command can never accept a value the type does not declare and `archived` can never be smuggled in as a custom status.
- Duplicate-slug rejection removes an order-dependent configuration defect that no user could see in the file.

**Negative:**

- The whole-map semantics is a behaviour change for existing projects: a file that lists three document types now really has three, not eleven. Projects that relied on the merge gain types back only by re-adding entries or running `init --force`, and a project that relied on the seeded `deploy` entry must now declare it.
- Case-insensitive reserved-name matching and prefix/directory uniqueness are stricter than the requirement text; a configuration that REQ-07.002 literally permits may be rejected. They remain flagged for confirmation.
- `config set` now rewrites the whole `document_types` map, producing a larger diff than a single-key edit and discarding comments in that section (the current writer already re-serializes YAML).
- The repair mode of decision 10 is a deliberate, narrowly scoped hole in "fail closed": the tolerant load and the validated write are two different code paths and must stay visibly distinct.

**Risks and mitigations:**

- *Upgrade breakage*: a project whose configuration was only valid because defaults were merged now fails at load, and a project that used the seeded `deploy` entry loses that command. Mitigation: the release note documents both, the error names each missing entry, and `init --force` regenerates the default set.
- *Validator/loader divergence*: `config get/set/unset` currently validate independently; they must call the same function, and a test asserts the same message from both surfaces. A second test asserts that repair mode rejects an invalid resulting configuration and writes nothing.
- *Stricter rules reject valid projects*: flagged as a requirement question; if the user prefers the literal reading, decisions 3 and 5/6 lose the case-insensitive and uniqueness extensions without affecting anything else.
- *Status list read from the wrong source*: repair mode must validate the resulting list, not the file's existing list, or a `status` command could later accept a stale value; the test matrix covers add, remove, and redefine attempts.
- *Path check bypass via a symlinked subdirectory*: `ensure_within` is applied at load and at every resolution, and a test covers a symlink inside `doc_directory` pointing outside the project.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| One aggregated, reject-only validator; whole-map `document_types`; repair mode limited to `config` (this ADR) | Selected | Satisfies REQ-07.002 and NFR-02.002/04.002 together, keeps removal expressible, and leaves a recovery path. |
| Warn on invalid entries and skip them | Rejected | Violates "no partially valid document-type set" and hides a configuration defect behind a runtime surprise. |
| Sanitize slugs (trim, lowercase, replace separators) | Rejected | NFR-02.002 requires rejection, not silent cleaning; normalization would also change command names behind the user's back. |
| Keep per-entry merge and express removal as `enabled: false` only | Rejected | Contradicts REQ-07.002's "removing an entry stops managing those records"; a disabled entry still reserves its prefix and still appears in lifecycle views as unmanaged. |
| Fail closed everywhere including `config` | Rejected by the owner decision of 2026-11-13 (D4) | Simplest rule, but makes an invalid configuration unrecoverable through Papyrus itself; the owner chose the two repair surfaces. |
| Validate lazily in each command | Rejected | Guarantees divergence between CLI, browser, and record layer, and allows a command to start on an invalid set. |

## Open Questions

- Should a slug that matches a built-in name with different case (`Req`) be rejected (current decision) or accepted?
- Should duplicate prefixes and duplicate enabled directories be rejected (current decision) or only the literal REQ checks performed?

## Downstream constraints

1. `Config::validate` is the only validator; `config get/set/unset/list` must not carry a private copy.
2. `cli::BUILTIN_COMMANDS` is the only reserved-name list.
3. `load_effective_value` must not merge `document_types` per entry; the key is wholesale.
4. No command may start on an unvalidated `document_types` set; the registry constructor re-asserts the invariants.
5. Every message must name `document_types.<slug>` and the offending field.
6. Repair mode may not write an entry that fails decisions 2–7 and may not relax NFR-02.002/NFR-04.002; the tolerant load reads, it never permits.
7. A status list is validated only here; the registry and the `status` command consume the validated list and never re-parse the file.

## References

- `docs/requirements/business/REQ-07.002.md` — slug, prefix, directory, status lists, duplicate keys, reserved names, fail-closed and repair surfaces.
- `docs/requirements/business/REQ-07.001.md` — main status, custom statuses, `status` command.
- `docs/requirements/business/REQ-05.011.md` — `status` exemption from the option rule.
- `docs/requirements/business/REQ-01.002.md` — `config set` as the configuration edit surface.
- `docs/requirements/business/REQ-01.003.md`, `REQ-01.004.md` — configuration selection and exit codes.
- `docs/requirements/non-functional/NFR-02.002.md` — sanitize and reject input.
- `docs/requirements/non-functional/NFR-04.002.md` — project-directory boundary.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — configuration-driven paths and containment.
- `docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md` — built-in name table and startup order.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — registry construction invariants.
- `docs/architecture/adr/ADR-0018-generic-document-content-model.md` — prefix-based claim rule.
- Implementation: `src/config.rs`, `src/commands.rs`, `src/sanitize.rs`, `src/context.rs`, `src/cli/` (new).
