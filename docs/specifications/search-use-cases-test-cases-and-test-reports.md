# Feature Specification — Expand Unified Search to Use Cases, Test Cases, and Test Reports

**Ticket:** TASK-80
**Milestone:** m-15 (Use Cases and Test Documentation)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Extend `papyrus search` to include use cases, test cases, and test reports. These are document records: they are stored under their configured `document_types` directories (default `use-cases/`, `test-cases/`, and `test-reports/`) and are searched whenever the documented `doc` type is selected (including the default all-types search). The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee established by `docs/specifications/search-requirements.md` and reused by later search increments are reused unchanged. This increment implements REQ-06.001 under REQ-05/REQ-05.012/REQ-05.013/REQ-05.014/REQ-05.007, and advances BR-015 and BR-018/BR-019/BR-020.

Search operates on the dedicated-type records managed by the `use-case`, `test-case`, and `test-report` commands (BR-018/BR-019/BR-020). The `req` wording in REQ-05.012/013/014 is superseded for implementation by those business rules; no `req`-managed records of these kinds exist.

## Scope

### In scope

- Searching active use-case, test-case, and test-report records using the existing `papyrus search <query> [--type <type>]... [--limit <n>]` shape.
- `--type doc` returns use-case, test-case, and test-report records together with the other document records.
- Default search (no `--type`) includes use-case, test-case, and test-report records together with task, milestone, release, decision, and `doc` records.
- Case-insensitive substring matching, quoted-phrase handling, deterministic ordering by record ID, text output, and `--json` output reused unchanged.
- `--limit <n>` capping the combined result set.
- Deprecation visibility: deprecated records of all three types are never returned; restored records appear again.
- Exclusion of disabled or absent `use-case`, `test-case`, and `test-report` document types (BR-016).
- Project-directory boundary enforcement, input sanitization, and a strict read-only guarantee, reused unchanged.
- Unit and integration tests for the new record types.
- Implementation inside the existing single crate, using existing dependencies only (ADR-0001, ADR-0004).

### Out of scope

- Searching any document type beyond the existing search stores and the three types added here — their dedicated search tasks and TASK-18.
- Adding a new `--type` value for use cases, test cases, or test reports (for example `--type use-case`); per-document-type filtering through `--type` is not part of the documented vocabulary. All three types are covered by `--type doc`.
- Searching deprecated records of these types — see Assumptions.
- Browser/web UI access for these record types — TASK-81 and browser milestones.
- Packaging and verification of the use-cases-and-test-documentation milestone — TASK-82.
- Configuration changes, new configuration keys, indexes, caches, or persisted search state.
- Fuzzy matching, stemming, token ranking, boolean operators, field-qualified queries, or regular expressions.
- Modifying, moving, creating, deprecating, restoring, or deleting records; `search` is read-only.
- UI mockups; this is a CLI-only increment.

## Behavior

### 1. Command shape, query, and options

1. The command shape is unchanged: `papyrus search <query> [--type <type>]... [--limit <n>]`.
2. The documented type vocabulary is unchanged: `task`, `milestone`, `release`, `decision`, and `doc`. No new type value is added; use-case, test-case, and test-report records are returned whenever `doc` is selected (including the default all-types search).
3. `--type` validation, repetition, defaulting to all five types, and `--limit` semantics are unchanged from `docs/specifications/search-requirements.md` sections 1 and 4.
4. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for the existing commands.

### 2. Record sources and visibility

1. In this increment, the searchable stores are the configured directories for the three document types:
   - `document_types.use-case` (default directory `use-cases/`, prefix `UC`),
   - `document_types.test-case` (default directory `test-cases/`, prefix `TC`),
   - `document_types.test-report` (default directory `test-reports/`, prefix `TR`),
   each resolved as `doc_directory` + `document_types.<key>.directory`. None is hardcoded.
