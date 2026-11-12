---
name: adr-index-visualization
description: 'Regenerate index.html — a list + detail visualization of the Architecture Decision Record (ADR) corpus — by running this skill bundled scripts/generate_adr_index.sh against the project docs/architecture/adr. Use after every change to ADR files (create, revise, accept, refuse, deprecate, supersede). This skill is the only owner of ADR index generation.'
argument-hint: 'Run after ADR changes, e.g. "regenerate the ADR index" or "refresh the architecture index".'
user-invocable: true
disable-model-invocation: false
---

# ADR Index Visualization

## Outcome

`docs/architecture/adr/index.html` is regenerated from the current ADR markdown corpus and its timestamp is newer than the ADR files it visualizes.

## When To Use

- Immediately after any ADR change under `docs/architecture/adr/`: create, revise, accept, refuse, deprecate, or supersede.
- On explicit request to regenerate or refresh the ADR index.

Do not use this skill for:

- Creating or editing ADRs — use the `architecture-decision-record` skill.
- Projects that do not keep ADRs as `ADR-NNNN-<kebab-case-title>.md` files.

This skill is the single owner of ADR index generation. Other agents and skills must invoke it rather than run the script or generate their own index.

## Prerequisites

- This skill bundles its parser at `scripts/generate_adr_index.sh` (POSIX sh + awk) and its presentation at `templates/index.html`.
- Rendering uses the shared POSIX sh/awk renderer from the `html-template-migration` skill.
- The target project has an ADR directory (default `docs/architecture/adr/`) containing one or more `ADR-NNNN-*.md` files.
- No separate markdown ADR index is required or maintained. A legacy `ADR-INDEX.md` is ignored if present.

If the bundled script is missing, stop and report it — do not create a replacement silently.

## Procedure

1. From the target project root, run the bundled script:

   ```
   ~/.config/opencode/skills/adr-index-visualization/scripts/generate_adr_index.sh
   ```

    It defaults to `docs/architecture/adr` relative to the current directory. Pass a directory to override; an optional second argument selects a different output path:

   ```
    ~/.config/opencode/skills/adr-index-visualization/scripts/generate_adr_index.sh path/to/architecture path/to/index.html
   ```

2. On success the script prints `wrote <output>: N decision records` and exits 0. Report that summary.
3. On failure the script exits non-zero, removes its temporary data file, and leaves the destination unchanged. Read stderr and fix the reported cause before retrying (see Failure Modes).
4. Never hand-edit `<adr-dir>/index.html` — it is a generated artifact; always regenerate it.

## What the Script Does

- Parses every `ADR-NNNN-*.md` file (files are the source of truth).
- Rejects ADR files that do not follow the naming convention and ADR ids that are duplicated.
- Builds a JSON payload and renders it through `templates/index.html`: left pane = filterable list of decisions (id, title, status dot); right pane = full ADR detail (status tag, then every `##` section in file order, including custom sections such as `Related Decisions`).
- Renders section content as lightweight markdown: paragraphs, bullet and numbered lists, tables (with header row), fenced code blocks (backticks or tildes, whitespace preserved), `###`/`####` subheadings, bold (`**text**`), emphasis rendered as `<b>` (`*text*`), inline code, and relative/`http(s)` links.
- Output has no external resources (works offline via `file://`) and is deterministic. The parser's temporary data file is created in the ADR directory; the renderer writes atomically beside the selected output.

## Failure Modes

| Message | Cause | Fix |
|---|---|---|
| `ADR directory not found: <dir>` | No `docs/architecture/adr` in the current directory and no directory argument given | Run from the project root or pass the directory explicitly |
| `bad ADR filename: <file> (expected ADR-NNNN-kebab-case-title.md)` | An `ADR-*.md` file does not follow the naming convention (e.g. `ADR-2-title.md`) | Rename it to `ADR-<NNNN>-<kebab-case-title>.md` and rerun |
| `duplicate ADR id(s): <ids>` | Two files share the same `ADR-NNNN` prefix | Renumber one ADR and update cross-references, then rerun |
| `missing title heading in <file>` | The ADR has no leading `# <Title>` line | Add the title heading, then rerun |
| `missing Status section in <file>` | The ADR has no `## Status` section or it is empty | Add the status per the ADR template, then rerun |
| `invalid status in <file>: <status>` | Status does not start with `Proposed`, `Accepted`, `Refused`, `Deprecated`, or `Superseded` | Fix the status using a legal value, then rerun |
| `missing value` / `unused value` / `unresolved placeholder` | Template and payload do not match | Report the details; do not edit `index.html` by hand |
| `expected N id entries` / `id ... missing from parsed data` | Parser or source problem | Check ADR filenames and rerun |

## Completion Checks

- Script exited 0 and printed the `wrote <output>: N decision records` summary.
- `N` equals the number of `ADR-NNNN-*.md` files in the ADR directory.
- `<adr-dir>/index.html` exists and is newer than the ADR files (default `docs/architecture/adr/index.html`).
- No markdown ADR index file was created or updated.
- Selecting a decision shows its status and all ADR sections, including tables and lists.
- The generated page includes a link back to the main documentation index (`docs/index.html`).
- No manual edits were made to `index.html`.
