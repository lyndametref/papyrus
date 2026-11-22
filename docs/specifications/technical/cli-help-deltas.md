# CLI help golden deltas (TASK-260, MS-034)

This record attributes every difference between the MS-032 frozen CLI help
snapshot (`tests/golden/cli-help`, captured by TASK-240 at commit `0e49427`)
and the regenerated set on the post-TASK-259 tree. It is evidence for
MS-034 AC-5 and AC-7 and is intended to be read with
[`EXPECTED-DELTAS.md`](../../../tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md).

The regeneration is a no-behaviour-change step: the engine and commands are
untouched. Only the goldens, this record, the expected-deltas document, and the
help-determinism test were updated.

## Cause codes

| Code | Documented cause | Authority |
| --- | --- | --- |
| `T` | Uniform title contract: positional `TITLE` on create and the uniform `--title` form, including the previously missing edit titles. | REQ-07.003; TASK-257 |
| `G` | `--term` removed from glossary create/edit; the positional `TITLE` or `--title` replaces it. | REQ-05.003, BR-011 |
| `P` | Engine-generated verb about lines and option/argument prose derived from the document model and field sequence. | ADR-042 item 2 |
| `V` | `--group` is no longer clap-required on `req`/`nfr create`; the engine validates the group and the title. | ADR-042 item 2; TASK-256 |
| `F` | Seed-derived required/optional fields and field order (ADR-036 decision 29), including removal of the `-d` short on `--description`. | ADR-036 decision 29 |
| `B` | Eight-builder retention. Retained builders are internal; they produce no help delta. | MS-034 AC-4; ADR-042 |

A file with several codes contains several independent deltas; each is covered
by the code that explains it. No delta is attributed to more than one cause.

## Scope and completeness

- Input snapshot: `0e49427` (TASK-240, MS-032).
- Regenerated from: `4ecc5a6` (TASK-259 merge) on branch `task/TASK-260`.
- Golden cases: 144 (`top`, `version`, `check`, 12 slug surfaces, every verb,
  the backlog/decision/project verbs, config/help/browser/search/init/import,
  and 10 usage/application error cases).
- Changed golden files: 93. Unchanged files include `top.json`, `version.json`,
  the backlog/decision/project verbs, config, browser, search, init, import,
  and most error cases.
- `golden_snapshot_set_is_complete` passes: no missing and no stale files.
- Two consecutive `PAPYRUS_UPDATE_CLI_HELP=1` regenerations produce
  byte-identical trees and leave `git diff -- tests/golden/cli-help` empty.

## Attribution summary

| Codes | Count | Files |
| --- | --- | --- |
| `P` | 72 | `slug-audit`, `slug-glossary`, `slug-nfr`, `slug-note`, `slug-req`, `slug-rule`, `slug-tech-spec`, `slug-test-case`, `slug-test-report`, `slug-ui-spec`, `slug-use-case`, `verb-audit-archive`, `verb-audit-delete`, `verb-audit-edit`, `verb-audit-list`, `verb-audit-status`, `verb-audit-view`, `verb-glossary-archive`, `verb-glossary-delete`, `verb-glossary-list`, `verb-glossary-status`, `verb-glossary-view`, `verb-nfr-archive`, `verb-nfr-delete`, `verb-nfr-list`, `verb-nfr-status`, `verb-nfr-view`, `verb-note-archive`, `verb-note-delete`, `verb-note-list`, `verb-note-status`, `verb-note-view`, `verb-req-archive`, `verb-req-delete`, `verb-req-list`, `verb-req-status`, `verb-req-view`, `verb-rule-archive`, `verb-rule-delete`, `verb-rule-list`, `verb-rule-status`, `verb-rule-view`, `verb-tech-spec-archive`, `verb-tech-spec-delete`, `verb-tech-spec-edit`, `verb-tech-spec-list`, `verb-tech-spec-status`, `verb-tech-spec-view`, `verb-test-case-archive`, `verb-test-case-delete`, `verb-test-case-edit`, `verb-test-case-list`, `verb-test-case-status`, `verb-test-case-view`, `verb-test-report-archive`, `verb-test-report-delete`, `verb-test-report-edit`, `verb-test-report-list`, `verb-test-report-status`, `verb-test-report-view`, `verb-ui-mockup-edit`, `verb-ui-spec-archive`, `verb-ui-spec-delete`, `verb-ui-spec-list`, `verb-ui-spec-status`, `verb-ui-spec-view`, `verb-use-case-archive`, `verb-use-case-delete`, `verb-use-case-edit`, `verb-use-case-list`, `verb-use-case-status`, `verb-use-case-view` |
| `TP` | 2 | `verb-ui-mockup-create`, `verb-ui-spec-edit` |
| `TV` | 1 | `err-missing-required` |
| `PF` | 4 | `verb-nfr-edit`, `verb-note-edit`, `verb-req-edit`, `verb-rule-edit` |
| `TGP` | 1 | `verb-glossary-edit` |
| `TPF` | 10 | `verb-audit-create`, `verb-nfr-create-group`, `verb-note-create`, `verb-req-create-group`, `verb-rule-create`, `verb-tech-spec-create`, `verb-test-case-create`, `verb-test-report-create`, `verb-ui-spec-create`, `verb-use-case-create` |
| `TGPF` | 1 | `verb-glossary-create` |
| `TPVF` | 2 | `verb-nfr-create`, `verb-req-create` |

