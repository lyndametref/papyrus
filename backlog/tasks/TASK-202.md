---
id: TASK-202
title: "Rename ADR-034 to a conventional slug and update the TC-036 path assertion"
created_date: '2026-11-18T22:21:37Z'
updated_date: '2026-11-18T22:36:32Z'
labels: []
references:
  - AUD-110
  - ADR-034
  - TC-036
  - REQ-06.018
status: Done
assignee:
  - architect
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 59
documentation:
  - decisions/ADR-034-test-adr.md
  - tests/browser_cli.rs
type: implementation
---

## Description
Correction for AUD-110 finding 2 (medium): the accepted ADR-034 record is filed under a test-adr slug that contradicts its title and the convention used by every other ADR, and TC-036 pins that filename so a legitimate rename breaks the test. Rename the record through the Papyrus CLI so the slug is conventional and matches the title, keeping the record ID, content, references, and history intact; do not create a second record. Update the TC-036 assertion in tests browser_cli.rs to the new path, or assert the decision content by record ID through the record API so a future rename does not break the test. Done when papyrus check reports no inconsistency and cargo test --test browser_cli passes.
## Comments
- **architect**: Papyrus decision has no rename command, so the file was moved on disk; content is byte-identical to HEAD (sha256 89040f578aa72d284b5a631b6f159af23c07e535653e6bba0c06660e683838c7), front matter, ID ADR-034, title, references, and dates are unchanged, and no second record was created.
- **architect**: Verification: cargo fmt --check pass, cargo clippy --test browser_cli -- -D warnings pass, cargo test --test browser_cli 181 passed 0 failed, papyrus decision view ADR-034 resolves to the new path, papyrus check shows only the 3 pre-existing archived-task frontmatter warnings (TASK-098, TASK-111, TASK-112).
- **architect**: Residual: the TASK-202 documentation field still lists the old path because papyrus task edit --documentation appends and cannot replace; the only other mention of the old filename is the AUD-110 finding text, kept as audit history.
- **architect**: Rename evidence: the ADR-034 record file was renamed from the test-adr slug to ADR-034-remove-configuration-managed-artifacts-and-agent-skills-sections-from-the-browser.md.
- **architect**: Test evidence: TC-036 no longer pins the filename; it resolves the record by its ADR-034 ID prefix under the decisions folder, then asserts the record ID and the content marker, so a future slug change cannot break the test.
