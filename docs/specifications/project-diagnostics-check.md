# Feature Specification — Project Diagnostics (`papyrus check`)

**Requirement:** REQ-01.008 (Run Project Diagnostics)
**Architecture:** ADR-0029 (Read-Only Corpus Diagnostics with Duplicate-ID Repair), decisions 1-9
**Milestone:** m-21
**UI mockups:** none (CLI only)
**Status:** In progress — read-only run implemented (TASK-137); duplicate-ID repair (`--fix`, `--force`) implemented (TASK-138); `check` reserved and the §1.1 command surface published (TASK-139)

This specification fixes the diagnostic contract: the command surface, the two-stage
configuration load, the scanned locations, the seven finding kinds, the finding model,
the output shapes, and the exit codes (sections 1-9), plus duplicate-ID repair through
`--fix` and `--force` (section 10). Rationale and alternatives live in ADR-0029; this
document is the implementation contract.

## 1. Command surface

- `papyrus check` is a built-in command with no subcommands. It takes two flags of its
  own, `--fix` and `--force` (section 10.1); `--json`, `--dir`, `--config`, and
  `--no-color` apply as global options (REQ-01.004).
- `check` is exempt from fail-closed startup (ADR-0019). `lib.rs::run` builds the
  command tree with no registry when the first positional is `check`, so
  `papyrus check --help` works on a project whose configuration is invalid
  (ADR-0029 decision 2).
- `check` is CLI-only in this release; no browser route runs it.

### 1.1 Published help text

`papyrus check --help` exits 0 and prints the following long description verbatim,
then a blank line, then clap's generated synopsis `Usage: papyrus check [OPTIONS]`
and the option list. The description is the clap `long_about` of the `check`
subcommand; the golden `tests/golden/cli-help/check.json` locks the rendered bytes.

```text
Run read-only project diagnostics.

Nothing is written unless --fix is given. Findings are reported one per line as
<kind>: <family> <subject> — <detail> (<locations>), sorted by kind, family,
subject, and location. The finding kinds are duplicate-id, self-dependency,
dependency-cycle, dangling-milestone, dangling-release, frontmatter-conflict,
and config-invalid. --json emits the same report as one object with the keys
findings, skipped, and repaired.

--fix repairs duplicate record IDs only. It prints the planned reassignments and
every reference they affect, then asks for confirmation; --force skips that
prompt and requires --fix. A planned repair that cannot be applied is reported
as repair-skipped. With repairs to apply, no terminal, and no --force, nothing
is written and the run exits 2.

The configuration is loaded tolerantly, like papyrus config get|set|unset|list:
an invalid configuration is reported as one config-invalid finding per
diagnostic line, the checks that need a valid document_types set are named as
skipped, and the remaining checks still run.

Exit status: 0 when nothing was found; 4 when at least one finding was reported,
including a run whose repairs succeeded; 1 on an I/O failure; 2 on a usage error.
```

The short `about` stays `Run read-only project diagnostics`, which is the line
`papyrus --help` lists; the long description never leaks into that list. No
`override_usage` is added, and the text carries no version, path, or
environment-specific detail.

## 2. Configuration load (two stages)

`check` never inherits the fail-closed configuration load. It loads its own.

**Stage A — tolerant self-load.** Parse the effective configuration through
`commands::load_effective_config_tolerant`, the loader used by
`config get|set|unset|list` (REQ-01.002, REQ-07.002). It merges the defaults with the
file and deserializes, without `Config::validate`.

- A load error (missing configuration file, malformed YAML, duplicate key, schema
  mismatch, invalid browser key) becomes exactly one `config-invalid` finding. No
  configuration value exists in that case, so every corpus check is skipped.
- On a successful parse, `Config::validate(Some(project_dir))` runs. Its result is a
  single string of newline-separated diagnostics; each non-empty line becomes one
  `config-invalid` finding.
- The parsed configuration stays usable for the checks that do not need a valid
  `document_types` set, even when validation failed.

**Stage B — corpus scan.** Two independent sub-stages:

- **B1, static families** (`task`, `milestone`, `release`, `decision`): runs whenever
  Stage A produced a parsed configuration, valid or not. These descriptors and their
  prefixes/directories do not come from `document_types`.
