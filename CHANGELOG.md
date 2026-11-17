# CHANGELOG (`CHANGELOG.md`)

**Policy:** Append-only chronological record of verified project changes by Stage, Phase, and Subphase. Never overwrite or delete historical entries.

---

## [Unreleased] — Stage 2: Backend and Processing Foundation

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.3`), Phase 1.4 Certification & Stage 1 Milestone Certification

#### Added
- **`[1.4.3]` UI Performance Benchmarks, Asset Budget Audits, Design Token Regression & Stage 1 Certification:**
  - Authored comprehensive performance and visual regression test suite `apps/web/tests/ui-performance.test.ts` (11 tests across 3 categories):
    - **Production Bundle Size & Dependency Budget:** Asserted `package.json` contains zero external runtime dependencies (`dependencies` is empty); verified absence of bloated third-party libraries (no lodash, moment, prismjs, highlight.js, rxjs, lucide-react, MUI, Chakra); verified client build asset footprints strictly satisfy budgets (largest JS chunk $57.4\text{ kB} \le 100\text{ kB}$ raw, largest CSS chunk $44.1\text{ kB} \le 60\text{ kB}$ raw, total client bundle $\sim 260\text{ kB} \le 350\text{ kB}$ raw).
    - **Algorithmic Throughput & Micro-Benchmarks:** Verified high-throughput execution for syntax highlighting ($> 20,000\text{ lines/sec}$ on 1,000 JSON lines; $< 40\text{ms}$ on 500+ XML lines), token budget utilization evaluation (10,000 calls in $< 50\text{ms}$, $> 200,000\text{ ops/sec}$), URL classification and SSRF pre-checking (5,000 calls in $< 50\text{ms}$), and Web Crypto SHA-256 integrity digests (500 KB payload in $< 50\text{ms}$).
    - **Visual Design System & Styling Token Regression:** Verified full CSS custom properties export in `tokens.css` (`--color-brand-*`, `--color-accent-*`, `--color-surface-*`, `--color-status-*`, `--elevation-*`, `--spacing-*`, `--radius-*`, `.dark` theme overrides); verified Tailwind semantic utility classes (`.bg-theme-canvas`, `.bg-theme-panel`, `.bg-theme-subtle`, `.text-theme-primary`, `.text-theme-secondary`, `.text-theme-muted`, `.border-theme-subtle`, `.border-theme-strong`); verified WCAG AAA contrast compliance across both Dark mode ($\ge 14:1$ canvas, $\ge 12:1$ panel) and Light mode ($\ge 14:1$), plus AA for brand buttons ($\ge 4.5:1$); audited all 12 frontend components confirming zero inline arbitrary hex color pollution.
  - Expanded total automated test suite to 184/184 tests passing across 15 test suites with 0 failures.
  - Formally certified completion of **Phase 1.4: UX validation** across all 3 subphases (`1.4.1` – `1.4.3` all `ACCEPTED`).
  - Formally certified completion of **Stage 1: Frontend and Product Experience** across all 15 subphases (`1.1.1` – `1.4.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.2`)

#### Added
- **`[1.4.2]` Mobile, Tablet, and Desktop Viewport Tests & Responsive Invariant Suite:**
  - Expanded `apps/web/tests/responsive-layout.test.ts` into a multi-viewport certification test suite (28 tests across 6 sections):
    - **Tailwind Breakpoint Spectrum:** Verified resolution mapping across representative devices for Mobile (`xs`: 320px–480px, including iPhone SE, iPhone 14/15, Pixel 7), Tablet (`sm`: 640px, `md`: 768px–834px, `lg`: 1024px, including iPad Mini, iPad 10th Gen, iPad Air), and Desktop (`xl`: 1280px–1440px, `2xl`: 1536px–2560px, including standard laptops, QHD, and 1080p FHD displays).
    - **Mobile Viewport Invariants (320px–480px):** Verified horizontal overflow prevention (`overflow-x-hidden`, `max-w-full`, `min-h-screen`), standard responsive container padding (`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8`), Hero typography scaling (`text-3xl sm:text-5xl lg:text-6xl`), full-width action button stacking (`flex-col`, `w-full min-h-[48px]`), format pills 2-column collapse, value metric 1-column collapse, `UrlInput` vertical stacking with iOS auto-zoom prevention (`text-sm sm:text-base`), content grids single-column collapse, and Header mobile hamburger menu button (`md:hidden`).
    - **Tablet Viewport Invariants (640px–1024px):** Verified horizontal button flow at `sm` (`sm:flex-row`), 3-column format pills (`sm:grid-cols-3`), 3-column metric cards (`sm:grid-cols-3`), 3-column workflow cards (`md:grid-cols-3`), 2-column sources and feature grids (`md:grid-cols-2`), and Header desktop navigation visibility (`hidden md:flex`).
    - **Desktop Viewport Invariants (1280px–1920px+):** Verified 6-column format matrix (`lg:grid-cols-6`), 3-column feature highlights (`lg:grid-cols-3`), 5-column footer matrix (`lg:grid-cols-5`), maximum headline typography (`lg:text-6xl`), and container constraints preventing ultra-wide stretching (`max-w-7xl mx-auto`, `max-w-6xl mx-auto`).
    - **Conversion Workspace Viewport Invariants:** Verified workspace header responsive alignment (`flex-col sm:flex-row items-start sm:items-center`), 4-stage pipeline execution cards collapsing (1-col mobile, 2-col tablet, 4-col desktop), completeness accounting matrix (2-col mobile, 4-col tablet+), `TokenBudgetBar` metric grid, `PreviewPanel` 12-column responsive layout with sidebar stacking on mobile and side-by-side splitting on tablet/desktop (`grid-cols-1 md:grid-cols-12`), `DownloadControls` artifact cards (`flex-col md:flex-row md:items-center`), and bounded execution telemetry log scroll (`max-h-64 overflow-y-auto`).
    - **Touch Target Accessibility (WCAG 2.1 AA/AAA):** Verified $\ge 44 \times 44\text{px}$ touch targets for Header hamburger toggle and theme buttons, $\ge 44\text{px}$ touch heights for mobile dropdown links, and $\ge 48\text{px}$ touch heights for Hero CTA and `UrlInput` controls.
  - Expanded total automated test suite to 173/173 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.1`)

