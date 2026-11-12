---
name: rust-style-audit
description: 'Audit Rust code for compliance with the official Rust Style Guide, rustfmt, and Clippy: naming, formatting, imports, error handling, ownership/borrowing, unsafe usage, and concurrency. Use when reviewing or cleaning up Rust code.'
argument-hint: 'Examples: audit this crate for Rust style; check naming and formatting; review unsafe and error handling.'
user-invocable: true
disable-model-invocation: false
---

# Rust Style Audit

## Outcome

Review Rust code against the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/), `rustfmt` defaults, and Clippy lints; report violations with concrete, actionable fixes.

## When To Use

- Reviewing or cleaning up Rust code in libraries or binaries.
- Preparing a PR for merge.
- Enforcing style in CI or code review.

Do not use this skill for other languages.

## Checklist

### Naming
- Crates/modules: snake_case.
- Types/traits/enums: CamelCase.
- Functions/methods/variables: snake_case.
- Constants/statics: UPPER_SNAKE_CASE.
- Type parameters: single uppercase letter or CamelCase (`T`, `Item`).
- Lifetimes: short lowercase (`'a`, `'de`).

### Formatting
- Run `rustfmt` (default config) as the source of truth; `cargo fmt --check` must pass.
- 4-space indentation; no tabs.
- Line width ≤ 100 characters where practical (rustfmt default).
- Trailing commas in multi-line lists/arguments.
- One item per line in use statements when formatted.
- No trailing whitespace.

### Imports
- Group and sort imports; prefer `use` paths over fully-qualified names.
- Remove unused imports (`cargo clippy`/`unused_imports`).
- Avoid wildcard imports (`use foo::*`) except in preludes/tests where justified.
- Prefer `pub use` re-exports deliberately; keep public API surface intentional.

### Error handling
- Use `Result<T, E>`; avoid panics in library code (`unwrap`/`expect` only with justification).
- Prefer `?` propagation; define domain error types (`thiserror`) or use `anyhow` at application boundaries.
- Don't swallow errors; log or propagate with context.
- Avoid `Box<dyn Error>` in public APIs when a concrete type is feasible.

### Ownership and borrowing
- Prefer borrowing (`&T`, `&mut T`) over cloning; clone only when needed and justified.
- Avoid unnecessary `Arc`/`Rc`/`Mutex`; use the simplest synchronization.
- Prefer iterators and combinators over manual indexing.
- Avoid indexing panics; use `.get()` or pattern matching where appropriate.

### Unsafe
- Minimize `unsafe` blocks; each must have a `// SAFETY:` comment justifying invariants.
- Encapsulate unsafe in small, audited modules; expose safe wrappers.
- Prefer safe abstractions (e.g., `MaybeUninit` carefully, or existing crates).

### Concurrency
- Use structured concurrency; avoid detached threads without join handles.
- Prefer `tokio`/`async-std` runtimes consistently; don't block async executors.
- Protect shared state with appropriate primitives; avoid data races (compile-time guarantees).
- Handle cancellation and timeouts explicitly.

### Documentation and API
- Public items should have `///` doc comments.
- Include `# Errors`, `# Panics`, `# Safety` sections where relevant.
- Prefer small, focused crates/modules; keep `pub` surface minimal.

### General
- Prefer `let` bindings and immutability by default.
- Use enums for state machines; avoid stringly-typed APIs.
- Prefer `match` exhaustiveness; avoid catch-all `_` when variants are meaningful.
- Keep functions short and single-purpose.
- Use `#[must_use]` on important return values.

## Completion checks

- `cargo fmt --check` passes.
- `cargo clippy` reports no new warnings (or justified allows).
- No `unwrap`/`expect`/`panic!` in library code without justification.
- Every `unsafe` block has a `// SAFETY:` comment.
- Naming follows conventions.
- Report lists each violation with file and line reference and a suggested fix.
