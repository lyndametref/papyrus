---
name: gradle-build
description: 'Build, test, and inspect Android/Gradle projects: common Gradle tasks, test execution, lint, dependency resolution, and troubleshooting. Use when building or testing a Gradle-based (Android) project.'
argument-hint: 'Examples: run unit tests; build debug APK; check lint; resolve dependency conflicts.'
user-invocable: true
disable-model-invocation: false
---

# Gradle Build

## Outcome

Build, test, and diagnose Android/Gradle projects using the Gradle wrapper and standard tasks, without breaking the build.

## When To Use

- Building or assembling an Android app/module.
- Running unit or instrumented tests.
- Running lint/static analysis.
- Resolving dependency or build configuration issues.

Do not use this skill for non-Gradle projects (use the project's own build tool).

## Conventions

- Always use the wrapper: `./gradlew` (never a globally installed `gradle`).
- Run from the module/project root that contains `gradlew`.
- Prefer offline when dependencies are cached: `--offline`.
- Use `--stacktrace` / `--info` only when diagnosing failures.

## Common tasks

- `./gradlew assembleDebug` — build debug APK/AAB.
- `./gradlew test` — run all unit tests (all modules).
- `./gradlew :app:testDebugUnitTest` — unit tests for one module/variant.
- `./gradlew connectedAndroidTest` — instrumented tests (device/emulator required).
- `./gradlew lint` / `./gradlew lintDebug` — Android lint.
- `./gradlew dependencies` / `./gradlew :app:dependencies` — dependency tree.
- `./gradlew clean` — clean build outputs.
- `./gradlew tasks --all` — list available tasks.

## Test execution

- Run the narrowest scope first: single module/variant, then broaden.
- Use `--tests "com.example.FooTest"` to run a single test class.
- Use `--tests "*should_*"` for pattern matching.
- Add `--info` to see test execution details on failure.

## Troubleshooting

- **Stale cache / corrupt build**: `./gradlew clean` then rebuild; or delete `.gradle/` cache if needed.
- **Dependency conflict**: inspect `./gradlew dependencies`; use `exclude` or force versions in `build.gradle.kts`.
- **Daemon issues**: `./gradlew --stop` then retry.
- **Out of memory**: increase `org.gradle.jvmargs` in `gradle.properties`.
- **Flaky instrumented tests**: rerun with `--rerun-tasks`; check device state.

## Completion checks

- Build succeeds (`BUILD SUCCESSFUL`).
- Target tests pass (no failures).
- Lint reports no new errors (warnings acceptable unless configured otherwise).
- No files outside the project folder were modified.
