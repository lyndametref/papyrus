# Feature Specification — Expand Unified Search to Technical Specifications

**Ticket:** TASK-56
**Milestone:** m-11 (Technical Specifications)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Extend `papyrus search` so that technical specifications are searched in addition to business and non-functional requirements, task, milestone, release, decision, glossary, UI specification, UI mockup, business rule, and audit records. Technical specifications are document records: they are stored under the configured `document_types.tech-spec` directory (default `docs/technical/`) and are searched whenever the documented `doc` type is selected (including the default all-types search). The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee established by `docs/specifications/search-requirements.md` and reused by later search increments are reused unchanged. This increment implements REQ-06.001 under REQ-05/REQ-05.009/REQ-05.007, and advances BR-015 and BR-017.

## Scope

### In scope

- Searching active technical specifications using the existing `papyrus search <query> [--type <type>]... [--limit <n>]` shape.
- `--type doc` returns technical specifications together with business-requirement, non-functional-requirement, glossary, UI specification, UI mockup, business rule, audit, and other document records.
- Default search (no `--type`) includes technical specifications together with task, milestone, release, decision, and `doc` records.
- Case-insensitive substring matching, quoted-phrase handling, deterministic ordering by record ID, text output, and `--json` output reused unchanged.
- `--limit <n>` capping the combined result set.
- Deprecation visibility: deprecated technical specifications are never returned; restored specifications appear again.
- Exclusion of disabled `tech-spec` document types (BR-016).
- Project-directory boundary enforcement, input sanitization, and a strict read-only guarantee, reused unchanged.
- Unit and integration tests for the new record type.
- Implementation inside the existing single crate, using existing dependencies only (ADR-0001, ADR-0004).

### Out of scope

- Searching UI mockups, business rules, audit records, deployment records, or any other document type beyond the existing search stores — their dedicated search tasks and TASK-18.
- Adding a new `--type` value for technical specifications (for example `--type tech-spec`); per-document-type filtering through `--type` is not part of the documented vocabulary. Technical specifications are covered by `--type doc`.
- Searching deprecated technical specifications — see Assumptions.
- Browser/web UI access for technical specification search — TASK-57 and browser milestones.
- Packaging and verification of the technical-specifications milestone — TASK-59.
- Configuration changes, new configuration keys, indexes, caches, or persisted search state.
- Fuzzy matching, stemming, token ranking, boolean operators, field-qualified queries, or regular expressions.
- Modifying, moving, creating, deprecating, restoring, or deleting records; `search` is read-only.
- UI mockups; this is a CLI-only increment.

## Behavior

### 1. Command shape, query, and options

1. The command shape is unchanged: `papyrus search <query> [--type <type>]... [--limit <n>]`.
2. The documented type vocabulary is unchanged: `task`, `milestone`, `release`, `decision`, and `doc`. No new type value is added; technical specifications are returned whenever `doc` is selected (including the default all-types search).
3. `--type` validation, repetition, defaulting to all five types, and `--limit` semantics are unchanged from `docs/specifications/search-requirements.md` sections 1 and 4.
4. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for the existing commands.

### 2. Record sources and visibility

1. In this increment, the searchable technical-specification store is the configured `document_types.tech-spec` directory (default `docs/technical/`), resolved as `doc_directory` + `document_types.tech-spec.directory`. It is never hardcoded as `docs/technical/`.
2. Only the active folder of the technical-specification directory is searched. The `deprecated/` subfolder is never scanned or read. Deprecated technical specifications are therefore never returned, consistent with deprecated requirement records and archived backlog/decision records.
3. Only files directly inside the configured active technical-specification directory with the `.md` extension whose file stem matches the configured record pattern (`<prefix>-<digits>`, for example `TECH-001`) are candidates. Subdirectories (including `deprecated/`), non-Markdown files, and files outside the configured directory are ignored. There is no recursive traversal. This matches `tech-spec list` (BR-017), which limits records to `<prefix>-<digits>.md` files at the top level of the technical-specification directory.
4. A technical specification is included only when the `tech-spec` document type is present in the effective configuration and its `enabled` flag is `true` (BR-016). A disabled or absent `tech-spec` type contributes no results and no error.
5. A document type key or enabled flag does not otherwise gate the store beyond the `tech-spec` entry: technical specifications are searched whenever `--type doc` is selected and the `tech-spec` type is enabled, regardless of other `document_types` entries.
6. A missing configured technical-specification directory yields no results for that store and no error. `search` must not create it.
7. An unreadable configured technical-specification directory is an IO error (exit 1). A technical specification file that cannot be read as UTF-8 is skipped rather than aborting the search.
8. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
9. A record ID is the file stem. Its title is the text after the ID on the first heading line (the technical specification title); if that text is empty, the ID is used as the title (consistent with `tech-spec list`).

### 3. Searchable text and matching

1. The searchable text of a technical specification is its entire persisted record text, including front matter, so metadata values are searchable: the top-level heading (ID and title) plus the `## Description` section, as well as any content added through `--editor`.
2. Matching is case-insensitive substring matching over that text, using Unicode-aware lowercasing of both the query and the record text.
3. The query is trimmed of leading and trailing whitespace before matching. Internal whitespace is preserved exactly; a quoted phrase is matched as one contiguous substring, including its internal spaces.
4. There is no tokenization, word splitting, stemming, fuzzy matching, ranking, boolean syntax, field-qualified syntax, or regular-expression interpretation. Characters such as `.`, `*`, `(`, `[`, and `?` are literal.
5. A record is returned at most once, regardless of how many times the query occurs in it.

### 4. Ordering, limit, type value, and output

