# Feature Specification — Complete Unified Search Across All Documented Record Types

**Ticket:** TASK-18
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Final integration increment of `papyrus search`: one query searches every documented record type that Papyrus manages, so that a contributor can find any project record without knowing which store or document family it lives in. The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, sanitization, and read-only guarantee established by `docs/specifications/search-requirements.md` and reused by every later search increment are reused unchanged; this increment adds no new `--type` value and no new option. It completes REQ-06.001 across all of REQ-05's document types plus REQ-02 (tasks), REQ-03 (milestones/releases), and REQ-04 (decisions), and advances BR-015. It also confirms that lazy front-matter migration (ADR-0007, ADR-0009) does not break or narrow search coverage.

## Scope

### In scope

- A single default `papyrus search <query>` (no `--type`) returns matching records from every documented record type:
  - business requirements and their requirement group documents (`document_types.req`),
  - non-functional requirements and their group documents (`document_types.nfr`),
  - task records (`backlog_directory/tasks/`),
  - milestone records (`backlog_directory/milestones/`),
  - release records (`backlog_directory/releases/`),
  - decision records (`decision_directory`),
  - glossary terms (`document_types.glossary`),
  - UI specifications (`document_types.ui-spec`),
  - UI mockups (`document_types.ui-mockup`, HTML records),
  - business rules (`document_types.rule`),
  - audit records (`document_types.audit`),
  - technical specifications (`document_types.tech-spec`).
- `--type doc` aggregates every document-type store above (all except `task`, `milestone`, `release`, `decision`) in one result set.
- `--type task` / `milestone` / `release` / `decision` continue to return only their own store.
- Case-insensitive substring matching over the whole persisted record text (including front matter), quoted-phrase handling, deterministic ordering by record ID, text output, `--json` output, and `--limit` capping the combined result set, all reused unchanged.
- Deprecation visibility: deprecated document records are never returned; restored records reappear.
- Archival visibility: archived task and milestone records are never returned; release records (no archived state) are all searched.
- Disabled or absent document types contribute no results and no error.
- Project-directory boundary enforcement, input sanitization, and a strict read-only guarantee, reused unchanged.
- Unit and integration tests, including a cross-type test that creates one record of every supported type and searches them together.
- Implementation inside the existing single crate, using existing dependencies only (ADR-0001, ADR-0004).

### Out of scope

- New `--type` values or any change to the documented `--type` vocabulary (`task`, `milestone`, `release`, `decision`, `doc`).
- New options, filters, sorting modes, output formats, or ranking.
- Fuzzy matching, stemming, tokenization, boolean operators, field-qualified queries, or regular expressions.
- Searching archived task/milestone records, even on request (a future increment may add an include flag).
- Searching deprecated records, even on request.
- Browser/web UI access — TASK-19.
- Packaging and release verification — TASK-20 / TASK-73.
- Configuration changes, new configuration keys, indexes, caches, or persisted search state.
- Modifying, moving, creating, archiving, restoring, or deleting records; `search` is read-only.
- UI mockups for the CLI; this is a CLI-only increment.

## Behavior

### 1. Command shape, query, and options

Unchanged from `docs/specifications/search-requirements.md` sections 1 and 4 and BR-015: `papyrus search <query> [--type <type>]... [--limit <n>]`, with the five documented type values, repeatable `--type`, positive `--limit`, and the same validation/exit codes (0 success including no matches, 2 usage, 4 validation, 1 IO/boundary; 3 never used). Global options behave as for existing commands.

### 2. Record sources and visibility

