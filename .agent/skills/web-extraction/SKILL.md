---
name: web-extraction
description: >-
  Use this skill when implementing or testing webpage acquisition, static HTML
  parsing (html5ever/scraper), article/documentation extraction, Playwright
  dynamic rendering fallback, or web crawling in crates/web-extractor and
  services/browser-worker.
---

# Web Extraction & Crawling Engineering Skill

## 1. Purpose
Convert publicly accessible webpages (static HTML articles, technical documentation, tables, code examples, and JS-rendered SPAs) into accurate, provenance-traceable `CdmDocument` representations.

## 2. When to Use
- Implementing HTTP fetching, content-type handling, and DOM parsing in `crates/web-extractor`.
- Implementing readability article extraction, technical documentation extraction, table normalization, and code-block language detection.
- Implementing or securing `services/browser-worker` (Playwright + Chromium dynamic fallback).
- Implementing sitemap discovery and bounded internal link crawling in Stage 8.

## 3. Required Procedures
1. Route all URL validation, DNS resolution, IP pinning, and redirect revalidation through `crates/url-security`.
2. Attempt fast streaming HTTP acquisition first; inspect extracted structural density before invoking `services/browser-worker`.
3. Strip non-content boilerplate (`<nav>`, `<footer>`, cookie modals, ad containers, tracking scripts) while strictly preserving `<main>`, `<article>`, heading hierarchies (`h1`–`h6`), `<table>` structures, and `<pre><code>` blocks.
4. Resolve all relative links and image/citation URLs against the document's canonical base URL.
5. Record HTTP status, `ETag`/`Last-Modified`, canonical URL, retrieval timestamp, and any extraction warnings in `CdmDocument`.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [04-extraction-accuracy.md](../../rules/04-extraction-accuracy.md)
- Enforces [05-github-security.md](../../rules/05-github-security.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- Unit and golden-dataset regression tests covering articles, documentation pages, complex tables, code blocks, and malformed HTML.
- Adversarial SSRF, redirect loop, decompression limit, and timeout tests.

## 6. Expected Outputs
- Validated `CdmDocument` instances with complete section hierarchies, preserved code/tables, and accurate provenance metadata.

## 7. Common Failure Modes
- **Code Corruption:** Stripping whitespace or collapsing newlines inside `<pre><code>` blocks during HTML text normalization.
- **Unnecessary Browser Rendering:** Spinning up Chromium for static pages where HTTP extraction already succeeded.
- **Subresource SSRF in Browser Worker:** Allowing a dynamically rendered webpage to `fetch("http://169.254.169.254/...")` or `http://127.0.0.1:...` from inside Chromium.