The observed delta for each group:

- `P` — every changed slug and verb help replaces the legacy per-type verb
  descriptions and about lines with engine-generated text, for example
  `Create a req document.`, `List req documents.`, `Set the status of a req
  document.`, and `Create a req group.`. `note` also gains the generated
  `<ID>`/`<VALUE>` metavars and argument prose. Option prose changes the same
  way (for example `--editor`, `--label`, `--reference`, `--story`,
  `--applicability`, `--view`, `--content`, `--audit-type`).
- `PT` — `verb-ui-mockup-create` gains the positional `TITLE` and `--title`
  (model-fixed `--task`, `--spec`, `--ref` stay unchanged);
  `verb-ui-spec-edit` gains the uniform `--title`.
- `TV` — `err-missing-required` changes from clap exit 2 listing `--group`,
  `--title`, and `<title>` to exit 4 `title is required`; the group is now
  engine-validated rather than clap-required.
- `FP` — `req`, `nfr`, `note`, and `rule` edit lose the `-d` short, gain the
  derived `--description` prose, and (rule) the generated
  `--requirement`/`--no-requirement` metavar and prose.
- `GPT` — glossary edit loses `--term`, gains `--title`, and uses generated
  about/option prose; `--definition` stays.
- `FPT` — every typed create loses the obsolete clap-required field flags,
  regenerates the field-derived option order/prose, and gains `TITLE`/`--title`;
  `audit create` also makes `--content` engine-validated instead of
  clap-required.
- `FGPT` — glossary create loses `--term` and the clap-required `--term`
  grammar, keeps `--definition` (now engine-validated), gains the positional
  `TITLE` and `--title`, and uses generated prose.
- `FPTV` — `req` and `nfr` create combine the typed-create delta with the
  engine-validated `--group`.

## Determinism and order

- `help_output_is_deterministic_and_free_of_volatile_tokens` runs seven
  representative cases (`top`, `req create`, `glossary create`, `audit create`,
  `ui-mockup create`, `req edit`, `err-missing-required`). Each case runs twice
  with identical environment and once with a different working directory,
  `LC_ALL`/`LANG` (`C` vs `C.UTF-8`), `COLUMNS` (80 vs 200), and `TZ` (`UTC` vs
  `Pacific/Kiritimati`). All three renders are byte-identical.
- The test rejects the manifest path, the project directory, the process id,
  the host name (except the program name and `localhost`), an ISO-8601
  timestamp, and a `0x` memory address in the rendered output.
- `top.json` is unchanged. The enabled-type order is `audit`, `glossary`,
  `nfr`, `note`, `req`, `rule`, `tech-spec`, `test-case`, `test-report`,
  `ui-mockup`, `ui-spec`, `use-case`: effective `document_types` map order
  (slug order), not seed `order`, per ADR-042 item 4.
  `descriptor_equivalence::cli_help_projection_matches_the_frozen_goldens`
  asserts the root order against `ArtifactRegistry::enabled` and the verb list
  against `ordered_verb_specs`; the test passes unchanged.
- `B` contributes zero golden deltas. `ui-mockup` keeps its model-fixed
  `--task`/`--spec`/`--ref` create options and `--status`/`--superseded-by`
  edit options (ADR-042 item 3); no diff is attributed to builder retention.

## Fixture replay (MS-032 corpus)

`cargo test --test characterization_replay` passes: all 254 current cases
(136 CLI, 118 HTTP) replay against the committed expected artifacts. Deltas
against the `0e49427` corpus, each with its cause:

| Delta | Cause | Cases |
| --- | --- | --- |
| Glossary create/edit take the term positionally (create) or through `--title` (edit); `--definition` stays. | `T`, `G` | `glossary/create`, `glossary/create-json`, `glossary/edit`, `glossary/edit-json` (case manifest argv) |
| `--term` is an exit-2 usage error and writes no record. | `G` | `glossary/create-term-removed`, `glossary/edit-term-removed` (new cases) |
| `ui-spec edit --title` now succeeds and writes the edited title; previously exit 2 `unexpected argument '--title'` with no record. | `T` | `ui-spec/edit`, `ui-spec/edit-json` (exit, record, payload, stderr) |
| Grouped members store the group link in front-matter `group` instead of `## Group`. | EXPECTED-DELTAS item 7 | `req/create`, `req/create-json`, `nfr/create`, `nfr/create-json` (record.md) |
| Markdown create/create-group writes `status` in front matter. | EXPECTED-DELTAS item 8 | `audit`, `glossary`, `nfr`, `req`, `rule`, `tech-spec`, `test-case`, `test-report`, `ui-spec`, `use-case` `create`/`create-json`, plus `req/create-group`, `nfr/create-group` (record.md; `note` already had status; `ui-mockup` is HTML and unchanged) |
| HTTP capture normalisation and added coverage. | TASK-292 capture fix, not a product delta: `{{VERSION}}`/`{{LENGTH}}` normalisation, removal of a duplicated renderer suffix, and two new `api-requirements` cases. | 87 artifacts under `expected/http/`; no CLI record or exit change |

