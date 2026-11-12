---
name: next-version
description: 'Choose the next semantic version (final release or milestone RC) by analyzing Done tasks in the milestones of the current release. Classifies changes as breaking (major), feature (minor), or chore/maintenance (patch) and handles RC numbering with reset on major increment. Asks the user when classification or scope is uncertain.'
argument-hint: 'Optionally provide the current release, milestone, or target ("rc" or "release"). If omitted, resolve from the backlog and ask when ambiguous.'
user-invocable: true
disable-model-invocation: false
---

# Next Version Selection

## Outcome

One recommended version string (for example `1.4.0`, `1.4.0-rc.3`, or `2.0.0-rc.1`) with a rationale, the increment class, the baseline version, the RC number (when applicable), and the list of Done tasks considered. No backlog state is changed. The recommendation is proposed to the user for confirmation; it is not recorded or tagged.

## When To Use

- Before cutting a milestone RC or a final release, to decide the version number.
- When the user asks "what should the next version be", "choose the next version", "bump version", or "next rc".
- After a set of tasks has been completed and the version needs to be derived from the delivered work.

Do not use this skill for:
- Closing a release or running the security audit (use `release-orchestration`).
- Delivering milestone work (use `milestone-orchestration`).
- Creating Git tags or modifying version files (report the recommendation only; tagging is the user's decision).

## Prerequisites

- **backlog-md skill (REQUIRED)** — Load it before any backlog read.
- Read access to the repository, including Git tags and project version files.
- Available: `git` for tags and history.

## Versioning Rules

This project follows Semantic Versioning (`MAJOR.MINOR.PATCH`) with these domain rules:

- **Breaking change** — any change to a public API, file format, or command-line argument (renamed/removed/changed semantics) requires a **major** increment.
- **New function** — a backward-compatible addition (new command, new option, new field, new capability) requires a **minor** increment.
- **Chore / maintenance** — documentation updates, dependency/library updates, refactors with no behavior change, tests, build changes require a **patch** increment.
- **Pre-1.0.0 (`0.x`)** — the API is unstable and may break at any time. Breaking changes before 1.0.0 are allowed, but the increment rule still applies unless the user states otherwise. When a breaking change is detected on a `0.x` baseline, ask the user whether to increment major (to `1.0.0`) or minor, because both are defensible under SemVer for `0.x`.
- **RC (release candidate)** — at the end of each milestone an RC is released: `X.Y.Z-rc.N`. No intensive audit occurs at milestone end. The final release `X.Y.Z` is cut only after all milestones of the release are Done and the end-of-release security audit passes.
- **RC reset** — when the computed base version (`X.Y.Z`) changes from the base of the latest RC (because a new breaking or feature change appeared after earlier RCs), the RC counter resets to `1`. When the base version is unchanged, increment the RC counter.
- **Only Done tasks count.** A task that is not yet implemented is considered cancellable and is excluded. `To Do`, `Initiated`, `Refined`, `Implemented`, and `Validated` tasks are ignored.
- **Highest wins.** If the considered Done tasks include at least one breaking change, the increment is major even if other tasks are features or chores. Otherwise, if at least one new function, minor. Otherwise patch.

## Procedure

### 1. Establish the baseline version

1. Load `backlog-md`.
2. Determine the last released version:
   - Run `git tag --list 'v*' --sort=-v:refname` (and variants without the `v` prefix). Take the highest SemVer tag.
   - If no tags, inspect completed release milestones: `backlog milestone list --show-completed --plain` and pick the highest `Release <version>` milestone that is completed/archived or whose release ticket is `Done`.
   - If neither exists, ask the user for the baseline version (do not guess `0.1.0` or `1.0.0` silently).
3. Cross-check project version files when present (`package.json`, `Cargo.toml`, `build.gradle*`, `pom.xml`, `pyproject.toml`, `VERSION`, etc.). If they disagree with the Git/milestone baseline, report the discrepancy and ask the user which is authoritative.
4. Record `baseline = MAJOR.MINOR.PATCH` (strip any pre-release suffix; the baseline is the last final release).

### 2. Resolve the current release scope

1. List milestones: `backlog milestone list --plain` (and `--show-completed` if needed).
2. Identify completed releases (baselines already shipped) from step 1.
3. Determine the **current release** = the set of milestones that belong to the in-progress release:
   - Prefer an explicit release milestone named `Release <version>` that is not yet complete; its tasks define the release scope, and any non-release milestones referenced by those tasks (or named as part of the same release) are included.
   - If the project groups work into feature milestones followed by a release milestone, include all feature milestones created after the last completed release and before the next `Release <version>` milestone.
   - If multiple plausible groupings exist, ask the user and stop.
4. List tasks for each milestone in scope: `backlog task list --plain` filtered by milestone. Exclude the release ticket (label `release`) from classification, but note it.
5. Collect only tasks with status `Done`. Record their IDs, titles, labels, descriptions, acceptance criteria, final summaries, and `modifiedFiles`.

If no Done tasks exist in scope, report that and ask whether to keep the baseline version unchanged (patch with no work is meaningless) or wait.

### 3. Classify each Done task

For each Done task, decide one of:

- **breaking** — the task changes or removes a public API, file format, or command argument; renames a command/flag; changes output format consumed by users; changes on-disk schema. Evidence: labels such as `breaking`, `api`, `cli`, `format`, `schema`, `migration`; AC or final summary mentioning "breaking", "incompatible", "rename", "remove", "change argument", "format change"; modified files in public contract paths.
- **feature** — adds new, backward-compatible functionality: new command, new option, new endpoint, new field, new capability. Evidence: labels `feature`, `enhancement`; AC describing "add", "new", "support".
- **chore** — documentation, dependency/library updates, refactors, tests, build/CI, internal cleanup with no user-visible behavior change.

Rules:
- When evidence is mixed or ambiguous, mark the task `unknown` and collect it for a user question. Do not silently classify.
- A task touching both a breaking surface and an internal refactor is `breaking`.
- Dependency updates that force an API change are `breaking`; pure version bumps are `chore`.
- Documentation of a new feature does not make a chore task a feature; the feature is the task that adds the capability.

Produce a classification table: Task ID | Title | Class | Evidence.

### 4. Determine the increment

1. If any task is `unknown`, ask the user to classify those tasks (breaking / feature / chore) and stop until answered.
2. If any `breaking` → increment = **major**.
   - Else if any `feature` → increment = **minor**.
   - Else → increment = **patch**.
3. Pre-1.0.0 special case: if baseline `< 1.0.0` and increment is major, ask the user whether to:
   - increment major (to `1.0.0` or `0.(MAJOR+1).0`? clarify) — recommend `1.0.0` only if the release is intended to mark API stability; otherwise `0.(MAJOR+1).0`.
   Stop for the answer.
4. Compute the new base version by applying the increment to the baseline:
   - major: `(MAJOR+1).0.0`
   - minor: `MAJOR.(MINOR+1).0`
   - patch: `MAJOR.MINOR.(PATCH+1)`

### 5. Determine RC vs final and RC number

1. Ask or infer the release kind:
   - If the user named a milestone (not a release) or the run is "end of milestone" → RC.
   - If the user named a release / "ship" / "final" → final (no suffix).
   - If unclear, ask.
2. For a final release: recommended version = new base version.
3. For an RC:
   - Find the latest RC tag or milestone for the same base version and the previous base version:
     - `git tag --list '*-rc.*' --sort=-v:refname`.
     - Also inspect release/milestone names for `rc`.
   - If the latest RC's base version equals the new base version → next RC number = latest RC number + 1.
   - If the latest RC's base version differs from the new base version (because the increment changed, e.g. a breaking change appeared after earlier minor RCs) → reset: RC number = 1.
   - If no prior RC for this release → RC number = 1.
   - Recommended version = `X.Y.Z-rc.N`.

### 6. Report

Print:
- Baseline version and its source.
- Current release scope (milestones included).
- Done tasks considered (IDs and titles).
- Classification table.
- Increment class and reason.
- RC handling (reset or increment) if applicable.
- **Recommended version**.
- Caveats: tasks not yet implemented were excluded and could still change the version if they land; if a later task is breaking, re-run this skill (the version and RC counter may reset).

Do not create tags, edit files, or change backlog state. Ask the user to confirm the recommended version; if they request a different one, record their override as the decision (no file write).

## Stop Conditions

Stop and ask the user when:
- The baseline version cannot be determined.
- The current release scope is ambiguous.
- Any Done task cannot be classified confidently.
- A breaking change is detected on a pre-1.0.0 baseline.
- The release kind (RC vs final) is not stated and cannot be inferred.
- No Done tasks exist in scope.

## Completion Checks

- Baseline version identified with source.
- Release scope and included milestones stated.
- Only Done tasks considered; non-Done tasks excluded and noted.
- Classification table produced with evidence.
- Increment class justified by the highest-severity change.
- RC number computed with reset logic explained.
- Recommended version printed separately from the rationale.
- No repository, backlog, or version file modified.

## Resume / interruption

The skill is stateless. Re-run from step 1 on re-invocation; re-read milestones, tasks, and tags. Do not reuse a previous recommendation without re-reading current state.