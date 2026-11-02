# Repository Root Single-Crate Layout for the Papyrus Rust Crate

## Status

Accepted

## Date

2026-11-02

## Context

Papyrus is a single Rust command-line tool and the repository is the project itself; it is not a multi-application monorepo. **NFR-03.001** requires that Papyrus be implemented in Rust, and **ADR-0001** selects Rust with Cargo as the implementation language and build system. The **TASK-1 specification** (`docs/specifications/rust-cli-and-debian-package-baseline.md`) requires a minimal single-crate layout (`src/main.rs` + `src/lib.rs`), explicitly excludes a multi-crate workspace, and assumes any move to a workspace or to module boundaries beyond that minimal split requires a new ADR.

`AGENTS.md` lists generic default write scopes such as `apps/rust/**`, `apps/backend/**`, and `apps/webFrontend/**` for multi-application projects. No requirement, specification, or ADR mandates an `apps/` subdirectory for Papyrus, and Papyrus has no other application components that would justify one. The current repository already places `Cargo.toml`, `src/`, `tests/`, `rust-toolchain.toml`, `scripts/`, and `packaging/` at the repository root, and both `build-deb.sh` and `docs/packaging.md` reference a root-level build producing `target/release/papyrus`.

This ADR formalizes that the root-level single-crate placement is deliberate rather than incidental, so that the layout is not silently "corrected" into an `apps/` directory and so that any future relocation is treated as an explicit architectural change.

## Decision

- Place the Papyrus Rust crate at the **repository root**: `Cargo.toml`, `src/` (with `src/main.rs` + `src/lib.rs`), `tests/`, `rust-toolchain.toml`, and build/packaging assets (`scripts/`, `packaging/`, `build-deb.sh`) all live at the repository root.
- Keep Papyrus as a **single crate** with binary and library targets; do **not** introduce a Cargo workspace and do **not** nest the crate under an `apps/<name>/` directory.
- Treat the generic `apps/**` write scopes in `AGENTS.md` as **not applicable** to Papyrus; Papyrus-specific code and build artifacts live at the repository root.
- Any future relocation of the crate into `apps/<name>/` (or any other directory), or conversion to a multi-crate workspace, **requires a new ADR and an accompanying refactor**. Ad-hoc moves are not permitted.

## Consequences

**Positive:**

- Matches the minimal single-crate layout required by the TASK-1 specification and ADR-0001 without added indirection.
- Cargo commands (`cargo build`, `cargo test`) run from the repository root, matching existing build and packaging references (`target/release/papyrus`, `build-deb.sh`, `docs/packaging.md`).
- Avoids an unnecessary directory level and keeps the project navigable as the project itself.

**Negative:**

- Contributors familiar with the generic multi-app convention must be aware that Papyrus does not use an `apps/` directory; its write scope is the repository root.
- If Papyrus is later absorbed into a larger multi-application repository, relocation and path updates will be required (mitigated by requiring a new ADR and refactor).

**Risks and mitigations:**

- *Accidental relocation*: guard against silent moves by treating the root layout as an accepted decision; any change requires a new ADR.
- *Scope confusion*: document in architecture/AGENTS context that the generic `apps/**` scopes do not apply to Papyrus.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Repository root single crate (this ADR) | Accepted | Matches the TASK-1 minimal layout and ADR-0001; simplest; consistent with existing root-level build/packaging references. |
| Nest under `apps/rust/` or `apps/papyrus/` | Rejected | No requirement or ADR mandates an `apps/` directory for Papyrus; adds path indirection and breaks existing build/packaging references without benefit. |
| Cargo workspace with multiple crates | Rejected | Violates the TASK-1 assumption of a minimal single-crate layout; out of scope and requires a new ADR. |

## References

- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust + Cargo implementation language.
- `docs/specifications/rust-cli-and-debian-package-baseline.md` — TASK-1 spec (minimal single-crate layout, no workspace).
- `docs/requirements/non-functional/NFR-03.001.md` — Implement Papyrus in Rust.
- `AGENTS.md` — generic default write scopes (`apps/rust/**`, etc.), not binding for Papyrus.
- `build-deb.sh`, `docs/packaging.md` — root-level build references (`target/release/papyrus`).
- Approval note: accepted per explicit user request on 2026-11-02; index regeneration left to the caller.