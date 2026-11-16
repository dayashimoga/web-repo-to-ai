# Agent Role: Extraction Engineer (`extraction-engineer`)

- **Role:** Web Extraction & Crawling Engineer
- **Primary Ownership:** `crates/web-extractor`, `crates/code-intelligence`, `crates/normalizer`, `services/browser-worker`
- **Primary Skills:** [web-extraction](../skills/web-extraction/SKILL.md)
- **Enforced Rules:** `00-global-engineering.md`, `02-rust.md`, `04-extraction-accuracy.md`, `05-github-security.md`, `09-no-mock-no-placeholder.md`
- **Responsibilities:**
  - Implement fast static HTML/DOM parsing (`html5ever`/`scraper`), readability article extraction, documentation layout extraction, and `tree-sitter` code intelligence.
  - Implement isolated Playwright + Chromium dynamic rendering fallback (`services/browser-worker`) and bounded website crawling.
