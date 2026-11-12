---
name: project-initialization
description: 'Create an approved base folder structure and documentation indexes from a fixed index set, and verify Backlog.md CLI readiness.'
user-invocable: false
disable-model-invocation: false
---

# Project Initialization

## Outcome

Prepare a project for the requirements → planning → feature specification → implementation workflow by gathering project context, creating only the authorized missing base folders and documentation indexes from a fixed index set, and confirming that Backlog.md can operate in the target project.

This skill is for project bootstrap, not ordinary backlog planning. It does not create substantive requirements, feature specifications, business rules, architecture decisions, milestones, or feature tickets.

## Required safeguards

- Identify the target project root before making changes. If the current working directory is clearly the target repository root, state that assumption; otherwise ask the user for the path.
- Inspect the target read-only before creating anything, including hidden entries, repository state, existing documentation, and backlog configuration. Preserve all existing, untracked, ignored, or modified content.
- Before creating each folder, inspect every existing component of its path. If any component is a file, symlink, or unexpected path, stop on that path and ask before proceeding.
- Reuse existing folders and conventions. Create only missing, authorized paths. Never overwrite, move, rename, delete, or normalize existing project content.
- Do not create stack-specific source folders until the platform and technology are known and the user wants that scaffold.
- Do not invent project scope, business priorities, requirements, technical choices, or architecture decisions. Mark unknowns and blockers explicitly.
- Use the `backlog-md` skill and Backlog.md CLI for backlog initialization and task operations. Never hand-create or edit Backlog-managed task, milestone, document, or decision files. Backlog.md requires its `statuses` list to be configured in the existing project configuration file; use that documented path only for the approved status-list change, preserve all other configuration, and verify it with the CLI.
- Do not install software or run dependency setup commands. If the Backlog.md CLI is unavailable, report the blocker and ask the user how to proceed.

## Procedure

### 1. Confirm the initialization target and scope

Confirm the target root and whether this is a new, existing, or partially initialized project. Ask whether the user also expects a delivery plan now or only project bootstrap. Do not create any folders or run modifying setup commands until the user approves the proposed structure in step 4. Do not create milestones or feature tickets as part of bootstrap unless separately requested and a validated planning baseline exists. Exception: a domain-modeling task (context map and business object model) and a technical-stack definition task (delegated to `architect`) may be created via the Backlog.md CLI with explicit user approval, because they are bootstrap prerequisites rather than feature tickets.

### 2. Inspect the project before proposing changes

Inventory the project root, including hidden entries and repository metadata. Review `git status --short` when it is a Git repository, and inspect ignored entries under paths proposed for initialization where needed to avoid collisions. Record the existing presence and status of all planned folder and index-file paths, including `docs/index.html` and each corpus `index.html`. Do not read potentially sensitive file contents merely because they are ignored or untracked. Identify existing source, documentation, conventions, project manifests, and Backlog.md files/configuration. Classify the project as:

- **New/empty:** no existing project-specific structure beyond repository metadata.
- **Existing:** project source, documentation, configuration, or backlog artifacts are present.
- **Partial/ambiguous:** conventions or initialization state are mixed or unclear.

Do not call a repository empty just because it has no application source. If the target is partial or ambiguous, preserve its conventions and ask before resolving conflicts.

### 3. Confirm the fixed documentation index set

Do not consult documentation owners during bootstrap. Use this hardcoded index set (paths are relative to `<project-root>/docs`):

- `index.html` (landing page)
- `architecture/adr/index.html`
- `requirements/business/index.html`
- `requirements/non-functional/index.html`
- `business-rules/index.html`
- `domain-model/business-glossary/index.html`

Collect only the project-level answers needed to proceed (purpose, scope, constraints, owners, unknowns) in one concise pass; specialist intake happens later during requirements, specification, and architecture work. Wait for the user's answers before creating folders or running `backlog init`. Record which answers are confirmed, undecided, or still open. If material questions remain, pause and ask whether the user wants to proceed with those explicitly listed unknowns; do not treat silence or a partial response as approval.

### 4. Propose and get approval for the base folder structure

Use project conventions where they exist. Otherwise propose the shared documentation structure used by this workflow:

```text
docs/
├── index.html
├── requirements/
│   ├── business/
│   └── non-functional/
├── specifications/
├── business-rules/
├── domain-model/
│   ├── business-glossary/
│   └── object-model/
├── business-processes/
├── architecture/
│   └── adr/
```

The documentation entry points initialized by this skill are `docs/index.html`, `docs/architecture/adr/index.html`, `docs/requirements/business/index.html`, `docs/requirements/non-functional/index.html`, `docs/business-rules/index.html`, and `docs/domain-model/business-glossary/index.html`.

The Backlog.md CLI owns its own folder and configuration structure. Do not create those paths manually. Include this board status order in the proposal for explicit approval:

```text
To Do → Initiated → Refined → Implemented → Validated → Done
```

`To Do` remains the initial status. `Done` is reserved for work the user has explicitly approved. Propose stack-specific source folders only after platform choices are confirmed.

Before writing, show the user the proposed folders and index files adapted to the inspected project. For every path, identify whether it will be **reused**, **created**, or **deferred**, and state why. Ask the user to approve the proposal, including creation of the documentation index files. Wait for explicit approval before creating any folders or files; if the user requests changes, revise the proposal and obtain approval for the revised structure. If approval is not given, make no filesystem changes.

After approval, create only the missing folders the user approved. Recheck each path and its ancestors immediately before creation. Do not create placeholder requirement/specification/ADR Markdown files or `.gitkeep` files unless the approved proposal or repository convention calls for them. Once the approved documentation folders exist, run the bundled initializer:

