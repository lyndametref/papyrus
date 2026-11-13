# REQ-07.002 Configure Document Types and Slugs

## User Story
As a project maintainer, I want to declare document types in the configuration file so that I can add, rename, disable, or remove a document type without changing Papyrus.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
`document_types` is a map keyed by **document slug**. The slug is the configuration key, the CLI command name (`papyrus <document-slug>`, REQ-07.003), and the identifier used for that document type elsewhere. No separate `document-slug` attribute exists, because the map key already carries that role.

Each entry declares `prefix` (ID prefix), `directory` (folder relative to `doc_directory`), and `enabled`, and may declare `statuses` (the document type's custom statuses, REQ-07.001) and `default_status` (the status a new record of that type starts at). `directory` is the type's own folder; archived records are stored in an `archived/` subfolder of it (REQ-05.007).

Entries are fully ordinary. The default entries seeded by `papyrus init` have no reserved status: any entry can be renamed, removed, disabled, or given a different prefix or folder, and new entries can be added by editing `papyrus.config.yml` or with `papyrus config set`. A project initialized with `papyrus init` writes the default set:

- `req` — `REQ`, `requirements/business/`
- `nfr` — `NFR`, `requirements/non-functional/`
- `glossary` — `GLO`, `glossary/`
- `rule` — `RUL`, `business-rules/`
- `ui-spec` — `UI`, `specifications/ui/`
- `ui-mockup` — `MOCK`, `specifications/ui/mockups/`
- `audit` — `AUD`, `audit-records/`
- `tech-spec` — `TECH`, `technical/`
- `use-case` — `UC`, `use-cases/`
- `test-case` — `TC`, `test-cases/`
- `test-report` — `TR`, `test-reports/`

All default entries are enabled unless the interactive walkthrough (REQ-01.006) disables an artifact. `deploy` is not part of the default set; a project may still add it, like any other user-defined type, by declaring a `deploy` entry.

Configuration loading validates every entry. A slug must be usable as a command name: non-empty, composed of ASCII letters, digits, `-`, and `_`, not starting with `-`, and free of whitespace, control characters, and path separators. A slug must not equal a built-in command name — `init`, `config`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` — so a configured document type can never shadow a built-in command; the comparison is case-insensitive, so `Config` is rejected as well as `config`. Each entry must have a non-empty `prefix` and a `directory` that is relative and stays inside the project directory (NFR-04.002). A `prefix` must be unique across all configured document types, including disabled entries. Two enabled entries must not declare the same `directory`. Values that fail these rules are rejected as configuration errors and are never silently cleaned or normalized (NFR-02.002).

A `document_types` mapping that declares the same slug key twice is invalid configuration and is rejected; a repeated key is never resolved by keeping the first or the last occurrence.

`statuses`, when present, must be a non-empty list with no duplicates and must include `active` and `archived` exactly once. Those two values are fixed: configuration cannot remove, rename, or replace them, and a custom status is any other value in the list. Every declared status follows the same character rules as a slug: non-empty, ASCII letters, digits, `-`, and `_`, not starting with `-`, with no whitespace, control characters, or path separators. `default_status`, when present, must be one of the declared statuses and must not be `archived`, so a new record never starts archived. A `statuses` declaration on the default UI mockup type `ui-mockup` is rejected, because that type keeps its state in HTML metadata and declares no custom statuses (REQ-05.008).

`statuses` is a list-valued configuration key. `papyrus config set document_types.<slug>.statuses active,archived,custom` writes it from a comma-separated value and stores it as a YAML flow sequence (REQ-01.002). Commas are separators with no escape form, so a status value can never contain a comma — which the status character rules already exclude — and an empty value is rejected rather than written as an empty list. The configuration that would result is validated before it is persisted, so an invalid list is refused and nothing is written.

A violating configuration is rejected with a validation outcome (REQ-01.004) that names the offending slug or key. Validation is fail-closed and there is no partially valid document-type set: every command refuses to run while the effective configuration is invalid, with two repair surfaces. `papyrus config get|set|unset|list` loads the configuration tolerantly, validates the configuration that would result from the change, and refuses an invalid write. `papyrus init --force` replaces the configuration file with the generated default set. A user can therefore always repair a broken configuration through Papyrus itself.

Adding an entry adds a document type and its command; disabling an entry removes its CLI command and browser views while leaving existing files untouched; removing an entry stops managing those records without deleting files.

## Acceptance Criteria
- `document_types` is keyed by document slug, and the slug is the CLI command name for that document type.
- No separate `document-slug` configuration attribute exists.
- Every entry declares `prefix`, `directory`, and `enabled`, and may declare `statuses` and `default_status`.
- `papyrus init` writes the default document-type set listed in this requirement, all enabled unless the interactive walkthrough (REQ-01.006) disables an artifact; `deploy` is not in that set.
- `deploy` can still be added as an ordinary user-defined document type.
- The default entries have no reserved status: any entry can be renamed, removed, disabled, or given a different prefix or folder by editing the configuration.
- A new document type can be added by editing `papyrus.config.yml` or with `papyrus config set`, without a code change.
- A slug is validated at configuration load: it is non-empty, uses only ASCII letters, digits, `-`, and `_`, does not start with `-`, and contains no whitespace, control character, or path separator; a violating value is rejected and not silently cleaned or normalized (NFR-02.002).
- A slug equal to `init`, `config`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` is rejected with a validation outcome naming the conflict, and the comparison is case-insensitive, so a slug such as `Config` or `SEARCH` is rejected too.
- Each entry must have a non-empty `prefix` and a `directory` that is relative and inside the project directory (NFR-04.002); a violating entry is rejected at configuration load.
- A `prefix` declared by more than one entry is rejected at configuration load, including when one of those entries is disabled.
- Two enabled entries that declare the same `directory` are rejected at configuration load; disabled entries do not take part in this check.
- A `statuses` list is rejected at configuration load when it is empty, contains duplicates, omits `active` or `archived`, or contains a value that violates the slug character rules.
- `papyrus config set document_types.<slug>.statuses <value>` accepts a comma-separated value such as `active,archived,custom` and stores it as a YAML flow sequence; `config get` returns the same values in the same order.
- An empty value for `statuses` is rejected with a validation outcome and nothing is written; commas are separators with no escape form, so a value containing a comma can only be written by editing `papyrus.config.yml` directly.
- A `statuses` value written by `config set` is validated as the resulting configuration before it is persisted: a list that violates these rules is refused with a validation outcome and nothing is written.
- `active` and `archived` are always declared for a document type and cannot be removed, renamed, or replaced by configuration; every other value in the list is a custom status.
- A `default_status` that is not one of the declared statuses, or that is `archived`, is rejected at configuration load.
- A `statuses` declaration on the `ui-mockup` entry is rejected at configuration load, because UI mockups keep their state in HTML metadata (REQ-05.008).
- A `document_types` mapping that declares the same slug key twice is rejected as invalid configuration, and neither occurrence is silently preferred.
- An invalid configuration blocks every command with a validation outcome (REQ-01.004); no partially valid document-type set is used.
- The repair surfaces for an invalid configuration are `papyrus config get|set|unset|list`, which load tolerantly, validate the resulting configuration, and refuse an invalid write, and `papyrus init --force`, which replaces the file with the generated default set.
- A disabled entry has no CLI command and no browser view, and its existing files are untouched.
- Removing an entry stops managing its records and deletes no file.
- The non-functional requirement family is addressed by its slug `nfr`; the former `nfreq` command name is not retained as an alias.

## Open Questions
- Whether removing an entry should leave its former records visible read-only somewhere, or simply unmanaged. Current requirement: unmanaged and untouched.
- Whether slug renames should rewrite existing record IDs or folders. Current requirement: renaming is a configuration-only change, so existing records stay in their folder and keep their IDs.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "any type of document can be configured in the config file. a default set is proposed at init but editing the file allows to create any document type unless the name is already used by a buit in document type. ... a New config attribute document-slug is needed in the config file."
- Project-owner decisions 2026-11-13 (refinement): the `document_types` key is the slug and no separate attribute is added; default entries are fully ordinary (renameable/removable, no reserved names); only collisions with built-in command names are rejected; the CLI is dynamic only, so built-in document commands are not retained.
- Project-owner decision 2026-11-13 (D2): `deploy` is removed from the default document-type set and remains available only as a user-added type.
- Project-owner decision 2026-11-13 (D4): configuration validation is fail-closed for every command except `config get|set|unset|list` (tolerant load, validate the resulting configuration, refuse an invalid write) and `init --force`, so a broken configuration is always repairable through Papyrus.
- Project-owner decision 2026-11-13 (D5): `document_types.<slug>.statuses` and `document_types.<slug>.default_status` are validated at load for non-emptiness, uniqueness, the presence of the fixed `active`/`archived` values, and slug-style characters.
- Owner-confirmed decision 2026-11-13 (validation): duplicate `document_types` keys are invalid configuration rather than last-wins; reserved built-in name matching is case-insensitive; `prefix` is unique across all entries including disabled ones; duplicate `directory` values among enabled entries are rejected.
- Project-owner decision 2026-11-13 (list-valued keys): `statuses` is set through `papyrus config set` from a comma-separated value written as a YAML flow sequence (REQ-01.002); commas are not escapable; an empty value is invalid.
- ADR-0019 — Fail-Closed Document-Type Configuration Validation (accepted 2026-11-13).
- Current state: `src/config.rs` already accepts arbitrary `document_types` keys with `prefix`, `directory`, and `enabled`.
- REQ-01.001 — Initialize a Papyrus Project, REQ-01.002 — Read and Edit Project Configuration
- REQ-01.006 — Interactive Init Walkthrough (default artifact set)
- REQ-07.001 — Organize Artifacts into Three Categories (document status model)
- REQ-07.003 — Dispatch Config-Defined Document Commands
- NFR-02.002 — Sanitize Input (validation rejects rather than silently cleans)
- NFR-04.002 — Restrict File Access to the Project Directory (entry directory boundary)
- BR-016 — Configuration Keys and Precedence (needs revision for slug and status semantics)
