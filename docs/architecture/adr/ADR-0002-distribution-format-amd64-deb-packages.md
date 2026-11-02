# Distribution Format — amd64 `.deb` Packages for Debian Trixie, Forky, and Later Releases

## Status

Accepted

## Date

2026-11-01

## Context

Papyrus is a project-level backlog and documentation manager delivered as a command-line tool. The NFR-03 group ("Implementation and Distribution") records validated constraints for implementation and distribution. **NFR-03.002** requires that Papyrus be distributed as `.deb` packages for the amd64 architecture on Debian Trixie, Forky, and later Debian releases.

The distribution target is amd64 Debian `.deb` packages. The available tooling in the verified environment includes `dpkg-deb`, Node.js (for browser UI assets at build time only), and Python 3 (auxiliary build/release tooling).

NFR-03.002 contains an explicit open question: the phrase "later Debian releases" does not specify a terminal release boundary or which future releases must be verified. This ADR records the decision as stated and captures that open question rather than resolving it silently.

## Decision

- Distribute Papyrus as **amd64 `.deb` packages** targeting **Debian Trixie, Forky, and later Debian releases** (NFR-03.002).
- Build the Debian package artifacts with the available `dpkg-deb` tooling; package metadata and control files are maintained as part of the release process.
- Node.js is available for browser UI asset tooling at build time only; it is not a runtime dependency of the distributed CLI.
- Python 3 is available as auxiliary build/release tooling; it is not a runtime dependency of the distributed CLI.

## Consequences

**Positive:**

- `.deb` packaging for amd64 is straightforward with `dpkg-deb` and matches the required distribution format.

**Negative:**

- Debian package metadata (control fields, dependencies, versioning) must be maintained and validated per release.
- Each claimed supported Debian release requires installation and verification work.

**Risks and mitigations:**

- *Unbounded "later releases" claim*: until the open question is resolved, only Trixie and Forky are advertised as supported; additional releases require explicit verification before being claimed (see Open Questions).
- *glibc/toolchain compatibility*: verify the built binary runs on each claimed release's glibc; prefer a conservative baseline.
- *Dependency drift*: audit Rust crate licenses and transitive dependencies during release review.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| AppImage / Flatpak / tarball distribution | Rejected | Violates NFR-03.002, which requires amd64 `.deb` packages. |
| Additional architectures (arm64, etc.) | Out of scope | NFR-03.002 specifies amd64 only; not added without a requirement change. |

## Open Questions

- **NFR-03.002 — "later Debian releases" boundary.** The phrase does not specify a terminal release boundary or which future releases must be verified. This ADR does not resolve it. Follow-up: define a supported-release list and verification matrix (install + smoke test on each claimed release) before advertising support beyond Debian Trixie and Forky. Tracked against NFR-03.002's Verification Method.

## References

- `docs/requirements/non-functional/NFR-03.md` — Implementation and Distribution group.
- `docs/requirements/non-functional/NFR-03.002.md` — Distribute Papyrus as a Debian Package.
- Verified environment: `dpkg-deb`, Node.js, Python 3.
- Approval note: Distribution format decision approved by the user on 2026-11-02; status changed from Proposed to Accepted. Open question on "later Debian releases" boundary remains unresolved (see Open Questions).
