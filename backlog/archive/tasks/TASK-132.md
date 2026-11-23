---
id: TASK-132
title: >-
  Implement the read-only `check` diagnostic command with descriptor-driven
  scanners
status: Done
assignee:
  - @rust-dev
created_date: '2026-11-15T16:11:00Z'
updated_date: '2026-11-15T17:17:00Z'
labels:
  - rust
  - cli
  - diagnostics
milestone: MS-022
dependencies: []
references:
  - docs/requirements/business/REQ-001.008.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - src/lib.rs
  - src/main.rs
  - src/commands.rs
  - src/registry.rs
  - docs/specifications/project-diagnostics-check.md
priority: high
ordinal: 41500
---


## Description
**Why.** REQ-001.008 adds `papyrus check` and ADR-0029 is Accepted. Three gaps block it today: `lib.rs::run` validates the configuration fail-closed before dispatch, so the command cannot diagnose a broken configuration; `CommandOutput` cannot carry a non-zero exit, so findings would have to be reported as an error; and no read-only corpus enumeration exists, because `record::resolve::list_in_dir` relocates records and therefore writes.

**What.** A `check` built-in command, a `src/check/` module with descriptor-driven read-only scanners, a tolerant self-load, a stable findings model, exit 4, and `--json`. The implementation contract is `docs/specifications/project-diagnostics-check.md` (finding model, detection rules, output shapes, exit codes); ADR-0029 supplies the rationale.

**In scope.**
- `BuiltinCmd::Check` (no flags yet) wired into dispatch and the command tree, with no registry in the tree so `check --help` works on a broken configuration. Flags are added by TASK-138; the reserved slug in `BUILTIN_COMMANDS`, the help text, and the `--help` golden are TASK-139.
- `lib.rs::run` gains an `is_check` branch beside `is_config`: the tree is built with no registry.
- `Context` construction for `check` does not require the configuration file to exist; a configuration path outside the project is still exit 1.
- Stage A: tolerant self-load through `commands::load_effective_config_tolerant` (the loader used by `config get|set|unset|list`). Any load error becomes exactly one `config-invalid` finding; on success run `Config::validate(Some(project_dir))` and emit one finding per non-empty diagnostic line.
- Stage B split. B1: the four static families (task, milestone, release, decision) always scan once a configuration value exists, valid or not. B2: document families from `registry.descriptors()` (enabled **and** disabled) scan only when validation passed and the registry builds; a registry build failure adds one `config-invalid` finding and skips B2.
- A new read-only primitive `record::scan::family_files(ctx, cfg, descriptor)` listing active, archived, read/import, and read-only legacy directories without relocating, migrating, or adopting front matter. Files are filtered to the descriptor's extension (`html` for `ui-mockup`, `md` otherwise).
- Finding kinds and their exact semantics: `duplicate-id` (canonical ID per family, grouped-family group and member spaces apart, `fixable` computed from the ADR-0029 refusal rules but never acted on), `self-dependency`, `dependency-cycle` (iterative DFS, one finding per cycle, canonical member order), `dangling-milestone`, `dangling-release`, `frontmatter-conflict`, `config-invalid` (one per diagnostic line; `locations` = configuration path, `detail` = the line verbatim, `subject` = `None`).
- `frontmatter-conflict` scope: retired task sections (`status`, `assignee`, `milestone`, `labels`, `dependencies`) and the milestone `release` section compared against the front matter, read the way the legacy migration reads them and compared as sets for list fields; an unreadable or duplicated section is always a conflict; agreement is not a finding; a field the front matter does not carry is not a finding. The `# <ID> <title>` heading check applies to document records only, because tasks and milestones keep no heading. Decisions are excluded entirely.
- `CommandOutput.exit_code` (`i32`, default 0); `main` prints the output and exits with it; the `AppError` mapping is unchanged (I/O 1, usage 2, not-found 3, validation 4).
- Exit 4 whenever the run produced at least one finding, 0 otherwise; `--json` emits `{"findings":[…],"skipped":[…],"repaired":[]}`, never the error envelope.
- Text report: one `<kind>: <family> <subject> — <detail> (<locations>)` line per finding, `no findings` on a clean run, and a `skipped: …` line naming skipped checks.
- Deterministic order `(kind, family, subject, locations)`.
- `deprecated/` is never read or inventoried and produces no finding; a task dependency that resolves to no task is not a finding.
- Tests: one characterization fixture per finding kind written by the real writer; invalid-configuration run; deterministic ordering golden; text and `--json` goldens; exit-code tests; a read-only test that every scanned file keeps its bytes and mtime.

