---
name: spring-boot-testing
description: 'Test Spring Boot applications: unit tests, slice tests (@WebMvcTest, @DataJpaTest), and integration tests with Testcontainers. Use when verifying backend behavior.'
argument-hint: 'Examples: add unit tests for a use case; slice-test a controller; integration-test an endpoint with Testcontainers.'
user-invocable: true
disable-model-invocation: false
---

# Spring Boot Testing

## Outcome

Verify backend behavior at the right level (unit, slice, integration) with deterministic, fast, and meaningful tests.

## When To Use

- Adding tests for services, controllers, repositories, or endpoints.
- Verifying validation, error handling, and persistence.
- Diagnosing flaky or slow tests.

Do not use this skill for:
- Android/instrumented tests (use `gradle-build` + Android test tooling).
- Pure unit tests with no Spring context (plain JUnit).

## Test levels

- **Unit**: domain logic and use cases with faked ports (fast, no Spring context).
- **Slice**: `@WebMvcTest` (controller + validation), `@DataJpaTest` (repository + DB).
- **Integration**: `@SpringBootTest` with Testcontainers (PostgreSQL) for end-to-end flows.

Prefer the lowest level that still exercises the real behavior.

## Conventions

- Use JUnit 5 + AssertJ (or the project's chosen assertions).
- Use `@MockBean`/`@MockkBean` sparingly; prefer fakes for domain ports.
- Use Testcontainers for real PostgreSQL in integration tests; never H2 for production-parity checks.
- Keep tests deterministic: no real network, fixed clocks, isolated data (clean DB per test or transactional rollback).
- Name tests after behavior: `should_<expected>_when_<condition>`.
- Test validation failures and error paths, not just happy paths.

## Procedure

1. **Identify the level** — unit, slice, or integration (see above).
2. **Write the test** — arrange (given), act (when), assert (then); one behavior per test.
3. **Run** — use `gradle-build` (`./gradlew test` or targeted `--tests`).
4. **Fix failures** — make the smallest change; re-run.
5. **Check coverage** — ensure ACs and edge cases are covered.

## Completion checks

- Tests pass (`BUILD SUCCESSFUL`).
- Each AC has at least one test.
- Error/validation paths are tested.
- Integration tests use real PostgreSQL (Testcontainers), not H2.
- No flaky or order-dependent tests.
