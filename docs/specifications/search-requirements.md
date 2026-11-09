# Feature Specification — Search Business and Non-Functional Requirements

**Ticket:** TASK-5
**Milestone:** m-1 (Search and Browser for Requirements)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the first increment of `papyrus search`: one read-only, project-local free-text search over the records implemented so far — business requirements, non-functional requirements, and their requirement group documents — with the full documented type vocabulary and option shape already in place for later increments. The command contract is REQ-06.001 and BR-015 (`papyrus(1)` → COMMANDS → search); where the man page is silent, this specification fixes explicit, testable defaults (see Assumptions and Open questions).

## Scope

### In scope

- `papyrus search <query> [--type <type>]... [--limit <n>]` exactly as documented in `papyrus(1)`.
- Repeatable `--type` accepting exactly the documented values `task`, `milestone`, `release`, `decision`, and `doc`; default is all types (REQ-06.001).
- Searching active business-requirement and non-functional-requirement records, including requirement group documents, from the configured document directories.
- Accepting the documented-but-not-yet-implemented types `task`, `milestone`, `release`, and `decision` with zero matches and no error.
- Case-insensitive substring matching, quoted-phrase handling, deterministic ordering by record ID, text output, and `--json` output.
- `--limit <n>` capping the result set.
- Deprecation visibility and disabled-document-type exclusion.
- Project-directory boundary enforcement, input sanitization, and a strict read-only guarantee.
- Unit and integration tests for the behavior in this specification.
- Implementation inside the existing single crate, using existing dependencies only (ADR-0004).

### Out of scope

- Searching task, milestone, release, or decision records — TASK-10 and TASK-14.
- Searching glossary terms, UI specifications, business rules, audit records, deployment records, or any other document type — TASK-18.
- Browser/web UI access — TASK-6.
- Packaging and release verification — TASK-7.
- Configuration changes, new configuration keys, indexes, caches, or persisted search state.
- Searching deprecated records, or any option to include them.
- Fuzzy matching, stemming, token ranking, boolean operators, field-qualified queries (`title:`), or regular expressions.
- Modifying, moving, or creating records; `search` is read-only.
- UI mockups; this is a CLI-only increment.

## Behavior

### 1. Command shape, query, and options

1. The command is `papyrus search <query> [--type <type>]... [--limit <n>]`.
2. `<query>` is one required positional argument. Omitting it is a usage error (exit 2). An unquoted multi-word query reaches the CLI as several positionals and is a usage error (exit 2); a phrase containing spaces must be quoted by the shell and then arrives as one argument.
3. A query beginning with `-` must be separated with `--` (for example `papyrus search -- -draft`); otherwise it is parsed as an option and is a usage error (exit 2).
4. `--type` is repeatable and takes one value per occurrence. Accepted values are exactly `task`, `milestone`, `release`, `decision`, and `doc`, matched case-sensitively. Any other value, an uppercase variant, or a comma-joined value such as `task,doc` is a usage error (exit 2).
5. Repeating the same `--type` value is allowed and has no additional effect. When `--type` is omitted, all five documented types are selected.
6. `--limit` takes a positive integer (`>= 1`). A non-integer, `0`, or a negative value is a usage error (exit 2). There is no documented upper bound.
7. The limit is applied after type filtering, matching, and ordering; it keeps the first N results in the final order.
8. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for the existing commands.

### 2. Record sources and visibility

1. In this increment, the `doc` type covers:
   - business-requirement records (`document_types.req`), and
   - non-functional-requirement records (`document_types.nfr`).
