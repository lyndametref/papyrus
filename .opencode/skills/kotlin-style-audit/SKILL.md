---
name: kotlin-style-audit
description: 'Audit Kotlin code (Android apps or backend services) for Google Kotlin style guide compliance: naming, formatting, imports, null safety, and coroutine usage. Use when reviewing or cleaning up Kotlin code.'
argument-hint: 'Examples: audit this module for Kotlin style; check naming and imports; review coroutine usage.'
user-invocable: true
disable-model-invocation: false
---

# Kotlin Style Audit

## Outcome

Review Kotlin code against the Google Kotlin Style Guide and report violations with concrete, actionable fixes.

## When To Use

- Reviewing or cleaning up Kotlin code in Android apps or backend services (Ktor, Spring, etc.).
- Preparing a PR for merge.
- Enforcing style in CI or code review.

Do not use this skill for:
- Java, Groovy, or other JVM languages.
- Performance or architecture review (use other skills).

## Checklist

### Naming
- Classes/objects: PascalCase.
- Functions/properties/variables: camelCase.
- Constants: UPPER_SNAKE_CASE.
- Backing properties: leading underscore (`_foo`).
- Package names: lowercase, no underscores.

### Formatting
- 4-space indentation; no tabs.
- Line length ≤ 100 characters where practical.
- One statement per line; braces on the same line (K&R).
- Trailing commas in multi-line lists/arguments.
- Blank line between members; no trailing whitespace.

### Imports
- No wildcard imports except for extension functions.
- Remove unused imports.
- Group and sort imports (std, third-party, project).
- No fully-qualified names when an import is possible.

### Null safety
- Prefer non-null types; avoid `!!`.
- Use safe calls (`?.`), Elvis (`?:`), and `let`/`run` idiomatically.
- Avoid platform types leaking from Java interop; annotate or convert.
- Prefer `require`/`check`/`error` for precondition failures over nullable returns.

### Coroutines
- Use structured concurrency; avoid `GlobalScope`.
- Android/UI: prefer `viewModelScope`/`lifecycleScope`/`rememberCoroutineScope`.
- Backend/services: use `coroutineScope`/`supervisorScope` (or the framework's scope); keep `runBlocking` at process boundaries only.
- Do not block threads inappropriately; use `Dispatchers.IO`/`Default` (or the framework's dispatcher) as appropriate.
- Handle cancellation; don't swallow `CancellationException`.
- Use `Flow`/`StateFlow`/`SharedFlow` instead of callbacks where reactive streams fit.

### General
- Prefer `val` over `var`; immutability by default.
- Use data classes for value objects; avoid boilerplate.
- Prefer expression bodies for simple functions.
- Avoid `Any`/raw types; use generics.
- Keep functions short and single-purpose.

## Completion checks

- No `!!` without justification.
- No wildcard/unused imports.
- Naming follows conventions.
- Coroutines use structured concurrency.
- Report lists each violation with file and line reference and a suggested fix.