- **B2, document families** (every configured `document_types` entry, enabled and
  disabled): runs only when Stage A validation passed *and* `ArtifactRegistry::build`
  succeeded. A registry build failure adds one `config-invalid` finding
  (`detail` = the registry error message) and skips B2.

A family whose configured directory cannot be resolved inside the project is treated
as already reported by Stage A: it contributes no files. It is not a second finding
and does not abort the run.

## 3. Scanned locations and the read-only guarantee

- Enumeration uses the read-only primitive
  `record::scan::family_files(ctx, cfg, descriptor) -> Result<Vec<PathBuf>, AppError>`,
  which lists the descriptor's `resolve_read_directories` set: the active directory,
  the lifecycle archive directory, the read/import archive directory, and the
  read-only legacy alias directory. A missing directory yields no paths.
- `family_files` never relocates, migrates, or adopts a record and never writes.
  `check` must not call `record::resolve::list_in_dir`, `store::relocate_if_archived`,
  `store::archive`, `store::set_status`, `store::delete`, `record::migrate`, or any
  `fs::write`.
- A file is a candidate only when its extension matches the descriptor: `html` for a
  `ContentModel::HtmlMockup` descriptor (`ui-mockup`), `md` for every other
  descriptor. HTML mockup records contribute their stem to duplicate-ID detection
  only; they carry no front matter, so no other scanner reads them.
- A path with a `deprecated` component is never listed or read, even if a configured
  directory points at one. Files under `deprecated/` are not candidates for any
  finding and produce no report entry (REQ-05.007).
- Reads go through `record::frontmatter` and `record::document` only: one parser, one
  projection, no second metadata model (ADR-0013, ADR-0018).

## 4. Finding model

One finding per detected problem:

| Field | Type | Meaning |
|---|---|---|
| `kind` | `&'static str` | Stable kind value from section 5. |
| `family` | `Option<String>` | `task`, `milestone`, `release`, `decision`, or a document-type slug. `None` for `config-invalid`. |
| `subject` | `Option<String>` | Record ID for record-scoped kinds. `None` for `config-invalid` and for findings that are not attributable to a single record. |
| `locations` | `Vec<String>` | Project-relative paths with `/` separators, sorted and deduplicated. `["papyrus.config.yml"]` or the configured configuration path for `config-invalid`. |
| `detail` | `String` | Human-readable description, fixed per kind in section 5. |
| `fixable` | `bool` | `false` for every kind except `duplicate-id` (section 5.1). |

## 5. Detection rules

Identifiers are canonicalised with `record::identity::parse_id_parts` against the
descriptor's configured prefix: case-insensitive prefix, optional leading zeros, and a
tolerated cosmetic `-<slug>` suffix. The canonical key is `ParsedId::key`; the display
form is `ParsedId::display`.

A record's canonical identity is the front-matter `id` when it parses to the family
shape, otherwise the file stem. A stem that does not parse for the family is not a
member of that family and is ignored.

### 5.1 `duplicate-id`

- Compare canonical keys per family across the family's active, archived, and
  read-only legacy directories.
- Grouped families (`req`, `nfr`) compare group records (`REQ-001`) and member
  records (`REQ-01.008`) in separate spaces: a key with one number and a key with two
  numbers never collide.
- A key seen in two or more distinct paths is one finding: `family` = family name,
  `subject` = display ID of the first path in sorted order, `locations` = all paths,
  `detail` = `"<n> records share this ID"`.
- `fixable` is `true` only when every one of these holds, and `false` otherwise:
  no member has a front-matter `id` that disagrees with its stem; no member lives in
  a read-only legacy or read/import directory; and the family is not a grouped
  family reported at the group level. Nothing is repaired unless `--fix` is given
  (section 10).

### 5.2 `self-dependency`

Task whose `dependencies` contain its own canonical key. `subject` = task ID,
`locations` = `[task path]`, `detail` = `"dependency list contains <ID>"`.

### 5.3 `dependency-cycle`

- Graph nodes are canonical task keys observed in the scanned task locations; an edge
  `A -> B` exists when task `A`'s `dependencies` resolve to a scanned task `B`
  (self-edges excluded, dangling edges ignored).
- Detection is an iterative depth-first search with a colour map (white/grey/black),
  no recursion and no crate. Encountering a grey neighbour reports the cycle consisting
  of the grey node through the current node; identical member sets are reported once.