```
~/.config/opencode/skills/project-initialization/scripts/initialize_docs_indexes.sh <project-root>
```

The initializer creates a `docs/index.html` landing page linking to the corpus indexes above (including separate business and non-functional requirement indexes). For each missing corpus index, it renders the visualization template with an empty data payload if that corpus has no records; if canonical records already exist, it calls that corpus's generator to create a populated index. Existing regular `index.html` files are preserved. Symlinks, non-directory path components, and other unexpected path collisions stop initialization for review. The initialization helper requires the approved base directories to exist and does not create folders itself. The visualization skills own all subsequent index regeneration after corpus changes.

### 5. Check, prepare, and configure Backlog.md

Load the `backlog-md` skill before any Backlog operation. From the confirmed project root:

1. Check CLI availability and version with `backlog --version`; inspect supported commands with `backlog --help`. Consult `backlog help <command>` for exact command details rather than guessing flags.
2. Check whether Backlog.md already works in this project using read-only commands: `backlog task list --plain` and `backlog milestone list --plain`. An empty result is acceptable; successful project-scoped queries are readiness evidence. If the project is initialized, continue to step 4. If it is not initialized, continue to step 3.
3. For an uninitialized project, inspect the documented `backlog init` behavior. Explain the setup and any files/configuration it will create or change, then obtain explicit approval before running `backlog init` in the confirmed project root. After initialization, run the task and milestone list commands and confirm they succeed. Do not rerun init when an existing configuration works.
4. Read the current status list with `backlog config get statuses` and inspect task statuses with `backlog task list --plain`. The desired ordered statuses are `To Do`, `Initiated`, `Refined`, `Implemented`, `Validated`, `Done`; do not retain `In Progress`. For an existing, ongoing project, compare its configured statuses with this list and explicitly identify any missing statuses. Include every missing desired status in the approved status-list update; do not assume the existing configuration is complete.
5. If an existing task uses a status that would be removed or is not in the desired list, do not change the status configuration yet. Ask the user how those tasks should be mapped. After explicit approval, migrate each affected task with `backlog task edit <id> -s <approved-status>` before removing its current status from the config; never change task statuses as an implicit side effect of initialization. Adding missing desired statuses does not authorize changing any task's status.
6. Once the user has approved the status list, update only the `statuses` array in the existing Backlog.md project config file (normally `backlog/config.yml` or the configured root-level `backlog.config.yml`). The CLI does not support setting this list with `backlog config set`; do not replace other settings or create the config file manually. Preserve the existing config location and all unrelated values. Use YAML such as:

   ```yaml
   statuses: ["To Do", "Initiated", "Refined", "Implemented", "Validated", "Done"]
   ```

7. Verify the saved list with `backlog config get statuses`, then run `backlog task list --plain` and `backlog milestone list --plain` and verify both succeed. Do not create milestones, tasks, or docs merely to make the checks non-empty.
8. If the CLI is unavailable or setup/read checks fail, report Backlog.md as **blocked** with the command and error. Do not install the CLI, switch tools, or change task statuses without user approval.

Report readiness accurately: distinguish **ready** (CLI available and both project-scoped reads succeed), **initialized but unverified** (setup completed but checks fail), and **blocked** (CLI or setup unavailable). Do not claim readiness based only on the executable or version.

### 6. Handoff and report

After scaffolding, re-check every planned path and confirm that only the intended missing folders and index files were added. Verify that a newly created `docs/index.html` links to all corpus indexes, that each missing empty-corpus index embeds its expected empty data shape and links back to `docs/index.html`, and that existing index files remain unchanged. If a pre-existing docs landing page lacks a link, report that without modifying it. If using Git, compare `git status --short` and ignored entries under the planned paths with the pre-initialization state. Investigate any unexpected changes; preserve pre-existing untracked, ignored, and modified files. Do not try to revert unexpected changes automatically.

Report:

- Target root and project classification.
- Specialist agents consulted: none during bootstrap (intake deferred to later work). Domain modeling (context map and business object model) and technical-stack definition (delegated to `architect`) are deferred to backlog tasks executed by `domain-modeler` and `architect`; they must be early roadmap tasks before feature work. Create these tasks via the Backlog.md CLI with explicit user approval (do not perform the work during bootstrap).
- Folders created, reused, or intentionally deferred.
- Documentation indexes created, preserved, or populated from existing corpus records.
- Backlog.md version, setup action (if any), configured status list, and results of the task and milestone readiness checks.
- Decisions still needed, blockers, and clear next handoffs to requirements-engineer, business-analyst, architect, and domain-modeler (for an initial context map and business object model and a technical-stack definition via backlog tasks).
- Whether a separate project-planning step can begin. Do not mark requirements validated simply because initialization inputs were collected.

## Completion checks

- Target root was explicit or confirmed before any writes.
- Existing project content and conventions were inspected and preserved.
- The fixed documentation index set was used without specialist consultation during bootstrap.
- The user approved the proposed structure before implementation; only approved, missing shared folders and indexes were created, with no specialist-owned substantive documents invented by the planner.
- A newly created `docs/index.html` links to all corpus indexes; each missing corpus index is rendered empty when its corpus is empty, or populated from existing canonical records. Each generated corpus index includes a link back to `docs/index.html`. Pre-existing indexes are preserved.
- Backlog.md was verified through successful project-scoped read commands, or clearly reported as blocked/unverified; its approved status list is `To Do`, `Initiated`, `Refined`, `Implemented`, `Validated`, `Done`, with `Done` reserved for explicit user approval.
- Any planned milestones or feature tickets are deferred unless separately requested and supported by validated requirements.
