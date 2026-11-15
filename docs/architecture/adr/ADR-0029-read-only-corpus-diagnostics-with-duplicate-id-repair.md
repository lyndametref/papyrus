# Read-Only Corpus Diagnostics (`papyrus check`) with Duplicate-ID Repair

## Status

Proposed (2026-11-15) — awaiting owner validation. Do not implement before acceptance.

## Date

2026-11-15

## Context

REQ-01.008 adds `papyrus check`: a read-only diagnostic over the project corpus and configuration that reports duplicate record IDs, self-referential task dependencies, dependency cycles, dangling `milestone`/`release` references, front-matter/section conflicts, and configuration validity. `--fix` repairs duplicate record IDs **only**; `--yes` skips its confirmation; any finding exits 4; an invalid configuration is reported first and the checks that need a valid configuration are skipped; the command is CLI-only and honours the global `--json`.

The current architecture does not accommodate it in four places:

1. **Startup is fail-closed before dispatch.** `lib.rs::run` loads and fully validates the configuration (ADR-0019) and builds the registry (ADR-0017) for every command except `init` and `config`. A project with an invalid configuration therefore cannot run `check` at all — the exact state `check` exists to diagnose. REQ-01.004 lists the configuration "repair surfaces" as `config get|set|unset|list` and `init --force`; `check` must join that set.
2. **A non-zero exit cannot carry a normal report.** `main.rs` prints `CommandOutput` and exits 0 on `Ok`, and on `Err(AppError)` prints the error (to stderr, or as `{"error":{...}}` with `--json`) and exits `err.code()`. A findings report is output, not an error, and must stay structured under `--json`.
3. **There is no read-only corpus enumeration.** `record::resolve::list_in_dir` calls `store::relocate_if_archived`, which *writes* (it moves a mis-placed record), and `browser::records::build_index` is a browser projection tied to a status filter. `record::resolve::duplicate_ids` exists and is unused; it is family-scoped and takes caller-supplied directories.
4. **There is no ID-rewrite primitive.** `record::store` writes content, moves records, archives, and deletes; nothing rewrites a record's identity in one crash-safe step, and ADR-0013 decision 7 forbids record writing outside `src/record/`.

Forces: one parser, one projection, one writer (ADR-0013, ADR-0018); no new crate (ADR-0005); exit codes 0/1/2/3/4 and the `--json` error object are fixed by REQ-01.004 and ADR-0016 constraint 4; the command tree is registry-derived, so `check` is a built-in name that must be reserved (ADR-0016 decision 2, ADR-0019); interactive confirmation must work without a terminal crate (ADR-0015 precedent); and `--fix` may not become a second migration mechanism.

## Decision

1. **`check` is a built-in command with two flags and no subcommands.** `BuiltinCmd::Check { fix: bool, yes: bool }`; `check` is added to `cli::BUILTIN_COMMANDS` (reserved name, ADR-0019). `--yes` without `--fix` is a usage error (exit 2). `--fix` and `--yes` are not global. `--json` applies as for any output-producing command. No browser route, no JSON API, no library exposure beyond `run`.

2. **`check` is exempt from fail-closed startup, and loads its own configuration.** `lib.rs::run` gains an `is_check` branch beside `is_config`: the command tree is built with no registry (`registry_opt = None`, so only built-ins exist and `check --help` works), and the handler then performs its own two-stage load:

   - Stage A: load and parse the configuration. A YAML parse failure (including duplicate keys) becomes a `config-invalid` finding. Otherwise run `Config::validate(Some(project_dir))` and split its newline-joined diagnostics into one finding per line.
   - Stage B: only when stage A produced no finding, build `ArtifactRegistry` and run the corpus scanners. An invalid registry build is itself a `config-invalid` finding.

   This is the only command allowed to proceed past an invalid configuration, and the exception is deliberate: it is the diagnostic surface for that state. It extends REQ-01.004's repair-surface list and needs no change to ADR-0016's dispatch order.

3. **A findings run is a successful run with a non-zero exit status.** `CommandOutput` gains `pub exit_code: i32` (0 by default) and main prints the output before exiting with it; `AppError` keeps its current mapping, so a runtime I/O failure is still exit 1 and a usage error still exit 2. Under `--json` the findings report is the JSON object (never the `{"error":…}` envelope). `check` exits 4 whenever the scan produced at least one finding — **including after successful repairs** — so a clean second run is the verification step. Exit codes: 0 no findings, 4 findings, 1 I/O failure, 2 usage.