- One finding per cycle: `subject` = the canonically first member, `locations` = the
  member paths sorted, `detail` = members sorted by canonical key and joined with
  `" -> "`.

### 5.4 `dangling-milestone`

Task whose non-empty `milestone` value does not resolve to a scanned milestone.
`subject` = task ID, `locations` = `[task path]`, `detail` =
`"milestone <value> does not resolve to a milestone"`.

### 5.5 `dangling-release`

Milestone whose non-empty `release` value does not resolve to a scanned release.
`subject` = milestone ID, `locations` = `[milestone path]`, `detail` =
`"release <value> does not resolve to a release"`.

### 5.6 `frontmatter-conflict`

Only a record that carries front matter is checked. Conflict means "front matter and
body both carry the value and disagree".

- **Retired task sections:** `## Status` (`status`), `## Assignee` (`assignee`),
  `## Milestone` (`milestone`), `## Labels` (`labels`), `## Dependencies`
  (`dependencies`). `## Release` (`release`) for milestones. No other type has retired
  sections (ADR-0012).
- Sections are read exactly as the legacy migration reads them: a scalar section
  (`status`, `milestone`, `release`) is one non-empty line; a list section
  (`assignee`, `labels`, `dependencies`) is one item per line with an optional `- `
  prefix. List values are compared as sets of entries.
- A conflict is reported when the front matter carries the field and the section value
  differs. A retained section that agrees with the front matter is not reported. A
  retained section whose field the front matter does not carry is not reported (the
  migration would adopt it).
- A section that cannot be reduced to a single value is reported regardless of the
  front matter, because agreement cannot be established: more than one non-empty line
  for a scalar field, an empty list item, or more than one occurrence of the same
  retired heading.
- **Body heading:** for a document record (a configured `document_types` entry) whose
  body starts with `# <ID> <title>`, compare the heading ID and title with the
  front-matter `id` and `title`. A disagreeing ID or title is one finding naming the
  field and both values. A missing heading is not a conflict. Tasks and milestones
  keep no `# <ID> <title>` heading, so the check does not apply to them
  (REQ-05.010).
- **Decisions are excluded entirely** from this kind. `## Status`, `## Date`, and
  `## References` are canonical decision sections, and a frozen ADR body keeps its own
  heading, so neither a retired-section nor a heading comparison can be expected to
  agree (REQ-01.008, REQ-05.010).
- `subject` = record ID, `locations` = `[record path]`, `detail` names the field and
  both values, for example `"section ## Status \"Done\" disagrees with front matter
  status \"In Progress\""` or `"heading title \"A\" disagrees with front matter title
  \"B\""`.

### 5.7 `config-invalid`

One finding per Stage A load error, per `Config::validate` diagnostic line, and per
registry build failure. `family` and `subject` are `None`; `locations` is the
project-relative configuration path; `detail` is the diagnostic text verbatim (each
diagnostic already names its key path).

## 6. Skipped checks

A skipped check is named in both output forms:

| Situation | Skipped |
|---|---|
| Stage A load failed (no configuration value) | every corpus check |
| Configuration parsed but invalid, or registry build failed | `duplicate-id` for document types and `frontmatter-conflict` for document records |

Stable identifiers: `duplicate-id`, `duplicate-id:document-types`,
`self-dependency`, `dependency-cycle`, `dangling-milestone`, `dangling-release`,
`frontmatter-conflict`, `frontmatter-conflict:document-records`. A repair run adds the
finding kind `repair-skipped` (section 10.9), which is an addition permitted by
ADR-0029 constraint 5. Additions are allowed; renames are not.

## 7. Output

### 7.1 Ordering

Findings are sorted by `(kind, family, subject, locations)` with absent options
sorting first, each component compared as a string, and compared stably so repeated
runs over an unchanged corpus produce identical output.

### 7.2 Text

One line per finding:

```
<kind>: <family> <subject> — <detail> (<locations>)
```

`<family> ` is omitted when `family` is `None`; `<subject> ` is omitted when `subject`
is `None`; `<locations>` is the sorted paths joined with `", "`. A clean run prints
exactly `no findings`. When any check was skipped, one final line follows:
`skipped: <identifiers joined with ", ">`. A repair run inserts its `repaired:` and
`reference:` lines between the finding lines and that `skipped:` line (section 10.10).

