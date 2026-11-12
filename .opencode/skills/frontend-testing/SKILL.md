---
name: frontend-testing
description: 'Test vanilla JavaScript and Alpine.js UIs: unit tests (Vitest/Jest) and end-to-end tests (Playwright). Use when verifying frontend behavior.'
argument-hint: 'Examples: add unit tests for a module; E2E test a form submit; test an Alpine component.'
user-invocable: true
disable-model-invocation: false
---

# Frontend Testing

## Outcome

Verify frontend behavior with fast, deterministic unit tests and targeted end-to-end tests, keeping the test setup light.

## When To Use

- Adding tests for UI logic, components, or fetch flows.
- Verifying validation, error states, and rendering.
- Diagnosing flaky or slow tests.

Do not use this skill for:
- Backend/API tests (use `spring-boot-testing`).
- Android/instrumented tests (use `gradle-build`).

## Test levels

- **Unit**: pure functions and small modules (fast, no DOM or with jsdom).
- **Component**: Alpine component logic with a lightweight DOM (jsdom/happy-dom).
- **E2E**: critical user flows in a real browser (Playwright).

Prefer the lowest level that still exercises the real behavior.

## Conventions

- Use Vitest or Jest for unit/component tests; Playwright for E2E.
- Mock `fetch` in unit tests; use a test server or MSW for integration.
- Keep tests deterministic: no real network, fixed data, isolated state.
- Name tests after behavior: `should_<expected>_when_<condition>`.
- Test loading, error, and empty states, not just happy paths.
- Avoid heavy test frameworks that require a complex build; keep config minimal.

## Procedure

1. **Identify the level** — unit, component, or E2E.
2. **Write the test** — arrange, act, assert; one behavior per test.
3. **Run** — `npm test` or `npx vitest run` / `npx playwright test`.
4. **Fix failures** — smallest change; re-run.
5. **Check coverage** — ensure ACs and edge cases are covered.

## Completion checks

- Tests pass.
- Each AC has at least one test.
- Error/loading/empty states are tested.
- No flaky or order-dependent tests.
- No heavy build tooling introduced.
