# List characterization fixtures (TASK-141)

Pre-change list evidence for every artifact family, captured **before** the
shared list layout existed. TASK-142 replays these fixtures against the shared
layout; this ticket only replays the reference family (milestones).

- **Schema:** `papyrus.list-characterization/1`
- **Producing commit:** `d60f71c9b5bc6a7e419e2cef682763e0c914160a`
- **Captured with:** the pre-change browser binary from that commit
- **Producer:** `tests/list_characterization_capture.rs` (`#[ignore]`d; kept for
  audit only — never regenerate these files from migrated code)

## Layout

- `project/` — committed seed project (config plus record files). It enables
  the eleven default document types and one config-defined type, `note`, and
  contains at least two active records per family, one archived record per
  archivable family, one requirement group with two members, one NFR group
  with two members, one requirement with an unresolved group reference, and an
  archived milestone moved to `backlog/archive/milestones/`.
- `project-empty/` — same configuration with no record files; every family is
  captured in its empty state.
- `expected/<family>.json` — populated state.
- `expected/<family>-empty.json` — empty state.

## Reproduction

```text
PAPYRUS_CAPTURE_COMMIT=d60f71c9b5bc6a7e419e2cef682763e0c914160a \
  cargo test --test list_characterization_capture -- --ignored
```

## Schema

```jsonc
{
  "schema": "papyrus.list-characterization/1",
  "family": "milestone",
  "url": "/?type=milestone",
  "captured_from_commit": "<sha>",
  "heading": "Milestones",
  "count_note": "3 milestones",
  "row_order": ["MS-001", "MS-002", "MS-003"],
  "baseline_columns": [],
  "baseline_row_shape": "ul",
  "records": [
    {
      "id": "MS-001",
      "title": "Milestone One",
      "status": "open",
      "created": "2026-11-15T19:07:08Z",
      "updated": "2026-11-15T19:07:08Z",
      "labels": ["m1"],
      "references": ["REF-1"],
      "slots": { "release": "REL-001" },
      "href": "/doc/MS-001",
      "actions": []
    }
  ],
  "grouped": [{ "group": "REQ-001", "children": ["REQ-001.001"] }],
  "excluded": ["MS-004"],
  "empty": false
}
```

- `row_order`, `records`, `grouped`, `excluded`, `heading`, and `count_note`
  are **replayed** targets.
- `baseline_columns` and `baseline_row_shape` are **evidence only**: they record
  the old contract (`ul`, `table`, or `board`) that the shared layout changes.
  `note` is the only family with a header row today (generic document table).
- `slots` records the declared family-slot values. Values that today's list does
  not render (rule applicability, UI-mockup task/specs) are read from the seed
  records; every other value is decoded from the rendered list.
- `excluded` lists archived IDs in the family's archive directory; they must
  never appear in a list.
- Tasks are captured from the REQ-006.013 board (the task family has no list
  route): `grouped` carries milestone groups and card order; cards carry no
  dates.