### 7.3 JSON

`--json` emits the report as the top-level JSON object (never the `{"error":…}`
envelope):

```json
{
  "findings": [
    { "kind": "duplicate-id", "family": "task", "subject": "TASK-003",
      "locations": ["backlog/tasks/task-3.md", "backlog/tasks/task-3-copy.md"],
      "detail": "2 records share this ID", "fixable": false }
  ],
  "skipped": ["duplicate-id:document-types"],
  "repaired": []
}
```

`findings`, `skipped`, and `repaired` are always present; `findings` and `skipped` are
empty arrays on a clean run; `repaired` is always `[]` for the read-only run, for a run
with nothing to repair, for a cancelled run, and for a skipped repair. A repair run
fills it with the entries of section 10.10.

## 8. Exit codes

| Code | Meaning |
|---|---|
| 0 | The run completed and found nothing. |
| 4 | The run completed and found at least one finding (the validation/conflict code of REQ-01.004, reused for findings), including a run in which `--fix` repaired every repairable finding and a run whose repair was cancelled (section 10.11). |
| 1 | I/O failure (a directory could not be listed, the project directory is unreadable). |
| 2 | Usage error: a clap parse error, `--force` without `--fix`, or the non-terminal `--fix` refusal of section 10.8. |

`CommandOutput` gains `exit_code: i32` (default 0); `main` prints the output and exits
with that code. The `AppError` mapping is unchanged, so error paths keep their codes
and their `{"error":…}` JSON envelope.

## 9. Verification

- One characterization fixture per finding kind, written by the real writer, proves
  the scanners agree with the writer.
- A read-only test asserts every scanned file keeps its bytes and modification time.
- Goldens pin the text report, the `--json` object, the deterministic order, and the
  clean-run output.
- Exit-code tests cover 0 and 4, plus 1 for an I/O failure and 2 for usage.
- Repair verification is listed in section 10.12.

## 10. Duplicate-ID repair (`--fix`, `--force`)

Repair is the only writing path of the command. It changes identity and nothing else,
it is driven by the findings of the scan that the same run produced, and it never
rewrites a reference (ADR-0029 decisions 7-9).

### 10.1 Flags

- `BuiltinCmd::Check { fix: bool, force: bool }`; `check` keeps no subcommands. Both
  flags are command-local, never global (ADR-0029 decision 1).
- `--force` is declared with clap `requires = "fix"`, so `--force` without `--fix` is
  a clap usage error: nothing is scanned, nothing is written, and the process exits 2
  after clap prints its own message naming `--fix`. The command does not hand-roll
  this check, and no test pins clap's exact wording.
- `--json` applies unchanged to the report; it does not change what is repaired.
- `--fix` never repairs a finding of any kind other than `duplicate-id`
  (sections 5.2-5.7 stay reported and unresolved).

### 10.2 What enters the plan

- Only `duplicate-id` findings with `fixable: true` (section 5.1) are planned.
- The plan is built from the findings of the current scan; there is no second
  enumeration of the corpus.
- With an invalid configuration, static-family duplicates remain repairable and
  document-type duplicates are absent because section 6 skipped that check. A repair
  never needs a valid registry.
- A run with no repairable finding performs no write, prints no plan, reads no input,
  and reports exactly like the read-only run with the exit code of section 8.
- Every reassignment in a group is either planned or the whole group is refused:
  `fixable` is evaluated per group in section 5.1, so no partial group is possible.

### 10.3 Keeper rule

Within one duplicate group:

- A member's `created_date` is its front-matter `created_date` scalar, trimmed.
- It is **valid** when it matches `YYYY-MM-DDTHH:MM:SS` followed by `Z` or `±HH:MM`.
  Valid values compare as UTC seconds, so offsets and equal instants compare correctly.
  This needs
  `pub fn timestamp_seconds(value: &str) -> Option<i64>` in `src/record/frontmatter.rs`,
  the inverse of the existing `utc_from_unix`; no new crate.
- Members whose `created_date` is missing or not valid sort after every valid member.
- Remaining ties break on the project-relative path, ascending, compared as strings.
  Two records cannot share a path, so the order is total.
