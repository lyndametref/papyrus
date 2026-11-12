---
name: alpine-js
description: 'Alpine.js patterns for lightweight reactive UIs: x-data, directives, stores, and fetch integration with REST APIs. Use when building or changing frontend views with Alpine.js.'
argument-hint: 'Examples: add a reactive form with Alpine; use a store for shared state; fetch data from the REST API.'
user-invocable: true
disable-model-invocation: false
---

# Alpine.js

## Outcome

Build small, reactive UIs with Alpine.js on plain HTML, keeping payloads light and avoiding a build step.

## When To Use

- Adding interactivity to server-rendered or static HTML.
- Managing small client-side state (forms, toggles, lists).
- Fetching and rendering data from a REST API.

Do not use this skill for:
- Large SPAs with complex routing/state (consider a heavier framework).
- Server-side rendering frameworks.

## Conventions

- Keep components small; one `x-data` per logical widget.
- Use `x-init` for initial data fetch; handle loading/error states.
- Stores (`Alpine.store`) for shared state across components.
- Escape output to avoid XSS; never inject untrusted HTML via `x-html` unless sanitized.
- Prefer `fetch` with async/await; handle non-2xx responses.
- Keep Alpine + app code in one or few small files; no bundler.

## Patterns

- **Reactive state**: `x-data="{ count: 0 }"`, `x-text`, `x-show`, `x-if`.
- **Events**: `@click`, `@submit.prevent`, `@input.debounce`.
- **Lists**: `x-for="(item, i) in items" :key="item.id"`.
- **Fetch on init**: `x-init="fetch('/api/x').then(r=>r.json()).then(d=>items=d)"`.
- **Form submit**: `@submit.prevent="submit()"` with validation before fetch.
- **Stores**: `Alpine.store('auth', { token: null })`, access via `$store.auth`.

## Completion checks

- No XSS: user content escaped or sanitized.
- Loading and error states handled for fetches.
- No heavy dependencies added; files stay small.
- Works without JS for core content where feasible.
