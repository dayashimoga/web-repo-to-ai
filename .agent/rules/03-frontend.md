# Rule 03: Frontend & UX Engineering Standards

1. **Stack Compliance:** Use SvelteKit, Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`), strict TypeScript, and Tailwind CSS in `apps/web`.
2. **Functional Controls Only:** Every button, input, toggle, and download control rendered in the UI must be wired to real state or tested API contracts.
3. **Accessibility (WCAG AA):** Use semantic HTML landmarks, keyboard-navigable controls, visible focus states, ARIA attributes for live progress/trees, and sufficient color contrast.
4. **Responsive Design:** All layouts must function cleanly across mobile (320px+), tablet (768px+), and desktop (1280px+) viewports without horizontal overflow.
5. **Large-Output Virtualization:** File trees and large document previews must use virtualized rendering or pagination to keep browser memory bounded.