2. Requirement group documents (`REQ-<n>` / `NFR-<n>` files) are records in their own right and are searchable alongside their member requirements.
3. `task`, `milestone`, `release`, and `decision` are accepted values but have no record store in this increment. They produce no matches and no error, alone or combined with `doc`.
4. Other document type keys — `glossary`, `rule`, `ui-spec`, `audit`, `deploy`, and any user-added key — are not searched in this increment, even if files exist in their configured directories. `--type doc` covers only the `req` and `nfr` keys until TASK-18.
5. A document type whose key is absent from the effective configuration, or whose `enabled` flag is `false`, contributes no results and no error. If both `req` and `nfr` are disabled, `doc` and default searches return no matches with exit 0.
6. Only active records are searched. The scanner reads files directly in the configured active directory and never scans or reads the `deprecated/` subfolder. Deprecated records therefore never appear in results.
7. Deprecating a group does not hide its active requirements (deprecation does not cascade); those requirements remain searchable while active.
8. Each type directory is `doc_directory` + `document_types.<key>.directory`, resolved from the effective configuration. The implementation must not hardcode `docs/` or the `REQ`/`NFR` prefixes: configured directories and prefixes are honored, and a record ID is taken from its file stem, so renamed prefixes appear correctly.
9. Only files directly inside the configured active directory with the `.md` extension are candidates. Subdirectories, non-Markdown files (for example `index.html`), and files outside the configured directory are ignored. There is no recursive traversal.
10. A record ID is the file stem. Its title is the text after the ID on the first heading line; if that text is empty, the ID is used as the title (consistent with `req list` / `nfreq list`).
11. A missing configured type directory yields no results for that type and no error. `search` must not create it.
12. A record file that cannot be read as UTF-8 is skipped rather than aborting the search (consistent with the tolerant read in `req list`). An unreadable configured type directory is an IO error (exit 1).

### 3. Searchable text and matching

1. The searchable text of a record is its entire persisted record text, including front matter, so metadata values are searchable: the top-level heading (ID and title) plus every body section, including `## User Story`, `## Description`, and `## Group`, as well as any sections added through `--editor`.
2. Matching is case-insensitive substring matching over that text, using Unicode-aware lowercasing of both the query and the record text.
3. The query is trimmed of leading and trailing whitespace before matching. Internal whitespace is preserved exactly; a quoted phrase is matched as one contiguous substring, including its internal spaces.
4. There is no tokenization, word splitting, stemming, fuzzy matching, ranking, boolean syntax, field-qualified syntax, or regular-expression interpretation. Characters such as `.`, `*`, `(`, `[`, and `?` are literal.
5. A record is returned at most once, regardless of how many times the query occurs in it.

### 4. Ordering, limit, and output

1. Results are ordered by record ID in ascending byte-wise lexicographic order. Ties (not expected, because record IDs are unique within a store) are broken by the documented type value and then by file path, so output is deterministic across runs.
2. In text mode, each result is printed on its own line as `<id> <type> <title>`, where `<type>` is the documented filter value (`doc` for every result in this increment). Lines are separated by a newline. There is no header, no trailing blank line, and no output at all when there are no matches.
3. With `--json`, the output is a JSON array of objects in the same order, each with:
   - `id` (string): the record ID,
   - `type` (string): the documented filter value (`doc` in this increment),
   - `title` (string): the record title,
   - `path` (string): the canonical absolute path of the record file, consistent with `req view` JSON.
4. When there are no matches, `--json` prints `[]`.
5. The limit, when supplied, keeps the first N results after ordering. If fewer records match than the limit, all matches are returned.
6. A successful search exits 0, including when it returns zero matches. `search` never exits 3 (not found), because a query is not an identifier lookup.

### 5. Boundary, sanitization, and read-only guarantees

