---
name: security-audit
description: 'Audit a release or feature for security vulnerabilities: secrets, dependency supply chain, authentication and authorization, input handling, data protection, platform-specific risks, and security NFR coverage. Produces a read-only, severity-ranked report with a Pass / Pass with findings / Block recommendation.'
argument-hint: 'Provide the release milestone, release ticket, feature ticket, branch, or audit target. Example: "Release 1.0" or "task-0042".'
user-invocable: true
disable-model-invocation: false
---

# Security Audit

## Outcome

One release or one explicitly named target receives an independent security audit report with severity-ranked findings and a release recommendation: `Pass`, `Pass with findings`, or `Block`. The audit is read-only; the orchestrator owns approval and release closure.

## When To Use

- At the end of every release, before the release is declared complete.
- The user asks for a security audit, security review, or security gate.
- A change touches authentication, authorization, secrets, personal data, file upload, network boundaries, or dependencies.

Do not use this skill for:

- Fixing vulnerabilities or writing code/configuration.
- Style, accessibility, or general code-quality review.
- Auditing multiple unrelated releases in one pass.

## Prerequisites

- **backlog-md skill (REQUIRED)** — Load it before any backlog read. All ticket reads go through the CLI.
- Read access to requirements, architecture, specifications, and application source.
- Optional: available scanners and test commands for the stacks present.

## Constraints

- Read-only: do not modify code, configuration, requirements, tickets, or status.
- Never edit ticket files directly; read via the backlog CLI only.
- Never reveal secrets, credentials, tokens, personal data, or exploit-ready payloads. Redact values and give the location.
- Report only findings traceable to evidence. Mark unverifiable controls as `not verified`, not as passed.
- Do not invent threats unrelated to the audited scope.
- Do not ask the user for approval and do not close the release; the orchestrator owns that.
- Do not create persistent reports. Return the report in the response. Use only project-local `./.tmp/` for transient scanner output and delete it before finishing, including when the audit is blocked.

## Procedure

### 1. Establish scope

1. Load `backlog-md`.
2. Read the target release ticket and milestone, or the named feature/branch.
3. List included tasks and confirm they are `Done`; report any exception.
4. Read linked requirements, security/privacy NFRs, specifications, architecture documents, and ADRs.
5. Record scope boundaries and anything explicitly out of scope.

### 2. Inventory the attack surface

Identify, per application in `apps/`:

- languages, frameworks, and runtime versions;
- entry points: HTTP endpoints, CLI commands, background jobs, message consumers, deep links, exported Android components, WebView bridges, IPC/FFI;
- authentication and authorization mechanisms;
- data stores and data flows, including personal or sensitive data;
- secrets and configuration sources;
- third-party dependencies and build plugins;
- file upload, download, import, and export paths;
- network egress and external integrations.

### 3. Run checks

Run every check that applies. Use available tools; if a tool is unavailable, record the limitation and perform a manual review.

**Secrets and configuration**
- Search tracked files and, when history access is available, repository history for hardcoded credentials, API keys, tokens, private keys, and connection strings. If history access is unavailable, record the limitation and audit the current tree.
- Verify secrets come from environment or a secret manager and are not committed.
- Verify debug flags, verbose errors, and test backdoors are disabled for release.

**Dependency and supply chain**
- Run the ecosystem audit tool when available: `npm audit`, `cargo audit`, `pip-audit`, Gradle dependency verification or OWASP dependency-check, or the project's equivalent.
- Flag known-vulnerable direct and transitive dependencies, unmaintained critical libraries, and unpinned or untrusted sources.

**Authentication and authorization**
- Verify authentication is required on every non-public entry point.
- Verify authorization is enforced server-side per object and per action, not only in the UI.
- Check session/token lifetime, rotation, revocation, storage, and transport.
- Check password and credential handling; verify no plaintext storage.
- Verify privilege boundaries between roles and tenants.