1. The complete set of stores is enumerated in Scope. No store is omitted, including the types added after TASK-5 (`glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`) and the backlog/decision stores.
2. Document types are resolved through the effective configuration (`document_types.<key>`) exactly as in the per-type increments: a key absent from configuration or with `enabled: false` contributes no results and no error.
3. Only active records are searched. For document types, only files directly inside the configured active directory are candidates; the `deprecated/` subfolder is never scanned or read. For task and milestone stores, only files directly inside the active folder are candidates; archived records live in `backlog/archived/` or `backlog/archive/<type>/` and are never scanned. Release records have no archived state.
4. Only files directly inside the configured active directory with the documented extension are candidates: `.md` for every store except UI mockups, which use `.html`. Subdirectories, non-record files, and files outside the configured directory are ignored; there is no recursive traversal.
5. Filename eligibility follows each store's documented rule:
   - `req`, `nfr`, `glossary`: any `.md` file directly in the active directory (group documents count as records).
   - `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`: only files whose stem matches `<prefix>-<digits>` (for example `UI-001.md`, `MOCK-0001.html`, `RUL-001.md`, `AUD-001.md`, `TECH-001.md`); registry, description, and other non-record files are ignored.
   - `task`, `milestone`, `release`: files named with the store's documented ID form (for example `TASK-001.md`, `MS-001.md`, `REL-001.md`).
   - `decision`: files whose stem is an enabled decision type's ID form (for example `ADR-001.md`), delegated to the decision module for consistent prefix matching, including hyphenated prefixes.
6. A missing configured store directory yields no results for that store and no error; `search` must not create it.
7. An unreadable configured store directory is an IO error (exit 1). A record file that cannot be read as UTF-8 is skipped rather than aborting the search.
8. A symbolic link that resolves outside the project directory is a boundary violation (exit 1) for every store, and the outside content is never read.
9. Record ID is the file stem. Title is the text after the ID on the first heading line for Markdown records, or the text after the ID in the mockup's `<h1>` heading for HTML mockups; if that text is empty, the ID is used as the title. For task, milestone, and release records the front-matter `title` is the authoritative source when the body has no `#` heading, consistent with the task/milestone/release list commands.
10. Lazy front-matter migration (ADR-0007, ADR-0009) may add a front-matter block to a record on first read. Search reads the file's persisted text after that read; a migration that cannot be applied safely leaves the file untouched and the record is still searched from its existing text. Search itself performs no writes.

### 3. Searchable text and matching

1. The searchable text of a record is its entire persisted record text, including front matter, so metadata values are searchable. This holds for migrated and unmigrated records alike.
2. Matching is case-insensitive substring matching over that text, using Unicode-aware lowercasing of both the query and the record text.
3. The query is trimmed of leading and trailing whitespace before matching. Internal whitespace is preserved exactly; a quoted phrase is matched as one contiguous substring, including its internal spaces.
4. There is no tokenization, word splitting, stemming, fuzzy matching, ranking, boolean syntax, field-qualified syntax, or regular-expression interpretation. Characters such as `.`, `*`, `(`, `[`, and `?` are literal.
5. A record is returned at most once, regardless of how many times the query occurs in it.

### 4. Ordering, limit, type value, and output

1. Each hit reports the documented filter value as its type: `task`, `milestone`, `release`, `decision`, or `doc` (for every document-type record).
2. Results are ordered by record ID in ascending byte-wise lexicographic order. Ties are broken by the documented type value and then by file path, so output is deterministic across runs.
3. In text mode, each result is printed on its own line as `<id> <type> <title>`. Lines are separated by a newline. There is no header, no trailing blank line, and no output at all when there are no matches.
4. With `--json`, the output is a JSON array of objects in the same order, each with `id`, `type`, `title`, and `path`, where `path` is the canonical absolute path of the record file.
5. When there are no matches, `--json` prints `[]`.
6. The limit, when supplied, keeps the first N results after ordering. If fewer records match than the limit, all matches are returned.
7. A successful search exits 0, including when it returns zero matches. `search` never exits 3.

### 5. Boundary, sanitization, and read-only guarantees

