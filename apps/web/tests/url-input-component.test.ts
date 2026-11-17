import { describe, it, expect } from "vitest";
import { UrlInput } from "../src/lib/components/input/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("UrlInput Component Integrity & Accessibility", () => {
  it("exports UrlInput component cleanly from input module", () => {
    expect(UrlInput).toBeDefined();
  });

  it("UrlInput component defines required semantic landmarks and ARIA contracts", () => {
    const componentPath = resolve(process.cwd(), "src/lib/components/input/UrlInput.svelte");
    const content = readFileSync(componentPath, "utf-8");

    // Semantic form container
    expect(content).toContain("<form");
    expect(content).toContain('role="search"');
    expect(content).toContain('aria-label="URL Ingestion Form"');

    // ARIA Live region for screen readers announcing classification updates
    expect(content).toContain('role="status"');
    expect(content).toContain('aria-live="polite"');

    // Main input ARIA attributes
    expect(content).toContain("aria-label=");
    expect(content).toContain("aria-invalid=");
    expect(content).toContain("aria-describedby=");

    // Clear control
    expect(content).toContain('aria-label="Clear URL input"');

    // Error alert banner
    expect(content).toContain('role="alert"');

    // Svelte 5 Runes usage
    expect(content).toContain("$bindable");
    expect(content).toContain("$derived");
    expect(content).toContain("$effect");
    expect(content).toContain("$props");
  });

  it("UrlInput component handles leading icons for all source modalities", () => {
    const componentPath = resolve(process.cwd(), "src/lib/components/input/UrlInput.svelte");
    const content = readFileSync(componentPath, "utf-8");

    expect(content).toContain("detection.sourceType === 'github_repo'");
    expect(content).toContain("detection.sourceType === 'sitemap'");
    expect(content).toContain("detection.sourceType === 'ssrf_blocked'");
  });
});
