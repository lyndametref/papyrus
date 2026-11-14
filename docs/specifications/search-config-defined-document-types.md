# Search Config-Defined Document Types

## Status

Specification for TASK-113 (registry-derived document search). Consumes the
artifact registry (ADR-0017, TASK-105), the record layer (TASK-107), and the
generic document handler (TASK-110). It changes no command shape, option
vocabulary, output format, or exit code established by
`docs/specifications/search-complete-unified.md`. CLI-only; no UI.

## Purpose

`papyrus search` currently walks the static `DOC_KEYS` table in
`src/search.rs`, so a document type declared in `document_types` is managed by
the CLI but invisible to search. This increment makes the artifact registry the
only source of the searched document-type set, so a type added, renamed,
re-pointed, or disabled in configuration changes search results with no code
change and no rebuild (REQ-06.001, REQ-07.002).

## Scope

In scope:

- Replace the static document-type enumeration in `src/search.rs` with the
  enabled `Document` descriptors of the artifact registry.
- Keep every existing search behavior: `papyrus search <query> [--type
  <type>]... [--limit <n>]`, the five documented type values, case-insensitive
  whole-text substring matching, ID ordering, `<id> <type> <title>` text
  output, `--json` shape, `--limit`, exit codes, sanitization, and
  boundary enforcement.
- Descriptor-driven file extension, ID shape, title source, and directory
  resolution, so built-in typed types and config-defined generic types share
  one code path.
- Active-directory-only indexing, archived/disabled exclusion, and read-path
  auto-relocation required by REQ-05.007.
- Unit and integration tests.

Out of scope:

- New `--type` values, per-document-type filters, or a `papyrus <slug> search`
  command (REQ-06.001 and BR-015 fix the vocabulary; see Assumption A1).
- New options, ranking, scoring, field-qualified queries, or output formats.
- Changing the registry model (ADR-0017), record storage (ADR-0013), or the
  generic content model (ADR-0018).
- Browser search or browser changes — TASK-114.
- Configuration validation — already TASK-109.

## Behavior

### 1. Document set

1. The searched document set is exactly
   `ArtifactRegistry::enabled(ArtifactCategory::Document)`, in registry order.
   No module-local list of document slugs exists in `src/search.rs`.
2. The registry is the instance already built at startup from the validated
   effective configuration; search does not build a second type model.
3. A document type with `enabled: false` contributes no candidates and no
   error. A type absent from `document_types` does not exist in the registry
   and contributes nothing.
4. Built-in typed document types (`req`, `nfr`, `glossary`, `ui-spec`,
   `ui-mockup`, `rule`, `audit`, `tech-spec`, `use-case`, `test-case`,
   `test-report`) and config-defined generic types are handled by the same
   descriptor-driven code path.
5. Task, milestone, release, and decision collection is unchanged.

### 2. Directory, files, and identity

1. A document descriptor's active directory is
   `<project>/<doc_directory>/<descriptor.directory.relative>`, contained by
   the existing project-boundary check. The configured `doc_directory` and the
   entry directory are honored; neither `docs/` nor a prefix is hardcoded.
2. Only files directly inside the active directory are candidates. There is no
   recursive scan; `archived/`, a legacy inert inactive folder left by an
   earlier version (ADR-0022 decision 6), and any other subdirectory are never
   read (REQ-05.007).
3. The candidate extension follows `content_model`: `HtmlMockup` → `.html`;
   every other document type → `.md`.
4. A candidate is a record only when its file stem parses with the
   descriptor's prefix and `id_shape`:
   - `Grouped`: try `<prefix>-<group>.<sequence>`, then `<prefix>-<sequence>`
     (requirement group documents);
   - `Dotted`: `<prefix>-<group>.<sequence>` only;
   - `Plain`: `<prefix>-<sequence>` only.
   No slug literal selects a parse rule.
5. A missing active directory yields no results for that type and no error.
   An unreadable directory is an IO error (exit 1). A non-UTF-8 file is skipped.
6. Record identity and title come from the shared projection used by list and
   view: front-matter `id`/`title` when present, otherwise the heading ID/title.
   UI mockups keep `ui_mockup::extract_title` (HTML `<h1>`/`<title>`). A record
   with no usable title is displayed with its ID as the title.

### 3. Matching, ordering, and output — unchanged

1. The searchable text is the whole persisted record text, including front
   matter. Title, description, labels, references, status, and body text are
   therefore all matchable; there is no field-restricted query.
2. Matching is case-insensitive Unicode substring matching with the query
   trimmed; a quoted phrase is one contiguous substring. No tokenization,
   stemming, fuzzy matching, ranking, or regular expressions.
3. Results are ordered by ID ascending byte-wise, then documented type value,
   then path. `--limit` keeps the first N after ordering.
4. Text output is `<id> <type> <title>` per line, `<type>` is `doc` for every
   document result. `--json` is an ordered array of `{id, type, title, path}`
   with the canonical absolute path. No matches → empty stdout / `[]`, exit 0.
5. `--type doc` returns document results from every enabled document type,
   including config-defined ones; `--type task|milestone|release|decision`
   never returns a document. When `--type` is omitted, all stores are searched.
6. A config-defined record's `type` value is `doc`; the document slug is not a
   `--type` value and does not appear in output.

### 4. Visibility

1. A record in the active directory whose stored main status is `archived` is
   auto-relocated to `<active dir>/archived/` with the ADR-0022 sequence before
   the result would be returned, and is not returned (REQ-05.007, ADR-0022
   decision 12). A successful relocation is reported on stderr; a custom-status
   record is not relocated. This relocation is the only permitted write on the
   search path; no other write occurs.