- The first member keeps the ID. Every other member is reassigned in that same order,
  which fixes the order of the plan, the writes, and the report.
- The keeper rule never consults directory iteration order, file stem casing, or the
  scan order.

### 10.4 Reassignment rule

- The new number is `record::identity::allocate_next` over the observed numbers of the
  record's own family, prefix, and shape, taken from the scan's `RecordMeta` values
  plus every number allocated earlier in the same run. No fresh directory walk is made.
  - **Plain record** (one number, e.g. `TASK-003`, `MS-002`, `REL-001`, `ADR-0029`, a
    plain document type): observed numbers are `numbers[0]` of every scanned record in
    the same family whose configured prefix matches case-insensitively; the new ID is
    `identity::format_id(prefix, n, cfg.id_padding)`.
  - **Member of a dotted or grouped family** (`REQ-01.008`, a dotted document member):
    observed numbers are `numbers[1]` of every scanned record in the same family, with
    the same configured prefix and the same `numbers[0]`; the group component is
    `identity::format_id(prefix, numbers[0], cfg.id_padding)`; the new ID is
    `identity::format_seq_id(group, n, cfg.id_padding)`.
- Prefix and padding come from the configuration (ADR-0029 decision 7), never from the
  file stem or the record's existing display form.
- The new stem is exactly the new ID: a cosmetic `-<slug>` or ` - <slug>` suffix is
  dropped, and the directory and extension are kept, so the new path is
  `<same directory>/<new id>.<ext>`.
- A member of a grouped family at the group level is never planned (section 5.1), so
  member reassignment cannot renumber a group.
- If the computed new path already exists when the repair runs, that reassignment is
  skipped and reported (section 10.9) instead of overwriting the file.

### 10.5 Write primitive

`record::store::reassign_id(ctx: &Context, path: &Path, new_id: &str) -> Result<PathBuf, AppError>`
is added to `src/record/store.rs` and exposed on the `Store` façade. It is the only
mutation `check` performs.

1. Resolve and read the record; compute the new path per section 10.4.
2. If the file carries no front matter (HTML mockups, and Markdown records that predate
   front matter), its identity lives in the stem, so the repair is a rename:
   `store::move_record(ctx, old, new)`. The bytes are unchanged.
3. Otherwise apply one patch through `record::patch`:
   - `PatchOp::SetFrontMatter("id", new_id)`.
   - `PatchOp::SetFrontMatter("updated_date", frontmatter::utc_now())`, refreshed once
     per write (ADR-0013 decision 9). The key is added when absent.
   - `PatchOp::SetHeadingId(new_id)` only when the body heading's first token equals the
     record's current front-matter `id`. The heading is left untouched when it disagrees,
     because that disagreement is a `frontmatter-conflict` finding that this command
     does not repair.
4. `store::write_atomic(ctx, new_path, &patched)` writes the new path first, then
   `fs::remove_file(old)` removes the old path. A crash in between leaves the new record
   plus the old one, and the next run reports and repairs the remainder.
5. Return the new path.

`PatchOp::SetHeadingId(String)` is added to `src/record/patch.rs`, mirroring
`SetHeadingTitle`: it replaces the ID component of an existing `# <ID> <title>` heading,
keeps the title, is a no-op when the ID already matches, and inserts no heading when the
record has none. `reassign_id` changes `id`, `updated_date`, and the heading ID only;
every other byte of the record is preserved by the existing patch no-op behaviour.

`src/check/fix.rs` contains no `fs::write`, no `fs::remove_file`, and no `store::`
mutator other than `reassign_id` and the read-only `version_token`.

### 10.6 Refusals

Reported and left untouched, with no write to any file of the group:

- A `duplicate-id` finding with `fixable: false`: a member whose front-matter `id` and
  file stem disagree; a member in a read-only legacy or read/import directory; a grouped
  family reported at the group level.
- Every non-`duplicate-id` finding: dangling `milestone`/`release` references,
  dependency cycles, self-dependencies, front-matter/section conflicts including heading
  disagreements, and invalid configuration.
- No `deprecated/` folder is read, migrated, or repaired (REQ-05.007).

A refusal does not abort the run: the finding stays in the report, other groups are
still repaired, and the exit code stays 4.