2. Only the active folder of each type directory is searched. The `deprecated/` subfolder is never scanned or read. Deprecated records of all three types are therefore never returned, consistent with deprecated requirement records and archived backlog/decision records.
3. Only files directly inside the configured active directory whose file stem matches the configured record pattern (`<prefix>-<digits>`, for example `UC-001`, `TC-001`, `TR-001`) are candidates. Subdirectories (including `deprecated/`), non-Markdown files, and files whose stem does not match `<prefix>-<digits>` are ignored. There is no recursive traversal. This matches `use-case list` / `test-case list` / `test-report list` (BR-018/BR-019/BR-020), which limit records to `<prefix>-<digits>.md` files at the top level of the type directory.
4. A record type is included only when its document type is present in the effective configuration and its `enabled` flag is `true` (BR-016). A disabled or absent `use-case`, `test-case`, or `test-report` type contributes no results and no error.
5. A document type key or enabled flag does not otherwise gate a store beyond its own entry: each type is searched whenever `--type doc` is selected and its type is enabled, regardless of other `document_types` entries.
6. A missing configured type directory yields no results for that store and no error. `search` must not create it.
7. An unreadable configured type directory is an IO error (exit 1). A record file that cannot be read as UTF-8 is skipped rather than aborting the search.
8. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
9. A record ID is the file stem. Its title is the text after the ID on the first heading line; if that text is empty, the ID is used as the title (consistent with `use-case list` / `test-case list` / `test-report list`).

### 3. Searchable text and matching

1. The searchable text of a record is its entire persisted record text, including front matter, so metadata values are searchable: the top-level heading (ID and title) plus the `## Description` and `## Applicability` sections (when present), as well as any content added through `--editor` or general front-matter fields such as `labels` and `references`.
2. Matching is case-insensitive substring matching over that text, using Unicode-aware lowercasing of both the query and the record text.
3. The query is trimmed of leading and trailing whitespace before matching. Internal whitespace is preserved exactly; a quoted phrase is matched as one contiguous substring, including its internal spaces.
4. There is no tokenization, word splitting, stemming, fuzzy matching, ranking, boolean syntax, field-qualified syntax, or regular-expression interpretation. Characters such as `.`, `*`, `(`, `[`, and `?` are literal.
5. A record is returned at most once, regardless of how many times the query occurs in it.

### 4. Ordering, limit, type value, and output

1. Each hit of these three types reports the documented filter value as its type: `doc` (not `use-case`, `test-case`, or `test-report`), consistent with all other document records.
2. Results are ordered by record ID in ascending byte-wise lexicographic order. Ties are broken by the documented type value and then by file path, so output is deterministic across runs.
3. In text mode, each result is printed on its own line as `<id> <type> <title>`. Lines are separated by a newline. There is no header, no trailing blank line, and no output at all when there are no matches.
4. With `--json`, the output is a JSON array of objects in the same order, each with `id`, `type`, `title`, and `path`, where `path` is the canonical absolute path of the record file.
5. When there are no matches, `--json` prints `[]`.
6. The limit, when supplied, keeps the first N results after ordering. If fewer records match than the limit, all matches are returned.
7. A successful search exits 0, including when it returns zero matches. `search` never exits 3.

### 5. Boundary, sanitization, and read-only guarantees

