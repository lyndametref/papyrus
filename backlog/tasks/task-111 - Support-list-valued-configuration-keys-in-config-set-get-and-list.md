---
id: TASK-111
title: Support list-valued configuration keys in config set, get, and list
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-109
documentation:
  - docs/architecture/adr/ADR-0024-list-valued-configuration-keys.md
  - docs/requirements/business/REQ-01.002.md
priority: high
ordinal: 41050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`config set` rejects a YAML sequence, so `document_types.<slug>.statuses` and the top-level task `statuses` can only be written by hand-editing `papyrus.config.yml`. That contradicts REQ-01.002 and leaves the ADR-0019 repair path incomplete for the keys most likely to be invalid. This task adds the declared list-key class and a comma-separated value form stored as a YAML flow sequence, with the resulting configuration validated before any write.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 config::LIST_VALUED_KEYS names the top-level statuses key and every document_types.<slug>.statuses key, and no other key is classified as list-valued.
- [ ] #2 config set splits a list value on commas, trims surrounding whitespace, writes a YAML flow sequence, and config get/list print the same flow form and order.
- [ ] #3 An empty list value is rejected with exit 4 naming the key and nothing is written; commas are never escaped or quoted and this limitation is stated in the command help.
- [ ] #4 The resulting configuration is validated with the ADR-0019 validator before persisting; an invalid list is refused and the file is unchanged.
- [ ] #5 Scalar keys keep their existing comma-as-data semantics, and config unset on a list key reverts to the default list with no clear operation.
- [ ] #6 Tests cover round-tripping document status lists, an empty value, an invalid element, a scalar key containing a comma, and the list/scalar classification constant.
<!-- AC:END -->
