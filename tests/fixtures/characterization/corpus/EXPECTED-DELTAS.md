# Expected deltas (TASK-240, MS-032)

The complete list of deliberate differences between this pre-refactor snapshot
and the post-refactor behaviour. Any other difference is a regression.

1. **Uniform title contract.** Every document type migrates to the same title
   input: a positional `TITLE` on create and a uniform title form on edit,
   including the glossary. Bytes: `*/create`, `*/edit`, `glossary/create`,
   `glossary/edit`, and `verb-*/create|edit` help goldens. Source: REQ-07.003,
   REQ-05.001, REQ-05.003, BR-011.
2. **`--term` removed, `--definition` kept, and answered as a usage error.**
   `glossary create` no longer takes `--term`; the term is the positional
   `TITLE` and `--definition` remains the definition option. `glossary edit`
   no longer takes `--term`; the term is edited through the uniform `--title`.
   A `--term` invocation becomes an exit-2 usage error and writes no record.
   Corpus cases:
   `glossary/create` and `glossary/create-json` (positional `TITLE`,
   `--definition` kept, exit 0), `glossary/edit` and `glossary/edit-json`
   (uniform `--title`, exit 0), and the captured usage-error cases
   `glossary/create-term-removed` and `glossary/edit-term-removed` (exit 2, no
   record). Bytes: those six cases plus `verb-glossary-create`,
   `verb-glossary-edit`. Source: REQ-05.003, BR-011.
3. **Help prose and derived field options.** Every verb about line is generated
   by the engine from the document model and field sequence; per-type about
   text is not preserved (ADR-042 item 2). Option prose is derived from the
   seed `label`/`help`/kind, field options follow the seed field sequence, and
   the `-d` short on `--description` disappears because `FieldSpec` has no
   short-name equivalent. Bytes: `tests/golden/cli-help/*` and the `* create`
   and `* edit` fixture stdout. Source: REQ-07.003, ADR-042 item 2,
   ADR-036 decision 29.
4. **Asset cache headers and versioned vendor URLs.** Static assets gain cache
   headers and the vendor URLs become version-qualified. Bytes:
   `http/asset-app-css`, `http/asset-app-js`, `http/asset-nav-state-js`
   (`http.json`) and any page that references a vendor asset. Source:
   NFR-007.003, ADR-038.
5. **Latency.** Response latency may change; the byte snapshot does not record
   time and must stay identical. Source: NFR-007.001, NFR-007.002.
6. **Conflict and not-found renderer collapse.** The nine conflict renderers
   collapse to one and the five not-found renderers collapse to one. Bytes:
   `http/conflict-*`, `http/not-found-*`, `http/not-found-path`. Source:
   ADR-039, proof in MS-035.

7. **Grouped member group link.** A newly created grouped member stores its
   engine-owned group link in the reserved front-matter `group` key instead of
   the legacy `## Group` body section. Existing records are not migrated and
   keep their stored section. Bytes: `req/create`, `req/create-json`,
   `nfr/create`, `nfr/create-json`. Source: REQ-05.001, REQ-05.002,
   REQ-05.010, ADR-036 decision 9.

8. **Seeded status on create.** Every Markdown document `create` and
   `create-group` now writes `status` into front matter: the type's configured
   `default_status`, or `active` when none is declared. A created group record
   therefore gains a front-matter block holding only `status`; created members
   and single Markdown records carry the key in key order. `SingleHtmlMockup`
   is unchanged and keeps `status: pending` in its HTML metadata. Existing
   records without `status` are not migrated, and no create option sets the
   value. Bytes: `*/create`, `*/create-json`, `req/create-group`,
   `nfr/create-group` for every Markdown type except `ui-mockup`. Source:
   REQ-05.011, TECH-042 7.4, TC-138.

Corrected behaviour without a byte delta: `nfr create` and `nfr create-group`
now write the `## User Story` section when `--story` is supplied. The seed
declares `story` for both nfr roles (REQ-05.002) and the pre-refactor module
ignored it; no corpus case supplies `--story` to nfr, so the snapshot is
unchanged. Source: REQ-05.002, ADR-036 decisions 26-29.

Recorded engine exception without a byte delta: the ui-spec editor
front-matter re-adoption quirk is expressed as one descriptor-keyed engine rule
(not a slug literal and not a name comparison) and covered by
`tests/ui_spec_cli.rs::ui_spec_editor_rewrites_without_renaming`. It preserves
the pre-refactor output, so no corpus byte delta is recorded. Source: MS-035
TASK-269 (269b); TECH-040.

## Measured verb-about-line delta (TASK-260)

The `TASK-260` regeneration measured one systematic help delta that the earlier
item 3 wording left implicit. Every per-verb about line becomes engine-generated
text from the document model and the field sequence (`ADR-042` item 2):

| Verb | Generated about line |
| --- | --- |
| `create` | `Create a <slug> document.` |
| `create-group` | `Create a <slug> group.` |
| `list` | `List <slug> documents.` |
| `view` | `View a <slug> document.` |
| `edit` | `Edit a <slug> document.` |
| `archive` | `Archive a <slug> document.` |
| `delete` | `Delete a <slug> document.` |
| `status` | `Set the status of a <slug> document.` |

The root about line (`Project-level backlog and documentation manager.`) and
each type's `Manage <slug> documents.` line are unchanged. Option prose follows
the same ruling: for example `--story` is `User story`, `--description` is
`Description Use "-" to read multiline text from standard input.`,
`--applicability` is `Applicability`, `--view` is `View`, and `--group` is
`Owning group`. `ui-mockup create` has no verb about line. Every measured help
delta, including the `-d` short removal and the engine title/group validation,
is attributed file by file in
[`docs/specifications/technical/cli-help-deltas.md`](../../../../docs/specifications/technical/cli-help-deltas.md).

The enabled-type order in `top.json` is unchanged and remains effective
configuration map order (slug order); seed `order` is presentation-only
(`ADR-042` item 4). No order delta is expected or recorded.

No other difference is expected.