4. **The scanners are read-only and live in `src/check/`.** `check::scan`, `check::findings`, `check::fix`, driven by `check::run(ctx, json, fix, yes)`. Read-only is a hard guarantee:

   - Enumeration uses a new upstream primitive in the record layer, `record::scan::family_files(ctx, descriptor) -> Vec<PathBuf>`, which lists the descriptor's active, archived, and read-only legacy directories without relocating, migrating, or adopting front matter.
   - `check` may not call `list_in_dir` (it writes), `store::archive`/`set_status`/`relocate`, `record::migrate`, or any `fs::write` outside `check::fix`.
   - A test runs `check` over a fixture corpus and asserts every file's bytes and mtime are unchanged.

5. **Detection is descriptor-driven and uses the existing parser and projection.** The scanner iterates `registry.descriptors()` — every category, enabled **and disabled**, because disabled types still own files on disk — and reads through `record::document` / `record::frontmatter` only. No new parser, no second metadata projection.

   | Kind (stable JSON value) | Detection |
   |---|---|
   | `duplicate-id` | Canonical ID per family (front-matter `id`, else stem) seen more than once across the family's active, archived, and read-only legacy directories. Grouped families compare group records and member records in their own spaces (`REQ-001` vs `REQ-01.008`). |
   | `self-dependency` | A task whose `dependencies` contain its own ID. |
   | `dependency-cycle` | A cycle in the task dependency graph, reported once per cycle with the member IDs in canonical order. Algorithm: iterative depth-first search with a colour map; no recursion, no crate. |
   | `dangling-milestone` | A task whose `milestone` does not resolve to an existing milestone. |
   | `dangling-release` | A milestone whose `release` does not resolve to an existing release. |
   | `frontmatter-conflict` | A value carried by both front matter and the body disagrees: `id`/`title` against the `# <ID> <title>` heading, or a retired metadata section against its front-matter field (ADR-0012). Agreement is not a finding; REQ-01.008 reports disagreement. |
   | `config-invalid` | Stage A/B diagnostics, one finding per diagnostic line, with the offending key path as the location. |

   A task dependency that resolves to no task is **not** a finding: REQ-01.008 closes the dangling-reference category to `milestone` and `release`. That gap is recorded as an open question rather than implemented silently.

6. **Findings are deterministic and carry a stable shape.**

   ```rust
   pub struct Finding {
       pub kind: &'static str,          // "duplicate-id", …
       pub family: Option<String>,      // "task", "req", "tech-spec", …
       pub subject: Option<String>,     // record ID, or the config key path
       pub locations: Vec<String>,      // project-relative paths, sorted
       pub detail: String,
       pub fixable: bool,
   }
   ```

   Output order is `(kind, family, subject, locations)`, all sorted, so text and JSON are stable for golden tests. Text form: one line per finding, `<kind>: <family> <subject> — <detail> (<locations>)`; then a repair summary when `--fix` ran. JSON form:

   ```json
   { "findings": [ … ], "repaired": [ { "kind": "duplicate-id", "family": "task",
       "id": "TASK-003", "path": "backlog/tasks/task-3.md", "new_id": "TASK-014" } ] }
   ```

7. **`--fix` repairs duplicate IDs only, through the single write path, with a deterministic owner rule.**

   - **Keeper rule** (deterministic, independent of directory iteration order): within a duplicate group the record with the earliest `created_date` keeps the ID; ties break on the lowest project-relative path; remaining ties cannot occur (two records cannot share a path).
   - **Reassignment rule**: every other record in the group gets the next free number of its own family and shape from `record::identity::allocate_next`, computed over every observed number in the family's active, archived, and legacy directories (ADR-0010 decision 5). Prefix and padding come from the descriptor, never from the file.
   - **Write primitive**: a new `record::store::reassign_id(ctx, path, new_id) -> Result<PathBuf>` builds the patched content with `record::patch` (front-matter `id`, plus the body heading where the type keeps one per REQ-05.010), writes the **new** path atomically, then removes the old path. Order is fixed: new file first (its ID is unique, so a crash leaves a valid record), old file second (a crash leaves a duplicate that the next run re-reports and re-repairs). `updated_date` is refreshed once per write (ADR-0013 decision 9). No other module writes.
   - **Refusals** (`fixable: false`, reported and left untouched): a record whose front-matter `id` and file stem disagree (`ResolvedRecord::mismatch`) — the duplicate is not unambiguous; a duplicate whose keeper or member lives in a read-only legacy directory (repairing would require a migration that REQ-01.008 forbids); duplicate **group** IDs in a grouped family (reassigning a group ID would renumber its members and their dotted IDs, which is a structural migration) — member-level duplicates in those families are repaired.
   - **Out of scope, by requirement**: dangling references, cycles, self-dependencies, front-matter conflicts, invalid configuration, and legacy `deprecated/` migration are never repaired.
   - **Idempotent**: a second `--fix` on a repaired corpus performs no write.

