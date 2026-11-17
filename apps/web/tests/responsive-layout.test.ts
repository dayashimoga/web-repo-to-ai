import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Responsive Layout & Viewport Invariants", () => {
  it("Root layout enforces horizontal overflow containment", () => {
    const layoutPath = resolve(process.cwd(), "src/routes/+layout.svelte");
    const layoutContent = readFileSync(layoutPath, "utf-8");

    expect(layoutContent).toContain("overflow-x-hidden");
    expect(layoutContent).toContain("max-w-full");
    expect(layoutContent).toContain("min-h-screen");
  });

  it("All landing sections use standard responsive container padding", () => {
    const heroPath = resolve(process.cwd(), "src/lib/components/landing/Hero.svelte");
    const heroContent = readFileSync(heroPath, "utf-8");
    expect(heroContent).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

    const workflowPath = resolve(process.cwd(), "src/lib/components/landing/WorkflowSteps.svelte");
    const workflowContent = readFileSync(workflowPath, "utf-8");
    expect(workflowContent).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

    const sourcesPath = resolve(
      process.cwd(),
      "src/lib/components/landing/SupportedSources.svelte"
    );
    const sourcesContent = readFileSync(sourcesPath, "utf-8");
    expect(sourcesContent).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

    const featuresPath = resolve(
      process.cwd(),
      "src/lib/components/landing/FeatureHighlights.svelte"
    );
    const featuresContent = readFileSync(featuresPath, "utf-8");
    expect(featuresContent).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

    const footerPath = resolve(process.cwd(), "src/lib/components/navigation/Footer.svelte");
    const footerContent = readFileSync(footerPath, "utf-8");
    expect(footerContent).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");
  });

  it("Responsive grids collapse cleanly to 1 or 2 columns on mobile viewports", () => {
    const heroPath = resolve(process.cwd(), "src/lib/components/landing/Hero.svelte");
    const heroContent = readFileSync(heroPath, "utf-8");
    expect(heroContent).toContain("grid-cols-2 sm:grid-cols-3 lg:grid-cols-6");
    expect(heroContent).toContain("grid-cols-1 sm:grid-cols-3");

    const workflowPath = resolve(process.cwd(), "src/lib/components/landing/WorkflowSteps.svelte");
    const workflowContent = readFileSync(workflowPath, "utf-8");
    expect(workflowContent).toContain("grid-cols-1 md:grid-cols-3");

    const sourcesPath = resolve(
      process.cwd(),
      "src/lib/components/landing/SupportedSources.svelte"
    );
    const sourcesContent = readFileSync(sourcesPath, "utf-8");
    expect(sourcesContent).toContain("grid-cols-1 md:grid-cols-2");

    const featuresPath = resolve(
      process.cwd(),
      "src/lib/components/landing/FeatureHighlights.svelte"
    );
    const featuresContent = readFileSync(featuresPath, "utf-8");
    expect(featuresContent).toContain("grid-cols-1 md:grid-cols-2 lg:grid-cols-3");

    const footerPath = resolve(process.cwd(), "src/lib/components/navigation/Footer.svelte");
    const footerContent = readFileSync(footerPath, "utf-8");
    expect(footerContent).toContain("grid-cols-1 md:grid-cols-2 lg:grid-cols-5");
  });
});

describe("Touch Target Accessibility Invariants (WCAG 2.1 AA / AAA)", () => {
  it("Header mobile menu button and theme toggle enforce >= 44x44px touch targets", () => {
    const headerPath = resolve(process.cwd(), "src/lib/components/navigation/Header.svelte");
    const headerContent = readFileSync(headerPath, "utf-8");

    expect(headerContent).toContain("min-h-[44px] min-w-[44px]");
    expect(headerContent).toContain('aria-label="Toggle navigation menu"');
    expect(headerContent).toContain('aria-controls="mobile-navigation"');
    expect(headerContent).toContain("aria-expanded={mobileMenuOpen}");
  });

  it("Mobile dropdown navigation links provide comfortable touch padding", () => {
    const headerPath = resolve(process.cwd(), "src/lib/components/navigation/Header.svelte");
    const headerContent = readFileSync(headerPath, "utf-8");

    expect(headerContent).toContain("min-h-[44px] flex items-center");
    expect(headerContent).toContain('id="mobile-navigation"');
  });

  it("Hero action buttons enforce full-width mobile stacking and >= 48px touch heights", () => {
    const heroPath = resolve(process.cwd(), "src/lib/components/landing/Hero.svelte");
    const heroContent = readFileSync(heroPath, "utf-8");

    expect(heroContent).toContain("flex flex-col sm:flex-row");
    expect(heroContent).toContain("w-full sm:w-auto min-h-[48px]");
  });

  it("UrlInput submit and clear controls provide responsive touch targets", () => {
    const inputPath = resolve(process.cwd(), "src/lib/components/input/UrlInput.svelte");
    const inputContent = readFileSync(inputPath, "utf-8");

    expect(inputContent).toContain("w-full sm:w-auto min-h-[48px]");
    expect(inputContent).toContain('aria-label="Clear URL input"');
  });
});
