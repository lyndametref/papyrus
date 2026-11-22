# Expected deltas (TASK-240, MS-032)

The complete list of deliberate differences between this pre-refactor snapshot
and the post-refactor behaviour. Any other difference is a regression.

1. **Uniform title contract.** Every document type migrates to the same title
   input: a positional `TITLE` on create and a uniform title form on edit,
   including the glossary. Bytes: `*/create`, `*/edit`, `glossary/create`,
   `glossary/edit`, and `verb-*/create|edit` help goldens. Source: REQ-07.003,
   REQ-05.001, REQ-05.003, BR-011.
2. **`--term` removed and answered as a usage error.** `glossary create` no
   longer takes `--term`/`--definition`, and `glossary edit` no longer takes
   `--term`; a `--term` invocation becomes an exit-2 usage error. Bytes:
   `glossary/create`, `glossary/edit`, `verb-glossary-create`,
   `verb-glossary-edit`. Source: REQ-05.003, BR-011.
3. **Help prose.** Document command help text changes with the title contract
   and the shared verbs; the prose, not the option grammar alone. Bytes:
   `tests/golden/cli-help/*`. Source: REQ-07.003.
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

Corrected behaviour without a byte delta: `nfr create` and `nfr create-group`
now write the `## User Story` section when `--story` is supplied. The seed
declares `story` for both nfr roles (REQ-05.002) and the pre-refactor module
ignored it; no corpus case supplies `--story` to nfr, so the snapshot is
unchanged. Source: REQ-05.002, ADR-036 decisions 26-29.

No other difference is expected.
