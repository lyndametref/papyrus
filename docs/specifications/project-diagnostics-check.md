# Feature Specification — Project Diagnostics (`papyrus check`, read-only run)

**Requirement:** REQ-01.008 (Run Project Diagnostics)
**Architecture:** ADR-0029 (Read-Only Corpus Diagnostics with Duplicate-ID Repair), decisions 1-6
**Milestone:** m-21
**UI mockups:** none (CLI only)
**Status:** Planned

This specification fixes the read-only diagnostic contract: the command surface, the
two-stage configuration load, the scanned locations, the seven finding kinds, the
finding model, the output shapes, and the exit codes. Rationale and alternatives live
in ADR-0029; this document is the implementation contract. Duplicate-ID repair
(`--fix`, `--force`) is specified separately and is not part of the read-only run.

## 1. Command surface

- `papyrus check` is a built-in command with no subcommands. In this release the
  read-only run takes no flags of its own; `--json`, `--dir`, `--config`, and
  `--no-color` apply as global options (REQ-01.004).
- `check` is exempt from fail-closed startup (ADR-0019). `lib.rs::run` builds the
  command tree with no registry when the first positional is `check`, so
  `papyrus check --help` works on a project whose configuration is invalid
  (ADR-0029 decision 2).
- `check` is CLI-only in this release; no browser route runs it.

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

- Enumeration uses the new read-only primitive
  `record::scan::family_files(ctx, descriptor) -> Result<Vec<PathBuf>, AppError>`,
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
  family reported at the group level. Nothing is repaired by this run.

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
`frontmatter-conflict`, `frontmatter-conflict:document-records`. Additions are
allowed; renames are not.

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
`skipped: <identifiers joined with ", ">`.

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
empty arrays on a clean run; `repaired` is always `[]` for the read-only run.

## 8. Exit codes

| Code | Meaning |
|---|---|
| 0 | The run completed and found nothing. |
| 4 | The run completed and found at least one finding (the validation/conflict code of REQ-01.004, reused for findings). |
| 1 | I/O failure (a directory could not be listed, the project directory is unreadable). |
| 2 | Usage error (clap). |

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

## 10. Traceability

- REQ-01.008 — run project diagnostics (behavior, finding categories, skipped checks).
- REQ-01.002, REQ-07.002 — tolerant configuration surfaces and validity diagnostics.
- REQ-01.004 — global `--json` and exit codes 0/1/2/3/4.
- REQ-01.005 — canonical ID shape and the single ID space.
- REQ-05.007 — an inert `deprecated/` folder.
- REQ-05.010 — front matter authority, heading policy, canonical decision sections.
- ADR-0029 — decisions 2-6 (exemption from fail-closed startup, exit-code path,
  read-only scanners, descriptor-driven detection, finding shape and order).
- ADR-0012 — retired metadata sections.
- ADR-0013, ADR-0018 — single parser, projection, and writer.