**Out of scope.** `--fix` and `--force` (TASK-138), reserved slug and help and README (TASK-139, TASK-140), dependency-reference findings, a browser surface, any migration of legacy folders, and any repair or rewriting of references.

**Assumptions.**
- Assumption: ADR-0029 decision 5 is the detection contract and decision 6 the finding shape and order; where decision 2's "Stage B only when stage A produced no finding" contradicts REQ-001.008 and the independent-checks rule, REQ-001.008 wins (see open questions).
- Assumption: `config-invalid` is one finding per diagnostic line, `subject` is `None`, `locations` is the project-relative configuration path, and `detail` keeps the key path verbatim.
- Assumption: a missing configuration file is a load error and therefore one `config-invalid` finding with exit 4, not a runtime exit 1 as `config list` behaves.
- Assumption: `deprecated/` exclusion is guaranteed by `resolve_read_directories` plus a defensive path-component filter, so no scanner needs a `deprecated` special case.
- Assumption: HTML mockup records have no front matter; they participate in `duplicate-id` by stem and in no other kind.
- Assumption: a family whose configured directory cannot be resolved contributes no files and is not a second finding, because Stage A already reports the invalid directory.
- Assumption: a task dependency that resolves to no task stays an open question recorded by ADR-0029, not a finding.

**Open questions (do not block implementation; defaults are fixed above and in the specification).**
1. ADR-0029 decision 2 says Stage B runs "only when stage A produced no finding", while REQ-001.008 and this ticket require the independent checks to run under an invalid configuration. The requirement wins; the ADR sentence should be corrected (owner: architect).
2. ADR-0029 decision 5 calls the offending key path the `config-invalid` location, while decision 6 makes it the `subject`. This ticket uses the configuration file as `locations` and `None` as `subject`, keeping the key path in `detail`, because a joined diagnostic line has no reliable key-path boundary. Confirm or amend.
3. No reserved-name entry for `check` lands in this ticket. Until `check` is added to the built-in name list, a project that configures a document type named `check` fails while the command tree is built. Accepted as a transient seam between consecutive changes in this milestone.

## Acceptance Criteria
- [x] `papyrus check` runs on a valid project and prints a plain findings report, exiting 0 when there are no findings.
- [x] Scanners read every registry descriptor, including disabled document types, through the existing parsers and projection, and never relocate, migrate, or adopt a record.
- [x] All seven finding kinds are detected: duplicate-id, self-dependency, dependency-cycle, dangling-milestone, dangling-release, frontmatter-conflict, and config-invalid.
- [x] Duplicate IDs are compared per family, keeping grouped-family group and member spaces apart, across decisions and every configured document type.
- [x] Front-matter conflicts cover the body heading and the retired task and milestone sections; agreement is not reported, an unreadable section is, and decision body sections are excluded.
- [x] A broken configuration does not abort the run: configuration findings are reported, the checks that need document_types are skipped and named as skipped, and the independent checks still run.
- [x] deprecated/ folders are neither read nor inventoried and produce no finding, and a task dependency that resolves to no task is not a finding.
- [x] CommandOutput.exit_code is honoured by main: a run with findings exits 4, a clean run exits 0, and --json emits the findings object rather than the error envelope.
- [x] Findings are deterministically ordered and both output forms match their golden fixtures.
- [x] A read-only test asserts that every scanned file keeps its bytes and modification time.

