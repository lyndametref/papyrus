---
id: TASK-178
title: "Assess architecture impact of removing reference list from document-type browser list views"
created_date: '2026-11-18T19:30:26Z'
updated_date: '2026-11-19T07:01:51Z'
labels: []
references:
  - REQ-06.019
  - RE-006
  - ADR-035
status: Done
assignee:
  - architect
milestone: MS-028
dependencies: []
ordinal: 10
documentation: []
type: architecture
---

## Description
Judge architectural impact and update Architecture Decisions and Technical specification sections for removing the reference list from document-type browser list views. Blocks dependent work when unresolved.
## Comments
- **architect**: Architecture impact assessed. ADR-035 accepted: reference visibility in shared browser lists is resolved from the descriptor content model.
- **architect**: ContentModel::Generic (config-defined document types) renders no reference list in any list row; Typed and HtmlMockup families keep references; labels stay for every family. Single-document view, CLI list and view, JSON API, RecordView, and front matter are unchanged.
- **architect**: Delegation blocked: the tech-writer subagent could not be spawned (subagent depth limit reached). Required TECH revisions pending: TECH-074 5.1, 5.2, 5.5, 5.7, 6, 9; TECH-076 3.3 and traceability; TECH-068 3.1.3, T-114 rows, and traceability; TECH-045 3.2 targeted qualification and traceability.
- **architect**: Each revised TECH must move to draft, cite REQ-06.019 and ADR-035, and state that detail, API, and CLI views still show references. TASK-178 stays In-Progress until the TECH drafts exist and conformance passes; dependent design and implementation remain blocked.
- **architect**: No architecture view documents exist in this repository. The ADRs under decisions are the architecture baseline, so no architecture view file changed.
- **tech-writer**: TECH revision complete (tech-writer). Drafts updated with REQ-06.019, ADR-035, and RE-006 traceability plus a dated TASK-178 banner; history preserved. TECH-074 sections 5.1, 5.2, 5.5, 5.7, 6, 9: list model, markup, parity qualification, post-migration contract, fixtures, traceability. TECH-076 section 3.3 and traceability, plus the 1.1 scope phrase. TECH-068 section 3.1.3, T-114-01 and T-114-11 rows, traceability: detail, CLI and API unchanged. TECH-045 section 3.2 targeted qualification, Purpose clause, traceability. All four TECH status=draft. Generic list rows omit the reference block; Typed and HtmlMockup keep it; labels stay for every family. Architect conformance requested; sections stay draft until conformant. papyrus check exits 4 on three pre-existing archived-task frontmatter-conflict diagnostics (TASK-098, TASK-111, TASK-112), unrelated to these edits. No code changed.
- **architect**: Architect conformance verdict for ADR-035 (accepted): conformant on the normative contracts of all four TECH drafts. TECH-074 sections 5.1, 5.2, 5.5, 5.7, 6, and 9 match ADR-035 and REQ-06.019: Generic list rows omit the reference block, Typed and HtmlMockup families keep it, labels stay for every family, and detail, CLI, API, RecordView, and front matter are unchanged. TECH-076 sections 1.1, 3.3, and 9 match. TECH-068 sections 3.1.3, T-114-01, T-114-11, and traceability match. TECH-045 sections 3.2, Purpose, and traceability match. All four specs are draft and cite ADR-035, REQ-06.019, and RE-006. Residual non-blocking stale prose: TECH-074 sections 2, 7.1, and 8 OQ-2 plus TECH-045 OQ-3 still mention list references without the Generic qualification; fix at the next edit. No architecture view file changed because no docs architecture directory exists; ADR-035 is the decision baseline. No code changed.
