---
id: TASK-240
title: "Build characterization fixtures and the behavior snapshot corpus"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-21T17:03:47Z'
labels: []
references:
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - TECH-069
  - TC-183
  - TC-184
  - REQ-05.001
  - BR-001
status: Done
assignee:
  - test-manager
milestone: MS-032
dependencies:
  - TASK-239
ordinal: 30
documentation:
  - docs/test-cases
  - tests/fixtures/characterization/corpus
  - tests/golden/cli-help
type: test-design
---

## Description
Context: every later milestone (MS-033 onward) proves equivalence against this corpus, so a missing case becomes an undetectable regression. Extend the existing conventions instead of inventing new ones: `tests/fixtures/characterization/lists/` with `tests/list_characterization_capture.rs` (`#[ignore]` capture) and `tests/list_characterization_replay.rs` (default-suite replay), and `tests/golden/cli-help/` with `tests/cli_help_golden.rs` (byte-for-byte help goldens, regenerated with `PAPYRUS_UPDATE_CLI_HELP=1`). Do not modify `tests/fixtures/characterization/lists/`: it is pinned to commit d60f71c9.

Inputs: MS-032 AC-2 and AC-6; TC-183 (strict bytes, JSON, status codes); TC-184 (semantic HTML for later milestones); REQ-07.003 and REQ-05.001 to REQ-05.014 (uniform title contract); BR-001, BR-009 to BR-014, BR-017 to BR-020.

Scope:

1. Corpus root `tests/fixtures/characterization/corpus/`.
   - `project/`: committed seed project. `papyrus.config.yml` enables the eleven built-in document types the suite already uses (`req`, `nfr`, `glossary`, `rule`, `audit`, `tech-spec`, `ui-spec`, `ui-mockup`, `use-case`, `test-case`, `test-report`) plus one user-defined `note` type (`prefix: NOTE`, `directory: notes/`, `statuses: [active, draft, archived]`), with `id_padding: 3`, and the backlog and decision categories enabled.
   - `project-empty/`: same configuration, no record files, for empty states.
   - One type per DocumentModel: SingleMarkdown = `note` (user-defined) and `rule` (built-in typed); GroupedMarkdown = `req` and `nfr`; SingleHtmlMockup = `ui-mockup`.
   - `REQ-001` group with three members: one member with a resolved reference (`REQ-001.001` to `REQ-002`), one member with the unresolved reference `REQ-999`, one plain member.
   - `REQ-002` group with no members.
   - `NFR-001` group with two members.
   - `MOCK-001` with `status: pending`, `MOCK-002` with `status: approved`, `MOCK-003` archived under `specifications/ui/mockups/archived/` with `status: archived`.
   - One archived front-matter record: `BR-002` under `business-rules/archived/`.
   - Zero-padding boundary: `NOTE-999` (width 3) and `NOTE-1000` (width 4), per `src/record/identity.rs::pad_number`.
   - At least one record per type carrying labels and references; fixed `created_date` and `updated_date` values in every record.

2. `cases.json`: ordered case manifest. Each case carries `id`, `kind` (`cli` or `http`), `argv`, optional `request` `{method, path, body}`, `document_type`, `model`, `renderer` when a page is involved, and `artifacts`. Case ids are stable paths such as `req/create`, `http/list-req`, `http/conflict-rule`.

3. `expected/<case-id>/` artifacts: `record.md` or `record.html` (stored file bytes), `stdout.txt`, `stderr.txt`, `exit.json` (`{"exit": <code>}`), `payload.json` (`--json` stdout or API body), `http.json` (`{"status": <u16>, "headers": [[name, value], ...]}` in wire order), `page.html` (response body). Omit an artifact that the case does not produce.

4. Determinism. Every ISO-8601 UTC timestamp written into a captured artifact is replaced by the literal token `{{TIMESTAMP}}` at capture and replay (`created_date`, `updated_date`, `created`, `updated`). The server emits no `Date` header, so `http.json` needs no normalisation. The capture embeds no wall-clock value, PID, absolute path, or port.

