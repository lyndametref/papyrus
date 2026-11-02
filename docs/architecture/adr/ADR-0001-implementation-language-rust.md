# Implementation Language — Rust

## Status

Accepted

## Date

2026-11-01

## Context

Papyrus is a project-level backlog and documentation manager delivered as a command-line tool. The NFR-03 group ("Implementation and Distribution") records validated constraints for implementation and distribution. **NFR-03.001** requires that the Papyrus application be implemented in Rust. This is a firm, decided constraint.

The available toolchain in the verified environment includes Rust 1.99.1, Cargo, and rustup. Node.js and Python 3 are available as auxiliary build/release tooling only and are not runtime dependencies of the distributed CLI.

## Decision

- Implement Papyrus in **Rust** as the sole implementation language (NFR-03.001).
- Use **Cargo** for dependency management, reproducible builds, and the testing workflow.
- Rust 1.99.1 (via rustup) is the pinned baseline toolchain for builds.
- Node.js and Python 3, where used, remain build-time/auxiliary tooling only and are never runtime dependencies of the distributed CLI.

## Consequences

**Positive:**

- A single compiled Rust binary keeps runtime dependencies minimal and startup fast.
- Cargo provides reproducible builds, dependency management, and a testing workflow.
- Rust's type system and ownership model support long-term maintainability and safety.

**Negative:**

- Rust compile times and a learning curve increase development cost.

**Risks and mitigations:**

- *Dependency drift*: audit Rust crate licenses and transitive dependencies during release review.
- *Toolchain pinning*: keep the Rust toolchain version recorded and reproducible across builds.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Python implementation | Rejected | Violates NFR-03.001; adds a runtime interpreter dependency and packaging complexity. |
| Go implementation | Rejected | Violates NFR-03.001. |
| Shell/Node.js CLI | Rejected | Violates NFR-03.001; not suitable for a structured backlog/documentation manager. |

## References

- `docs/requirements/non-functional/NFR-03.md` — Implementation and Distribution group.
- `docs/requirements/non-functional/NFR-03.001.md` — Implement Papyrus in Rust.
- Verified environment: Rust 1.99.1, Cargo, rustup.
