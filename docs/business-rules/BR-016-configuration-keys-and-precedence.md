# BR-016: Papyrus Configuration Keys and Precedence

## Description

`papyrus.config.yml` is the single source of project paths, ID formats, workflow statuses, document types, and browser settings, and its effective value is resolved in the fixed precedence order command-line options, environment variables, the project configuration file, then built-in defaults.

## Rationale

Based on REQ-01.002 — Read and Edit Project Configuration, REQ-01.003 — Select Project and Configuration Context, REQ-07.001 — Organize Artifacts into Three Categories, REQ-07.002 — Configure Document Types and Slugs, and REQ-01.008 — Run Project Diagnostics, this rule exists because configuration is the only place document types, their slugs, prefixes, directories, and status vocabularies are declared; the loader must therefore validate the whole map fail-closed and still leave a repair path, or one bad entry would make every command unusable and the project could not be fixed through Papyrus itself.

## Applicability

Applies to every `papyrus.config.yml` file and to the project data it governs.

### Precedence

1. Command-line options.
2. Environment variables (`PAPYRUS_DIR`, `PAPYRUS_CONFIG`).
3. `papyrus.config.yml` in the project directory.
4. Built-in defaults.

This order is unchanged by the document-type model.

### General, path, and workflow keys

- `project_name`, `task_prefix`, `milestone_prefix`, `release_prefix`, `id_padding` (1–6, default 3), `date_format`, and `editor`.
- `backlog_directory`, `decision_directory`, and `doc_directory`; each document type's `directory` is relative to `doc_directory`.
- `statuses` and `default_status` configure task statuses only. The milestone (`open -> closed`), release (`planned -> released`), and decision lifecycles stay fixed, and the document main statuses `active` and `archived` are not configurable.

### document_types

- The map is keyed by document slug; the key is the command name (`papyrus <slug>`) and the identifier used elsewhere. There is no separate document-slug attribute.
- Each entry declares `prefix`, `directory` (relative to `doc_directory`), and `enabled`, and may declare `statuses` and `default_status`. Archived records are stored in an `archived/` subfolder of the entry's directory.
- `papyrus init` seeds eleven entries: `req` (REQ, `requirements/business/`), `nfr` (NFR, `requirements/non-functional/`), `glossary` (GLO, `glossary/`), `rule` (RUL, `business-rules/`), `ui-spec` (UI, `specifications/ui/`), `ui-mockup` (MOCK, `specifications/ui/mockups/`), `audit` (AUD, `audit-records/`), `tech-spec` (TECH, `technical/`), `use-case` (UC, `use-cases/`), `test-case` (TC, `test-cases/`), and `test-report` (TR, `test-reports/`). `deploy` is not seeded and can be added as an ordinary user-defined entry.
- Default entries are fully ordinary: any entry can be renamed, removed, disabled, or given another prefix or directory.
- The map is whole: when the key is present, exactly its entries exist and no seeded entry is re-added. Removing an entry stops managing its records and deletes no file; `enabled: false` removes the command and browser view while keeping the entry, its prefix reservation, and its files.
- `statuses`, when present, is a non-empty duplicate-free list that contains `active` and `archived` exactly once; `active` and `archived` are fixed and every other value is a custom status. Every declared value follows the slug character rule. `default_status`, when present, must be a declared status and must never be `archived`. An entry without `statuses` has only the two main statuses and starts `active` unless `default_status` is set. `statuses` and `default_status` are rejected on `ui-mockup`, which keeps its state in HTML metadata.
- `statuses` is list-valued: `papyrus config set document_types.<slug>.statuses active,archived,review` writes the value as a YAML flow sequence; commas are separators with no escape form, and an empty value is rejected.

### Validation

- Fail-closed: every command except the repair surfaces (`config get|set|unset|list`), `init --force`, and `check` refuses to run while the effective configuration is invalid, and no partially valid document-type set is used.
- A violation is a validation outcome (exit code 4) that names `document_types.<slug>` and the offending field.
- Slug: non-empty, ASCII letters, digits, `-`, and `_`, not starting with `-`, with no whitespace, control character, or path separator; values are rejected, never trimmed or normalized.
- Reserved names: a slug equal to `init`, `config`, `check`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` is rejected. The comparison is case-insensitive, so `Config` and `SEARCH` are rejected too.
- `prefix` is non-empty and unique across all entries, including disabled ones.
- `directory` is relative, stays inside the project directory, and is unique among enabled entries.
- A repeated `document_types` slug key is invalid configuration; neither occurrence is preferred.

### Repair surfaces

- `papyrus config get|set|unset|list` load tolerantly, validate the configuration that would result, and refuse an invalid write.
- `papyrus init --force` replaces the file with the generated default set.
- `papyrus check` loads tolerantly, reports each configuration diagnostic as a `config-invalid` finding, runs only the checks that do not depend on the effective `document_types` set, and repairs no configuration value (REQ-01.008).
