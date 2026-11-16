---
name: frontend-engineering
description: >-
  Use this skill when building, styling, or testing the SvelteKit + Svelte 5 +
  TypeScript + Tailwind CSS web application in apps/web, including accessibility,
  SSE progress streaming, virtualized trees, and responsive UX.
---

# Web Frontend & UX Engineering Skill

## 1. Purpose
Build a responsive, accessible, high-performance web application (`apps/web`) where every UI control performs its advertised action against real state and verified backend API contracts.

## 2. When to Use
- Implementing design tokens, reusable UI components, pages, or state stores in `apps/web`.
- Connecting the frontend to `services/api` REST endpoints and Server-Sent Events (SSE) progress streams.
- Implementing virtualized repository file trees, syntax-highlighted output previews, and token statistics dashboards.
- Running accessibility, keyboard navigation, and responsive viewport validations.

## 3. Required Procedures
1. Use **Svelte 5** runes (`$state`, `$derived`, `$effect`, `$props`) and strict **TypeScript** types matching `docs/API.md`.
2. Ensure every interactive control has keyboard support (`Tab`, `Enter`, `Space`, `Escape`, arrow keys for trees), visible focus rings, and explicit ARIA labels/live regions.
3. Render large file trees and multi-megabyte previews using virtualized lists or windowed pagination to prevent DOM lag.
4. Handle loading, partial completion, exclusion warnings, and error states explicitly—never fail silently or show fake progress.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [03-frontend.md](../../rules/03-frontend.md)
- Enforces [06-testing.md](../../rules/06-testing.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- `npm run check` (svelte-check + TypeScript) passes with 0 errors and 0 warnings.
- `npm run lint` and `npm test` pass 100%.
- Responsive verification across mobile (`375px`), tablet (`768px`), and desktop (`1440px`) viewports.

## 6. Expected Outputs
- Accessible, responsive Svelte 5 components, routes, and state modules in `apps/web` backed by automated component and E2E tests.

## 7. Common Failure Modes
- **Dead Controls:** Rendering filter checkboxes, format buttons, or settings toggles that do not actually include their values in the conversion job payload.
- **DOM Freezing:** Rendering 10,000+ repository tree nodes or a 5 MB Markdown preview directly into the DOM without virtualization.
- **SSE Leaks:** Failing to close `EventSource` connections when a job completes or when the component unmounts.