## Definition of Done
- [x] #1 The new check integration tests, fixtures, and goldens pass, and the full cargo test suite passes.
- [x] #2 cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
- [x] #3 src/check/ contains no fs::write, no record::store mutator, no record::migrate path, and no record::resolve::list_in_dir call; the read-only test asserts unchanged bytes and mtimes for every scanned file.
- [x] #4 Finding kinds, deterministic ordering, text and --json shapes, and exit codes 0/4 match docs/specifications/project-diagnostics-check.md exactly.
- [x] #5 Existing command behavior is unchanged apart from the mechanical CommandOutput field addition, and tests/golden/cli-help/top.json is regenerated for the new subcommand.
- [x] #6 No new crate or dependency is added; requirements, business rules, and ADRs are not edited.


## Implementation Notes
Contract, in one place (full detail in `docs/specifications/project-diagnostics-check.md`):
- Finding: `kind: &'static str`, `family: Option<String>`, `subject: Option<String>`, `locations: Vec<String>` (project-relative, `/` separators, sorted, deduped), `detail: String`, `fixable: bool`.
- Kinds: `config-invalid`, `dangling-milestone`, `dangling-release`, `dependency-cycle`, `duplicate-id`, `frontmatter-conflict`, `self-dependency` (stable strings).
- Order: `(kind, family, subject, locations)`, absent options first, all ascending.
- Text: `<kind>: <family> <subject> — <detail> (<locations>)` per finding (omit `family `/`subject ` when absent; locations joined with `", "`); `no findings` on a clean run; a trailing `skipped: <ids joined with ", ">` line when any check was skipped.
- JSON: `{"findings":[{kind,family,subject,locations,detail,fixable},…],"skipped":[…],"repaired":[]}`, top level, never the error envelope; all three keys always present.
- Exit: 0 no findings, 4 at least one finding, 1 I/O failure, 2 usage.
- Skipped ids: `duplicate-id`, `duplicate-id:document-types`, `self-dependency`, `dependency-cycle`, `dangling-milestone`, `dangling-release`, `frontmatter-conflict`, `frontmatter-conflict:document-records`.
- Details: duplicate-id `"<n> records share this ID"`; self-dependency `"dependency list contains <ID>"`; dependency-cycle members sorted and joined with `" -> "`; dangling-milestone `"milestone <value> does not resolve to a milestone"`; dangling-release `"release <value> does not resolve to a release"`; frontmatter-conflict `"section ## <Name> \"<body value>\" disagrees with front matter <key> \"<front-matter value>\""` or `"heading <id|title> \"<heading value>\" disagrees with front matter <id|title> \"<front-matter value>\""`; config-invalid the diagnostic line verbatim.

1. `src/record/scan.rs` (new) + `pub mod scan;` in `src/record/mod.rs`.
   - `pub fn family_files(ctx: &Context, cfg: &Config, descriptor: &RecordDescriptor) -> Result<Vec<PathBuf>, AppError>`.
   - Directories come from `descriptor.resolve_read_directories(cfg, &ctx.project_dir)`; a directory that does not exist yields nothing; other read errors return `AppError::Runtime` (exit 1).
   - List regular files whose extension is `md` or `html`; skip any path with a `deprecated` component; canonicalise through `Context::ensure_within`; dedupe and sort by project-relative path.
   - No write, no relocate, no migrate, no adoption: it never calls `store::relocate_if_archived`.
   - Note: the signature carries `cfg` because `check` scans with its own tolerant configuration and must not reload the file; ADR-0029 shows the same primitive without `cfg`.

