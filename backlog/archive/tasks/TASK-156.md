---
id: TASK-156
title: "Refine the browser origin validation fix"
created_date: '2026-11-16T20:02:21Z'
updated_date: '2026-11-17T18:47:29Z'
labels:
  - bug
references:
  - RE-003
  - NFR-01.002
  - REQ-06.002
  - BR-004
status: Done
assignee:
  - task-refiner
milestone: MS-024
dependencies:
  - TASK-155
ordinal: 20
documentation:
  - src/browser/mod.rs
type: refinement
---

## Description
Outcome: confirm all information for implementing the browser origin validation fix is available and complete the task refinement fields. Root cause: validate_origin compares the Origin host only to the raw request Host and only handles http origins plus port 80 normalization, so legitimate same-site browser POST edits (alias hosts, IPv6 bracket forms, https origins) are rejected with 403. Repair: validate Origin host membership in the allowed host set after normalizing scheme (http or https), IPv6 brackets, and port. Scope assumptions and acceptance criteria are in the refinement comment.
## Acceptance Criteria
- [ ] The defect and required code locations are identified: validate_origin and allowed_host_values in the browser module.
- [ ] The repair accepts any Origin whose normalized host is in the allowed host set (alias hosts), per MS-024 AC-1.
- [ ] Ports, IPv6 brackets (bracketed and unbracketed), and https scheme are normalized before the membership check, per MS-024 AC-2.
- [ ] Origin: null remains rejected with 403, per MS-024 AC-3.
- [ ] Cross-host origins, disallowed ports, origins with a path or userinfo component, and unknown schemes are rejected with 403, per MS-024 AC-4.
- [ ] Missing Origin header and non-POST requests are not rejected, per MS-024 AC-1 and AC-5.
- [ ] Legitimate same-site edits no longer return 403, per MS-024 AC-5.
- [ ] Traceability to TC-007..TC-014, REQ-06.002, NFR-01.002, BR-004, and RE-003 is recorded.
- [ ] No open material questions remain.
## Comments
- **task-refiner**: Refinement complete. Root cause: validate_origin compares Origin host only to raw request Host and only handles http origins plus port 80 normalization, rejecting legitimate same-site browser POST edits. Repair: parse and normalize Origin (scheme http or https, host, optional port, IPv6 brackets) and accept when normalized host is in allowed_host_values; reject null, cross-host, disallowed port, path or userinfo, unknown scheme. Assumptions: allowed host set is allowed_host_values(state) with no new config key; scheme does not change host identity; default ports 80 for http and 443 for https when omitted; Origin null always rejected; applies only to POST; missing Origin accepted; no host validation or route changes. Implementation: change validate_origin signature to accept server state or allowed hosts, add normalize_origin helper, keep missing Origin and non-POST as Ok. Code locations: validate_origin and allowed_host_values in browser module. Traceability: TC-007 to TC-014, MS-024 AC-1 to AC-5, REQ-06.002, NFR-01.002, BR-004, RE-003. No open material questions.