8. **Confirmation reuses stdin, not a terminal crate.** `--fix` prints the planned repairs, then reads one line from stdin: `y`/`yes` confirms, anything else cancels with no write. `--yes` skips the prompt. When stdin is not a terminal and `--yes` is absent, `check` writes nothing and exits 2 with a message naming `--yes` — a deterministic contract for scripts and CI rather than a hang. No raw mode, no `stty`, no new dependency (ADR-0015 is precedent for the "std only" rule).

9. **Concurrency safety is per repair, not per run.** The scan is a snapshot with no locking. Immediately before each repair the file is re-read, its version token compared with the token captured during the scan, and the repair skipped (with a finding) if the file changed; a concurrent writer therefore costs a skipped repair, never a lost update (ADR-0006's token, used defensively on the CLI side as well).

10. **No new crate, no browser surface, no background work.** The scanner uses `std::fs`, `std::collections`, the existing `serde_json` for `--json`, and the existing parsers. It builds no index file, caches nothing across runs, and writes nothing except the repairs of decision 7.

11. **Documentation follows the command.** `papyrus(1)` gains the command, its flags, the finding kinds, the exit-status paragraph, and the read-only guarantee; the README command section is updated by the tech writer. The man page is still unbuilt (ADR-0021 deferred), so this ADR records the text requirement without creating a man-page deliverable.

## Consequences

**Positive:**

- Configuration damage stops being a dead end: the one command that explains a broken configuration runs while it is broken.
- Findings have a stable machine contract, so CI can gate on `papyrus check --json` and scripts can distinguish "clean", "dirty", and "failed".
- The read-only guarantee is architectural (a dedicated enumeration primitive and a test on bytes and mtimes), not a convention.
- Duplicate repair reuses the existing identity allocator, so repaired IDs follow the same prefix, padding, and grouping rules as newly created records.
- No second migration path is created: the refusals of decision 7 are explicit.

**Negative:**

- `CommandOutput` gains a field, so ~50 struct literals change and `main.rs` gains an exit-code path that is not an error path.
- `check` is the second command to bypass the fail-closed configuration gate, which weakens "the configuration is always valid before dispatch" from a global invariant to a rule with two named exceptions.
- `--fix` can leave references pointing at the *keeper* of a reassigned ID, because reference rewriting is out of scope — the repair is safe for identity uniqueness and unsafe for reference intent. This is reported, not fixed (see risks).
- Scanning every family in every directory makes `check` the slowest command on a large corpus (no index, no cache), and it re-reads files that other commands have already parsed.
- Duplicate group IDs in `req`/`nfr` remain unfixable, so `--fix` cannot reach zero findings on a corpus with that damage.

**Risks and mitigations:**

- *A repair changes which record a reference means*: highest-severity semantic risk. The confirmation prompt lists every reassignment, the report lists affected reference values read from other records, and the keeper rule favours the oldest record (the one references most likely target). A test asserts a document with a reference to a duplicated ID is reported before and after a repair.
- *Crash during a repair leaves two files*: fixed write order (new first, old second) plus idempotent re-run; a test interrupts between the two steps and asserts the corpus stays readable.
- *`check` accidentally writes*: the read-only test on bytes and mtimes, plus a grep rule that `src/check/` contains no `fs::write` and no `store::` mutator outside `fix.rs`.
- *Exit 4 after successful repairs surprises users*: documented in the help text and the release notes; a clean second run exits 0, which is the intended loop.
- *Findings format churn*: `kind` strings and the JSON keys are declared a stable public contract for the release; additions are allowed, renames are not.
- *Scanners drift from the writer*: every scanner reads through `record::document`/`record::frontmatter` and the registry, and a characterization test per finding kind is built from a fixture written by the real writer.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| `check` as a built-in with its own tolerant load, read-only scanners, and a `reassign_id` primitive (this ADR) | Selected | The only shape that satisfies "diagnose an invalid configuration", "read-only by default", "repair through the single write path", and the fixed exit-code contract. |
| Reuse `AppError::Validation` to produce exit 4 | Rejected | The report would go to stderr and `--json` would emit the error envelope, losing the findings structure; findings are output, not an error. |
| A dedicated `--check` mode on each family command | Rejected | Duplicates six scanners per family and cannot report project-wide duplicates or cross-family references. |
| Run `check` inside the browser (a page with a "run diagnostics" button) | Rejected | REQ-01.008 makes it CLI-only in this release, and the browser has no write/diagnostic execution surface. |
| Emit a findings file (`.papyrus-check.json`) | Rejected | A new on-disk artifact, a new format, and a new staleness problem; `--json` already covers automation. |
| Repair by deleting the newer duplicate | Rejected | Destroys a record and its references; ID reassignment preserves content. |
| Repair references too | Rejected | REQ-01.008 scopes `--fix` to duplicate IDs; reference rewriting is a different, riskier migration. |
| Use `record::resolve::list_in_dir` for enumeration | Rejected | It relocates mis-placed records (writes) and would break the read-only guarantee. |

## Open Questions

1. **Exit code after a fully successful `--fix`**: current decision 4 per REQ-01.008's literal "one or more findings" wording; the alternative is 0. Recommended: keep 4.
2. **Should duplicate **group** IDs in grouped families be repairable?** Recommended: no in this release (renumbering members is a migration). Owner: product.
3. **Should an unresolved task dependency be reported?** REQ-01.008 excludes it. Recommended: add it as a `dangling-dependency` finding in a follow-up requirement, since it is the same class of defect.
4. **Should disabled document types be scanned?** Recommended: yes (their files exist and can collide); confirm that scanning a disabled type is not surprising.
5. **Confirmation prompt granularity**: one prompt for all repairs (recommended) versus per record. Per record is unusable on a corpus with many duplicates.
6. **Is `--fix` allowed to refresh `updated_date`?** Recommended: yes, one clock per write (ADR-0013 decision 9); the alternative leaves a repair invisible in the record's history.

## Downstream constraints

1. `check` may not run before or instead of configuration validation for any other command; the exemption is scoped to `check` in `lib.rs::run`.
2. Every scanner reads through `record::document`/`record::frontmatter`; no new parser, no second projection, no browser module reuse.
3. `src/check/` performs no write outside `check::fix`, and `check::fix` writes only through `record::store::reassign_id`.
4. `record::scan::family_files` is read-only; it must never relocate, migrate, or adopt front matter.
5. Finding `kind` strings and JSON keys are a stable contract for the release; additions are allowed, renames and removals are not.
6. Exit codes remain 0/1/2/3/4; `CommandOutput::exit_code` is the only new non-error exit path and is used only by `check`.
7. `--fix` never repairs anything but duplicate IDs and never migrates a legacy `deprecated/` directory.
8. `papyrus(1)`/README text for the command, its flags, the finding kinds, and the exit behaviour is owned by the tech writer; the requirement text is not edited by this ADR.

## References

- `docs/requirements/business/REQ-01.008.md` — run project diagnostics.
- `docs/requirements/business/REQ-01.004.md` — global options, exit codes, repair surfaces.
- `docs/requirements/business/REQ-01.005.md` — stable record IDs and the single ID space.
- `docs/requirements/business/REQ-05.010.md` — front matter as the authoritative metadata source.
- `docs/requirements/business/REQ-02.001.md`, `REQ-03.001.md` — task dependencies, milestone `release` reference.
- `docs/requirements/business/REQ-07.002.md` — configuration validity and repair surfaces.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — version tokens and atomic writes.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — identity allocation over all locations.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — authoritative fields and retired sections.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — single parser, patcher, identity, writer; no-drift rule.
- `docs/architecture/adr/ADR-0015-cursor-driven-init-list-via-stty-raw-mode.md` — std-only interactive input precedent.
- `docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md` — command tree, startup phases, reserved names.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md`, `ADR-0019-fail-closed-document-type-configuration-validation.md` — registry and validation.
- `docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md`, `ADR-0023-consolidated-record-layout-and-field-map.md` — archive locations and milestone/release lifecycles used by the scanners.
- `docs/architecture/document-type-config-validation-contract.md` — `Config::validate` diagnostic format.
- Implementation: `src/check/` (new), `src/record/scan.rs` (new), `src/record/store.rs` (`reassign_id`), `src/cli/builtin.rs`, `src/cli/dispatch.rs`, `src/lib.rs`, `src/main.rs`.
