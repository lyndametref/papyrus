---
id: TECH-001
title: Language
created_date: '2026-11-27T20:35:35Z'
updated_date: '2026-11-27T20:35:35Z'
labels: []
references:
  - NFR-003
  - NFR-003.001
status: verified
---
# TECH-001 Language

## Description
Covers the implementation language, the normative style, and the code-quality rules for Papyrus source. Build tooling, packaging, and distribution are separate specifications.

## Language

- Papyrus is implemented in Rust (NFR-003.001).
- The language edition and toolchain are fixed by the build system.

## Style

- The normative style is the [official Rust Style Guide](https://doc.rust-lang.org/style-guide/) and `rustfmt` with its default configuration.
- `cargo fmt --check` passes. Indentation is four spaces, tabs are not used, the line width is 100 where practical, and multiline lists end with a trailing comma.
- Naming follows the guide: `snake_case` for modules, functions, and variables; `CamelCase` for types, traits, and enums; `UPPER_SNAKE_CASE` for constants and statics; short lowercase lifetimes.
- Imports are grouped and sorted. A wildcard import is allowed only for a prelude or a test module.
- Public items carry `///` documentation, with `# Errors`, `# Panics`, and `# Safety` sections where they apply.
- `cargo clippy` reports no new warning. An `allow` is local and justified in a comment at the site.

## Pitfalls to Avoid

- Do not use `unwrap()`, `expect()`, or `panic!` as error handling.
- Do not clone, or reach for `Rc<RefCell<_>>` or global mutable state, just to satisfy the borrow checker.
- Do not build deep `if`/`match` nesting or encode a closed state set in strings or booleans.

## Error Handling

- A fallible operation returns `Result`; `?` propagates the error.
- `unwrap()`, `expect()`, and `panic!` are not error handling. They are reserved for a programmer invariant, with a message that names the invariant.
- An error is never swallowed; it is propagated or converted at an application boundary.

## Ownership

- Borrow before cloning. A clone is justified by a lifetime or ownership need, not by a borrow-checker workaround.
- Prefer plain ownership and borrowing over `Rc<RefCell<_>>` or global mutable state.
- The project baseline is safe Rust. An `unsafe` block needs a dedicated design and a `// SAFETY:` comment that states its invariant.

## Short Code and Complexity

- Short means one responsibility and few branches, not a dense one-liner.
- Use early returns and guard clauses instead of nested `if` or `match`.
- Prefer iterators and combinators over manual index loops; do not index where the index can be out of bounds.
- Use an enum for a state or a closed set of variants; do not encode state in strings or booleans.
- Target the project complexity gate: cyclomatic complexity at most 10, cognitive complexity at most 15, and function length at most 40 lines.
- The medium band (cyclomatic 11-15, cognitive 16-25, length 41-80 lines) needs a justification in review; the high and blocker bands are not accepted.
- Keep the public surface minimal; a helper that is not needed outside its module stays private.