### 10.7 Affected references (listed, never rewritten)

- Candidate fields read from every scanned record: `milestone`, `release`,
  `dependencies`, `references`, and `documentation`. A scalar value is one candidate; a
  list contributes one candidate per entry.
- A candidate matches when `identity::parse_id_parts` against the reassigned record's
  family prefix and shape yields the same canonical `key` as the record being
  reassigned, before the repair.
- The reassigned record's own path is never listed, so a self-dependency inside the
  edited file is not reported as an inbound reference.
- One tuple per match: referring family, referring record display ID, project-relative
  path, field name, and the value exactly as written (raw, not reformatted).
- Tuples are sorted by `(family, subject, path, field, value)` and deduplicated.
- The list is computed once, before the writes, and reused by the prompt and the report.
- No referring record is edited, renumbered, or moved (ADR-0029 constraint 9).

### 10.8 Confirmation, cancellation, and non-terminal input

The plan and the prompt are written to **stderr**, so stdout carries only the report in
both output modes and `--json` stays parseable.

Exactly one prompt covers the whole run. When the plan is empty nothing is printed and
no input is read. Otherwise the run prints the plan, one line per reassignment in
section 10.3 order followed by one line per reference in section 10.7 order:

```
fix plan: 2 reassignments
reassign: task TASK-001 backlog/tasks/TASK-001-copy.md -> TASK-002
reference: task TASK-005 (backlog/tasks/TASK-005.md) milestone = "TASK-001"
apply repairs? [y/N]
```

- `fix plan: <n> reassignment` for one planned repair and
  `fix plan: <n> reassignments` for any other count.
- `reassign: <family> <old display id> <old project-relative path> -> <new id>`.
- `reference: <family> <referring id> (<project-relative path>) <field> = "<raw value>"`.
- The prompt line is `apply repairs? [y/N]`, terminated by a newline, and the stream is
  flushed before reading.
- The answer is one line read from stdin, trimmed, ASCII-lowercased: `y` or `yes`
  confirms. Any other line, an empty line, and end-of-input cancel.
- On cancel nothing is written, `repair cancelled; nothing written` is printed to
  stderr, stdout carries the unchanged scan report with `repaired: []`, and the exit
  code is 4 (a plan implies at least one finding).
- `--force` skips both the prompt and the plan block; the repairs are applied and the
  report lists them.
- When `--fix` is given, `--force` is absent, the plan is non-empty, and stdin is not a
  terminal (`std::io::IsTerminal`), the run writes nothing, prints
  `error: --fix requires --force when standard input is not a terminal` to stderr,
  prints nothing to stdout, and exits 2. This is a usage error, so `--json` emits no
  report object, exactly like a clap usage error.
  - Interpretation: the refusal is gated on a non-empty plan. A run with nothing to
    repair never prompts and therefore does not fail, which satisfies REQ-01.008's
    intent that an unattended run never blocks on a prompt.
- The confirmation reads one line from stdin with `std::io`; no terminal crate, no raw
  mode, no `stty` (ADR-0029 decision 8).

### 10.9 Concurrency re-check

- `RecordMeta` gains `path: PathBuf`, `prefix: String`, `numbers: Vec<i64>`, and
  `version: String`, the last captured with `store::version_token(&fs::metadata(path)?)`
  during the scan.
- Immediately before each repair the file's metadata is re-read and its token compared
  with the scan-time token. A difference skips that repair: nothing is written, and a
  finding is added so the skip is visible.
- The skip finding has kind `repair-skipped` (an addition permitted by ADR-0029
  constraint 5), `family` and `subject` of the affected record, `locations` = its path,
  `fixable: false`, and one of these details:
  - `record changed since the scan; repair skipped`
  - `target path <project-relative path> already exists; repair skipped` when the new
    path appeared between the scan and the repair.
- After the token matches, the content is re-read and patched, so a write is never based
  on stale bytes. A matching token is the only state the write is based on.
- A skipped repair is not a `repaired` entry. The original `duplicate-id` finding stays
  in the report, and the run keeps exit code 4.
- The scan itself stays lock-free and snapshot-based; concurrency safety is per repair,
  not per run (ADR-0029 decision 9).

### 10.10 Report additions

