# Characterization corpus (TASK-240, MS-032)

Frozen pre-refactor behaviour of every document type and every browser route
(254 cases, 789 artifacts),
captured as byte-for-byte artifacts. MS-033 onward replays this corpus to prove
that the refactor changed nothing except the deltas listed in
[`EXPECTED-DELTAS.md`](EXPECTED-DELTAS.md).

This corpus is separate from `tests/fixtures/characterization/lists/`, which is
pinned to commit `d60f71c9` and must not change.

## Layout

- `project/` — committed seed project: `papyrus.config.yml` plus the record
  files. It enables the eleven built-in document types and one config-defined
  type (`note`), with `id_padding: 3`, the backlog and decision categories, and
  fixed `created_date`/`updated_date` values in every record.
- `project-empty/` — the same configuration with no record files.
- `cases.json` — the ordered case manifest (see below).
- `expected/<case-id>/` — captured artifacts of one case.

### Content-model coverage

| DocumentModel | Types |
| --- | --- |
| `SingleMarkdown` | `note` (config-defined, generic) and `rule` (built-in, typed) |
| `GroupedMarkdown` | `req` and `nfr` |
| `SingleHtmlMockup` | `ui-mockup` |

### Seed records

- `REQ-001` group with three members: `REQ-001.001` references the resolved
  group `REQ-002`, `REQ-001.002` carries the unresolved reference `REQ-999`,
  `REQ-001.003` is plain. `REQ-002` is a group with no members.
- `NFR-001` group with two members (`NFR-001.001`, `NFR-001.002`).
- `MOCK-001` pending, `MOCK-002` approved, `MOCK-003` archived under
  `docs/specifications/ui/mockups/archived/`.
- `BR-002` archived with a front-matter status under
  `docs/business-rules/archived/`.
- Zero-padding boundary: `NOTE-999` (archived, width 3) and `NOTE-1000`
  (active, width 4). Archiving `NOTE-999` keeps it in the corpus for
  `pad_number` while the not-found case can still request `/doc/NOTE-999` and
  receive the generic 404 page. `note/create` therefore allocates `NOTE-1001`.
- Every type has at least one record carrying labels and references.

## Case manifest

```jsonc
{
  "schema": "papyrus.characterization-cases/1",
  "unreachable": [],
  "cases": [
    {
      "id": "req/create",              // stable path; also the expected/ subdirectory
      "kind": "cli",                    // "cli" or "http"
      "argv": ["req", "create", "..."], // CLI cases; --dir is injected by the harness
      "request": { "method": "POST", "path": "/doc/REQ-001.001", "body": "..." },
      "document_type": "req",
      "model": "GroupedMarkdown",       // SingleMarkdown | GroupedMarkdown | SingleHtmlMockup
      "renderer": "page.html renderer", // required when an artifact is page.html
      "record": "docs/.../REQ-002.001.md", // stored record captured after the run
      "project": "project-empty",        // optional seed project; defaults to "project"
      "artifacts": ["record.md", "stdout.txt", "stderr.txt", "exit.json"]
    }
  ]
}
```

- `argv` never contains `--dir`; the harness prepends `--dir <copy>`.
- `record` names the stored file whose bytes become `record.md`/`record.html`.
  `delete` produces no record artifact because the file is gone.
- `project` selects the seed project. It is omitted for the populated project and
  set to `project-empty` for the empty-state cases.
- `artifacts` is the exact expected file set. An artifact is present only when
  the case produces it: `record.*`, `stdout.txt`, `stderr.txt`, `exit.json`,
  `payload.json` (`--json` stdout or API body), `http.json`, `page.html`
  (rendered page), `body.txt` (non-HTML, non-JSON response body such as assets).
- `unreachable` records renderers that have no call site. Since TASK-267 every
  declared renderer is reachable; the list is empty.

## Coverage

- Route matrix per type (12): `GET /?type=<declared key>` (legacy keys
  `business-rule` and `audit-record`), `GET /doc/<ID>`, `GET /doc/<ID>/edit`,
  `GET /api/docs/<slug>`, `GET /api/doc/<ID>`, and every legacy path declared in
  `Presentation::api_paths`.
