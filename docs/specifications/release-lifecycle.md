# Release Lifecycle Semantics

## Purpose

Define the lifecycle of a Papyrus release record so that REQ-03.002, the README manpage (`papyrus(1)`), BR-007, and the CLI implementation agree. The transition described here is implemented by TASK-118.

## Sources

- `README.md`, `papyrus(1)` → COMMANDS → release (manpage release command entry).
- `README.md`, `papyrus(1)` → STATUSES → Releases.
- `README.md`, `papyrus(1)` → STATUSES → Milestones (for contrast).
- `docs/requirements/business/REQ-03.002.md` — Manage Releases.
- `docs/business-rules/BR-007-release-command-behavior.md` — release command behavior.
- `docs/requirements/business/REQ-07.001.md` — Organize Artifacts into Three Categories.
- `docs/domain-model/business-glossary/release.md` — Release term.
- `docs/domain-model/object-model/business-object-model.md` — Release object description.
- TASK-22 (research task), TASK-8 (initial implementation), TASK-118 (`close` transition).

## Documented release behavior

The release command exposes these verbs only:

- `papyrus release create <title>`
- `papyrus release list`
- `papyrus release view <id>`
- `papyrus release edit <id> [--title <title>] [-d <description>]`
- `papyrus release close <id>`

No `archive`, `restore`, `delete`, or `status` command exists for releases. Release edit supports only title and description; it has no `--status` option.

The lifecycle is fixed by the backlog category (REQ-07.001) at `planned -> released`, reached only through `papyrus release close <id>`. It is not configurable through `statuses` or `default_status`.

## Implemented lifecycle

1. **State is stored.** A release stores its lifecycle state in the front-matter `status` field (REQ-05.010). The only valid states are `planned` and `released`.
2. **Create starts planned.** `papyrus release create <title>` writes `status: planned` after `references` in canonical front-matter order.
3. **Close performs the transition.** `papyrus release close <id>` resolves the release, reads its current status, and on success writes `status: released`, refreshing `updated_date` while preserving title, description, labels, references, and body.
4. **Legacy tolerance.** A missing or empty front-matter `status` is read as `planned`, so imported and legacy release records remain closable.
5. **Non-planned states are refused.** Any stored status other than `planned` is rejected with a validation outcome (exit code 4) that names the current status. No write occurs and the file stays byte-identical.
6. **Unknown IDs are not found.** Closing an unknown release ID fails with exit code 3.
7. **Output.** Text mode prints the release ID, matching `release create`; `--json` emits exactly one object `{"id": <id>, "status": "released"}`.
8. **One write path.** The transition goes through the shared atomic record writer (`record::store::write_atomic`, ADR-0013). No direct file write is added in the release path.
9. **Not archiving.** `released` represents delivery completion and is distinct from archiving; releases are not archived, and no release is moved to a document `archived/` folder.

## Resolved open questions

1. **Mechanism to mark a release as released (OQ-1, resolved by TASK-118).** The transition is `papyrus release close <id>`. It is implemented, covered by CLI integration tests, and is the only transition command.
2. **Stored vs. derived state (OQ-2, resolved by TASK-118).** `released` is a stored front-matter attribute, not derived from milestone or task completion.
3. **README update (resolved by TASK-118).** The `papyrus(1)` manpage in `README.md` matches this specification: COMMANDS → release lists `papyrus release close <id>` with its own entry, and STATUSES → Releases reads `planned -> released` and names `papyrus release close <id>` as the only transition. BR-001/BR-007 excerpts already use `planned -> released`.

## Affected artifacts

- `src/release.rs` — `ReleaseCmd::Close`, `close()`, and `status: planned` on create.
- `src/registry.rs` — `Verb::Close` declared in `RELEASE_VERBS`.
- `src/browser/render.rs` — browser lifecycle text and command snippet.
- `docs/specifications/milestone-and-release-management.md` — lifecycle, create, and close behavior.
- `tests/release_close_cli.rs` — close success, preservation, rejection, JSON, and legacy-status coverage.
- `docs/requirements/business/REQ-03.002.md`, `docs/business-rules/BR-007-release-command-behavior.md`, and `docs/requirements/business/REQ-07.001.md` already describe the implemented behavior; this specification does not modify them.

## Traceability

| Claim | Source |
|---|---|
| Release commands are create/list/view/edit/close only | `README.md`, `papyrus(1)` → COMMANDS → release; BR-007 |
| No release archive/restore/delete/status command is documented | BR-007; REQ-03.002 |
| Release lifecycle is `planned -> released` | `README.md`, `papyrus(1)` → STATUSES → Releases; REQ-03.002; REQ-07.001; BR-007 |
| `close` is the only transition | REQ-03.002; BR-007 |
| Status is stored in front matter and seeded `planned` on create | REQ-05.010; TASK-118 |
| Missing/empty status reads as `planned`; other values are refused | TASK-118 implementation |
| Writes use the shared atomic writer | ADR-0013; TASK-118 |

## Assumptions

- Assumption: `README.md` and `papyrus(1)` remain the behavior source for `papyrus release` (BR-007); no command beyond `close` is inferred.
- Assumption: "released" is a delivery-completion state, not a record-management/archive state.
- Assumption: Requirements, business-rule, and ADR edits remain with their owning agents; this specification records only the implemented behavior.
