---
id: ADR-044
title: "Non-loopback browser binding is an explicit, unauthenticated risk"
created_date: '2026-11-23T11:30:42Z'
updated_date: '2026-11-26T18:24:53Z'
labels: []
references:
  - REQ-006.002
  - BR-004
  - TECH-007
  - AUD-153
superseded_by: null
---
# ADR-044 Non-loopback browser binding is an explicit, unauthenticated risk

## Status
deprecated
## Decision
1. **Trust model.** Papyrus is a single-user, local desktop tool. The browser server's only supported default is loopback (`127.0.0.1`, port `6420`). The server has **no authentication, no session, and no authorization layer by design**: there is no login, no token, no cookie-based identity, no per-user ownership, and no credential storage. This is a deliberate architectural choice for 1.0; it avoids a new dependency, secret provisioning, and credential rotation in a tool with one local operator.

2. **Rule for a non-loopback bind.** A non-loopback bind must be an **explicit opt-in**, never silently derived. The effective host is selected only from explicit user input, in this precedence: `--host`, then `PAPYRUS_BROWSER_HOST`, then the `browser_host` configuration key, then the loopback default. The server must not auto-detect an interface, upgrade the bind on its own, or infer a network address from the environment.

3. **Warning at startup.** When the resolved host is not loopback (not `127.0.0.0/8`, `::1`, or `localhost`), startup must print **one prominent line** to standard error that names the exposure: the interface is unauthenticated and anyone who can reach the port can read and modify every record. A generic "reachable from the network" statement is not sufficient because it omits the absence of authentication and the read/write scope.

4. **Controls retained on every bind.** A non-loopback bind does not relax any existing control: the Host allowlist and Origin checks stay in force, CSP and other security headers stay in force, and `Cache-Control: no-store` behaviour stays in force. There is no TLS; the transport is plain HTTP in all cases.

5. **Documentation obligation.** The README browser/config reference and the configuration documentation must state, next to the host setting, that binding to a non-loopback address exposes an unauthenticated interface and that anyone who can reach the port can read and modify every record. A warning in a manual is not a substitute for the startup warning, and the startup warning is not a substitute for the documentation.

6. **Residual risk accepted for 1.0.** The residual risk of an unauthenticated, unencrypted read/write interface on a non-loopback bind is accepted for 1.0 because exposure requires an explicit user opt-in and the default and documented posture is local-first, single-user, loopback-only. This acceptance is conditional on decisions 2-5 being enforced.

7. **Future token option.** If a multi-user or shared-LAN use case is approved later, a future ADR may add a per-process bearer token compared in constant time (with no credential storage added for 1.0). Such an ADR would supersede this one; no token is implemented now.

8. **Code follow-up status.** Current behaviour was checked on branch `task/TASK-316`. The explicit opt-in in decision 2 is **already satisfied**: `resolve_options` in `src/browser/mod.rs` resolves CLI > environment > configuration > loopback default with no derivation, and `validate_browser_host` in `src/config.rs` validates syntax only and does not reject non-loopback hosts. A one-line startup warning also exists in `run()` at `src/browser/mod.rs` for non-loopback hosts. However, that warning's text ("the interface is reachable from the network") does not name the absence of authentication or the read/modify scope required by decision 3. A small follow-up change is therefore required, and is **not implemented by this ADR**: replace the warning text with a single line such as `warning: binding to non-loopback address <host> without authentication; anyone who can reach this port can read and modify every record`, and update the README/config reference per decision 5. No other code change is needed to enforce the opt-in rule.
## Context
- **Source.** Security audit AUD-153 finding M-2: the browser server warns on a non-loopback bind but has no authentication. The host validator accepts any syntactically valid host, and there is no session or token. Correction options listed by the audit: an explicit network opt-in and/or a per-process bearer token, or a recorded accepted-risk decision.
- **Existing contract.** REQ-006.002 and BR-004 permit a non-loopback host with a warning. TECH-007 section 9.10 already states that a non-loopback bind "warns and exposes an unauthenticated read/write interface to the network". The approved specification therefore permits the behaviour; the gap is that the risk was never recorded as an explicit architecture decision and the startup warning is weaker than the specification's statement.
- **Why authentication is out of scope for 1.0.** Papyrus has no user accounts, no roles, and no multi-tenant data. A token would introduce a new dependency, credential generation and storage, token logging concerns, and a second failure mode in a tool that runs on one machine. For the documented single-user case it adds cost without changing the trust boundary.
- **Options considered.**
  1. *Bearer token in every build* — strongest control, but adds credential handling and complexity disproportionate to a local single-user tool; deferred.
  2. *Refuse non-loopback binds entirely* — closes the exposure, but contradicts the approved browser requirement and specification and removes a documented explicit mode.
  3. *Explicit opt-in plus a prominent warning plus a recorded accepted risk* — keeps the approved behaviour, makes the exposure deliberate, and documents it. Chosen.
- **Force at play.** The audit requires mitigation or explicit acceptance before release. Acceptance must be visible to release reviewers, not buried in a code comment.
- **Related controls.** Loopback default, Host allowlist, Origin checks, CSP, `Cache-Control: no-store`, and no outbound connections already limit the attack surface. These are not authentication and do not remove the exposure when the port is reachable.
## Consequences
- **Positive.** No authentication code, no secrets, no credential storage, and no new dependency for 1.0. The default posture remains loopback-only. The residual risk is recorded, visible, and tied to an audit finding rather than left implicit.
- **Positive.** A single explicit rule (explicit opt-in, one-line exposure warning, retained Host/Origin/CSP/no-store controls, documentation statement) is testable and reviewable.
- **Negative.** When a user deliberately binds to a non-loopback address, there is no identity check: anyone who can reach the port can read, create, edit, archive, and delete every record. Traffic is plain HTTP with no TLS. The warning and documentation are advisory, not enforcement.
- **Negative.** The follow-up warning-text change and the README/config-reference wording are outstanding at the time of this decision; until they land, the warning does not fully name the exposure.
- **Migration.** None. No configuration key, default, CLI flag, or route changes. Existing explicit binds continue to work and remain warned.
- **Revisit trigger.** A multi-user, shared-host, or LAN use case, or a future audit that rejects unauthenticated network exposure. A superseding ADR must then introduce a per-process bearer token compared in constant time.