#### Added
- **`[1.4.1]` Keyboard Navigation and Accessibility Audits (WCAG 2.1 AA & AAA):**
  - Authored comprehensive accessibility audit test suite `apps/web/tests/a11y-audit.test.ts` (19 tests) asserting compliance across 8 distinct accessibility dimensions:
    - **Semantic HTML Landmarks:** Validated `<header>`, `<nav aria-label="Main Navigation">`, `<main id="main-content">`, and `<footer>` on both landing page and workspace routes; verified explicit `<section>` regions with `aria-labelledby` across `Hero.svelte`, `WorkflowSteps.svelte`, `SupportedSources.svelte`, and `FeatureHighlights.svelte`.
    - **WAI-ARIA Progressbar Pattern:** Validated compliance in `JobWorkspace.svelte` (overall pipeline progress) and `TokenBudgetBar.svelte` (context utilization progressbar) asserting `role="progressbar"`, dynamic `aria-valuenow`, `aria-valuemin={0}`, `aria-valuemax={100}`, and descriptive `aria-label`.
    - **WAI-ARIA Tabs Design Pattern:** Validated bidirectional linkage in `PreviewPanel.svelte` format tabs asserting `role="tablist"`, interactive tab controls (`role="tab"`, `id="tab-{id}"`, dynamic `aria-selected`, `aria-controls`), and panel containers (`role="tabpanel"`, `id="panel-{id}"`, `aria-labelledby`).
    - **Polite Live Regions & Assertive Alerts:** Verified non-disruptive feedback live regions (`role="status"`, `role="log"`, `aria-live="polite"`) across `UrlInput`, `JobWorkspace`, `PreviewPanel`, and `DownloadControls`, alongside assertive error and budget overflow banners (`role="alert"`).
    - **Screen Reader Exclusion of Decorative Content:** Verified `aria-hidden="true"` on line number gutter cells (`line.lineNumber`) and decorative background SVGs and visual effects.
    - **Universal Visible Focus Indicators:** Verified WCAG 2.4.7 visible focus styles (`focus-visible:ring` and `focus-visible:outline`) across interactive primitives (`Button`, `Input`, `Toggle`, `UrlInput`, `ConversionSettings`, `PreviewPanel`, `TokenBudgetBar`, `DownloadControls`).
    - **Color Contrast Certification:** Mathematically verified WCAG AAA ($\ge 7.0:1$) compliance for dark canvas ($14.0:1$) and panels ($12.0:1$) with white text, light canvas with dark slate text ($14.0:1$), and WCAG AA ($\ge 4.5:1$) for brand primary buttons.
    - **Keyboard Navigation Shortcuts:** Validated Escape key listeners in `UrlInput` (clearing input) and `Modal` (dialog dismissal), and Enter/Space keyboard toggling on `Toggle` switches (`role="switch"`, `aria-checked`).
  - Total automated test suite expanded to 152/152 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.4`) & Phase 1.3 Certification

#### Added
- **`[1.3.4]` Export Artifact Download Controls, SHA-256 Verification & Phase 1.3 Certification:**
  - Defined export artifact contracts and cryptographic SHA-256 generators in `apps/web/src/lib/types/download.ts`:
    - `computeSha256Hex`: Generates standard 64-character hexadecimal digests via Web Crypto API (`crypto.subtle.digest`).
    - `deriveArtifactBaseName`: Sanitizes filesystem-safe artifact base names from GitHub repositories and web documentation URLs.
    - `buildExportArtifacts`: Asynchronously generates concrete `ExportArtifact` records for all 5 formats with real content, byte sizes, and SHA-256 digests.
    - `generateChecksumsTxt`: Produces standard coreutils `sha256sum` formatted checksum lists (`<hash>  <filename>`).
    - `generateChecksumManifestJson`: Produces schema v1.0.0 JSON checksum manifests matching CDM specifications.
    - `triggerBlobDownload`: Client-side Blob download trigger with automatic Object URL cleanup.
  - Authored accessible, interactive `DownloadControls.svelte` in `apps/web/src/lib/components/workspace/DownloadControls.svelte`:
    - Region landmark (`role="region"`, `aria-label="Format Selection and Download Controls"`).
    - Batch download controls: "Download All Formats", "Checksums (.txt)", and "manifest.json".
    - Individual artifact cards with format icons, extension badges, byte sizes, truncated SHA-256 pills with one-click copy feedback, and download buttons.
    - Collapsible checksum manifest drawer rendering coreutils `checksums.txt` content.
    - Screen reader announcement live region (`role="status"`, `aria-live="polite"`).
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `DownloadControls` into `JobWorkspace.svelte` directly below the preview panel.
  - Authored comprehensive automated test suite `apps/web/tests/download-controls.test.ts` (11 tests) validating SHA-256 hashing, base name derivation, artifact generation, checksums.txt formatting, manifest JSON schema, and component ARIA landmarks.
  - Expanded total automated test suite to 133/133 passing tests across 13 test suites.
  - Formally certified completion of **Phase 1.3: Conversion workspace** across all 4 subphases (`1.3.1` – `1.3.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.3`)

#### Added
- **`[1.3.3]` Token Statistics Interface, Context Window Budget Bar & Headroom Metrics:**
  - Defined token budget evaluation domain model and calculations in `apps/web/src/lib/types/token-budget.ts`:
    - Mathematical helpers: `calculateSavingsPct` (token reduction ratio), `calculateUtilizationPct` (budget consumption), and `calculateHeadroom` (available conversational reasoning buffer).
    - Evaluation engine `evaluateBudgetUtilization` supporting 3 status levels: `optimal` (utilization $\le 85\%$), `warning` ($> 85\%$ and $\le 100\%$), and `exceeded` ($> 100\%$).
    - Calibration across AI model context windows (Gemini 2M, Gemini 1M, Claude 200k, GPT-4o 128k, Local 32k).
  - Authored accessible, interactive `TokenBudgetBar.svelte` in `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`:
    - Region landmark (`role="region"`, `aria-label="Token Statistics and Context Budget Bar"`).
    - 4-card metric grid: Raw Ingested Tokens, Deduplicated Tokens, Final Context Output, and Token Optimization Savings with reduction badge.
    - Dual-layer progress bar (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`) showing Final Tokens, ghost Deduplication Savings segment, and 80% safety margin threshold line.
    - Dynamic model preset switcher allowing instant re-evaluation against all major model context windows.
    - Headroom callout displaying remaining tokens for conversation prompts and completions.
    - Accessible context overflow alert banner (`role="alert"`) with one-click model upgrade switch to Gemini 1.5 Pro.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `TokenBudgetBar` into `JobWorkspace.svelte` positioned between the 4-stage pipeline execution cards and completeness accounting matrix.
  - Authored comprehensive automated test suite `apps/web/tests/token-budget.test.ts` (15 tests) validating savings math, headroom math, utilization bounds, ARIA landmarks, preset switching, and workspace integration.
  - Expanded total automated test suite to 122/122 passing tests across 12 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.2`)

#### Added
- **`[1.3.2]` Syntax-Highlighted Content Preview Panel & Multi-Format Exporters:**
  - Defined preview domain contracts and multi-format exporters in `apps/web/src/lib/types/preview.ts`:
    - 5 standard export formats: Markdown (`.md`), JSON CDM schema v1.0.0 (`.json`), Claude XML (`.xml`), Plaintext (`.txt`), and JSONL (`.jsonl`).
    - Canonical Context Document Model (CDM) serializer matching `docs/DATA_MODEL.md` with completeness accounting.
    - Claude context XML hierarchy serializer with `<documents>`, `<document index="...">`, and `<document_content>`.
    - Deterministic token estimation heuristic `estimateTokens` and sample extracted repository/article generator `generateSampleExtractedFiles`.
  - Authored zero-dependency, high-performance syntax highlighter in `apps/web/src/lib/utils/syntax-highlighter.ts`:
    - Strict HTML escaping (`escapeHtml`) preventing cross-site scripting (XSS).
    - Syntax token classification and styling for JSON (keys, strings, numbers, booleans, null), XML (tags, attributes, values, comments), Markdown (headings, blockquotes, code blocks, lists), and general code (keywords, comments, strings, numbers).
    - Line splitter `highlightLines` with 1-based line numbers.
  - Authored accessible, interactive `PreviewPanel.svelte` in `apps/web/src/lib/components/workspace/PreviewPanel.svelte`:
    - Accessible tablist (`role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`).
    - Extracted files sidebar with instant path filtering, file icons, byte size, token counts, and full concatenated context mode.
    - Copy to clipboard button with animated visual confirmation (`✓ Copied!`) and polite ARIA announcement (`role="status"`, `aria-live="polite"`).
    - Line numbers gutter with `aria-hidden="true"`, word wrap toggle, search within preview text, and direct format download.
    - Context metrics banner displaying estimated tokens, line counts, and byte sizes.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `PreviewPanel` into `JobWorkspace.svelte` with toggleable visibility and automatic target format synchronization.
  - Authored comprehensive automated test suite `apps/web/tests/preview-panel.test.ts` (17 tests) validating domain models, exporters, syntax highlighters, ARIA accessibility, and workspace integration.
  - Expanded total automated test suite to 107/107 passing tests across 11 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.1`)

