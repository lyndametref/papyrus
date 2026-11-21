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

No other difference is expected.