Text: the finding lines of section 7.2 come first and stay unchanged; then one
`repaired:` line per applied repair in plan order; then one `reference:` line per
affected reference in section 10.7 order; then the `skipped:` line when present.

```
repaired: task TASK-001 -> TASK-002 (backlog/tasks/TASK-001-copy.md -> backlog/tasks/TASK-002.md)
reference: task TASK-005 (backlog/tasks/TASK-005.md) milestone = "TASK-001"
```

The `reference:` line format is identical to the prompt's. The report repeats the
references of applied repairs so the consequence of the repair is visible after the
prompt is gone; a cancelled or skipped repair lists none.

JSON: `repaired` carries one object per applied repair, with the ADR-0029 decision 6
keys plus two additions, `new_path` and `references`:

```json
{
  "kind": "duplicate-id",
  "family": "task",
  "id": "TASK-001",
  "path": "backlog/tasks/TASK-001-copy.md",
  "new_id": "TASK-002",
  "new_path": "backlog/tasks/TASK-002.md",
  "references": [
    { "family": "task", "subject": "TASK-005",
      "path": "backlog/tasks/TASK-005.md", "field": "milestone", "value": "TASK-001" }
  ]
}
```

`id` and `path` describe the record before the repair, `new_id` and `new_path` after it.
`references` uses the same tuples and order as the text form and is `[]` when the repair
has no affected reference. Entries keep the key order shown (`serde_json` with
`preserve_order`).

### 10.11 Exit codes with `--fix`

- 4 whenever the scan produced at least one finding, including a run whose repairs all
  succeeded, a cancelled run, and a run whose repairs were all skipped.
- 0 only when the scan found nothing; there is then nothing to repair.
- 2 for `--force` without `--fix` and for the non-terminal refusal of section 10.8.
- 1 for an I/O failure while scanning or repairing.

### 10.12 Verification for the repair

- Unit tests in `src/check/fix.rs`: keeper ordering (earliest `created_date`, offset
  equivalence, tie on path, missing or invalid date last), sequential allocation of two
  members of one group in a single run, plain versus member allocation, and confirmation
  parsing (`y`, `Y`, `yes`, `n`, empty, end-of-input).
- `reassign_id` unit tests: front-matter `id`, `updated_date`, and heading ID patched;
  heading untouched when it disagrees with the front-matter ID; a record without front
  matter renamed with identical bytes; the old path gone and the new path parsing to the
  new ID; every other byte of the record identical.
- CLI tests in `tests/check_cli.rs`: keeper and tie-break, the three refusal classes,
  reference listing before the write with the referring file byte-identical after it,
  `--force` bypass, non-terminal exit 2, `--force` without `--fix` exit 2, idempotence
  (a second `--fix` writes nothing and the second run is clean), exit 4 after a
  successful repair, a crash state (new file and old file both present) repaired by the
  next run, and a `--fix --force` golden report plus JSON.
- Goldens assert the ISO-8601 UTC shape of `updated_date` rather than its value, because
  the value is the clock at write time.
- The prompt path itself is unit-tested through the injected reader/writer seam; the
  terminal gate is exercised through a piped stdin at the CLI level, because a pipe is
  not a terminal. The interactive end-to-end path is verified manually.

## 11. Traceability

- REQ-01.008 — run project diagnostics (behavior, finding categories, skipped checks).
- REQ-01.002, REQ-07.002 — tolerant configuration surfaces and validity diagnostics.
- REQ-01.004 — global `--json` and exit codes 0/1/2/3/4.
- REQ-01.005 — canonical ID shape and the single ID space.
- REQ-05.007 — an inert `deprecated/` folder.
- REQ-05.010 — front matter authority, heading policy, canonical decision sections.
- ADR-0029 — decisions 2-6 (exemption from fail-closed startup, exit-code path,
  read-only scanners, descriptor-driven detection, finding shape and order) and
  decisions 7-9 (keeper and reassignment rules, the refusals, no inbound reference
  rewriting, stdin confirmation, per-repair concurrency re-check).
- ADR-0010 decision 5 — identity allocation over every record location.
- ADR-0013 decisions 7 and 9 — the single writer, and `updated_date` refreshed once per
  write.
- ADR-0012 — retired metadata sections.
- ADR-0013, ADR-0018 — single parser, projection, and writer.
