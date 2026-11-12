---
name: tdd
description: 'Test-driven development workflow applicable to any language: write a failing test first, implement the smallest change to make it pass, then refactor. Use before implementing any feature or fix.'
argument-hint: 'Examples: implement login with TDD; fix the off-by-one error test-first; add tests for the new use case.'
user-invocable: true
disable-model-invocation: false
---

# Test-Driven Development (TDD)

## Outcome

Implement behavior through short red-green-refactor cycles so every change is verified by an automated test before it lands. Language- and framework-agnostic.

## When To Use

- Implementing a new feature or fixing a bug.
- Adding or changing behavior in any module.
- Before writing production code when tests are feasible.

Do not use this skill for:
- Pure documentation, configuration, or asset changes with no logic.
- Exploratory spikes where tests would be throwaway (prototype first, then TDD the real change).

## Cycle

1. **Red** — Write one small, focused test that describes the desired behavior. Run it; confirm it fails for the right reason (not a compile error or wrong assertion).
2. **Green** — Write the minimum production code to make the test pass. Do not add behavior beyond what the test requires.
3. **Refactor** — Clean up (naming, duplication, structure) while keeping tests green. Run tests after each refactor step.
4. Repeat for the next behavior.

## Rules

- Never write production code without a failing test first (when feasible).
- One behavior per cycle; keep tests small and independent.
- Tests must be deterministic and fast; avoid real network, clock, or filesystem unless the behavior under test requires it (use fakes/seams).
- Name tests after behavior: `should_<expected>_when_<condition>`.
- A test that never fails is not a test — verify the red step.
- Refactor only with green tests; stop refactoring when tests stay green but code is clean.

## Test levels

- **Unit**: domain logic, use cases, pure functions (fast, isolated).
- **Integration**: adapters, repositories, serialization (real dependencies, faked externals).
- **End-to-end / UI**: critical user flows (slow; keep few).

Match the level to the behavior; prefer the lowest level that still exercises the real logic.

## Edge cases to cover

- Empty/null/zero inputs, boundary values, off-by-one.
- Error and failure paths, not just happy paths.
- Concurrency/ordering where relevant.
- Regression tests for every bug fixed.

## Completion checks

- Every new behavior has at least one test that failed before implementation.
- All tests pass after implementation and refactoring.
- No production code was added without a corresponding test (except trivial glue).
- Tests are readable, deterministic, and free of flaky dependencies.
