# Feature Specification — Expand Unified Search to Decision Records

**Ticket:** TASK-14
**Milestone:** m-3 (Decision Records)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Extend `papyrus search` so that decision records are searched in addition to task, milestone, release, and `doc` (business and non-functional requirement) records. The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee established by `docs/specifications/search-requirements.md` and reused by `docs/specifications/search-tasks-milestones-and-releases.md` are reused unchanged; this increment adds the decision record store to the set of searchable types. It implements REQ-06.001 under REQ-04/REQ-04.001/REQ-04.002, and advances BR-015 and BR-008.

## Scope

### In scope

- Searching active decision records using the existing `papyrus search <query> [--type <type>]... [--limit <n>]` shape.
- `--type decision` now returns matching decision records instead of empty success.
- Default search (no `--type`) includes decision records together with `task`, `milestone`, `release`, and `doc` records.
- Case-insensitive substring matching, quoted-phrase handling, deterministic ordering by record ID, text output, and `--json` output reused from the requirements and backlog increments.
- `--limit <n>` capping the combined result set.
- Exclusion of disabled decision types (BR-016) and archived decision records.
- Project-directory boundary enforcement, input sanitization, and a strict read-only guarantee, reused unchanged.
- Unit and integration tests for the new record type.
- Implementation inside the existing single crate, using existing dependencies only (ADR-0001, ADR-0004).

### Out of scope

- Searching glossary terms, UI specifications, business rules, audit records, deployment records, technical specifications, or any other document type — TASK-18.
- Searching archived decision records — see Assumptions and Open questions.
- Browser/web UI access for decisions — TASK-15.
- Packaging and release verification of the decision-record milestone — TASK-16.
- Configuration changes, new configuration keys, indexes, caches, or persisted search state.
- Per-decision-type filtering through `--type` (for example `--type adr`); `--type` accepts only the documented values, and `decision` covers all enabled decision types.
- Fuzzy matching, stemming, token ranking, boolean operators, field-qualified queries, or regular expressions.
- Modifying, moving, creating, archiving, restoring, or deleting records; `search` is read-only.
- UI mockups; this is a CLI-only increment.

## Behavior

### 1. Command shape, query, and options

1. The command shape is unchanged: `papyrus search <query> [--type <type>]... [--limit <n>]`.
2. The documented type vocabulary is unchanged: `task`, `milestone`, `release`, `decision`, and `doc`. `decision` is now implemented and returns matching decision records.
3. `--type` validation, repetition, defaulting to all five types, and `--limit` semantics are unchanged from `docs/specifications/search-requirements.md` sections 1 and 4.
4. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for the existing commands.

### 2. Record sources and visibility

1. In this increment, the searchable decision store is the configured `decision_directory` (default `decisions/`), which is outside the backlog store. It is resolved from the effective configuration (`decision_directory`), joined with the project directory; it is never hardcoded as `decisions/`.
2. Only the active folder of the decision directory is searched. The `archived/` subfolder is never scanned or read. Archived decision records are therefore never returned, consistent with archived task and milestone records and deprecated requirement records.
3. Only files directly inside the configured active decision directory with the `.md` extension are candidates. Subdirectories (including `archived/`), non-Markdown files, and files outside the configured directory are ignored. There is no recursive traversal.
4. A decision record is included only when its ID maps to an enabled decision type in the effective `decision_types` configuration (using the configured prefix). Decisions of disabled types are never returned, consistent with `decision list` and BR-016. A decision file whose ID does not map to any configured (enabled) type is ignored.
5. A document type key or enabled flag does not gate the decision store: decision records are searched whenever `--type decision` is selected, regardless of `document_types` configuration. There is no `enabled` flag for the decision store itself beyond the per-type `decision_types.<key>.enabled` flags.
6. A missing configured decision directory yields no results for that store and no error. `search` must not create it.
7. An unreadable configured decision directory is an IO error (exit 1). A decision record file that cannot be read as UTF-8 is skipped rather than aborting the search.
8. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
9. A record ID is the file stem. Its title is the text after the ID on the first heading line; if that text is empty, the ID is used as the title (consistent with `decision list` and with requirement records).

### 3. Searchable text and matching

1. The searchable text of a decision record is its entire persisted file text: the top-level heading (ID and title) plus every body section, including `## Status`, `## Decision`, `## Context`, and `## Consequences`, as well as any editor-added sections.
2. Matching is case-insensitive substring matching over that text, using Unicode-aware lowercasing of both the query and the record text.
3. The query is trimmed of leading and trailing whitespace before matching. Internal whitespace is preserved exactly; a quoted phrase is matched as one contiguous substring, including its internal spaces.
4. There is no tokenization, word splitting, stemming, fuzzy matching, ranking, boolean syntax, field-qualified syntax, or regular-expression interpretation. Characters such as `.`, `*`, `(`, `[`, and `?` are literal.
5. A record is returned at most once, regardless of how many times the query occurs in it.

### 4. Ordering, limit, type value, and output

