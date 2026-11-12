---
name: frontend-style-audit
description: 'Audit frontend (JavaScript, HTML, CSS, Alpine.js) code for style, accessibility, and security compliance. Use when reviewing or cleaning up frontend code.'
argument-hint: 'Examples: audit this view for style and accessibility; check Alpine usage; review CSS naming.'
user-invocable: true
disable-model-invocation: false
---

# Frontend Style Audit

## Outcome

Review frontend code (JavaScript, HTML, CSS, Alpine.js) against common style, accessibility, and security conventions and report violations with concrete fixes.

## When To Use

- Reviewing or cleaning up frontend code.
- Preparing a PR for merge.
- Enforcing style/accessibility in CI or code review.

Do not use this skill for:
- Backend, Android, or non-web code.
- Performance or architecture review (use other skills).

## Checklist

### JavaScript
- Use `const`/`let`; never `var`.
- camelCase for variables/functions; PascalCase for classes/components; UPPER_SNAKE for constants.
- Prefer small, pure functions; one responsibility per function.
- No unused variables/imports; remove dead code.
- Use strict equality (`===`); avoid `==`.
- Handle errors (`try/catch`, `.catch`) and avoid unhandled promises.
- No `eval` or `Function` constructor; no `innerHTML` with untrusted data.
- Prefer `async`/`await` over chained `.then` where clearer.

### HTML
- Semantic elements (`header`, `nav`, `main`, `section`, `button`) over generic `div`/`span`.
- Every interactive element keyboard-accessible; visible focus styles.
- Form inputs have associated `<label>`; use `aria-*` only when semantic HTML is insufficient.
- Images have meaningful `alt` (or `alt=""` for decorative).
- No inline event handlers (`onclick=`); use `addEventListener` or Alpine directives.

### Alpine.js
- One `x-data` per logical widget; keep state minimal.
- Use stores (`Alpine.store`) for shared state; avoid prop drilling via globals.
- Escape output; never use `x-html` with untrusted data.
- Name event handlers consistently (`@click="submit"` not inline logic).
- Debounce noisy inputs (`@input.debounce`).

### CSS
- Consistent naming (BEM or project convention); no deep nesting.
- Mobile-first responsive; avoid `!important` unless overriding third-party.
- Use CSS variables for theming/tokens.
- No unused rules; keep selectors simple.
- Respect `prefers-reduced-motion` and `prefers-color-scheme` where relevant.

### Security
- Escape/sanitize all user content rendered in the DOM.
- Store tokens in memory or httpOnly cookies, not localStorage when possible.
- Validate inputs client-side for UX, but never trust client validation for security.
- Set CSP and other security headers on the server.

### General
- Keep files small; split by feature.
- No console.log in production code.
- Consistent indentation (2 spaces) and formatting (Prettier or project standard).

## Completion checks

- No `var`, `==`, or `eval`/`innerHTML` with untrusted data.
- Semantic HTML and keyboard accessibility verified.
- Alpine directives used correctly (no unsafe `x-html`).
- CSS follows naming convention; no excessive `!important`.
- Report lists each violation with file/line and suggested fix.
