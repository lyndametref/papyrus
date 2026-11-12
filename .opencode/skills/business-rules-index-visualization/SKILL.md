---
name: business-rules-index-visualization
description: 'Regenerate docs/business-rules/index.html — a searchable detail view of business rules — from BR-XXX Markdown files. Use after any business-rule file change or when asked to refresh the index.'
argument-hint: 'Run after business-rule changes, e.g. "regenerate the business rules index" or "refresh index.html".'
user-invocable: true
disable-model-invocation: false
---

# Business Rules Index Visualization

## Outcome

`docs/business-rules/index.html` is the only generated business-rules index. It is regenerated from the current `BR-XXX-*.md` corpus. Markdown files are source inputs; no Markdown index is required or generated.

## When To Use

- After creating, updating, renaming, or removing a business-rule file.
- On explicit request to regenerate or refresh the business-rules index.

## Prerequisites

- This skill bundles its parser at `scripts/generate_business_rules_index.sh` (POSIX sh + awk) and its presentation at `templates/index.html`.
- Rendering uses the shared POSIX sh/awk renderer from the `html-template-migration` skill.
- The target project has `docs/business-rules/` with one or more `BR-XXX-short-name.md` files.
- Each rule uses the canonical `# BR-XXX: Title` heading and `Description`, `Rationale`, and `Applicability` sections.

If the bundled script is missing, stop and report it; do not create a replacement silently.

## Procedure

1. From the target project root, run:

   ```
   ~/.config/opencode/skills/business-rules-index-visualization/scripts/generate_business_rules_index.sh
   ```

    It defaults to `docs/business-rules`. Pass a directory to override; an optional second argument selects a different output path:

   ```
    ~/.config/opencode/skills/business-rules-index-visualization/scripts/generate_business_rules_index.sh path/to/business-rules path/to/index.html
   ```

2. On success, report `wrote <output>: N business rules`.
3. On failure, inspect stderr and fix the reported source issue before retrying. The script removes its temporary data file and leaves the destination unchanged.
4. Never hand-edit `index.html`; regenerate it from the rule files.

## What The Script Does

- Reads and validates each BR filename, title, and required section.
- Rejects malformed filenames, duplicate BR IDs, and missing required content.
- Builds a JSON payload and renders it through `templates/index.html`, which provides a filterable rule list and full Description, Rationale, and Applicability details.
- Does not read, create, or update a Markdown index.

## Failure Modes

| Message | Cause | Fix |
|---|---|---|
| `business-rules directory not found: <dir>` | Directory is missing | Run from project root or pass the directory explicitly |
| `no business-rule files found in <dir>` | No `BR-*.md` files are present | Add canonical business-rule files before generating the index |
| `bad business-rule filename: <file>` | Filename does not match `BR-XXX-short-name.md` | Rename the file to the canonical format |
| `duplicate BR ID(s): <ids>` | Multiple files use the same ID | Assign each rule a unique BR ID |
| `bad title` / `missing <section>` | Rule heading or required section is malformed or empty | Correct the business-rule source file |
| `missing value` / `unused value` / `unresolved placeholder` | Template and payload do not match | Report the details; do not edit generated HTML by hand |
| `expected N id entries` / `id ... missing from parsed data` | Parser or source problem | Check BR filenames and rerun |

## Completion Checks

- Script exits 0 and reports the number of business rules written.
- The reported count equals the number of valid BR files.
- `<business-rules-dir>/index.html` exists and is newer than its source rule files.
- The generated page includes a link back to the main documentation index (`docs/index.html`).
- No Markdown index was created or updated.
