---
name: postgresql-jpa
description: 'Persistence with PostgreSQL and JPA/Hibernate: entities, repositories, queries, and schema migrations (Flyway/Liquibase). Use when adding or changing database-backed features.'
argument-hint: 'Examples: add a JPA entity for X; write a repository query; create a Flyway migration.'
user-invocable: true
disable-model-invocation: false
---

# PostgreSQL + JPA

## Outcome

Implement persistence that is correct, migration-safe, and keeps database concerns in the adapter layer, with the domain model at the center.

## When To Use

- Adding or changing JPA entities and repositories.
- Writing queries (JPQL, native, or Spring Data derived).
- Creating or modifying schema migrations (Flyway/Liquibase).
- Tuning fetch strategies or transactions.

Do not use this skill for:
- In-memory or non-relational stores.
- Schema design without a migration (always migrate).

## Conventions

- Entities are persistence adapters; domain models stay framework-agnostic (map between them).
- Use Spring Data JPA repositories; keep queries in the repository, not the service.
- Prefer fetch joins / entity graphs over N+1; avoid `FetchType.EAGER` on collections.
- Always use migrations (Flyway or Liquibase) for schema changes; never rely on `ddl-auto` in production.
- Migrations are immutable once applied: add new files, never edit applied ones.
- Use transactions (`@Transactional`) at the use-case boundary; read-only where applicable.
- Validate uniqueness and constraints at the DB level (unique constraints, not just application checks).
- Use parameterized queries; never concatenate SQL.

## Procedure

1. **Model the domain** — define the domain entity first (framework-agnostic).
2. **Create the JPA entity** — map domain ↔ JPA entity in an adapter.
3. **Create the repository** — Spring Data interface; add derived/custom queries as needed.
4. **Create a migration** — Flyway/Liquibase script for schema changes; test it on a clean DB.
5. **Wire the adapter** — implement the port interface using the repository.
6. **Write tests** — repository slice tests (`@DataJpaTest`) and integration tests with Testcontainers (PostgreSQL).
7. **Verify** — run `gradle-build` tests; confirm migration applies cleanly.

## Completion checks

- Schema changes are in a migration file (no `ddl-auto` reliance).
- No N+1 queries in hot paths (verified via tests/logs).
- Entities are not exposed via REST (DTOs used).
- Repository tests pass against real PostgreSQL (Testcontainers).
- Transactions and read-only flags are set correctly.