#### Added
- **`[1.3.1]` Conversion Job Workspace, Pipeline Tracker & File Completeness Accounting:**
  - Defined conversion job domain model, state transitions, and file completeness invariant in `apps/web/src/lib/types/job.ts`:
    - Strict lifecycle states: `queued`, `leased`, `running`, `completed`, `partial`, `failed`, `cancelled`.
    - Standard 4-stage pipeline definitions: Acquisition & SSRF Defense, CDM Canonicalization, MinHash Deduplication, Export & Token Fit.
    - Verified invariant helper `verifyCompletenessInvariant` enforcing $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed}$ per `docs/DATA_MODEL.md` to prevent silent file drops.
    - Weighted progress calculator `calculateOverallProgress` and deterministic mock-free factory `createInitialJob`.
  - Authored accessible, responsive `JobWorkspace.svelte` component in `apps/web/src/lib/components/workspace/JobWorkspace.svelte`:
    - Overall progress bar landmark (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`).
    - 4-stage execution cards highlighting active, completed, failed, and pending stages with step numbers and status badges.
    - Repository completeness accounting matrix displaying discovered, extracted, excluded, and failed counts with verification badge.
    - Live execution telemetry log terminal with windowed DOM slice (most recent 200 items per Rule 03 bounded memory) and `role="log"`, `aria-live="polite"`.
    - Failure recovery alert banner (`role="alert"`) with error diagnostics and interactive retry trigger.
    - Action buttons for interactive job cancellation and pipeline retry.
  - Exported component via `apps/web/src/lib/components/workspace/index.ts`.
  - Authored dedicated workspace route in `apps/web/src/routes/workspace/+page.svelte`:
    - Reads query parameters (`?url=...&format=...&preset=...&budget=...`) on client load with fallback to initial verified target.
    - Integrated with site `Header`, `Footer`, breadcrumbs navigation, and SEO meta tags.
  - Updated `apps/web/src/routes/+page.svelte` to provide direct "Open Workspace →" action in conversion feedback banner.
  - Authored comprehensive automated test suite `apps/web/tests/job-workspace.test.ts` (14 tests) validating pipeline stages, job factories, completeness invariants, weighted progress math, ARIA attributes, and route integration.
  - Expanded total automated unit and invariant test suite to 90/90 passing tests across 10 test suites.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.4`) & Phase 1.2 Certification