5. Route matrix. For each of the twelve types: `GET /?type=<url key>` (the declared legacy key where one exists, for example `rule` to `business-rule` and `audit` to `audit-record`), `GET /doc/<ID>`, `GET /doc/<ID>/edit`, `GET /api/docs/<slug>`, `GET /api/doc/<ID>`, and every legacy path declared in `Presentation::api_paths` (`src/browser/routes.rs`). Shared surfaces: `GET /` (inventory), `GET /?type=task` (board), `GET /assets/app.css`, `GET /assets/app.js`, `GET /assets/nav-state.js`, an unknown path (404), and a document route with a disallowed method (405).
   - Conflict cases: `POST /doc/<ID>` carrying a stale `version` field. The stale-version branch of `src/browser/mod.rs::form_post` selects seven functions from `src/browser/render.rs`: `document_conflict_page`, `conflict_page_ui_mockup`, `conflict_page_business_rule`, `conflict_page_audit_record`, `conflict_page_tech_spec`, `conflict_page_use_case` and the fallback `conflict_page`. Capture one case per reachable page across the nine document types that reach them (`note`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`, `req`). The remaining two renderer functions, `conflict_page_test_case` and `conflict_page_test_report`, have no call site: record that fact in the case manifest rather than claiming them as covered here; TASK-242 owns the full nine-function inventory.
   - Not-found cases: one request per `src/browser/routes.rs::NotFoundPage` variant (`TECH-999`, `UC-999`, `TC-999`, `TR-999`, `NOTE-999`). The dispatcher is the 404 branch of `src/browser/mod.rs::handle_request` (`src/browser/mod.rs:644` to `661`), which selects through `BrowserRoutes::not_found_page`.

6. CLI capture per type: `create` with the minimum required options, `create --json`, `list`, `list --json`, `view`, `view --json`, `edit`, `edit --json`, `archive`, `delete` (on a throwaway copy), `status` where the type declares it, `supersede` for `ui-mockup`, and `create-group` for `req` and `nfr`. Each case captures the stored record bytes, stdout, stderr, exit code, and the JSON payload when `--json` is used.

7. Title and term bytes (MS-032 AC-6): capture the current pre-refactor form `glossary create --term <X> --definition <Y>` and `glossary edit <ID> --term <Z>` (see `src/glossary.rs`), the help text that documents `--term`, and the positional `TITLE` create form plus today's `--title` edit form for every other type. The uniform title contract of REQ-07.003 and BR-011 must stay measurable from these bytes.

8. CLI help snapshot (MS-032 AC-5): extend `tests/cli_help_golden.rs::cases()` and `tests/golden/cli-help/` with `config --help`, `config get --help`, `config set --help`, `config unset --help`, `config list --help`, `help --help`, `browser --help`, `search --help`, `init --help`, `import --help`, and the `task`, `milestone`, `release`, and `decision` verb helps the current case list omits (`release` has none). Keep `golden_snapshot_set_is_complete` and the `PAPYRUS_UPDATE_CLI_HELP=1` regeneration path.

9. `tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md`: the deliberate differences a later milestone may observe, each with its source. Uniform title contract (REQ-07.003, REQ-05.001, REQ-05.003, BR-011); `--term` removed and answered as a usage error (REQ-05.003, BR-011); help prose (REQ-07.003); asset cache headers and versioned vendor URLs (NFR-007.003, ADR-038); latency (NFR-007.001, NFR-007.002); conflict and not-found renderer collapse from 9 to 1 and 5 to 1 (ADR-039, proven in MS-035). No other difference is expected.

10. `README.md`: the single regeneration command `PAPYRUS_CAPTURE_COMMIT=$(git rev-parse HEAD) cargo test --test characterization_capture -- --ignored`, the case schema, the timestamp normalisation rule, and the corpus hash rule (sha256 over sorted `<relative path>\0<sha256(bytes)>` lines).

11. Replay gate `tests/characterization_replay.rs`, part of the default `cargo test`: copy `project/` into `./.tmp/`, replay every case, compare byte-for-byte, remove the copy. HTML is compared byte-for-byte inside MS-032 because no behaviour may change; later milestones may apply TC-184's semantic comparison instead.

Out of scope: any change under `src/`; deciding post-refactor behaviour; deltas beyond item 9.

Definition of done: a second capture run is idempotent (`git status --porcelain tests/fixtures/characterization/corpus tests/golden/cli-help` is empty), the replay gate passes from a clean checkout, and `README.md` plus `EXPECTED-DELTAS.md` are complete.
## Acceptance Criteria
- [ ] #1 The characterization corpus exists as a committed seed project plus an empty project with README, case manifest and expected-delta list under the corpus directory named in the description, and the existing list characterization fixtures stay unmodified (MS-032 AC-2).
- [ ] #2 The corpus covers all three DocumentModels: SingleMarkdown through the user-defined note type and rule, GroupedMarkdown through req and nfr, and SingleHtmlMockup through ui-mockup.
- [ ] #3 The corpus contains REQ-001 with three members (one resolved reference to REQ-002, one unresolved reference REQ-999), a memberless REQ-002 group, NFR-001 with two members, MOCK-001 pending, MOCK-002 approved, MOCK-003 archived, the archived BR-002, and the padding boundary pair NOTE-999 and NOTE-1000.
- [ ] #4 Every case carries the documented artifacts: stored record bytes, stdout, stderr, exit code, JSON payload, HTTP status and ordered headers, and rendered page bytes, each present only when the case produces it.
- [ ] #5 Timestamps are normalised to the fixed token and two consecutive capture runs produce byte-identical expected trees with no wall-clock value, PID, absolute path or port in any artifact.
- [ ] #6 The route matrix covers list, view, edit, the per-slug JSON list, every declared legacy JSON path and the per-record JSON route for all twelve types, plus inventory, the task board, the three app assets, a 404 case and a 405 case.
- [ ] #7 Every conflict page the stale-version dispatcher can select is exercised by a stale-version POST: document_conflict_page, conflict_page_ui_mockup, conflict_page_business_rule, conflict_page_audit_record, conflict_page_tech_spec, conflict_page_use_case and conflict_page, across the nine document types that reach them, and conflict_page_test_case plus conflict_page_test_report are recorded as unreachable rather than claimed as covered.
- [ ] #8 Every NotFoundPage variant is exercised by a request: TECH-999, UC-999, TC-999, TR-999 and NOTE-999, five cases in total.
- [ ] #9 The CLI matrix captures create, create with JSON, list, list with JSON, view, view with JSON, edit, edit with JSON, archive, delete, status where declared, supersede for ui-mockup and create-group for req and nfr, with record bytes, stdout, stderr and exit codes (MS-032 AC-2).
- [ ] #10 The pre-refactor glossary create and edit calls that pass a term option, their help text, and the positional title create plus the current title-option edit form of every other type are captured, so the REQ-07.003 title migration is measurable from bytes (MS-032 AC-6).
- [ ] #11 The CLI help golden set covers every document command and the config subcommands config, config get, config set, config unset, config list, help, browser, search, init and import, plus the missing task, milestone, release and decision verb helps, and the golden set completeness check passes (MS-032 AC-5).
- [ ] #12 The expected-delta list states exactly the six documented deltas with their sources: uniform title contract, term option removal, help prose, asset cache headers, latency, and the nine-to-one and five-to-one renderer collapse; no other difference is listed.
- [ ] #13 One documented command regenerates the expected tree from a clean checkout, and the replay test passes in the default cargo test run.
- [ ] #14 The replay gate is proven meaningful: a negative case shows that a one-byte change to any expected artifact fails the comparison.
- [ ] #15 Rendered page captures normalise the shell bind note to a fixed token, because every page embeds the bound host and port through the shell template, so no ephemeral port appears in any expected page.
## Comments
- **task-refiner**: Refined by task-refiner for MS-032: description grounded in the current routes, renderers, harness conventions and root-cause call sites; acceptance criteria added; references and documentation set. No dependency, ordinal or milestone change. Status left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done].
- **test-manager**: test-manager evidence corpus: 233 cases in cases.json - 134 CLI and 99 HTTP. 12 document types across all three DocumentModels, 9 stale-version conflict pages, 5 not-found pages, and the two conflict renderers without a call site recorded as unreachable. 746 expected artifacts under tests fixtures characterization corpus.
- **test-manager**: test-manager evidence stability: three consecutive capture runs produced an identical sha256 listing of the expected tree, and a scan found no wall-clock timestamp, port, absolute path or pid in any artifact. Capture command: set PAPYRUS_CAPTURE_COMMIT, then run cargo test --test characterization_capture -- --ignored.
- **test-manager**: test-manager evidence replay: characterization_replay runs in the default cargo test suite in about 4.5 s and passes all 3 tests; the negative test proves one changed byte and one extra artifact both fail the comparison. HTML is compared byte for byte in this milestone.
- **test-manager**: test-manager evidence suite: cargo test 1133 passed, 24 ignored, 2 failed. Both failures pre-date TASK-240 and touch no file this task owns: list_characterization_replay expects the pinned empty decision fixture to render no table while the tracked ADR-001 record makes it render one, and no_drift reports a heuristic false positive on src import.rs line 1202. Run with EDITOR unset because config_editor_json_null_when_unset picks up the session editor. cargo clippy --all-targets is clean and no src file was modified.
- **test-manager**: test-manager evidence snapshot: per case record md or html, stdout txt, stderr txt, exit json, payload json, http json with status and ordered headers, page html, and body txt for assets. Tokens normalised: TIMESTAMP for every ISO-8601 UTC value, BIND for the shell host and port, PROJECT for the absolute project path, VERSION for the mtime version token. Nothing else is normalised.
- **test-manager**: test-manager evidence CLI help: golden set now has 144 snapshots, 27 of them added - config and its four subcommands, help, browser, search, init, import, plus the previously missing task, milestone, release and decision verbs. Completeness and regeneration tests pass.
- **test-manager**: test-manager evidence correction: the final corpus has 236 cases - 137 CLI and 99 HTTP - and 752 expected artifacts. Three empty-state cases run against project-empty for inventory, the requirement list and the board. The earlier count of 233 predates those empty-state cases.
- **test-manager**: test-manager evidence correction 2: the exact split is 134 CLI and 102 HTTP, where the HTTP block is 99 populated cases plus 3 empty-state cases. Total 236 cases and 752 artifacts, corpus directory 3.8 MB.
- **orchestrator**: Acceptance met: 236-case characterization corpus and 752 frozen expectation artifacts over 12 document types, every route class including 7 reachable conflict pages and 5 not-found variants, 27 new CLI help goldens, replay harness stable across consecutive captures.