1. The project directory is selected and canonicalized as for existing commands, and the configuration path is boundary-checked as for existing commands.
2. Each configured type directory and every candidate record file are validated with the existing within-project boundary check. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
3. The query is normalized before use: leading/trailing whitespace is trimmed; an empty or whitespace-only query, or a query containing NUL, carriage return, or line feed, is a validation error (exit 4). The query is never used as a path, so `/` and `\` are allowed as literal search characters.
4. `--type` values are validated against the fixed documented set by the CLI and are never used as paths.
5. `search` performs no writes: it does not create, modify, move, or delete files or directories, and it writes nothing to stdout/stderr except results and error messages.
6. `search` reads only the configuration file, the configured type directories, and candidate `.md` files inside the canonical project directory. It performs no network access.

### 6. Empty results

1. A valid query with no matches exits 0, prints nothing in text mode, and prints `[]` in `--json` mode.
2. A valid query restricted to a store with no use-case, test-case, or test-report records (for example an uninitialized or empty type directory) behaves as an empty result, not an error.
3. A valid query restricted to `--type doc` on a project with all three types disabled behaves as an empty result for these types, not an error.

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
| T-80-01 | Default search returns active use-case, test-case, and test-report records together with requirement, task, milestone, release, decision, and other document records | Integration test on a project with one record of each store |
| T-80-02 | `--type doc` returns use-case, test-case, and test-report records; `--type task`/`milestone`/`release`/`decision` never return them | Type-filter integration tests |
| T-80-03 | Matching is case-insensitive substring matching over the whole record text of all three types | Query in different case returns the same records |
| T-80-04 | A quoted phrase is matched as one contiguous substring across all three types | Quoted-phrase integration test |
| T-80-05 | Deprecated records of all three types never appear; restored records appear again | Deprecate/restore integration test per type |
| T-80-06 | Disabled or absent `use-case`, `test-case`, or `test-report` document type excludes its records | Config with `document_types.<type>.enabled: false` per type |
| T-80-07 | Records of all three types are returned with type value `doc` and title equal to the record title | Exact stdout/JSON assertions |
| T-80-08 | Results are ordered by record ID ascending across stores | Multi-record ordering assertion |
| T-80-09 | `--limit` caps the combined result set after ordering | Limit integration test |
| T-80-10 | Missing type directory yields empty success without creation | Filesystem snapshot before/after |
| T-80-11 | Symlink escape exits 1 and does not expose outside content | Symlink fixture |
| T-80-12 | `search` creates no files or directories | Filesystem snapshot |
| T-80-13 | Non-record files (for example canonical `BR-*.md`-style names or other stems) and the `deprecated/` subfolder are never scanned | Fixture with non-conforming file names |
| T-80-14 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |

## Assumptions

- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004). `search` is extended inside the existing crate; no new crate, application, technology stack, external interface, or dependency is introduced, so no new ADR or architecture contract is required.
- Assumption: the man page and BR-015 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded as open questions.
- Assumption: deprecated use-case, test-case, and test-report records are hidden from search, mirroring deprecated requirement records and archived backlog/decision records; a future increment may add an option to include deprecated records.
- Assumption: the searchable text is the whole persisted record text, not only parsed fields, so editor-added body sections and general front-matter values are searchable.
- Assumption: text output uses `<id> <type> <title>` with the documented filter value as `<type>` (`doc`), not the document type key or prefix.
- Assumption: record IDs come from file stems, so configured prefixes are honored without parsing or hardcoding `UC`, `TC`, or `TR`.
- Assumption: a record is included only when its document type is present and enabled, mirroring `use-case list` / `test-case list` / `test-report list` and BR-016; disabled types are never searched.
- Assumption: records live in the configured `document_types.use-case`, `document_types.test-case`, and `document_types.test-report` directories. Relocating a type directory is a configuration decision, not part of this ticket.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: `search` performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: no dedicated type filter)** — The documented `--type` vocabulary contains no use-case, test-case, or test-report value. This specification searches all three types under `--type doc`, consistent with other document records. A future revision of the man page could add a dedicated value.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text, consistent with TASK-5, TASK-10, and TASK-14.
- **OQ-3 (recommended default: enabled types only)** — The man page does not state whether disabled document types are searched. This specification excludes disabled `use-case`, `test-case`, and `test-report` types, consistent with their `list` commands and BR-016.
- **OQ-4 (recommended default: fixed `document_types` directories)** — The man page and configuration define a single directory per type. This specification searches those directories, consistent with TASK-78.
- **OQ-5 (recommended default: hide deprecated records)** — BR-018/BR-019/BR-020 state that deprecation hides records from lists, search, and the web UI. This specification hides deprecated records of all three types, consistent with that contract.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05, REQ-05.007, REQ-05.010, REQ-05.012, REQ-05.013, REQ-05.014 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior), BR-016 (configuration keys and precedence), BR-018 (use-case document type behavior), BR-019 (test-case document type behavior), BR-020 (test-report document type behavior) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-80 |
| UI mockups | none |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/search-technical-specifications.md` (TASK-56) |
| Related specification | `docs/specifications/use-cases-test-cases-and-test-reports-management.md` (TASK-78) |