2. `src/check/findings.rs` (new).
   - `pub struct Finding` as above plus `FindingKind` string constants.
   - `pub fn sort(findings: &mut Vec<Finding>)`, `pub fn render_text(findings: &[Finding], skipped: &[&'static str]) -> String`, `pub fn to_json(findings: &[Finding], skipped: &[&'static str]) -> serde_json::Value` (`repaired` always `[]`).
   - `pub fn config_finding(ctx: &Context, detail: String) -> Finding` with `locations = [project-relative config path]`, `family = None`, `subject = None`, `fixable = false`.

3. `src/check/scan.rs` (new).
   - `pub struct FamilyScan<'a> { name: String, descriptor: &'a RecordDescriptor, prefixes: Vec<String>, dotted: bool }`.
   - `static_families(cfg) -> Vec<FamilyScan>`: `record::descriptor::descriptors()` (task, milestone, release, decision). Decision is one family merging all `cfg.decision_types` prefixes, so its directory is enumerated once.
   - `document_families(registry, cfg) -> Vec<FamilyScan>`: `registry.descriptors()` in category `Document`, family name = slug, descriptor = the registry descriptor's `record` (`Some` for every document type, enabled or disabled). Files filtered to `html` when `content_model` is `HtmlMockup`, else `md`.
   - `record_meta(cfg, family, path) -> Result<Option<RecordMeta>, AppError>`: `RecordMeta { path, key, display, frontmatter_id: Option<String>, mismatch: bool }`. Read content with `fs::read_to_string`; canonical identity is the front-matter `id` when it parses for a family prefix (via `record::identity::parse_id_parts`), else the stem; a stem that does not parse is not a member. `mismatch` = front-matter id present and its key differs from the stem key. HTML mockups: stem only.
   - `duplicate_ids(families, ctx, cfg) -> Vec<Finding>`: group by `(family, key)`; one finding per key with 2+ paths; `fixable` per the ADR-0029 decision 7 refusals (no mismatch member, no member in a read/import or legacy directory, family not a grouped-family group ID). Never repair.
   - `self_dependencies(families)`: task `dependencies` containing the task's own key.
   - `dependency_cycles(tasks)`: build the graph from dependencies that resolve to a scanned task key; iterative DFS with a colour map; report each cycle once from the grey node encountered; skip self-edges (already reported as `self-dependency`).
   - `dangling_references(tasks, milestones, releases)`: `dangling-milestone` and `dangling-release` against the scanned canonical key sets.
   - `frontmatter_conflicts(cfg, family, records)`: parse with `record::document::parse` and `record::frontmatter`; retired sections for task/milestone using the legacy reader rules (`record::migrate`'s `section_scalar`/`section_list` — expose them as `pub(crate)` if needed, or add a `pub(crate) fn legacy_section_kind(descriptor) -> &'static [(&'static str, &'static str, bool)]` built from the existing `retire_legacy_body_ops` table); heading check for document families only; decisions excluded.

4. `src/check/mod.rs` (new) + `pub mod check;` in `src/lib.rs`.
   - `pub fn run(ctx: &Context) -> Result<CommandOutput, AppError>`.
   - Stage A: `commands::load_effective_config_tolerant(&ctx.config_path, &ctx.project_dir)`. `Err` → one `config-invalid` finding, `skipped` = every corpus check id, exit 4.
   - On `Ok(cfg)`: `cfg.validate(Some(&ctx.project_dir))` → `Err(lines)` adds one `config-invalid` finding per non-empty line and sets `skipped = ["duplicate-id:document-types", "frontmatter-conflict:document-records"]`.
   - B1 always runs on `static_families(&cfg)`.
   - B2 runs only when validation passed: `ArtifactRegistry::build(&cfg)` → `Ok` scans `document_families`; `Err(e)` adds one `config-invalid` finding and the document skipped ids.
   - Sort, render text and (when `ctx.json`) JSON, set `exit_code = if findings.is_empty() { 0 } else { 4 }`.

5. `src/lib.rs`.
   - `CommandOutput` gains `pub exit_code: i32` and `#[derive(Default)]`, so existing struct literals can add `..Default::default()`.
   - Phase 2: `let is_check = first.as_deref() == Some("check");` and a branch returning `(None, None)` beside `is_init`/`is_config`, so the tree is built with no registry.
   - Do not add `check` to `BUILTIN_COMMANDS` (TASK-139); accept the documented seam.

6. `src/cli/builtin.rs` and `src/cli/dispatch.rs`.
   - `BuiltinCmd::Check` variant with a one-line doc comment (help text is TASK-139) and no flags.
   - Dispatch arm: build a check context and call `crate::check::run(&ctx)`.
   - `build_check_context` mirrors `build_context` but does not require the configuration file to exist; a configuration path outside the project stays exit 1.

7. `src/main.rs`.
   - `Ok(out) => { print_output(&out, json); exit(out.exit_code); }`; the `Err` path is unchanged.

8. Mechanical updates.
   - Every `CommandOutput { text, json }` literal (about 80, `src/` and `tests/`) gains `..Default::default()`; no behavior change.
   - Regenerate `tests/golden/cli-help/top.json` (`PAPYRUS_UPDATE_CLI_HELP=1 cargo test --test cli_help_golden`) because the top-level help now lists `check`. The dedicated `check --help` golden stays with TASK-139.

9. Tests (`tests/check_cli.rs`, `tests/golden/check/`).
   - Characterization: per finding kind, build the corpus with the real commands (`init`, `task create`, `milestone create`, `release create`, `req create`, `tech-spec create`), then create the defect (duplicate file copy, self dependency, two-task cycle, missing milestone/release, retained retired section, hand-edited heading) and assert the finding, its fields, and the exit code.
   - Valid configuration: clean run prints `no findings`, exits 0, `--json` yields `{"findings":[],"skipped":[],"repaired":[]}`.
   - Invalid configuration (for example `statuses: []` or a malformed YAML file): exit 4, `config-invalid` findings, the skipped line names exactly the two document checks, and task-level findings (`self-dependency`, `duplicate-id` for tasks) are still reported. A malformed file names every corpus check.
   - Determinism: a corpus with two findings in each kind produces byte-identical output on two runs and matches the ordering golden.
   - Read-only: snapshot `(bytes, mtime)` of every file in the project before and after a run that includes archived and legacy-aliased records; assert unchanged, and assert archived files stay in place.
   - Exclusions: a duplicate inside `docs/<type>/deprecated/` produces no finding; a dependency naming a non-existent task produces no finding; a decision with a divergent `## Status` and a frozen ADR heading produces no `frontmatter-conflict`.
   - `--json` shape: keys, order, `null` for absent `family`/`subject`, and no `{"error":…}` envelope, with exit 4.
   - Exit codes: 0, 4, and 2 for an unknown flag; an I/O case (unreadable directory) asserts 1 and is skipped when running as root.

10. Verification.
   - `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
   - `rg "fs::write|store::(archive|set_status|delete|relocate)|list_in_dir" src/check/` returns nothing.
   - Keep `docs/specifications/project-diagnostics-check.md` in step with the implementation; requirements and ADRs are not edited.


## Notes
Implemented src/check/{mod,findings,scan}.rs, record::scan::family_files, CommandOutput.exit_code, and CLI dispatch wiring per docs/specifications/project-diagnostics-check.md. Quality review: pass, all 10 AC and 6 DoD verified independently (873 tests pass, clippy/fmt clean, read-only grep clean, manual exit-code matrix 0/4/1/2).

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Read-only 'papyrus check' shipped: descriptor-driven scanners detect all seven finding kinds over enabled and disabled document types without writing to disk; invalid configurations are reported instead of aborting the run, with document-type-dependent checks named as skipped. Deterministic ordering, text and --json output match goldens; exit 4 on findings, 0 when clean. Deferred by scope to TASK-138/139/140: --fix/--force, the reserved built-in slug and check --help golden, README and business rules.
