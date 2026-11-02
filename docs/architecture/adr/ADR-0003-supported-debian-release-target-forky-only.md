# Supported Debian Release Target — Forky Only (amd64 `.deb`)

## Status

Accepted

## Date

2026-11-02

## Context

Papyrus is a project-level backlog and documentation manager delivered as a command-line tool. The NFR-03 group ("Implementation and Distribution") records validated constraints for implementation and distribution. **NFR-03.002** requires that Papyrus be distributed as `.deb` packages for the amd64 architecture on Debian Trixie, Forky, and later Debian releases.

**ADR-0002** (Accepted) advertised support for Debian Trixie, Forky, and later releases. During refinement of TASK-4 it was determined that Debian Trixie cannot be easily tested in the available environment. To avoid claiming support that is not verified, Papyrus narrows its advertised and verified support to **Debian Forky only** (amd64 `.deb`). Debian Trixie and any other Debian release are **not** claimed or verified as supported.

This ADR **supersedes ADR-0002**. ADR-0002's body remains frozen; only its status transitions to `Superseded` once this ADR is accepted.

NFR-03.002 still states "Debian Trixie, Forky, and later Debian releases" and therefore does not yet match this decision. Amending NFR-03.002 is out of architect scope and is tracked as an open item for the requirements-engineer (see Open Questions).

## Decision

- Advertise and verify support for **Debian Forky only**, delivered as amd64 `.deb` packages (supersedes ADR-0002).
- Do **not** claim or verify support for Debian Trixie or any other Debian release.
- Only releases that are explicitly installed and smoke-tested may be advertised as supported; at this time that set is Forky alone.
- Build the Debian package artifacts with the available `dpkg-deb` tooling; package metadata and control files are maintained as part of the release process.

## Consequences

**Positive:**

- The advertised support set matches what can actually be verified in the available environment, avoiding unverified support claims.
- Verification matrix is reduced to a single, testable release.

**Negative:**

- Users on Debian Trixie or other releases are not covered by the supported-release commitment.
- Future releases require explicit verification work before being added to the supported set.

**Risks and mitigations:**

- *Support drift*: any future addition to the supported set requires an explicit verification pass and a follow-up ADR/requirement update.
- *glibc/toolchain compatibility*: verify the built binary runs on Debian Forky's glibc; prefer a conservative baseline.
- *Dependency drift*: audit Rust crate licenses and transitive dependencies during release review.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Trixie + Forky (as ADR-0002) | Superseded | Trixie cannot be easily tested in the available environment; unverified support is not claimed. |
| Forky only | Accepted (this ADR) | Matches the verifiable environment; avoids unverified support claims. |
| All Debian releases | Rejected | Unbounded, unverifiable claim; violates the verification discipline. |

## Open Questions

- **NFR-03.002 amendment (requirements-engineer).** NFR-03.002 still requires amd64 `.deb` on Debian Trixie, Forky, and later Debian releases. It must be amended to Debian Forky only to match this ADR. This is out of architect scope; tracked as an open item for the requirements-engineer.
- **"later Debian releases" boundary.** No future release is claimed; adding one requires explicit verification and a requirement/ADR update.

## References

- `docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md` — superseded by this ADR.
- `docs/requirements/non-functional/NFR-03.md` — Implementation and Distribution group.
- `docs/requirements/non-functional/NFR-03.002.md` — Distribute Papyrus as a Debian Package (to be amended).
- Verified environment: `dpkg-deb`, Node.js, Python 3.