#### Added
- **`[1.2.4]` Responsive Layout Auditing, Mobile Touch Targets & Phase 1.2 Certification:**
  - Audited and hardened responsive layout rules across all Phase 1.2 components:
    - Root layout (`apps/web/src/routes/+layout.svelte`) configured with `w-full max-w-full overflow-x-hidden` ensuring zero horizontal page scrolling across mobile viewports down to `320px`.
    - Enforced WCAG 2.1 AA recommended touch target sizing ($\ge 44 \times 44\text{px}$) across mobile hamburger button, theme switch toggle, mobile navigation drawer links, clear buttons, and submit controls.
    - Updated `Hero.svelte` action buttons with mobile full-width vertical stacking and $\ge 48\text{px}$ touch targets.
    - Updated `UrlInput.svelte` submit button with mobile full-width stacking (`w-full sm:w-auto min-h-[48px]`).
  - Authored automated responsive layout test suite `apps/web/tests/responsive-layout.test.ts` (7 tests) validating overflow containment, container padding, responsive grid collapsing, and touch target minimums.
  - Total automated test suite count expanded to 76/76 passing tests with zero failures across 9 test suites.
  - Formally certified completion of **Phase 1.2: Main website** across all 4 subphases (`1.2.1` – `1.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.2: Main Website (`1.2.3`)

#### Added
- **`[1.2.3]` Conversion Settings Panel, AI Model Presets & Content Normalization Toggles:**
  - Defined conversion settings contracts and calibrated AI model presets in `apps/web/src/lib/types/settings.ts`:
    - Model presets for Gemini 1.5 Pro (2M window), Gemini 1.5 Flash (1M window), Claude 3.5 Sonnet (200k window), GPT-4o (128k window), Local LLMs (32k/128k windows), and Custom budget.
    - CDM normalization and filtering options: `stripBoilerplate`, `stripCodeComments`, `excludeTests`, `enableDynamicRendering`, `semanticDeduplication`, `maxFileSizeKb`.
    - Validation utility enforcing budget bounds (1,000 to 2,000,000 tokens) and file size bounds (64 to 10,240 KB).
  - Authored accessible conversion settings component in `apps/web/src/lib/components/settings/ConversionSettings.svelte`:
    - Collapsible disclosure panel with `aria-expanded`, `aria-controls`, and `role="region"`.
    - Preset selection grid updating calibrated token ceilings.
    - Context window token budget range slider with live percentage progress bar and tabular number formatting.
    - Accessible switches (`Toggle`) for boilerplate stripping, code comment stripping, test exclusion, and MinHash semantic deduplication.
    - Dynamic Chromium rendering fallback toggle and file size threshold number input.
    - Exported component via `apps/web/src/lib/components/settings/index.ts`.
  - Integrated `ConversionSettings` into `apps/web/src/routes/+page.svelte` in `#convert`, binding settings state directly to conversion job feedback.
  - Authored automated unit and component invariant test suite `apps/web/tests/conversion-settings.test.ts` (6 tests), expanding total test suite count to 69/69 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.2`)

#### Added
- **`[1.2.2]` URL Input Component, Real-Time Source Detection & SSRF Pre-validation Engine:**
  - Authored URL detection and classification utility `apps/web/src/lib/utils/url-detector.ts`:
    - Full classification supporting standard public GitHub repositories, branch/tree subpaths, file blobs, XML sitemaps, sitemap indexes, and generic web documentation.
    - Implemented client-side zero-trust SSRF pre-check (`isBlockedHost`) preventing submission to loopback addresses (`127.0.0.0/8`, `::1`), RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata endpoints (`169.254.169.254`, `metadata.google.internal`), and non-HTTP protocols (`file:`, `javascript:`, `data:`, `ftp:`).
    - Extracted GitHub metadata including owner, repo, ref, and subpath.
  - Authored dedicated accessible URL input component in `apps/web/src/lib/components/input/UrlInput.svelte`:
    - Accessible search form landmark (`role="search"`, `aria-label="URL Ingestion Form"`).
    - Dynamic source badge with ARIA live region (`role="status"`, `aria-live="polite"`).
    - Contextual leading icons matching detected source modalities (GitHub Octocat, Sitemap XML, Security Warning Shield, Globe).
    - Clear button control (`aria-label="Clear URL input"`), `Escape` key handling, and submit button with SVG loading spinner.
    - Accessible error alert banner (`role="alert"`) rendering explicit SSRF and syntax error diagnostics.
    - Exported component via `apps/web/src/lib/components/input/index.ts`.
  - Integrated `UrlInput` into `apps/web/src/routes/+page.svelte` replacing the prototype input in `#convert` with real-time classification, dynamic target format suggestion, and verified submission feedback.
  - Authored automated unit test suites `apps/web/tests/url-detector.test.ts` (14 tests) and `apps/web/tests/url-input-component.test.ts` (3 tests), expanding total test suite count to 63/63 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.1`)

#### Added
- **`[1.2.1]` Production Landing Page, Semantic Navigation & Deterministic Pipeline Showcase:**
  - Authored navigation components in `apps/web/src/lib/components/navigation/`:
    - `Header.svelte`: Sticky semantic navigation header with brand AI icon, stage badge, desktop anchor navigation (`#convert`, `#workflow`, `#sources`, `#features`), external documentation and GitHub repository badges, accessible dark/light mode toggle with dynamic ARIA attributes, and responsive mobile dropdown drawer.
    - `Footer.svelte`: Semantic footer (`role="contentinfo"`) with copyright, MIT license, compliance badges (CPU-only 16 GB, <= 30 GB Podman, $0 / zero API costs), and external links to system architecture, SSRF defense, CDM data model, and performance targets.
    - `index.ts`: Unified navigation exports.
  - Authored landing feature components in `apps/web/src/lib/components/landing/`:
    - `Hero.svelte`: Production hero headline, value proposition subtitle, primary "Start Conversion" and "Explore How It Works" call-to-action buttons, 6 multi-format exporter pills (`.md`, `.json`, `.xml`, `.txt`, `.jsonl`, `.zip`), and hardware/operational metric badges.
    - `WorkflowSteps.svelte`: 3-step structured cards detailing Acquisition & SSRF Defense, CDM Canonicalization & Optimization, and Multi-Format Export & Token Fit.
    - `SupportedSources.svelte`: 4-source responsive grid matrix highlighting Static Documentation (HTML5 parser), Dynamic JavaScript SPAs (isolated Chromium fallback), Public GitHub Repositories (Git Trees API), and Private Repositories (GitHub App ephemeral tokens).
    - `FeatureHighlights.svelte`: 6 architectural pillar cards covering zero-trust SSRF defense, AST code intelligence, zero mandatory paid APIs, 30 GB Podman storage ceiling, deterministic token budgeting, and privacy-first self-hostability.
    - `index.ts`: Unified landing exports.
  - Assembled root `apps/web/src/routes/+page.svelte` integrating semantic landmarks (`header`, `main`, `footer`), quick conversion entry with URL input, reactive source detection, format selector pills, and live verification feedback banner.
  - Enhanced `apps/web/src/lib/components/ui/Button.svelte` with optional `href`, `target`, and `rel` attributes for accessible link buttons.
  - Enhanced `apps/web/src/lib/components/ui/Input.svelte` with optional `ariaLabel` attribute for headless input labeling.
  - Authored comprehensive test suite `apps/web/tests/landing-page.test.ts` validating semantic landmarks, navigation links, export formats, 3-step pipeline, 4 sources, and 6 feature pillars (expanding total passing tests to 46/46).

### 2026-11-17 — Phase 1.1: Design System (`1.1.3`)

#### Added
- **`[1.1.3]` Reusable Accessible UI Component Primitives & Phase 1.1 Completion:**
  - Authored accessible UI components in `apps/web/src/lib/components/ui/`:
    - `Button.svelte`: 5 variants (`primary`, `secondary`, `outline`, `ghost`, `danger`), 3 sizes (`sm`, `md`, `lg`), SVG loading spinner (`aria-busy`), disabled state, keyboard focus rings.
    - `Input.svelte`: accessible text/url input with label, helper text, error message, clear button, and ARIA described-by linkage.
    - `Toggle.svelte`: accessible switch primitive with `role="switch"`, `aria-checked`, label, description, and keyboard toggle (`Space`, `Enter`).
    - `Badge.svelte`: status badge with 6 semantic variants and optional status dot.
    - `Modal.svelte`: accessible dialog with backdrop overlay, `Escape` key dismissal, click-outside dismissal, `role="dialog"`, `aria-modal="true"`, and header/footer action slots.
    - `Tooltip.svelte`: contextual popover appearing on hover and keyboard focus (`role="tooltip"`).
  - Exported all UI primitives via `apps/web/src/lib/components/ui/index.ts`.
  - Updated showcase page `apps/web/src/routes/+page.svelte` demonstrating live interactive UI components.
  - Authored unit test suite `apps/web/tests/ui-components.test.ts`, expanding total test suite count to 37/37 passed tests.
  - Formally certified completion of **Phase 1.1: Design System** (Subphases `1.1.1`, `1.1.2`, `1.1.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.1: Design System (`1.1.2`)

#### Added
- **`[1.1.2]` Typography Hierarchy, Spacing Primitives & WCAG Contrast Systems:**
  - Authored accessible typography scale components in `apps/web/src/lib/components/typography/`: `Heading.svelte` supporting levels 1–6 (`h1`–`h6`) with proportional sizes `xs` to `5xl`, `Text.svelte` supporting body sizes, weights, and semantic tone variants, `Code.svelte` for inline and block code, and `Mono.svelte` for tabular numbers and token metrics.
  - Authored layout primitives in `apps/web/src/lib/components/layout/`: `Container.svelte` with responsive breakpoint max-widths, `Stack.svelte` supporting vertical/horizontal direction and 4px baseline gap spacing (`gap-0` to `gap-12`), and `Grid.svelte` supporting responsive 1–12 columns.
  - Implemented color contrast and accessibility utilities in `apps/web/src/lib/tokens/color-utils.ts` verifying WCAG 2.1 AA/AAA compliance thresholds, automatic text contrast selection, token budget status badge styling, and universal visible focus rings.
  - Published comprehensive design system specification in `docs/DESIGN_SYSTEM.md` covering typography hierarchy, 4px baseline rhythm, container breakpoints, surface layers, token budget bands, and WCAG AA/AAA certification tables.
  - Updated showcase page `apps/web/src/routes/+page.svelte` integrating typography and layout components with interactive budget controls.
  - Authored unit test suites `apps/web/tests/typography.test.ts` and `apps/web/tests/color-utils.test.ts`, expanding total test count to 31/31 passed tests.

### 2026-11-17 — Phase 1.1: Design System (`1.1.1`)

#### Added
- **`[1.1.1]` SvelteKit 2 + Svelte 5 Foundation & Visual Design Tokens:**
  - Initialized `apps/web` with SvelteKit 2, Svelte 5 runes (`$state`, `$derived`, `$props`), strict TypeScript, Tailwind CSS, PostCSS, Autoprefixer, `@sveltejs/adapter-node`, Vitest, and Prettier.
  - Implemented atomic design token module `apps/web/src/lib/tokens/index.ts` defining brand primary AI violet/indigo (`#6366f1`), accent telemetry cyan (`#06b6d4`), neutral slate surfaces (`#090d16` canvas to `#f8fafc`), semantic status indicators (`success`, `warning`, `error`, `info`), and context token budget indicators (`safe` <=80%, `warning` >80%, `critical` >100%).
  - Implemented CSS custom properties stylesheet `apps/web/src/lib/styles/tokens.css` with `:root` and `.dark` variables, alongside base styling `apps/web/src/lib/styles/app.css`.
  - Configured `apps/web/tailwind.config.ts` cleanly integrating design tokens into the Tailwind theme engine.
  - Authored root layout `+layout.svelte` and interactive token budget test showcase `+page.svelte`.
  - Authored automated Vitest suite `apps/web/tests/tokens.test.ts` verifying token completeness, hex regex validation, budget threshold logic, WCAG AAA dark background contrast (>= 7:1), WCAG AA brand contrast (>= 4.5:1), and CSS variable flattening.
  - Verified all 5 monorepo verification gates with 0 errors across PowerShell and POSIX runners.

---

## Stage 0: Project Bootstrap and Engineering Foundation (`COMPLETED` / `ACCEPTED`)

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.5`)

#### Added
- **`[0.3.5]` Setup Documentation, Disaster Recovery Runbooks & Stage 0 Certification:**
  - Authored comprehensive Environment Variables Reference in `docs/DEPLOYMENT.md` detailing all 19 configuration variables from `deploy/.env.example` (PostgreSQL, Axum API, Worker, Playwright, SvelteKit, and GitHub App credentials).
  - Authored PostgreSQL Logical Backup & Restore Runbook (`pg_dump`/`pg_restore`) for both Windows PowerShell and POSIX bash with clean table recreation and volume copying.
  - Authored Artifact Storage Volume Backup & Restore Runbook for `deploy_artifact_data`.
  - Authored Emergency Cold Disaster Recovery Runbook guaranteeing complete system recovery from cold workstation restart or storage corruption in under 5 minutes.
  - Authored Production Hardening Guidelines covering Caddy automatic HTTPS reverse proxy configuration with TLS headers, secret manager injection, block-level encryption at rest, AES-256-GCM token encryption, read-only container filesystems, dropped Linux capabilities, and container resource limits.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 223/223 automated checks.
  - Formally certified completion of **Stage 0: Project Bootstrap and Engineering Foundation** (Phases `0.1`, `0.2`, `0.3`, 15 subphases `0.1.1` – `0.3.5` all `ACCEPTED`).

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.3`)

#### Added
- **`[0.3.3]` Development Environment Lifecycle Scripts:**
  - Authored `scripts/dev-up.ps1` and `scripts/dev-up.sh` for starting the compose stack, ensuring `deploy/.env` initialization, verifying the 30 GB storage ceiling prior to launch, supporting service subsets (`-Services`) and image rebuilding (`-Build`), and awaiting database healthcheck readiness.
  - Authored `scripts/dev-down.ps1` and `scripts/dev-down.sh` for service teardown with interactive protection for persistent database volumes (`postgres_data`, `artifact_data`) and post-teardown cache pruning (`-PruneCaches`).
  - Authored `scripts/dev-logs.ps1` and `scripts/dev-logs.sh` supporting service filtering, line tailing, and real-time log streaming (`-Follow`).
  - Authored `scripts/dev-restart.ps1` and `scripts/dev-restart.sh` for safe service restarts and healthcheck validation.
  - Authored `scripts/dev-status.ps1` and `scripts/dev-status.sh` reporting runtime container state, healthcheck status, exposed ports, and storage budget compliance in table and structured JSON formats.
  - Documented complete developer lifecycle workflow in `docs/DEVELOPMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 213/213 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.2: GitHub and CI Foundation (`0.2.1` – `0.2.5`)

#### Added
- **`[0.2.5]` Security & Dependency Scanning:**
  - Configured `deny.toml` with strict RustSec advisory checks, permissive license allowlists (MIT, Apache-2.0, BSD, ISC, Unicode, PostgreSQL, CC0, Zlib, Unlicense), copyleft rejection, and registry restrictions.
  - Authored `.github/workflows/security.yml` executing secret leakage scans, `cargo-deny` checks, and `npm audit`.
  - Authored `.github/workflows/dependency-review.yml` gating PRs on dependency vulnerabilities and license compliance.
  - Authored `.github/workflows/codeql.yml` running automated CodeQL static security analysis.
  - Extended verification suite (`scripts/verify-bootstrap.ps1` & `scripts/check-all.ps1`) to 187 automated checks.
- **`[0.2.4]` Linting, Formatting & Quality Check Runners:**
  - Configured `rustfmt.toml` enforcing Rust 2021 edition, 100 character width, Unix LF line endings, import reordering, and comment formatting.
  - Configured `.prettierrc.json` and `.prettierignore` for web, JSON, Markdown, and TypeScript styling.
  - Authored cross-platform monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `scripts/verify-bootstrap.ps1` with lint/format configuration assertions.
- **`[0.2.3]` GitHub Actions CI/CD Workflows:**
  - Authored `.github/workflows/ci.yml` with concurrency controls, bootstrap integrity checks, and conditional Rust and SvelteKit test gates.
  - Authored `.github/workflows/build.yml` with separate artifact uploads for frontend build bundle, backend binaries, and SHA-256 checksums.
  - Authored `.github/workflows/container.yml` auditing Containerfile specifications and enforcing the 30 GB storage ceiling.
  - Authored `.github/workflows/release.yml` with automated release gate verification, CycloneDX SBOM generation, SHA-256 checksums, and GitHub Release asset publishing.
  - Created `scripts/validate-workflows.py` for automated PyYAML validation and integrated it into `scripts/verify-bootstrap.ps1`.
- **`[0.2.2]` Branch Protection & Repository Governance:**
  - Configured repository merge strategy via GitHub REST API: disabled merge commits (`allow_merge_commit: false`), enabled squash merge (`allow_squash_merge: true`), and enabled rebase merge (`allow_rebase_merge: true`) to enforce linear history.
  - Enabled automatic head branch pruning on PR merge (`delete_branch_on_merge: true`).
  - Enabled Dependabot vulnerability alerts and automated security fixes.
  - Enforced branch protection on `main`: blocked force pushes (`allow_force_pushes: false`), blocked branch deletions (`allow_deletions: false`), and required PR conversation resolution (`required_conversation_resolution: true`).
  - Documented repository governance and protection rules in `docs/DEVELOPMENT.md`.
- **`[0.2.1]` GitHub Remote Repository & Push:**
  - Verified Git Credential Manager OAuth scopes (`gist, repo, workflow`) for authenticated user `dayashimoga`.
  - Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai` via GitHub REST API with issues enabled.
  - Configured Git remote `origin` pointing to `https://github.com/dayashimoga/web-repo-to-ai.git`.
  - Pushed `main` branch with upstream tracking (`git push -u origin main`).
  - Verified remote tracking and commit SHA alignment (`98cc1decf23f270aef970877eb3a4b574b86d822 refs/heads/main`).

### 2026-11-16 — Phase 0.1: Repository and Governance Initialization (`0.1.1` – `0.1.5`)

#### Added
- **`[0.1.1]` Environment & Tooling Inspection:**
  - Verified Windows host environment (14 logical processors, 16.64 GB physical RAM, 33.51 GB free disk on `C:`), `git 2.55.0.windows.5`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (`podman-machine-default` WSL VM with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager authentication status (`dayashimoga`).
- **`[0.1.2]` Monorepo & Git Foundation (`58e0f80`):**
  - Initialized local Git repository on `main` branch with LF line-ending normalization (`.gitattributes`, `core.autocrlf=false`).
  - Added `.gitignore`, `.editorconfig`, MIT `LICENSE`, and `README.md`.
  - Created monorepo structure across `apps/web`, `services/{api,worker,browser-worker}`, 15 shared Rust crate directories (`crates/{core,cdm,url-security,web-extractor,github-extractor,code-intelligence,normalizer,deduplication,tokenizer,context-optimizer,chunking,exporters,storage,job-engine,observability}`), `docs/`, `deploy/`, `scripts/`, `.agent/`, and `.github/workflows/`.
- **`[0.1.3]` Engineering Contract, Requirements & Baseline Documentation (`6372d8a`):**
  - Created `GLOBAL_ENGINEERING_CONTRACT.md` enforcing product integrity, CDM-centered Rust architecture, tiered performance, completeness accounting, SSRF/OAuth security, and one-subphase-at-a-time execution.
  - Created `REQUIREMENTS.md`, `docs/REQUIREMENTS.md`, and `DECISIONS.md` (`ADR-001` through `ADR-007`).
  - Created `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DATA_MODEL.md`, `docs/API.md`, `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md`, `docs/USER_GUIDE.md`, `docs/TESTING.md`, `docs/PERFORMANCE.md`, and `docs/LICENSING.md`.
- **`[0.1.4]` Agent System, Skills, Rules & Workflows (`f2aa8e9`):**
  - Created `AGENTS.md` and registered 9 specialized agent roles (`architecture-lead`, `frontend-engineer`, `rust-backend-engineer`, `extraction-engineer`, `github-integration-engineer`, `security-reviewer`, `qa-engineer`, `devops-engineer`, `documentation-reviewer`) in the Antigravity runtime and `.agent/agents/*.md`.
  - Created 10 domain skills under `.agent/skills/*/SKILL.md` (`architecture`, `rust-engineering`, `frontend-engineering`, `web-extraction`, `github-integration`, `context-optimization`, `security-engineering`, `testing-quality`, `devops-cicd`, `documentation`).
  - Created 10 engineering rules under `.agent/rules/00-global-engineering.md` through `09-no-mock-no-placeholder.md`.
  - Created 10 reusable workflows under `.agent/workflows/*.md`.
- **`[0.1.5]` Staged Roadmap, Ledger & Bootstrap Verification:**
  - Created `ROADMAP.md` and `docs/ROADMAP.md` defining all 12 stages (`Stage 0` – `Stage 11`), 48 phases, and 197 subphases.
  - Created `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1`.