1. The project directory is selected and canonicalized as for existing commands, and the configuration path is boundary-checked as for existing commands.
2. Every configured store directory and every candidate record file is validated with the existing within-project boundary check.
3. The query is normalized before use: leading/trailing whitespace is trimmed; an empty or whitespace-only query, or a query containing NUL, carriage return, or line feed, is a validation error (exit 4). The query is never used as a path, so `/` and `\` are allowed as literal search characters.
4. `--type` values are validated against the fixed documented set by the CLI and are never used as paths.
5. `search` performs no writes: it does not create, modify, move, or delete files or directories, and it writes nothing to stdout/stderr except results and error messages.
6. `search` reads only the configuration file, the configured store directories, and candidate record files inside the canonical project directory. It performs no network access.

### 6. Empty results

1. A valid query with no matches exits 0, prints nothing in text mode, and prints `[]` in `--json` mode.
2. A valid query restricted to a store with no records behaves as an empty result, not an error.

## Inputs and outputs

| Input / output | Description |
|---|---|
| `<query>` | One required positional free-text term or shell-quoted phrase. |
| `--type <type>` | Repeatable; one of `task`, `milestone`, `release`, `decision`, `doc`; default all five. |
| `--limit <n>` | Optional positive integer cap applied after ordering. |
| `--json` | Global flag; emits the JSON result array or the standard JSON error object. |
| Text output | Zero or more `<id> <type> <title>` lines in ID order. |
| JSON output | Ordered array of `{id, type, title, path}` objects, or `[]`. |
| Exit status | `0` success (including no matches), `2` usage, `4` validation, `1` IO/boundary. `3` is never used. |

## Validation and error behavior

| Condition | Exit | Output |
|---|---|---|
| Missing `<query>` | 2 | Usage message on stderr. |
| More than one positional argument (unquoted multi-word query) | 2 | Usage message on stderr. |
| Unknown, uppercase, or comma-joined `--type` value | 2 | Usage message on stderr. |
| `--limit` non-integer, `0`, or negative | 2 | Usage message on stderr. |
| Empty or whitespace-only query | 4 | Validation message on stderr. |
| Query containing NUL, CR, or LF | 4 | Validation message on stderr. |
| IO failure or project-directory boundary violation | 1 | Runtime message on stderr. |
| No matches | 0 | Empty stdout, or `[]` with `--json`. |
| Matches | 0 | Result lines, or the JSON array. |

With `--json`, errors use the existing convention: a JSON object `{"error":{"code":<exit>,"message":"<message>"}}` on stderr.

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-18-01 | Default search returns matches from every documented record type in one query | Integration test creating one record of each type carrying a shared phrase |
| T-18-02 | `--type doc` returns all document-type records; `--type task`/`milestone`/`release`/`decision` return only their store | Type-filter integration tests across all stores |
| T-18-03 | Matching is case-insensitive substring matching over the whole record text, including front matter | Query in different case; query matching a front-matter value |
| T-18-04 | A quoted phrase is matched as one contiguous substring across all stores | Quoted-phrase integration test |
| T-18-05 | Results are ordered by record ID ascending across stores, ties by type then path | Multi-record ordering assertion spanning multiple types |
| T-18-06 | `--limit` caps the combined result set after ordering | Limit integration test with more matches than the limit |
| T-18-07 | Deprecated document records never appear; restored records appear again | Deprecate/restore integration test for each document type |
| T-18-08 | Archived task and milestone records never appear; release records are searched | Archive then search integration test |
| T-18-09 | Disabled document types are excluded; absent directories yield empty success | Config with disabled types; missing-directory snapshot |
| T-18-10 | Configured directories and prefixes are honored for every store | Project with a custom document directory and prefix |
| T-18-11 | Non-record files (registry, descriptions, notes, non-`.md`/non-`.html`) are ignored | Non-record file fixtures per store |
| T-18-12 | Symlink escape exits 1 and does not expose outside content for every store | Symlink fixture per store family |
| T-18-13 | `search` creates no files or directories and performs no writes | Filesystem snapshot before/after |
| T-18-14 | Front-matter migration does not narrow coverage: migrated records are searchable and metadata values are searchable | Search after triggering lazy migration on a record |
| T-18-15 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |

## Assumptions

- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004). `search` is extended inside the existing crate; no new crate, application, technology stack, external interface, or dependency is introduced, so no new ADR or architecture contract is required for search behavior itself. Front-matter migration is governed by ADR-0007 and ADR-0009.
- Assumption: the man page (BR-015) and REQ-06.001 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded as open questions with recommended defaults.
- Assumption: the `doc` type remains the only aggregate filter; no per-document-type filter value is added, consistent with REQ-06.001 and BR-015.
- Assumption: archived task and milestone records are hidden from search, mirroring deprecated requirement records; a future increment may add an option to include them.
- Assumption: release records have no archived state and are all searched.
- Assumption: the searchable text is the whole persisted record text, not only parsed fields, so editor-added body sections and front-matter metadata are searchable.
- Assumption: text output uses `<id> <type> <title>` with the documented filter value as `<type>`; the man page does not prescribe a result format.
- Assumption: `--limit 0` is an invalid value rather than "unlimited".
- Assumption: record IDs come from file stems, so configured prefixes are honored without parsing or hardcoding any prefix.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: `search` performs no writes at all, including not creating missing configured directories and not triggering migrations itself beyond the read path's own lazy behavior.
- Assumption: no UI mockups are needed because the increment is CLI-only.
- Assumption: front-matter migration is lazy and per record (ADR-0007 decision 9); search must remain correct for both migrated and unmigrated records.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: no dedicated type filter)** — The documented `--type` vocabulary contains no per-document-type value. This specification keeps `doc` as the aggregate for all document types. A future change to add per-type filters would be a man-page/REQ-06.001 change, not a search change.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text. A narrower field set would risk hiding relevant matches.
- **OQ-3 (recommended default: hide deprecated and archived)** — BR-014/BR-015 do not state whether deprecated or archived records are searchable. This specification hides them, consistent with every prior search increment and the list commands.
- **OQ-4 (recommended default: title from heading, front-matter fallback for task/milestone/release)** — The man page does not prescribe title extraction. This specification uses the heading text for Markdown records and the front-matter `title` for task/milestone/release records that have no `#` heading, consistent with the per-type list commands.
- **OQ-5 (recommended default: no include-deprecated / include-archived flag)** — No option to include hidden records is documented. This specification adds none; a future increment may.
- **OQ-6 (recommended default: migration is read-path behavior, not search behavior)** — Lazy migration may write front matter during a read. Search does not treat that as a search write; if migration fails on a read-only filesystem, the record is still searched from its existing text and the error is surfaced by the read path. Confirm this matches the product's read-only guarantee expectation.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05, REQ-05.001–REQ-05.011, REQ-02, REQ-02.001, REQ-02.003, REQ-03, REQ-03.001, REQ-03.002, REQ-04, REQ-04.001, REQ-04.002 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary), NFR-04.003 (Backlog.md task/milestone interoperability) |
| Business rule | BR-015 (search command behavior), BR-005 (task), BR-006 (milestone), BR-007 (release), BR-008 (decision), BR-011 (glossary), BR-012 (UI specification), BR-013 (business rule), BR-014 (audit), BR-016 (configuration keys and precedence), BR-017 (technical specification) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate), ADR-0007 (YAML front matter), ADR-0009 (shared record envelope) |
| Ticket | TASK-18 |
| UI mockups | none |
| Domain model | `docs/domain-model/business-glossary/` (record-type terms) |
| Related specifications | `docs/specifications/search-requirements.md` (TASK-5), `docs/specifications/search-tasks-milestones-and-releases.md` (TASK-10), `docs/specifications/search-decision-records.md` (TASK-14), `docs/specifications/search-glossary-terms.md` (TASK-31), `docs/specifications/search-ui-specifications.md` (TASK-35), `docs/specifications/search-ui-mockups.md` (TASK-39), `docs/specifications/search-business-rules.md` (TASK-44), `docs/specifications/search-audit-records.md` (TASK-48), `docs/specifications/search-technical-specifications.md` (TASK-56) |
| Related specifications | `docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md` (TASK-68), `docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md` (TASK-69), `docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md` (TASK-70) |