**Input handling and injection**
- Verify server-side validation and canonicalization for every external input.
- Check SQL/JPQL/native query construction for injection.
- Check OS command, shell, path, template, LDAP, XPath, and expression-language injection.
- Check deserialization of untrusted data.
- Check file upload type, size, name, and storage-path handling; block path traversal and executable content.
- Check SSRF and unsafe outbound requests.
- Check XML parsing for external entity expansion.

**Output and client-side**
- Check HTML/JS/CSS output encoding and DOM sinks for XSS.
- Check CSRF protection for state-changing requests.
- Check CORS policy and allowed origins.
- Check security headers: CSP, HSTS, X-Content-Type-Options, frame ancestors, referrer policy.
- Check client-side storage of tokens and sensitive data.

**Data protection and privacy**
- Verify transport encryption and certificate validation.
- Verify sensitive data at rest is protected where required.
- Verify logs, errors, analytics, and crash reports do not leak secrets or personal data.
- Verify data minimization, retention, and deletion behavior against requirements.

**Platform specifics**

- Android: exported components, intent handling, permissions, storage modes, WebView settings, network security config, backup rules, root/debug detection where required, and hardcoded secrets in resources.
- Backend: Spring Security configuration, endpoint exposure, actuator endpoints, JPA query construction, multipart handling, and error responses.
- Frontend: DOM XSS, Alpine.js expression injection, unsafe `innerHTML`, token storage, CSP compatibility, and third-party scripts.
- Rust: `unsafe` blocks, FFI boundaries, integer overflow in release mode, panic exposure, command/path handling, and TLS configuration.

**Security NFR coverage**
- Map each referenced security/privacy NFR to evidence.
- Flag NFRs with no implementation evidence as gaps.

### 4. Verify

- Reproduce or confirm each finding with a concrete command, test, or code path.
- Run the project's security-relevant tests when available.
- Do not mark a finding confirmed based only on a comment or a specification claim.
- Separate confirmed findings from observations and unverified controls.

### 5. Classify and decide

Assign severity:

- **Critical** — exploitable now with severe impact: auth bypass, remote code execution, exposed production secrets, unauthenticated sensitive-data access.
- **High** — serious impact requiring limited preconditions: stored XSS, SQL injection behind authentication, missing object-level authorization, vulnerable critical dependency.
- **Medium** — meaningful risk with mitigating preconditions or limited scope.
- **Low** — hardening issue with low impact.
- **Info** — observation or recommendation.

Decision:

- `Block` — any unresolved Critical or High finding.
- `Pass with findings` — no unresolved Critical or High; Medium/Low/Info findings remain and are documented with owners.
- `Pass` — no findings or only informational items.

### 6. Report

Produce the report in the response and hand it to the orchestrator. Do not change status, close the release, or request approval.

## Output Format

- **Audit target**: release, milestone, feature, branch, or commit.
- **Scope covered**: applications, components, entry points, requirement IDs, and exclusions.
- **Method**: checks and scanners run, commands, and limitations.
- **Findings**: table with severity | finding | evidence | impact | recommendation.
- **Severity counts**: Critical / High / Medium / Low / Info.
- **NFR security coverage**: addressed / not applicable / gaps.
- **Secrets check**: result and redacted locations.
- **Dependency and supply-chain check**: result.
- **Release recommendation**: `Pass` / `Pass with findings` / `Block`, with rationale.
- **Accepted risks and follow-ups**: owner and suggested ticket for each Medium/Low finding.
- **Open questions**: missing context or unverifiable controls.
- **Next required action**: what the orchestrator must do next.

Use `none` when a section does not apply.

## Completion Checks

- Target scope and exclusions stated.
- Every included task's status verified.
- All applicable check families attempted or explicitly marked as not applicable/unavailable.
- Every finding has severity and concrete evidence.
- Secrets are redacted in the report.
- Security/privacy NFRs mapped to evidence or reported as gaps.
- Decision follows the severity rule; `Block` for any unresolved Critical/High.
- No file, ticket, status, or release was modified.
- Temporary artifacts under `./.tmp/` removed.

## Resume / interruption

The audit is a single pass over a fixed scope. If interrupted, restart from scope establishment; no persistent state is kept.
