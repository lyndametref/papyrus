---
name: spring-boot-rest
description: 'Design and implement REST APIs with Spring Boot: controllers, DTOs, validation, error handling, and status codes. Use when building or changing HTTP endpoints.'
argument-hint: 'Examples: add a REST endpoint for X; implement CRUD for Y; fix validation on Z.'
user-invocable: true
disable-model-invocation: false
---

# Spring Boot REST

## Outcome

Implement REST endpoints that are consistent, validated, and return appropriate status codes, keeping web concerns in the adapter layer.

## When To Use

- Adding or changing HTTP endpoints in a Spring Boot app.
- Designing request/response DTOs and validation.
- Implementing error handling and status codes.

Do not use this skill for:
- Persistence or database work (use `postgresql-jpa`).
- Non-HTTP APIs (gRPC, messaging).

## Conventions

- Controllers are thin adapters: parse/validate input, call a use case/port, map output to a response.
- Business logic lives in the domain/use-case layer, not in controllers.
- Use DTOs for request/response; never expose JPA entities directly.
- Validate input with Bean Validation (`@Valid`, `@NotNull`, etc.) at the controller boundary.
- Return proper status codes: 200/201/204, 400, 404, 409, 500.
- Use a global exception handler (`@RestControllerAdvice`) for consistent error responses.
- Version APIs via path (`/api/v1/...`) or headers when needed.

## Procedure

1. **Define the contract** — method, path, request/response shapes, status codes, error cases. Confirm with `@architect` if unresolved.
2. **Create DTOs** — request/response records/classes with validation annotations.
3. **Create the controller** — thin adapter delegating to a use case/port.
4. **Implement error handling** — map domain exceptions to HTTP statuses via `@RestControllerAdvice`.
5. **Write tests** — slice tests (`@WebMvcTest`) for controller logic; integration tests for the full flow.
6. **Verify** — run `gradle-build` tests; check status codes and error payloads.

## Completion checks

- Endpoints return correct status codes for success and error cases.
- Input validation rejects invalid payloads (400).
- No JPA entities leaked in responses.
- Error responses are consistent and do not leak internals.
- Tests cover success, validation failure, and not-found cases.