2. If relocation fails, search fails with a validation outcome (exit 4) naming
   the source record; the source file is left intact and no result is emitted.
3. A record carrying a custom status (for example `review`) stays in the active
   directory and is searchable; custom statuses are not visibility switches.
4. A UI mockup whose HTML metadata status is `archived` is never returned. HTML
   status is not part of the front-matter relocation contract, so search does
   not relocate it (Assumption A2).
5. Archived files inside `archived/` are never scanned, so an archived record
   cannot reappear after relocation.

## Implementation plan (task level)

1. Thread the startup `ArtifactRegistry` into `search::run` (dispatch passes
   `&ArtifactRegistry`; no second `ArtifactRegistry::build`).
2. Delete `DOC_KEYS` and the `collect_type` per-key branching. Replace it with
   one `collect_document_type(ctx, cfg, descriptor, needle, hits)` driven by
   `TypeDescriptor` fields: `directory` (`doc_directory` + relative),
   `content_model` (extension, title source), `id_shape` (ID parse), `prefix`.
3. Keep `collect_backlog_store` and `collect_decision_store` unchanged; filter
   their calls by the documented `--type` values as today.
4. In `collect_document_type`, call `record::store::relocate_if_archived`
   before matching, report a successful move on stderr, and skip the candidate
   when the returned path left the active directory. Map relocation failure to
   `AppError::Validation` per Behavior 4.2. Do not add per-slug arms.
5. Keep the existing sort, limit, text, and JSON rendering unchanged.

Files: `src/search.rs` (main), `src/cli/dispatch.rs` and
`src/cli/builtin.rs` only as needed to pass the registry, `tests/search_cli.rs`
(or a sibling `tests/search_config_types_cli.rs`) for tests.

## Test plan

| ID | Scenario | Assertion |
|---|---|---|
| T-113-01 | User-added generic type (`note`, prefix `NOTE`, directory `notes/`), records created through `papyrus note create` | Default search by title, description, label, and reference each returns `NOTE-001 doc <title>`; `--type doc` does too |
| T-113-02 | Same record with `--json` | One object `{id, type:"doc", title, path}`; path is the canonical `notes/NOTE-001.md` |
| T-113-03 | Same query with `--type task`/`milestone`/`release`/`decision` | No `NOTE-001` result; exit 0 |
| T-113-04 | Disable the type with `config set document_types.note.enabled false` | No result, exit 0; re-enable without rebuild restores it |
| T-113-05 | `papyrus note archive NOTE-001` | Query unique to the record returns nothing; a file hand-written in `notes/archived/` is never scanned |
| T-113-06 | Hand-edit a record in `notes/` to `status: archived`; run search | Record is not returned, file is relocated to `notes/archived/`, diagnostic on stderr, exit 0 |
| T-113-07 | Hand-add a file under a legacy inactive subdirectory | Never returned |
| T-113-08 | Record with a configured custom status (`review`) | Still returned by search |
| T-113-09 | Rename prefix and directory of the user type in configuration | Results and JSON path follow the new values |
| T-113-10 | Existing built-ins: `req` group + member IDs, `ui-mockup` HTML, `rule` typed | A group-title query returns the group record and a member-title query returns the member (`REQ-001` and `REQ-001.001` both resolvable); the mockup title comes from HTML; the rule is found |
| T-113-11 | Mixed result set ordering and `--limit` | ID order preserved; limit applied after ordering; no duplicate from repeated `--type doc` |
| T-113-12 | Missing directory, unreadable directory, symlink escape | Missing → empty success; unreadable → exit 1; outside-project symlink → exit 1 |
| T-113-13 | Unit check on the document collector | For a fixture config, collected type set equals the enabled Document descriptors; `DOC_KEYS` and slug matches are absent |

## Acceptance criteria mapping

- AC 1 → T-113-13, code review (no `DOC_KEYS`, no slug arm).
- AC 2 → T-113-01, T-113-02, T-113-10.
- AC 3 → T-113-05, T-113-06, T-113-07, T-113-04.
- AC 4 → T-113-04, T-113-09, T-113-03.
- AC 5 → T-113-04..T-113-08.

## Assumptions

- **A1** — "Per-type search" means the documented type-restricted search
  (`--type doc`, plus the four non-document values), not a per-document-slug
  filter. REQ-06.001 and BR-015 fix the five documented values; a
  per-document-type filter would require a man-page/requirement change, as
  recorded in `search-complete-unified.md` OQ-1.
- **A2** — Auto-relocation on read applies to front-matter status (REQ-05.007
  wording). Archived UI mockups are excluded by their HTML status but are not
  relocated by search.
- **A3** — Matching remains whole persisted text (no narrowing to
  title/description/labels/references) to preserve current behavior and avoid
  hiding matches; "same metadata projection" applies to ID/title derivation.
- **A4** — A registry build failure is already an exit-4 validation outcome at
  startup (ADR-0017 decision 9); search does not duplicate validation.

## Open questions

- None blocking. A future per-document-type `--type` value remains a
  requirement/man-page change, not a search change.

## Traceability

- REQ-06.001 — Search Project Records.
- REQ-05.007 — Manage Document Archive State and Deletion (archived visibility,
  auto-relocation, inert legacy folder).
- REQ-07.002 — Configure Document Types and Slugs (enabled/disabled, prefix,
  directory).
- REQ-06.004 — Front-matter metadata projection used for ID/title.
- ADR-0017 — Artifact category registry and type descriptors (single type
  model; no second list).
- ADR-0022 — Document lifecycle state and archive location (visible only in
  the active directory; auto-relocate on read).
- `docs/specifications/search-requirements.md`,
  `search-complete-unified.md` — unchanged command, matching, ordering,
  output, and exit-code contract.
