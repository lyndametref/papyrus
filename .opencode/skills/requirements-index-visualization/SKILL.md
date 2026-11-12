---
name: requirements-index-visualization
description: 'Regenerate index.html — a tree + detail visualization of the requirements corpus — by running this skill bundled scripts/generate_index.sh against the project docs/requirements. Use after any change to REQ/NFR files (create, update, reorganize, validate) to keep the visual index in sync with the markdown sources.'
argument-hint: 'Run after requirement changes, e.g. "regenerate the requirements index" or "refresh index.html".'
user-invocable: true
disable-model-invocation: false
---

# Requirements Index Visualization

## Outcome

Two generated indexes are produced: `docs/requirements/business/index.html` for business requirements and `docs/requirements/non-functional/index.html` for non-functional requirements. They are regenerated from the current REQ/NFR Markdown corpus and their timestamps are newer than the requirement files they visualize. The Markdown files are source inputs; the skill does not generate or maintain a Markdown index.

## When To Use

- After creating, updating, renaming, or reorganizing any requirement file under `docs/requirements/business/` or `docs/requirements/non-functional/`.
- After validation corrections that changed requirement content.
- On explicit request to regenerate or refresh the requirements index.

Do not use for projects whose requirements do not use the canonical atomic structure (group files `REQ-XX.md`/`NFR-XX.md` and atomic files `REQ-XX.XXX.md`/`NFR-XX.XXX.md`).

## Prerequisites

- This skill bundles its parser at `scripts/generate_index.sh` (POSIX sh + awk) and its presentation at `templates/index.html`.
- Rendering uses the shared POSIX sh/awk renderer from the `html-template-migration` skill.
- The target project has `<requirements-dir>/business/` and `<requirements-dir>/non-functional/` directories containing group and atomic requirement files. Markdown `INDEX.md` files are not required or generated.

If the bundled script is missing, stop and report it — do not create a replacement silently.

## Procedure

1. From the target project root, run the bundled script:

    ```
    ~/.config/opencode/skills/requirements-index-visualization/scripts/generate_index.sh
    ```

     It defaults to `docs/requirements` relative to the current directory and generates both `business/index.html` and `non-functional/index.html`. Pass a corpus directory (for example `docs/requirements/business`) to regenerate one index; an optional second argument selects a different output path:

    ```
     ~/.config/opencode/skills/requirements-index-visualization/scripts/generate_index.sh docs/requirements/business docs/requirements/business/index.html
    ```


2. On success the script prints `wrote <output>: N requirements (M groups)` and exits 0. Report that summary.
3. On failure the script exits non-zero, removes its temporary data file, and leaves the destination unchanged. Read stderr and fix the reported cause before retrying (see Failure Modes).
4. Never hand-edit `<requirements-dir>/index.html` — it is a generated artifact; always regenerate it.

## What the Script Does

- Parses every group and atomic requirement file; these files are the sole source of truth.
- Builds a JSON payload and renders it through `templates/index.html`: collapsible tree (requirement groups → requirements) with a filter box; right pane = full requirement detail (criticality, user story, description, acceptance criteria for REQ; description, rationale, verification method for NFR). Markdown files are read-only inputs and no Markdown index is generated.
- Output has no external resources (works offline via `file://`) and is deterministic. The parser's temporary data file is created in the corpus directory; the renderer writes atomically beside the selected output.

## Failure Modes

| Message | Cause | Fix |
|---|---|---|
| `requirements directory not found: <dir>` | No `docs/requirements` in the current directory and no directory argument given | Run from the project root or pass the directory explicitly |
| `group <ID> not seen before <atomic-ID>` | An atomic file exists whose group file is missing or not named `REQ-XX.md`/`NFR-XX.md` | Create or rename the group file, then rerun |
| `bad title: ...` | An `# ` heading does not match `REQ-XX[.XXX] Title` / `NFR-XX[.XXX] Title` | Fix the heading, then rerun |
| `missing value` / `unused value` / `unresolved placeholder` | Template and payload do not match | Report the details; do not edit `index.html` by hand |
| `expected N id entries` / `id ... missing from parsed data` | Parser or source problem | Check requirement IDs and rerun |

## Completion Checks

- Script exited 0 and printed the `wrote <output>: N requirements (M groups)` summary.
- `N` equals the number of atomic REQ files plus atomic NFR files.
- `docs/requirements/business/index.html` and `docs/requirements/non-functional/index.html` exist and are newer than the requirement files.
- Each generated page includes a link back to the main documentation index (`docs/index.html`).
- No manual edits were made to `index.html`.
