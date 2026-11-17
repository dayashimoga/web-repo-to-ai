import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  isWcagAaCompliant,
  isWcagAaaCompliant,
  getContrastRatio,
  getAccessibleTextColor,
} from "../src/lib/tokens/color-utils";
import { colors } from "../src/lib/tokens/index";

function readComponentSource(relativePath: string): string {
  const filePath = resolve(process.cwd(), relativePath);
  return readFileSync(filePath, "utf-8");
}

describe("Accessibility Audit (WCAG 2.1 AA/AAA) — Monorepo UX Certification", () => {
  describe("1. Semantic HTML Landmarks & Region Hierarchy", () => {
    it("verifies Header and Footer landmarks", () => {
      const headerContent = readComponentSource("src/lib/components/navigation/Header.svelte");
      expect(headerContent).toContain("<header");
      expect(headerContent).toContain('<nav aria-label="Main Navigation"');

      const footerContent = readComponentSource("src/lib/components/navigation/Footer.svelte");
      expect(footerContent).toContain("<footer");
      expect(footerContent).toContain('rel="noopener noreferrer"');
    });

    it("verifies Page-level landmarks and main content landmark on routes", () => {
      const landingPage = readComponentSource("src/routes/+page.svelte");
      expect(landingPage).toContain('<main id="main-content"');
      expect(landingPage).toContain("Header");
      expect(landingPage).toContain("Footer");

      const workspacePage = readComponentSource("src/routes/workspace/+page.svelte");
      expect(workspacePage).toContain('<main id="main-content"');
      expect(workspacePage).toContain("Header");
      expect(workspacePage).toContain("Footer");
      expect(workspacePage).toContain("JobWorkspace");
    });

    it("verifies explicit section landmarks on landing page", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("<section");
      expect(hero).toContain('aria-labelledby="hero-title"');

      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain("<section");
      expect(workflow).toContain('id="workflow"');
      expect(workflow).toContain('aria-labelledby="workflow-heading"');

      const sources = readComponentSource("src/lib/components/landing/SupportedSources.svelte");
      expect(sources).toContain("<section");
      expect(sources).toContain('id="sources"');
      expect(sources).toContain('aria-labelledby="sources-heading"');

      const features = readComponentSource("src/lib/components/landing/FeatureHighlights.svelte");
      expect(features).toContain("<section");
      expect(features).toContain('aria-labelledby="features-heading"');
    });
  });

  describe("2. WAI-ARIA Progressbar Pattern Compliance", () => {
    it("validates JobWorkspace overall pipeline progressbar contract", () => {
      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain('role="progressbar"');
      expect(workspace).toContain("aria-valuenow={overallPct}");
      expect(workspace).toContain("aria-valuemin={0}");
      expect(workspace).toContain("aria-valuemax={100}");
      expect(workspace).toContain('aria-label="Overall conversion pipeline progress"');
    });

    it("validates TokenBudgetBar context utilization progressbar contract", () => {
      const budgetBar = readComponentSource("src/lib/components/workspace/TokenBudgetBar.svelte");
      expect(budgetBar).toContain('role="progressbar"');
      expect(budgetBar).toContain("aria-valuenow={report.utilizationPct}");
      expect(budgetBar).toContain("aria-valuemin={0}");
      expect(budgetBar).toContain("aria-valuemax={100}");
      expect(budgetBar).toContain('aria-label="Context window token budget utilization"');
    });
  });

  describe("3. WAI-ARIA Tabs Design Pattern Compliance", () => {
    it("validates PreviewPanel export format tablist, tabs, and tabpanel relationships", () => {
      const preview = readComponentSource("src/lib/components/workspace/PreviewPanel.svelte");

      // Tablist container
      expect(preview).toContain('role="tablist"');
      expect(preview).toContain('aria-label="Export format preview tabs"');

      // Tab controls
      expect(preview).toContain('role="tab"');
      expect(preview).toContain('id="tab-{fmt.id}"');
      expect(preview).toContain("aria-selected={selectedFormat === fmt.id}");
      expect(preview).toContain('aria-controls="panel-{fmt.id}"');

      // Tabpanel target
      expect(preview).toContain('role="tabpanel"');
      expect(preview).toContain('id="panel-{selectedFormat}"');
      expect(preview).toContain('aria-labelledby="tab-{selectedFormat}"');
    });
  });

  describe("4. Screen Reader Polite Live Regions & Assertive Alerts", () => {
    it("verifies polite live regions for non-critical status updates", () => {
      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");
      expect(urlInput).toContain('role="status"');
      expect(urlInput).toContain('aria-live="polite"');

      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain('role="log"');
      expect(workspace).toContain('aria-live="polite"');

      const preview = readComponentSource("src/lib/components/workspace/PreviewPanel.svelte");
      expect(preview).toContain('role="status"');
      expect(preview).toContain('aria-live="polite"');

      const download = readComponentSource("src/lib/components/workspace/DownloadControls.svelte");
      expect(download).toContain('role="status"');
      expect(download).toContain('aria-live="polite"');
    });

    it("verifies assertive alert banners for critical errors and budget overflows", () => {
      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");
      expect(urlInput).toContain('role="alert"');

      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain('role="alert"');

      const budgetBar = readComponentSource("src/lib/components/workspace/TokenBudgetBar.svelte");
      expect(budgetBar).toContain('role="alert"');
    });
  });

  describe("5. Screen Reader Exclusion of Decorative Elements", () => {
    it("hides line numbers gutter from screen readers", () => {
      const preview = readComponentSource("src/lib/components/workspace/PreviewPanel.svelte");
      expect(preview).toContain('aria-hidden="true"');
      expect(preview).toContain("line.lineNumber");
    });

    it("hides decorative SVGs and indicators across components", () => {
      const header = readComponentSource("src/lib/components/navigation/Header.svelte");
      expect(header).toContain('aria-hidden="true"');

      const settings = readComponentSource("src/lib/components/settings/ConversionSettings.svelte");
      expect(settings).toContain('aria-hidden="true"');

      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain('aria-hidden="true"');
    });
  });

  describe("6. Universal Visible Focus Rings (WCAG 2.4.7)", () => {
    it("enforces focus visible ring classes across all interactive components", () => {
      const button = readComponentSource("src/lib/components/ui/Button.svelte");
      expect(button).toContain("focus-visible:ring");

      const input = readComponentSource("src/lib/components/ui/Input.svelte");
      expect(input).toContain("focus-visible:ring");

      const toggle = readComponentSource("src/lib/components/ui/Toggle.svelte");
      expect(toggle).toContain("focus-visible:ring");

      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");
      expect(urlInput).toContain("focus-visible:ring");

      const settings = readComponentSource("src/lib/components/settings/ConversionSettings.svelte");
      expect(settings).toContain("focus-visible:ring");

      const preview = readComponentSource("src/lib/components/workspace/PreviewPanel.svelte");
      expect(preview).toContain("focus-visible:ring");

      const budgetBar = readComponentSource("src/lib/components/workspace/TokenBudgetBar.svelte");
      expect(budgetBar).toContain("focus-visible:ring");

      const download = readComponentSource("src/lib/components/workspace/DownloadControls.svelte");
      expect(download).toContain("focus-visible:outline-none");
    });
  });

  describe("7. Color Contrast Certification (WCAG 2.1 AA & AAA)", () => {
    it("validates Dark Canvas with White text satisfies WCAG AAA (>= 7.0:1)", () => {
      const ratio = getContrastRatio("#ffffff", colors.surface[950]);
      expect(ratio).toBeGreaterThanOrEqual(14.0);
      expect(isWcagAaaCompliant("#ffffff", colors.surface[950])).toBe(true);
    });

    it("validates Dark Panel with White text satisfies WCAG AAA (>= 7.0:1)", () => {
      const ratio = getContrastRatio("#ffffff", colors.surface[900]);
      expect(ratio).toBeGreaterThanOrEqual(12.0);
      expect(isWcagAaaCompliant("#ffffff", colors.surface[900])).toBe(true);
    });

    it("validates Light Canvas with Slate-900 text satisfies WCAG AAA (>= 7.0:1)", () => {
      const ratio = getContrastRatio(colors.surface[950], colors.surface[50]);
      expect(ratio).toBeGreaterThanOrEqual(14.0);
      expect(isWcagAaaCompliant(colors.surface[950], colors.surface[50])).toBe(true);
    });

    it("validates Brand-600 with White text satisfies WCAG AA (>= 4.5:1)", () => {
      const ratio = getContrastRatio("#ffffff", colors.brand[600]);
      expect(ratio).toBeGreaterThanOrEqual(4.5);
      expect(isWcagAaCompliant("#ffffff", colors.brand[600])).toBe(true);
    });

    it("validates automated text color picker chooses correct contrast polarity", () => {
      expect(getAccessibleTextColor(colors.surface[950])).toBe("#ffffff");
      expect(getAccessibleTextColor(colors.surface[50])).toBe(colors.surface[950]);
    });
  });

  describe("8. Keyboard Shortcuts & Form Navigation", () => {
    it("verifies Escape key handling in UrlInput", () => {
      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");
      expect(urlInput).toContain("e.key === 'Escape'");
      expect(urlInput).toContain("handleClear");
    });

    it("verifies Modal Escape key dismissal", () => {
      const modal = readComponentSource("src/lib/components/ui/Modal.svelte");
      expect(modal).toContain("e.key === 'Escape'");
      expect(modal).toContain('role="dialog"');
      expect(modal).toContain('aria-modal="true"');
    });

    it("verifies Toggle WAI-ARIA switch keyboard activation", () => {
      const toggle = readComponentSource("src/lib/components/ui/Toggle.svelte");
      expect(toggle).toContain('role="switch"');
      expect(toggle).toContain("aria-checked");
    });
  });
});