- Grouped requirement index: `GET /api/requirements?type=req` and
  `GET /api/requirements?type=nfr` return the JSON index for each grouped type
  (TASK-292 CR-3).
- Shared surfaces: `GET /`, `GET /?type=task`, `/assets/app.css`,
  `/assets/app.js`, `/assets/nav-state.js`, an unknown path (404), and a
  document route with a disallowed method (405).
- Conflicts: twelve stale-version `POST /doc/<ID>` cases, one per document
  type across `note`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `use-case`,
  `test-case`, `test-report`, `req`, `nfr`, `glossary`, and `ui-spec`. Every
  document type now renders the one `document_conflict_page` (ADR-039
  decision 23).
- Writes: `POST /api/doc/<ID>` success (200 JSON) and stale-version conflict
  (409 JSON); `POST /doc/<ID>` form success (303 redirect) and validation
  failure (422 edit page with field errors).
- Not found: one case per document type — `REQ-999.001`, `NFR-999.001`,
  `GLO-999`, `BR-999`, `AUD-999`, `TECH-999`, `UC-999`, `TC-999`, `TR-999`,
  `UI-999`, `MOCK-999`, and `NOTE-999`.
- Empty states: `project-empty` supplies the inventory, requirement list, and
  board cases.
- CLI per type: `create`, `create --json`, `list`, `list --json`, `view`,
  `view --json`, `edit`, `edit --json`, `archive`, `delete --force`, `status`
  where declared, `supersede` for `ui-mockup`, and `create-group` for `req` and
  `nfr`. Create uses the uniform positional `TITLE` form; `audit create` also
  keeps `--title` because the positional is additive there. Glossary create
  uses the positional term and glossary edit uses `--title`, because the term
  is the record title. Separate `glossary/create-term-removed` and
  `glossary/edit-term-removed` cases record that the removed `--term` form is
  an exit-2 usage error.

## Determinism and normalisation

The capture replaces four volatile tokens in every artifact:

| Token | Replaces |
| --- | --- |
| `{{TIMESTAMP}}` | every ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`), including fresh `created_date`/`updated_date` |
| `{{BIND}}` | the shell bind note `127.0.0.1:<port>` embedded in every page |
| `{{PROJECT}}` | the absolute project directory echoed by `--json view` |
| `{{VERSION}}` | the mtime-derived token in `name="version" value="..."` on edit and conflict pages |
| `{{LENGTH}}` | the response `Content-Length` header when the body echoes a version token, whose digit count varies with the mtime |

The server emits no `Date` header, so `http.json` needs no other header
normalisation. Captured artifacts contain no wall-clock value, PID, absolute
path, or ephemeral port. HTTP cases run against a throwaway copy of the seed
project, so the write cases (form and API POST) never modify the committed
fixture.

`{{VERSION}}` and `{{PROJECT}}` extend the timestamp rule of the task: both are
wall-clock or checkout derived and would otherwise break AC-5.

## Regeneration

```text
PAPYRUS_CAPTURE_COMMIT=$(git rev-parse HEAD) \
  cargo test --test characterization_capture -- --ignored --nocapture
```

The capture writes `expected/` only; `cases.json` is authored and reviewed, not
generated. A second run must leave the tree unchanged:

```text
git status --porcelain tests/fixtures/characterization/corpus
```

Replay is part of the default suite:

```text
cargo test --test characterization_replay
```

## Corpus hash rule

The corpus hash is the SHA-256 over the sorted lines
`<relative path>\0<sha256(bytes)>` of every file in the corpus tree:

```python
import hashlib, os

lines = []
for root, _, files in os.walk("."):
    for name in files:
        path = os.path.join(root, name)
        rel = os.path.relpath(path, ".").encode()
        digest = hashlib.sha256(open(path, "rb").read()).hexdigest().encode()
        lines.append(rel + b"\0" + digest)
print(hashlib.sha256(b"\n".join(sorted(lines))).hexdigest())
```