1. Each technical specification hit reports the documented filter value as its type: `doc` (not `tech-spec`), consistent with all other document records.
2. Results are ordered by record ID in ascending byte-wise lexicographic order. Ties are broken by the documented type value and then by file path, so output is deterministic across runs.
3. In text mode, each result is printed on its own line as `<id> <type> <title>`. Lines are separated by a newline. There is no header, no trailing blank line, and no output at all when there are no matches.
4. With `--json`, the output is a JSON array of objects in the same order, each with `id`, `type`, `title`, and `path`, where `path` is the canonical absolute path of the record file.
5. When there are no matches, `--json` prints `[]`.
6. The limit, when supplied, keeps the first N results after ordering. If fewer records match than the limit, all matches are returned.
7. A successful search exits 0, including when it returns zero matches. `search` never exits 3.

### 5. Boundary, sanitization, and read-only guarantees

1. The project directory is selected and canonicalized as for existing commands, and the configuration path is boundary-checked as for existing commands.
2. The configured technical-specification directory and every candidate record file are validated with the existing within-project boundary check. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
3. The query is normalized before use: leading/trailing whitespace is trimmed; an empty or whitespace-only query, or a query containing NUL, carriage return, or line feed, is a validation error (exit 4). The query is never used as a path, so `/` and `\` are allowed as literal search characters.
4. `--type` values are validated against the fixed documented set by the CLI and are never used as paths.
5. `search` performs no writes: it does not create, modify, move, or delete files or directories, and it writes nothing to stdout/stderr except results and error messages.
6. `search` reads only the configuration file, the configured technical-specification directory, and candidate `.md` files inside the canonical project directory. It performs no network access.

### 6. Empty results

1. A valid query with no matches exits 0, prints nothing in text mode, and prints `[]` in `--json` mode.
2. A valid query restricted to a store with no technical specifications (for example an uninitialized or empty technical-specification directory) behaves as an empty result, not an error.
3. A valid query restricted to `--type doc` on a project with a disabled `tech-spec` type behaves as an empty result for technical specifications, not an error.

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
| T-56-01 | Default search returns active technical specifications together with requirement, task, milestone, release, decision, and other document records | Integration test on a project with one record of each store |
| T-56-02 | `--type doc` returns technical specifications; `--type task`/`milestone`/`release`/`decision` never return technical specifications | Type-filter integration tests |
| T-56-03 | Matching is case-insensitive substring matching over the whole technical specification text | Query in different case returns the same specifications |
| T-56-04 | A quoted phrase is matched as one contiguous substring across technical specifications | Quoted-phrase integration test |
| T-56-05 | Deprecated technical specifications never appear; restored specifications appear again | Deprecate/restore integration test |
| T-56-06 | Disabled `tech-spec` document type excludes its specifications | Config with `document_types.tech-spec.enabled: false` |
| T-56-07 | Technical specifications are returned with type value `doc` and title equal to the specification title | Exact stdout/JSON assertions |
| T-56-08 | Results are ordered by record ID ascending across stores | Multi-record ordering assertion |
| T-56-09 | `--limit` caps the combined result set after ordering | Limit integration test |
| T-56-10 | Missing technical-specification directory yields empty success without creation | Filesystem snapshot before/after |
| T-56-11 | Symlink escape exits 1 and does not expose outside content | Symlink fixture |
| T-56-12 | `search` creates no files or directories | Filesystem snapshot |
| T-56-13 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |

## Assumptions

- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004). `search` is extended inside the existing crate; no new crate, application, technology stack, external interface, or dependency is introduced, so no new ADR or architecture contract is required.
- Assumption: the man page and BR-015 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded as open questions.
- Assumption: deprecated technical specifications are hidden from search, mirroring deprecated requirement records and archived backlog/decision records; a future increment may add an option to include deprecated records.
- Assumption: the searchable text is the whole persisted record text, not only parsed fields, so editor-added body sections are searchable.
- Assumption: text output uses `<id> <type> <title>` with the documented filter value as `<type>` (`doc`), not the document type key or prefix.
- Assumption: record IDs come from file stems, so configured prefixes are honored without parsing or hardcoding `TECH`.
- Assumption: a technical specification file is included only when the `tech-spec` document type is present and enabled, mirroring `tech-spec list` and BR-016; disabled types are never searched.
- Assumption: technical specifications live in the configured `document_types.tech-spec` directory (default `docs/technical/`). Relocating the type directory is a configuration decision, not part of this ticket.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: `search` performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: no dedicated type filter)** — The documented `--type` vocabulary contains no technical-specification value. This specification searches technical specifications under `--type doc`, consistent with other document records. A future revision of the man page could add a dedicated value.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text, consistent with TASK-5, TASK-10, and TASK-14.
- **OQ-3 (recommended default: enabled types only)** — The man page does not state whether disabled document types are searched. This specification excludes disabled `tech-spec` types, consistent with `tech-spec list` and BR-016.
- **OQ-4 (recommended default: fixed `document_types.tech-spec` directory)** — The man page and configuration define a single technical-specification directory. This specification searches that directory, consistent with TASK-55.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05, REQ-05.009, REQ-05.007 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior), BR-017 (tech-spec command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-56 |
| UI mockups | none |
| Domain model | `docs/domain-model/business-glossary/technical-specification.md` |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/search-tasks-milestones-and-releases.md` (TASK-10) |
| Related specification | `docs/specifications/search-decision-records.md` (TASK-14) |
| Related specification | `docs/specifications/search-glossary-terms.md` (TASK-31) |
| Related specification | `docs/specifications/search-ui-specifications.md` (TASK-35) |
| Related specification | `docs/specifications/technical-specifications-management.md` (TASK-55) |