1. The project directory is selected and canonicalized as for existing commands, and the configuration path is boundary-checked as for existing commands.
2. Every configured type directory and every candidate record file is validated with the existing within-project boundary check. A symbolic link that resolves outside the project directory is a boundary violation (exit 1), and the outside content is never read.
3. The query is normalized before use: leading/trailing whitespace is trimmed; an empty or whitespace-only query, or a query containing NUL, carriage return, or line feed, is a validation error (exit 4). The query is never used as a path, so `/` and `\` are allowed as literal search characters.
4. `--type` values are validated against the fixed documented set by the CLI and are never used as paths.
5. `search` performs no writes: it does not create, modify, move, or delete files or directories, and it writes nothing to stdout/stderr except results and error messages.
6. `search` reads only the configuration file, the configured active type directories, and candidate `.md` files inside the canonical project directory. It performs no network access.

### 6. Empty results

1. A valid query with no matches exits 0, prints nothing in text mode, and prints `[]` in `--json` mode.
2. A valid query restricted to types that have no records in this increment (for example `--type task`) behaves as an empty result, not an error.

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
| T-01 | Command shape and documented type vocabulary appear in help and parse as specified | `papyrus search --help`; parse tests for each valid and invalid type |
| T-02 | Default search returns active business and non-functional requirement records, including group documents | Integration test on a project with a group plus a requirement of each type |
| T-03 | Matching is case-insensitive substring matching | Query in different case returns the same records |
| T-04 | A quoted phrase is matched as one contiguous substring | Quoted phrase matches; separated words do not match that phrase |
| T-05 | ID, title, user story, description, and other body text are searchable | One query per field, including an editor-added section |
| T-06 | `--type doc` restricts to requirement documents; `--type task`/`milestone`/`release`/`decision` return empty success | Type-filter integration tests, including combinations |
| T-07 | `--limit` caps results after ordering; a larger limit returns all matches | Limit tests with fewer, equal, and more matches than the limit |
| T-08 | Results are ordered by record ID ascending | Multi-record ordering assertion |
| T-09 | Deprecated records never appear; restoring one makes it appear again | Deprecate/restore integration test |
| T-10 | Disabled document types are excluded | Config with `document_types.req.enabled: false` |
| T-11 | Configured `doc_directory`, type directories, and prefixes are honored | Project with a custom document directory and prefix |
| T-12 | Text output is `<id> <type> <title>` per line with no output on no matches | Exact stdout assertions |
| T-13 | JSON output is the ordered `{id,type,title,path}` array, `[]` when empty | JSON parsing assertions |
| T-14 | Missing query, invalid type, and invalid limit exit 2 | Exit-code assertions |
| T-15 | Empty/whitespace/control-character query exits 4 | Exit-code assertions |
| T-16 | Symlink escape exits 1 and does not expose outside content | Symlink fixture pointing outside the project |
| T-17 | `search` creates no files or directories | Filesystem snapshot before/after a search on an uninitialized type directory |
| T-18 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |

## Assumptions

- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004). `search` is added as a module/subcommand inside the existing crate; no new crate, application, technology stack, external interface, or dependency is introduced, so no new ADR or architecture contract is required.
- Assumption: the man page and BR-015 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded in Open questions with recommended defaults.
- Assumption: requirement group documents count as searchable records; TASK-5 covers both group documents and member requirements.
- Assumption: only the `req` and `nfr` document types are searched in this increment; other document types wait for TASK-18.
- Assumption: the searchable text is the whole persisted record text, not only the parsed ID/title/user-story/description fields, so editor-added body sections are searchable.
- Assumption: text output uses `<id> <type> <title>` with the documented filter value as `<type>`; the man page does not prescribe a result format.
- Assumption: `--limit 0` is an invalid value rather than "unlimited"; the man page says "maximum number of results" and does not define a zero convention.
- Assumption: record IDs come from file stems, so configured prefixes are honored without parsing or hardcoding `REQ`/`NFR`.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: `search` performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only; the browser increment is TASK-6.

## Open questions

None of these block implementation: each has a specified default in this document. User confirmation is recommended for OQ-1 before release.

- **OQ-1 (recommended default: `<id> <type> <title>`)** — The man page does not prescribe the text result format. This specification fixes one line per result as `<id> <type> <title>` and JSON objects with `id`, `type`, `title`, and `path`. A different display format would be a presentation change only.
- **OQ-2 (recommended default: only `req` and `nfr`)** — The man page's `doc` type is broader than the document types implemented so far. This increment searches only `req` and `nfr`; `glossary`, `rule`, `ui-spec`, `audit`, and `deploy` files are ignored until TASK-18.
- **OQ-3 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text (ID, title, and all body sections) rather than only the parsed user-story/description fields.
- **OQ-4 (recommended default: error)** — `--limit 0` is treated as an invalid value (exit 2), not as "no limit". No zero convention is documented.
- **OQ-5 (recommended default: case-sensitive)** — `--type` values are accepted only in the documented lowercase spelling; uppercase variants are usage errors. This keeps the accepted vocabulary identical to the man page.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05.001, REQ-05.002, REQ-05.007 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-5 |
| UI mockups | none |
| Related specification | `docs/specifications/business-and-non-functional-requirements-management.md` (TASK-3) |
| Related specification | `docs/specifications/requirements-management-milestone-packaging-and-verification.md` (TASK-4) |