1. Each decision hit reports the documented filter value as its type: `decision`.
2. Results are ordered by record ID in ascending byte-wise lexicographic order. Ties are broken by the documented type value and then by file path, so output is deterministic across runs.
3. In text mode, each result is printed on its own line as `<id> <type> <title>`. Lines are separated by a newline. There is no header, no trailing blank line, and no output at all when there are no matches.
4. With `--json`, the output is a JSON array of objects in the same order, each with `id`, `type`, `title`, and `path`, where `path` is the canonical absolute path of the record file.
5. When there are no matches, `--json` prints `[]`.
6. The limit, when supplied, keeps the first N results after ordering. If fewer records match than the limit, all matches are returned.
7. A successful search exits 0, including when it returns zero matches. `search` never exits 3.

### 5. Boundary, sanitization, and read-only guarantees

1. The project directory is selected and canonicalized as for existing commands, and the configuration path is boundary-checked as for existing commands.
2. The configured decision directory and every candidate record file is validated with the existing within-project boundary check. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
3. The query is normalized before use: leading/trailing whitespace is trimmed; an empty or whitespace-only query, or a query containing NUL, carriage return, or line feed, is a validation error (exit 4). The query is never used as a path, so `/` and `\` are allowed as literal search characters.
4. `--type` values are validated against the fixed documented set by the CLI and are never used as paths.
5. `search` performs no writes: it does not create, modify, move, or delete files or directories, and it writes nothing to stdout/stderr except results and error messages.
6. `search` reads only the configuration file, the configured decision directory, and candidate `.md` files inside the canonical project directory. It performs no network access.

### 6. Empty results

1. A valid query with no matches exits 0, prints nothing in text mode, and prints `[]` in `--json` mode.
2. A valid query restricted to a store with no records (for example an uninitialized decision directory) behaves as an empty result, not an error.
3. A valid query restricted to `--type decision` on a project with no enabled decision types behaves as an empty result, not an error.

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
| T-14-01 | Default search returns active decision records together with task, milestone, release, and requirement records | Integration test on a project with one record of each store |
| T-14-02 | `--type decision` returns only decision records | Type-filter integration test |
| T-14-03 | Matching is case-insensitive substring matching over the whole decision record text | Query in different case returns the same records |
| T-14-04 | A quoted phrase is matched as one contiguous substring across decision records | Quoted-phrase integration test |
| T-14-05 | Archived decision records never appear | Archive then search integration test |
| T-14-06 | Disabled decision types are excluded | Config with a disabled decision type; its records never appear |
| T-14-07 | Decision records are searched in full (status, decision, context, consequences) | Content search integration test |
| T-14-08 | Results are ordered by record ID ascending across stores | Multi-record ordering assertion |
| T-14-09 | `--limit` caps the combined result set after ordering | Limit integration test |
| T-14-10 | Text output is `<id> <type> <title>` per line; JSON output is the ordered `{id,type,title,path}` array | Exact stdout/JSON assertions |
| T-14-11 | Missing decision directory yields empty success without creation | Filesystem snapshot before/after |
| T-14-12 | Symlink escape exits 1 and does not expose outside content | Symlink fixture |
| T-14-13 | `search` creates no files or directories | Filesystem snapshot |
| T-14-14 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |

## Assumptions

- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004). `search` is extended inside the existing crate; no new crate, application, technology stack, external interface, or dependency is introduced, so no new ADR or architecture contract is required.
- Assumption: the man page and BR-015 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded as open questions.
- Assumption: archived decision records are hidden from search, mirroring archived task and milestone records and deprecated requirement records; a future increment may add an option to include archived records.
- Assumption: the searchable text is the whole persisted record text, not only parsed fields, so editor-added body sections are searchable.
- Assumption: text output uses `<id> <type> <title>` with the documented filter value as `<type>` (`decision`), not the decision type key or prefix.
- Assumption: record IDs come from file stems, so configured prefixes are honored without parsing or hardcoding `ADR`/`BDR`.
- Assumption: a decision file is included only when its ID maps to an enabled decision type, mirroring `decision list` and BR-016; disabled types are never searched.
- Assumption: decision records live in the configured `decision_directory` (default `decisions/`), outside the backlog store, and are not gated by `document_types` or `backlog_directory`.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: `search` performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: active records only)** — Whether archived decision records should ever be searchable is not documented. This specification hides them, consistent with archived task and milestone records and deprecated requirement records. A future option could include them.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text, consistent with TASK-5 and TASK-10.
- **OQ-3 (recommended default: enabled types only)** — The man page does not state whether disabled decision types are searched. This specification excludes them, consistent with `decision list` and BR-016.
- **OQ-4 (recommended default: fixed `decision_directory`)** — The man page and configuration define a single `decision_directory`, not per-type search directories. This specification searches that directory, consistent with TASK-13.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-04, REQ-04.001, REQ-04.002 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior), BR-008 (decision command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-14 |
| UI mockups | none |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/search-tasks-milestones-and-releases.md` (TASK-10) |
| Related specification | `docs/specifications/decision-records-management.md` (TASK-13) |