All other CLI record files and exit codes match `0e49427` byte for byte. No
difference is unattributed. The HTTP table entry is outside the six MS-034
cause codes because it was already accepted in TASK-292 (AUD-136); it is
recorded here so no changed corpus file is left unexplained.

## Appendix: every changed golden

| Golden (`tests/golden/cli-help/`) | Cause codes |
| --- | --- |
| `err-missing-required.json` | TV |
| `slug-audit.json` | P |
| `slug-glossary.json` | P |
| `slug-nfr.json` | P |
| `slug-note.json` | P |
| `slug-req.json` | P |
| `slug-rule.json` | P |
| `slug-tech-spec.json` | P |
| `slug-test-case.json` | P |
| `slug-test-report.json` | P |
| `slug-ui-spec.json` | P |
| `slug-use-case.json` | P |
| `verb-audit-archive.json` | P |
| `verb-audit-create.json` | TPF |
| `verb-audit-delete.json` | P |
| `verb-audit-edit.json` | P |
| `verb-audit-list.json` | P |
| `verb-audit-status.json` | P |
| `verb-audit-view.json` | P |
| `verb-glossary-archive.json` | P |
| `verb-glossary-create.json` | TGPF |
| `verb-glossary-delete.json` | P |
| `verb-glossary-edit.json` | TGP |
| `verb-glossary-list.json` | P |
| `verb-glossary-status.json` | P |
| `verb-glossary-view.json` | P |
| `verb-nfr-archive.json` | P |
| `verb-nfr-create.json` | TPVF |
| `verb-nfr-create-group.json` | TPF |
| `verb-nfr-delete.json` | P |
| `verb-nfr-edit.json` | PF |
| `verb-nfr-list.json` | P |
| `verb-nfr-status.json` | P |
| `verb-nfr-view.json` | P |
| `verb-note-archive.json` | P |
| `verb-note-create.json` | TPF |
| `verb-note-delete.json` | P |
| `verb-note-edit.json` | PF |
| `verb-note-list.json` | P |
| `verb-note-status.json` | P |
| `verb-note-view.json` | P |
| `verb-req-archive.json` | P |
| `verb-req-create.json` | TPVF |
| `verb-req-create-group.json` | TPF |
| `verb-req-delete.json` | P |
| `verb-req-edit.json` | PF |
| `verb-req-list.json` | P |
| `verb-req-status.json` | P |
| `verb-req-view.json` | P |
| `verb-rule-archive.json` | P |
| `verb-rule-create.json` | TPF |
| `verb-rule-delete.json` | P |
| `verb-rule-edit.json` | PF |
| `verb-rule-list.json` | P |
| `verb-rule-status.json` | P |
| `verb-rule-view.json` | P |
| `verb-tech-spec-archive.json` | P |
| `verb-tech-spec-create.json` | TPF |
| `verb-tech-spec-delete.json` | P |
| `verb-tech-spec-edit.json` | P |
| `verb-tech-spec-list.json` | P |
| `verb-tech-spec-status.json` | P |
| `verb-tech-spec-view.json` | P |
| `verb-test-case-archive.json` | P |
| `verb-test-case-create.json` | TPF |
| `verb-test-case-delete.json` | P |
| `verb-test-case-edit.json` | P |
| `verb-test-case-list.json` | P |
| `verb-test-case-status.json` | P |
| `verb-test-case-view.json` | P |
| `verb-test-report-archive.json` | P |
| `verb-test-report-create.json` | TPF |
| `verb-test-report-delete.json` | P |
| `verb-test-report-edit.json` | P |
| `verb-test-report-list.json` | P |
| `verb-test-report-status.json` | P |
| `verb-test-report-view.json` | P |
| `verb-ui-mockup-create.json` | TP |
| `verb-ui-mockup-edit.json` | P |
| `verb-ui-spec-archive.json` | P |
| `verb-ui-spec-create.json` | TPF |
| `verb-ui-spec-delete.json` | P |
| `verb-ui-spec-edit.json` | TP |
| `verb-ui-spec-list.json` | P |
| `verb-ui-spec-status.json` | P |
| `verb-ui-spec-view.json` | P |
| `verb-use-case-archive.json` | P |
| `verb-use-case-create.json` | TPF |
| `verb-use-case-delete.json` | P |
| `verb-use-case-edit.json` | P |
| `verb-use-case-list.json` | P |
| `verb-use-case-status.json` | P |
| `verb-use-case-view.json` | P |